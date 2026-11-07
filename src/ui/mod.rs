use crate::App;
use eframe::egui::{self, Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, pos2, vec2};

const BG: Color32 = Color32::from_rgb(246, 247, 250);
const WHITE: Color32 = Color32::WHITE;
const INK: Color32 = Color32::from_rgb(34, 38, 49);
const MUTED: Color32 = Color32::from_rgb(132, 139, 155);
const LINE: Color32 = Color32::from_rgb(231, 234, 240);
const ACCENT: Color32 = Color32::from_rgb(113, 85, 224);
const TINT: Color32 = Color32::from_rgb(241, 237, 253);
const GREEN: Color32 = Color32::from_rgb(46, 153, 118);

fn text(p: &Painter, at: Pos2, value: &str, size: f32, color: Color32) {
    p.text(
        at,
        Align2::LEFT_CENTER,
        value,
        FontId::proportional(size),
        color,
    );
}
fn heading(p: &Painter, at: Pos2, value: &str, size: f32) {
    p.text(
        at,
        Align2::LEFT_CENTER,
        value,
        FontId::new(size, egui::FontFamily::Name("heading".into())),
        INK,
    );
}
fn card(p: &Painter, rect: Rect) {
    p.rect_filled(
        rect.translate(vec2(0., 2.)),
        12.,
        Color32::from_black_alpha(3),
    );
    p.rect_filled(rect, 12., WHITE);
    p.rect_stroke(rect, 12., Stroke::new(1., LINE), egui::StrokeKind::Inside);
}
fn line(p: &Painter, a: Pos2, b: Pos2) {
    p.line_segment([a, b], Stroke::new(1., LINE));
}
#[derive(Clone, Copy)]
enum Icon {
    Plane,
    Cursor,
    Check,
    Alert,
    Map,
    Clock,
    Power,
}
fn icon(p: &Painter, center: Pos2, kind: Icon, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    let path = |points: &[(f32, f32)]| {
        p.add(egui::Shape::line(
            points.iter().map(|&(x, y)| center + vec2(x, y)).collect(),
            stroke,
        ));
    };
    match kind {
        Icon::Plane => {
            path(&[(-8., -1.), (8., -7.), (3., 8.), (-1., 3.), (-8., -1.)]);
            path(&[(-1., 3.), (8., -7.)]);
        }
        Icon::Cursor => path(&[
            (-5., -8.),
            (7., 1.),
            (1., 2.),
            (4., 8.),
            (1., 10.),
            (-2., 3.),
            (-6., 7.),
            (-5., -8.),
        ]),
        Icon::Check => {
            p.circle_stroke(center, 8., stroke);
            path(&[(-3., 0.), (-1., 3.), (4., -3.)]);
        }
        Icon::Alert => {
            p.circle_stroke(center, 8., stroke);
            path(&[(0., -4.), (0., 1.)]);
            p.circle_filled(center + vec2(0., 4.), 0.8, color);
        }
        Icon::Map => {
            path(&[
                (-8., -6.),
                (-3., -8.),
                (3., -5.),
                (8., -7.),
                (8., 6.),
                (3., 8.),
                (-3., 5.),
                (-8., 7.),
                (-8., -6.),
            ]);
            path(&[(-3., -8.), (-3., 5.)]);
            path(&[(3., -5.), (3., 8.)]);
        }
        Icon::Clock => {
            p.circle_stroke(center, 8., stroke);
            path(&[(0., -4.), (0., 0.), (4., 2.)]);
        }
        Icon::Power => {
            p.add(egui::Shape::line(
                (0..=24)
                    .map(|i| {
                        let angle = 0.75 + i as f32 / 24. * (std::f32::consts::TAU - 1.5);
                        center + vec2(angle.sin() * 8., -angle.cos() * 8.)
                    })
                    .collect(),
                stroke,
            ));
            path(&[(0., -10.), (0., 0.)]);
        }
    }
}
pub(crate) fn window_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../../assets/app.png"))
        .expect("embedded application icon")
        .into_rgba8();
    egui::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    }
}
impl App {
    pub(crate) fn draw(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(BG))
            .show(ctx, |ui| {
                let style = ui.style_mut();
                style
                    .text_styles
                    .insert(egui::TextStyle::Body, FontId::proportional(13.));
                style
                    .text_styles
                    .insert(egui::TextStyle::Button, FontId::proportional(13.));
                style.visuals.override_text_color = Some(INK);
                style.visuals.selection.bg_fill = TINT;
                style.visuals.selection.stroke = Stroke::new(1., ACCENT);
                let area = ui.max_rect();
                let p = ui.painter().clone();
                let top = area.top();
                let side = area.left() + 156.;
                let x = side + 24.;
                let right = area.right() - 24.;
                p.rect_filled(
                    Rect::from_min_max(area.min, pos2(side, area.bottom())),
                    0.,
                    WHITE,
                );
                line(&p, pos2(side, top), pos2(side, area.bottom()));

                let brand = pos2(area.left() + 30., top + 34.);
                p.rect_filled(Rect::from_center_size(brand, vec2(28., 28.)), 8., ACCENT);
                p.add(egui::Shape::convex_polygon(
                    vec![
                        brand + vec2(0., -8.),
                        brand + vec2(6., 0.),
                        brand + vec2(0., 8.),
                        brand + vec2(-6., 0.),
                    ],
                    WHITE,
                    Stroke::NONE,
                ));
                heading(&p, pos2(brand.x + 24., brand.y), "修仙助手", 16.);
                for (page, title, kind, dy) in [
                    (true, "自动派遣", Icon::Plane, 83.),
                    (false, "连点器", Icon::Cursor, 129.),
                ] {
                    let rect =
                        Rect::from_min_size(pos2(area.left() + 12., top + dy), vec2(132., 39.));
                    let selected = self.dispatch_page == page;
                    let response = ui
                        .interact(rect, ui.id().with(title), Sense::click())
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if selected || response.hovered() {
                        p.rect_filled(rect, 7., if selected { TINT } else { BG });
                    }
                    let color = if selected { ACCENT } else { MUTED };
                    icon(&p, pos2(rect.left() + 22., rect.center().y), kind, color);
                    text(
                        &p,
                        pos2(rect.left() + 44., rect.center().y),
                        title,
                        13.5,
                        if selected { ACCENT } else { INK },
                    );
                    if response.clicked() {
                        self.dispatch_page = page;
                    }
                }
                let connection = Rect::from_min_size(
                    pos2(area.left() + 16., area.bottom() - 76.),
                    vec2(124., 58.),
                );
                p.rect_filled(connection, 8., BG);
                let connected = self.game.is_some();
                p.circle_filled(
                    connection.min + vec2(15., 19.),
                    3.,
                    if connected { GREEN } else { MUTED },
                );
                text(
                    &p,
                    connection.min + vec2(26., 19.),
                    if connected {
                        "游戏已连接"
                    } else {
                        "等待游戏启动"
                    },
                    12.,
                    INK,
                );
                text(
                    &p,
                    connection.min + vec2(15., 40.),
                    "懒人修仙传",
                    11.,
                    MUTED,
                );

                heading(
                    &p,
                    pos2(x, top + 33.),
                    if self.dispatch_page {
                        "自动派遣"
                    } else {
                        "连点器"
                    },
                    22.,
                );
                text(
                    &p,
                    pos2(x, top + 60.),
                    if self.dispatch_page {
                        "小队归来后，重新派遣并返回原地图"
                    } else {
                        "为游戏窗口设置连续点击"
                    },
                    12.,
                    MUTED,
                );

                if self.tray.is_some() {
                    let response = ui
                        .put(
                            Rect::from_min_size(pos2(right - 88., top + 20.), vec2(88., 28.)),
                            egui::Button::new(
                                egui::RichText::new("后台运行").size(12.).color(ACCENT),
                            )
                            .fill(TINT)
                            .stroke(Stroke::NONE)
                            .corner_radius(7.),
                        )
                        .on_hover_text("隐藏到系统托盘，继续运行已开启的任务");
                    if response.clicked() {
                        self.background(ctx);
                    }
                }

                let hero = Rect::from_min_max(pos2(x, top + 87.), pos2(right, top + 173.));
                card(&p, hero);
                let on = if self.dispatch_page {
                    self.running
                } else {
                    self.clicking
                };
                let status = if self.dispatch_page {
                    &self.status
                } else {
                    &self.click_status
                };
                let status_color = if on { GREEN } else { ACCENT };
                let badge = hero.min + vec2(35., 43.);
                p.rect_filled(
                    Rect::from_center_size(badge, vec2(38., 38.)),
                    11.,
                    if on {
                        Color32::from_rgb(235, 247, 241)
                    } else {
                        TINT
                    },
                );
                icon(&p, badge, Icon::Power, status_color);
                heading(&p, hero.min + vec2(68., 30.), status, 15.);
                text(
                    &p,
                    hero.min + vec2(68., 55.),
                    if self.dispatch_page {
                        "检查间隔 30 秒"
                    } else {
                        "仅在鼠标指向游戏时点击"
                    },
                    12.,
                    MUTED,
                );
                let switch =
                    Rect::from_min_size(pos2(right - 66., hero.center().y - 12.), vec2(42., 24.));
                let response = ui
                    .interact(switch, ui.id().with("switch"), Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let t = ctx.animate_bool(
                    ui.id().with("switch-animation").with(self.dispatch_page),
                    on,
                );
                p.rect_filled(
                    switch,
                    12.,
                    if on {
                        ACCENT
                    } else {
                        Color32::from_rgb(218, 222, 232)
                    },
                );
                p.circle_filled(
                    pos2(switch.left() + 12. + 18. * t, switch.center().y),
                    9.,
                    WHITE,
                );
                text(
                    &p,
                    pos2(right - 120., hero.center().y),
                    if on { "已开启" } else { "已关闭" },
                    12.,
                    MUTED,
                );
                if response.clicked() && !self.closing {
                    if self.dispatch_page {
                        self.toggle()
                    } else {
                        self.click_switch()
                    }
                }

                if self.dispatch_page {
                    self.dispatch_view(
                        ui,
                        &p,
                        Rect::from_min_max(pos2(x, top + 189.), pos2(right, area.bottom() - 24.)),
                    );
                } else {
                    self.clicker_view(
                        ui,
                        &p,
                        Rect::from_min_max(pos2(x, top + 189.), pos2(right, area.bottom() - 24.)),
                    );
                }
            });
    }
    fn dispatch_view(&self, ui: &mut egui::Ui, p: &Painter, area: Rect) {
        let gap = 12.;
        let w = (area.width() - 2. * gap) / 3.;
        for (i, (label, value, kind, color)) in [
            ("派遣成功", self.ok.to_string(), Icon::Check, GREEN),
            (
                "派遣失败",
                self.fail.to_string(),
                Icon::Alert,
                Color32::from_rgb(207, 120, 109),
            ),
            ("当前地图", self.map.clone(), Icon::Map, ACCENT),
        ]
        .into_iter()
        .enumerate()
        {
            let rect = Rect::from_min_size(area.min + vec2(i as f32 * (w + gap), 0.), vec2(w, 96.));
            card(p, rect);
            text(p, rect.min + vec2(18., 25.), label, 12., MUTED);
            icon(p, pos2(rect.right() - 25., rect.top() + 25.), kind, color);
            heading(
                p,
                rect.min + vec2(18., 64.),
                &value,
                if i == 2 { 19. } else { 28. },
            );
        }
        let activity = Rect::from_min_max(area.min + vec2(0., 112.), area.max);
        card(p, activity);
        heading(p, activity.min + vec2(20., 26.), "运行记录", 14.);
        text(
            p,
            pos2(activity.right() - 106., activity.top() + 26.),
            "Esc 停止派遣",
            11.,
            MUTED,
        );
        line(
            p,
            activity.min + vec2(0., 51.),
            pos2(activity.right(), activity.top() + 51.),
        );
        let content =
            Rect::from_min_max(activity.min + vec2(18., 63.), activity.max - vec2(18., 12.));
        if self.logs.is_empty() {
            let c = content.center();
            p.circle_filled(c - vec2(0., 18.), 22., BG);
            icon(
                p,
                c - vec2(0., 18.),
                Icon::Clock,
                Color32::from_rgb(173, 180, 195),
            );
            let galley = p.layout_no_wrap("暂无运行记录".into(), FontId::proportional(12.), MUTED);
            p.galley(pos2(c.x - galley.size().x / 2., c.y + 16.), galley, MUTED);
        } else {
            ui.scope_builder(egui::UiBuilder::new().max_rect(content), |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("activity")
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        for (i, entry) in self.logs.iter().enumerate() {
                            if i > 0 {
                                ui.add_space(8.);
                                ui.separator();
                                ui.add_space(8.);
                            }
                            let (time, message) = entry.split_once("     ").unwrap_or(("", entry));
                            ui.horizontal_top(|ui| {
                                ui.add_sized(
                                    [78., 20.],
                                    egui::Label::new(
                                        egui::RichText::new(time).size(12.).color(MUTED),
                                    ),
                                );
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(message).size(13.).color(INK),
                                    )
                                    .wrap(),
                                );
                            });
                        }
                    });
            });
        }
    }
    fn clicker_view(&mut self, ui: &mut egui::Ui, p: &Painter, area: Rect) {
        card(p, area);
        heading(p, area.min + vec2(22., 26.), "点击设置", 14.);
        let count = format!("已点击 {} 次", self.clicks);
        p.text(
            pos2(area.right() - 22., area.top() + 26.),
            Align2::RIGHT_CENTER,
            count,
            FontId::proportional(12.),
            MUTED,
        );
        line(
            p,
            area.min + vec2(0., 51.),
            pos2(area.right(), area.top() + 51.),
        );
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(area.shrink2(vec2(22., 12.))),
            |ui| {
                ui.add_space(56.);
                ui.add_enabled_ui(!self.clicking, |ui| {
                    ui.spacing_mut().button_padding = vec2(14., 7.);
                    egui::Grid::new("clicker-settings")
                        .spacing([48., 25.])
                        .show(ui, |ui| {
                            ui.label("点击间隔");
                            ui.add(
                                egui::DragValue::new(&mut self.interval)
                                    .range(30..=60000)
                                    .suffix(" 毫秒"),
                            );
                            ui.end_row();
                            ui.label("鼠标按键");
                            ui.horizontal(|ui| {
                                ui.selectable_value(&mut self.right, false, "左键");
                                ui.selectable_value(&mut self.right, true, "右键");
                            });
                            ui.end_row();
                            ui.label("启停热键");
                            ui.horizontal(|ui| {
                                for (key, title) in [(0x75, "F6"), (0x76, "F7"), (0x78, "F9")] {
                                    ui.selectable_value(&mut self.key, key, title);
                                }
                            });
                            ui.end_row();
                        });
                });
            },
        );
        text(
            p,
            pos2(area.left() + 22., area.bottom() - 26.),
            "自动派遣操作期间，连点会暂时暂停。",
            12.,
            MUTED,
        );
    }
}
