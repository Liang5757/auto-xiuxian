#![windows_subsystem = "windows"]
mod native;
mod session;
mod vision;
use eframe::egui::{self, Color32, RichText};
use session::{Event, Session};
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, Instant},
};
struct App {
    dispatch_page: bool,
    running: bool,
    clicking: bool,
    stop: Arc<AtomicBool>,
    lock: Arc<Mutex<()>>,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    status: String,
    map: String,
    click_status: String,
    ok: u64,
    fail: u64,
    clicks: u64,
    logs: VecDeque<String>,
    interval: u64,
    right: bool,
    key: i32,
    held: bool,
    game: Option<isize>,
    scan: Instant,
    next: Instant,
    closing: bool,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(bytes) = std::fs::read(format!(
            r"{}\Fonts\msyh.ttc",
            std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into())
        )) {
            fonts
                .font_data
                .insert("cjk".into(), Arc::new(egui::FontData::from_owned(bytes)));
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push("cjk".into())
            }
        }
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let (tx, rx) = mpsc::channel();
        Self {
            dispatch_page: true,
            running: false,
            clicking: false,
            stop: Arc::new(AtomicBool::new(false)),
            lock: Default::default(),
            tx,
            rx,
            status: "已停止".into(),
            map: "待识别".into(),
            click_status: "已停止".into(),
            ok: 0,
            fail: 0,
            clicks: 0,
            logs: Default::default(),
            interval: 100,
            right: false,
            key: 0x75,
            held: false,
            game: None,
            scan: Instant::now(),
            next: Instant::now(),
            closing: false,
        }
    }
    fn log(&mut self, s: String) {
        let mut t = unsafe { std::mem::zeroed() };
        unsafe { windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut t) };
        self.logs.push_front(format!(
            "{:02}:{:02}:{:02}     {}",
            t.wHour, t.wMinute, t.wSecond, s
        ));
        self.logs.truncate(100)
    }
    fn toggle(&mut self) {
        if self.running {
            self.stop.store(true, Ordering::Relaxed);
            self.status = "正在停止".into()
        } else {
            self.stop.store(false, Ordering::Relaxed);
            self.running = true;
            self.status = "正在监听".into();
            self.log("立即检查小队返回提示".into());
            let job = Session::new(self.stop.clone(), self.tx.clone(), self.lock.clone());
            std::thread::spawn(move || job.run());
        }
    }
    fn click_switch(&mut self) {
        self.clicking = !self.clicking;
        self.click_status = if self.clicking {
            "等待鼠标指向游戏"
        } else {
            "已停止"
        }
        .into()
    }
    fn poll(&mut self) {
        while let Ok(e) = self.rx.try_recv() {
            match e {
                Event::State(s) => self.status = s,
                Event::Map(s) => self.map = s,
                Event::Log(s) => self.log(s),
                Event::Success(n) => {
                    self.ok += 1;
                    self.log(format!("本轮成功，重新派遣 {n} 支小队"))
                }
                Event::Failure(e) => {
                    self.fail += 1;
                    self.log(e)
                }
                Event::Stopped => {
                    self.running = false;
                    self.status = "已停止".into()
                }
            }
        }
        let now = Instant::now();
        if now >= self.scan {
            self.game = native::game().ok().flatten();
            self.scan = now + Duration::from_secs(2)
        }
        if self.running && native::key(0x1b) {
            self.stop.store(true, Ordering::Relaxed);
        }
        let held = native::key(self.key);
        if held && !self.held && !self.closing {
            self.click_switch()
        }
        self.held = held;
        if self.clicking && now >= self.next {
            self.next = now + Duration::from_millis(self.interval);
            if let Ok(_g) = self.lock.try_lock() {
                match self.game.map(|h| native::repeat_click(h, self.right)) {
                    Some(Ok(true)) => {
                        self.clicks += 1;
                        self.click_status = "正在连点".into()
                    }
                    Some(Err(e)) => {
                        self.clicking = false;
                        self.click_status = e
                    }
                    _ => self.click_status = "等待鼠标指向游戏".into(),
                }
            } else {
                self.click_status = "派遣检查中".into()
            }
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.poll();
        if ctx.input(|i| i.viewport().close_requested()) && self.running {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.closing = true;
            self.clicking = false;
            self.stop.store(true, Ordering::Relaxed);
            self.status = "完成当前操作后关闭".into()
        }
        if self.closing && !self.running {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close)
        }
        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(180.)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(37, 38, 49))
                    .inner_margin(16),
            )
            .show(ctx, |ui| {
                ui.add_space(12.);
                ui.heading(RichText::new("◆ 修仙助手").color(Color32::WHITE));
                ui.add_space(30.);
                for (page, title) in [(true, "自动派遣"), (false, "连点器")] {
                    let selected = self.dispatch_page == page;
                    let b = egui::Button::new(RichText::new(title).color(Color32::WHITE).size(17.))
                        .fill(if selected {
                            Color32::from_rgb(98, 73, 136)
                        } else {
                            Color32::TRANSPARENT
                        });
                    if ui.add_sized([148., 40.], b).clicked() {
                        self.dispatch_page = page
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.colored_label(
                        Color32::LIGHT_GREEN,
                        if self.game.is_some() {
                            "● 游戏已连接"
                        } else {
                            "○ 等待游戏启动"
                        },
                    );
                });
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(247, 247, 252))
                    .inner_margin(24),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(if self.dispatch_page {
                        "自动派遣"
                    } else {
                        "连点器"
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let on = if self.dispatch_page {
                            self.running
                        } else {
                            self.clicking
                        };
                        if ui
                            .add_sized(
                                [90., 32.],
                                egui::Button::new(if on {
                                    "● 已开启"
                                } else {
                                    "○ 已关闭"
                                })
                                .selected(on),
                            )
                            .clicked()
                        {
                            if self.dispatch_page {
                                self.toggle()
                            } else {
                                self.click_switch()
                            }
                        }
                    });
                });
                ui.add_space(15.);
                ui.separator();
                if self.dispatch_page {
                    ui.label(format!("{}     │     每 30 秒检查", self.status));
                    ui.separator();
                    ui.add_space(10.);
                    ui.columns(3, |cols| {
                        for (col, (k, v)) in cols.iter_mut().zip([
                            ("成功", self.ok.to_string()),
                            ("失败", self.fail.to_string()),
                            ("当前地图", self.map.clone()),
                        ]) {
                            col.label(k);
                            col.heading(v);
                        }
                    });
                    ui.add_space(15.);
                    ui.separator();
                    ui.heading("最近活动");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for s in &self.logs {
                            ui.label(s);
                            ui.separator();
                        }
                    });
                } else {
                    ui.label(&self.click_status);
                    ui.add_enabled_ui(!self.clicking, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("点击间隔（毫秒）");
                            ui.add(egui::DragValue::new(&mut self.interval).range(30..=60000));
                        });
                        ui.horizontal(|ui| {
                            ui.label("鼠标按键");
                            ui.selectable_value(&mut self.right, false, "左键");
                            ui.selectable_value(&mut self.right, true, "右键");
                        });
                        ui.horizontal(|ui| {
                            ui.label("启停热键");
                            for (k, v) in [(0x75, "F6"), (0x76, "F7"), (0x78, "F9")] {
                                ui.selectable_value(&mut self.key, k, v);
                            }
                        });
                    });
                    ui.separator();
                    ui.heading(format!("已点击 {} 次", self.clicks));
                    ui.label("鼠标指向游戏时连点。派遣检查期间暂停连点。");
                }
            });
        ctx.request_repaint_after(Duration::from_millis(if self.clicking { 15 } else { 50 }));
    }
}
fn main() -> eframe::Result {
    eframe::run_native(
        "修仙助手",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([940., 525.])
                .with_min_inner_size([760., 440.]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
