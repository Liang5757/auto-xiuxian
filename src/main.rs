#![windows_subsystem = "windows"]
mod clicker;
mod native;
mod session;
mod tray;
mod ui;
mod vision;
use eframe::egui;
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
    clicker: clicker::Clicker,
    tray: Option<tray::Tray>,
    hidden: bool,
    game: Option<isize>,
    scan: Instant,
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
                fonts
                    .families
                    .entry(family)
                    .or_default()
                    .insert(0, "cjk".into())
            }
        }
        let heading = egui::FontFamily::Name("heading".into());
        let mut heading_fonts = fonts.families[&egui::FontFamily::Proportional].clone();
        if let Ok(bytes) = std::fs::read(format!(
            r"{}\Fonts\msyhbd.ttc",
            std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into())
        )) {
            fonts.font_data.insert(
                "cjk-bold".into(),
                Arc::new(egui::FontData::from_owned(bytes)),
            );
            heading_fonts.insert(0, "cjk-bold".into());
        }
        fonts.families.insert(heading, heading_fonts);
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        cc.egui_ctx.style_mut(|style| {
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(22, 23, 29));
            style.visuals.selection.bg_fill = egui::Color32::from_rgb(121, 75, 234);
            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(17.));
            style
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(16.));
        });
        let (tx, rx) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let lock = Arc::new(Mutex::new(()));
        let clicker = clicker::Clicker::new(lock.clone(), stop.clone());
        let tray_result = tray::Tray::new(cc.egui_ctx.clone());
        let mut app = Self {
            dispatch_page: true,
            running: false,
            clicking: false,
            stop,
            lock,
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
            clicker,
            tray: None,
            hidden: false,
            game: None,
            scan: Instant::now(),
            closing: false,
        };
        match tray_result {
            Ok(tray) => app.tray = Some(tray),
            Err(error) => app.log(error),
        }
        app
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
        self.clicker.enabled.fetch_xor(true, Ordering::Relaxed);
        self.clicking = self.clicker.enabled.load(Ordering::Relaxed);
        *self.clicker.status.lock().unwrap() = if self.clicking {
            "等待鼠标指向游戏"
        } else {
            "已停止"
        }
        .into();
    }
    fn background(&mut self, ctx: &egui::Context) {
        if self.tray.is_some() {
            self.hidden = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
    }
    fn exit(&mut self) {
        self.closing = true;
        self.clicker.stop();
        self.stop.store(true, Ordering::Relaxed);
        self.status = "完成当前操作后退出".into();
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
        self.clicking = self.clicker.enabled.load(Ordering::Relaxed);
        self.clicks = self.clicker.count.load(Ordering::Relaxed);
        self.click_status = self.clicker.status.lock().unwrap().clone();
        *self.clicker.settings.lock().unwrap() = clicker::Settings {
            interval: self.interval,
            right: self.right,
            key: self.key,
        };
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.poll();
        let events: Vec<_> = self
            .tray
            .as_ref()
            .map(|tray| tray.events.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                tray::Event::Show => self.hidden = false,
                tray::Event::Exit => {
                    self.hidden = false;
                    self.exit();
                }
            }
        }
        let close = ctx.input(|i| i.viewport().close_requested());
        if close && !self.closing {
            if self.tray.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.background(ctx);
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.exit();
            }
        }
        if !self.hidden && !self.closing && ctx.input(|i| i.viewport().minimized == Some(true)) {
            self.background(ctx);
        }
        if self.closing && !self.running {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::Q)) {
            self.exit();
        }
        if !self.hidden {
            self.draw(ctx);
        }
        ctx.request_repaint_after(Duration::from_millis(if self.hidden { 250 } else { 50 }));
    }
}
fn main() -> eframe::Result {
    if tray::show_existing() {
        return Ok(());
    }
    eframe::run_native(
        "修仙助手",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800., 540.])
                .with_icon(ui::window_icon())
                .with_min_inner_size([760., 510.]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
