use eframe::egui::{self, Color32, Stroke, Vec2, Frame, Margin, CornerRadius};

const TEAL: Color32 = Color32::from_rgb(0, 188, 162);
const TEAL_DIM: Color32 = Color32::from_rgba_premultiplied(0, 188, 162, 38);
const BG_DEEP: Color32 = Color32::from_rgb(26, 26, 26);
const BG_EDITOR: Color32 = Color32::from_rgb(30, 30, 30);
const BG_PANEL: Color32 = Color32::from_rgb(37, 37, 38);
const BG_HOVER: Color32 = Color32::from_rgb(45, 45, 45);
const BG_ACTIVE: Color32 = Color32::from_rgb(55, 55, 61);
const BG_RAISED: Color32 = Color32::from_rgb(51, 51, 51);
const BORDER: Color32 = Color32::from_rgb(60, 60, 60);
const TEXT: Color32 = Color32::from_rgb(204, 204, 204);
const TEXT_SEC: Color32 = Color32::from_rgb(150, 150, 150);
const TEXT_HEADING: Color32 = Color32::from_rgb(255, 255, 255);

const CR6: CornerRadius = CornerRadius::same(6);
const CR8: CornerRadius = CornerRadius::same(8);
const CR12: CornerRadius = CornerRadius::same(12);

#[derive(PartialEq)]
enum Screen { Installer, Welcome, Chrome, Sidebar, Palette, About }

struct CodApp { screen: Screen }

impl Default for CodApp { fn default() -> Self { Self { screen: Screen::Welcome } } }

impl eframe::App for CodApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        style_ctx(ctx);

        egui::TopBottomPanel::top("tabs")
            .frame(Frame { fill: BG_PANEL, ..Default::default() })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    for (label, screen) in [
                        ("Installer", Screen::Installer), ("Welcome", Screen::Welcome),
                        ("Title Bar", Screen::Chrome), ("Sidebar", Screen::Sidebar),
                        ("Palette", Screen::Palette), ("About", Screen::About),
                    ] {
                        if ui.selectable_label(self.screen == screen, label).clicked() {
                            self.screen = screen;
                        }
                    }
                });
            });

        match self.screen {
            Screen::Installer => show_installer(ctx),
            Screen::Welcome => show_welcome(ctx),
            Screen::Chrome => show_chrome(ctx),
            Screen::Sidebar => show_sidebar(ctx),
            Screen::Palette => show_palette(ctx),
            Screen::About => show_about(ctx),
        }
    }
}

fn style_ctx(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.dark_mode = true;
    style.visuals.panel_fill = BG_PANEL;
    style.visuals.window_fill = BG_PANEL;
    style.visuals.hyperlink_color = TEAL;
    style.visuals.selection.bg_fill = TEAL_DIM;
    style.visuals.selection.stroke = Stroke::new(1.0, TEAL);
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.spacing.button_padding = Vec2::new(12.0, 6.0);
    ctx.set_style(style);
}

fn card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    Frame { fill: BG_PANEL, corner_radius: CR8, stroke: Stroke::new(1.0, BORDER), ..Default::default() }
        .show(ui, add_contents);
}

fn logo(ui: &mut egui::Ui, size: f32, font_size: f32, r: CornerRadius) {
    Frame { fill: TEAL, corner_radius: r, ..Default::default() }.show(ui, |ui| {
        ui.set_min_size(Vec2::new(size, size));
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("COD").size(font_size).strong().color(BG_DEEP));
        });
    });
}

fn tb(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).size(13.0).color(Color32::WHITE)).fill(TEAL).rounding(CR6)
}
fn sb(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).size(13.0).color(TEXT)).fill(BG_RAISED).stroke(Stroke::new(1.0, BORDER)).rounding(CR6)
}
fn ob(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).size(13.0).color(TEXT)).fill(Color32::TRANSPARENT).stroke(Stroke::new(1.0, BORDER)).rounding(CR6)
}

// ═══  INSTALLER  ═══
fn show_installer(ctx: &egui::Context) {
    egui::CentralPanel::default().frame(Frame { fill: BG_DEEP, ..Default::default() }).show(ctx, |ui| {
        ui.allocate_space(Vec2::new(0.0, 40.0));
        let a = ui.available_size();
        egui::Area::new(egui::Id::new("installer")).fixed_pos(egui::pos2((a.x - 560.0).max(0.0) * 0.5, 40.0)).show(ctx, |ui| {
            ui.set_min_width(560.0);
            card(ui, |ui| {
                Frame { fill: Color32::from_rgb(26, 26, 46), corner_radius: CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 }, ..Default::default() }.show(ui, |ui| {
                    ui.horizontal(|ui| { ui.add_space(28.0); logo(ui, 48.0, 22.0, CR12); ui.add_space(16.0);
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("COD Editor Setup").size(18.0).strong().color(TEXT_HEADING));
                            ui.label(egui::RichText::new("Version 1.127.0 — Developer Preview").size(12.0).color(TEXT_SEC));
                        });
                    }); ui.add_space(20.0);
                });
                ui.add_space(16.0);
                Frame { fill: BG_DEEP, corner_radius: CornerRadius::same(4), ..Default::default() }.show(ui, |ui| {
                    ui.allocate_ui_with_layout(Vec2::new(504.0, 6.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        Frame { fill: TEAL, corner_radius: CornerRadius::same(4), ..Default::default() }.show(ui, |ui| { ui.set_min_size(Vec2::new(504.0 * 0.35, 6.0)); });
                    });
                });
                ui.add_space(16.0);
                ui.label(egui::RichText::new("INSTALL TYPE").size(11.0).color(TEXT_SEC).strong());
                ui.horizontal(|ui| { let mut typical = true; ui.radio_value(&mut typical, true, "Typical (recommended)"); ui.radio_value(&mut typical, false, "Custom"); });
                ui.add_space(8.0);
                ui.label(egui::RichText::new("INSTALL LOCATION").size(11.0).color(TEXT_SEC).strong());
                ui.add(egui::TextEdit::singleline(&mut "C:\\Program Files\\COD Editor".to_owned()).font(egui::TextStyle::Monospace).text_color(TEXT).background_color(BG_DEEP).desired_width(504.0));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("LICENSE AGREEMENT").size(11.0).color(TEXT_SEC).strong());
                Frame { fill: BG_DEEP, corner_radius: CR6, stroke: Stroke::new(1.0, BORDER), ..Default::default() }.show(ui, |ui| {
                    egui::ScrollArea::vertical().id_salt("license").max_height(60.0).show(ui, |ui| {
                        ui.label(egui::RichText::new("COD Editor License — MIT License. Copyright (c) COD Contributors…").size(11.0).color(TEXT_SEC));
                    });
                });
                let mut accept = true;
                let _ = ui.checkbox(&mut accept, "I accept the terms in the License Agreement");
                ui.add_space(4.0);
                let mut ctx_menu = true;
                let mut add_path = true;
                let _ = ui.checkbox(&mut ctx_menu, "Add \"Open with COD\" to context menu");
                let _ = ui.checkbox(&mut add_path, "Add COD to PATH");
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(tb("Install")).clicked() {} ui.add_space(8.0); let _ = ui.add(sb("Back")); let _ = ui.add(sb("Cancel"));
                    });
                }); ui.add_space(8.0);
            });
        });
    });
}

// ═══  WELCOME  ═══
fn show_welcome(ctx: &egui::Context) {
    egui::CentralPanel::default().frame(Frame { fill: BG_EDITOR, ..Default::default() }).show(ctx, |ui| {
        let a = ui.available_size();
        egui::Area::new(egui::Id::new("welcome")).fixed_pos(egui::pos2((a.x - 640.0).max(0.0) * 0.5, 40.0)).show(ctx, |ui| {
            ui.set_min_width(640.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| { logo(ui, 52.0, 24.0, CR12); ui.add_space(14.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Welcome to COD").size(26.0).strong().color(TEXT_HEADING));
                        ui.label(egui::RichText::new("Your streamlined code editor — fast, focused, yours.").size(13.0).color(TEXT_SEC));
                    });
                });
                ui.add_space(24.0);
                ui.horizontal(|ui| { if ui.add(tb("+ New File")).clicked() {} ui.add_space(8.0); let _ = ui.add(ob("Open Folder…")); let _ = ui.add(ob("Clone Git Repository…")); });
                ui.add_space(24.0);
                ui.label(egui::RichText::new("RECENT").size(12.0).color(TEXT_SEC).strong());
                ui.add_space(8.0);
                for (name, p) in [("cod", "C:\\Users\\nanda\\Desktop\\Github\\COD"), ("my-project", "C:\\Users\\nanda\\Projects\\my-project"), ("docs-site", "C:\\Users\\nanda\\Sites\\docs-site")] {
                    egui::Frame::none().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📁").size(12.0).color(TEAL)); ui.add_space(8.0);
                            ui.label(egui::RichText::new(name).size(13.0).color(TEXT));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(p).size(11.0).color(TEXT_SEC).family(egui::FontFamily::Monospace));
                            });
                        }); ui.add_space(2.0);
                    });
                }
                ui.add_space(24.0);
                egui::Grid::new("cards").min_col_width(300.0).max_col_width(300.0).spacing(Vec2::new(16.0, 12.0)).show(ui, |ui| {
                    for (icon, title, desc) in [
                        ("⌨️", "Keyboard Shortcuts", "Learn the shortcuts that speed up your workflow"),
                        ("🎨", "Color Theme", "Customize COD's appearance"),
                        ("🧩", "Extensions", "Browse extensions for your tools"),
                        ("⚙️", "Settings", "Configure editor and language settings"),
                    ] {
                        Frame { fill: BG_PANEL, corner_radius: CR8, stroke: Stroke::new(1.0, BORDER), ..Default::default() }.show(ui, |ui| {
                            ui.set_min_size(Vec2::new(280.0, 70.0)); ui.add_space(12.0);
                            ui.horizontal(|ui| { ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(icon).size(18.0)); ui.add_space(4.0);
                                    ui.label(egui::RichText::new(title).size(14.0).strong().color(TEXT_HEADING));
                                    ui.label(egui::RichText::new(desc).size(11.0).color(TEXT_SEC));
                                });
                            });
                        });
                        ui.end_row();
                    }
                });
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("COD v1.127.0").size(11.0).color(TEXT_SEC).family(egui::FontFamily::Monospace));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Release Notes · Privacy · Help").size(11.0).color(TEXT_SEC));
                    });
                });
            });
        });
    });
}

// ═══  WINDOW CHROME  ═══
fn show_chrome(ctx: &egui::Context) {
    egui::CentralPanel::default().frame(Frame { fill: BG_DEEP, ..Default::default() }).show(ctx, |ui| {
        ui.allocate_space(Vec2::new(0.0, 16.0));
        let a = ui.available_size(); let cw = 720.0f32.min(a.x - 32.0);
        egui::Area::new(egui::Id::new("chrome")).fixed_pos(egui::pos2((a.x - cw) * 0.5, 16.0)).show(ctx, |ui| {
            ui.set_min_width(cw);
            for (label, active) in [("— Active window —", true), ("— Inactive window —", false)] {
                ui.label(egui::RichText::new(label).size(12.0).color(TEXT_SEC));
                ui.add_space(4.0);
                let alpha = if active { 1.0 } else { 0.55 };
                Frame { fill: BG_PANEL, corner_radius: CR8, stroke: Stroke::new(1.0, BORDER), ..Default::default() }.show(ui, |ui| {
                    ui.set_opacity(alpha);
                    Frame { fill: Color32::from_rgb(if active { 45 } else { 37 }, if active { 45 } else { 37 }, if active { 45 } else { 37 }), corner_radius: CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 }, ..Default::default() }.show(ui, |ui| {
                        ui.horizontal(|ui| { ui.set_min_height(32.0); ui.add_space(12.0);
                            logo(ui, 14.0, 8.0, CornerRadius::same(4)); ui.add_space(8.0);
                            ui.label(egui::RichText::new(if active { "COD · cod (Development) — COD Editor" } else { "COD · cod" }).size(12.0).color(if active { TEXT } else { TEXT_SEC }));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                for ch in &["─", "□", "✕"] {
                                    ui.add(egui::Button::new(*ch).min_size(Vec2::new(46.0, 32.0)).fill(Color32::TRANSPARENT));
                                }
                            });
                        });
                    });
                    ui.horizontal(|ui| { ui.set_min_height(28.0); ui.add_space(8.0);
                        for item in &["File", "Edit", "Selection", "View", "Go", "Run", "Terminal", "Help"] {
                            ui.label(egui::RichText::new(*item).size(12.0).color(TEXT_SEC));
                        }
                    });
                    Frame { fill: BG_EDITOR, corner_radius: CornerRadius { nw: 0, ne: 0, sw: 8, se: 8 }, ..Default::default() }.show(ui, |ui| {
                        ui.set_min_size(Vec2::new(cw, 100.0));
                        ui.vertical_centered(|ui| { ui.label(egui::RichText::new(if active { "Editor — active" } else { "Editor — inactive" }).size(13.0).color(TEXT_SEC)); });
                    });
                });
                ui.add_space(16.0);
            }
        });
    });
}

// ═══  SIDEBAR  ═══
fn show_sidebar(ctx: &egui::Context) {
    egui::CentralPanel::default().frame(Frame { fill: BG_DEEP, ..Default::default() }).show(ctx, |ui| {
        ui.allocate_space(Vec2::new(0.0, 16.0));
        let a = ui.available_size(); let cw = 720.0f32.min(a.x - 32.0); let h = 420.0f32.min(a.y - 32.0);
        egui::Area::new(egui::Id::new("sidebar")).fixed_pos(egui::pos2((a.x - cw) * 0.5, 16.0)).show(ctx, |ui| {
            ui.set_min_size(Vec2::new(cw, h));
            Frame { fill: BG_PANEL, corner_radius: CR8, stroke: Stroke::new(1.0, BORDER), ..Default::default() }.show(ui, |ui| {
                ui.horizontal(|ui| {
                    Frame { fill: Color32::from_rgb(51, 51, 51), corner_radius: CornerRadius { nw: 8, sw: 8, ne: 0, se: 0 }, ..Default::default() }.show(ui, |ui| {
                        ui.set_min_size(Vec2::new(48.0, h));
                        ui.vertical(|ui| {
                            for (i, icon) in ["📂", "🔍", "⎇", "🐛", "🧩"].iter().enumerate() {
                                Frame { fill: if i == 0 { BG_ACTIVE } else { Color32::TRANSPARENT }, ..Default::default() }.show(ui, |ui| {
                                    ui.set_min_size(Vec2::new(48.0, 44.0));
                                    ui.vertical_centered(|ui| { ui.label(egui::RichText::new(*icon).size(16.0).color(if i == 0 { TEXT } else { TEXT_SEC })); });
                                });
                            }
                            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| { ui.set_min_height(h - 5.0 * 44.0); ui.label(egui::RichText::new("⚙️").size(16.0).color(TEXT_SEC)); });
                        });
                    });
                    Frame { fill: BG_PANEL, ..Default::default() }.show(ui, |ui| {
                        ui.set_min_size(Vec2::new(220.0, h));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| { ui.set_min_height(32.0); ui.add_space(12.0);
                                ui.label(egui::RichText::new("EXPLORER").size(11.0).color(TEXT_SEC).strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(egui::RichText::new("−").size(14.0).color(TEXT_SEC)); });
                            });
                            egui::ScrollArea::vertical().id_salt("tree").show(ui, |ui| {
                                for (indent, chevron, icon, name) in &[
                                    (0, "▼", "📁", "COD"), (1, "▸", "📁", "src"), (2, "", "📄", "main.ts"),
                                    (2, "", "📄", "app.ts"), (1, "▸", "📁", "styles"), (1, "▸", "📁", "components"),
                                    (0, "", "📄", "package.json"), (0, "", "📄", "tsconfig.json"),
                                ] {
                                    ui.horizontal(|ui| { ui.add_space(8.0 + 16.0 * *indent as f32);
                                        ui.label(egui::RichText::new(*chevron).size(10.0).color(TEXT_SEC)); ui.add_space(4.0);
                                        ui.label(egui::RichText::new(*icon).size(12.0).color(TEXT_SEC)); ui.add_space(4.0);
                                        ui.label(egui::RichText::new(*name).size(12.0).color(TEXT));
                                    }); ui.add_space(2.0);
                                }
                            });
                        });
                    });
                    Frame { fill: BG_EDITOR, corner_radius: CornerRadius { nw: 0, ne: 8, sw: 0, se: 8 }, ..Default::default() }.show(ui, |ui| {
                        ui.set_min_size(Vec2::new(cw - 48.0 - 220.0, h));
                        ui.vertical_centered(|ui| { ui.label(egui::RichText::new("Open Editors").size(13.0).color(TEXT_SEC)); });
                    });
                });
            });
        });
    });
}

// ═══  COMMAND PALETTE  ═══
fn show_palette(ctx: &egui::Context) {
    egui::CentralPanel::default().frame(Frame { fill: BG_DEEP, ..Default::default() }).show(ctx, |ui| {
        let a = ui.available_size();
        egui::Area::new(egui::Id::new("pal-overlay")).fixed_pos(egui::pos2(0.0, 0.0)).show(ctx, |ui| {
            ui.set_min_size(a); ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), a), CornerRadius::ZERO, Color32::from_black_alpha(80));
        });
        egui::Area::new(egui::Id::new("palette")).fixed_pos(egui::pos2((a.x - 480.0) * 0.5, a.y * 0.12)).show(ctx, |ui| {
            ui.set_min_width(480.0);
            Frame { fill: Color32::from_rgba_premultiplied(37, 37, 38, 247), corner_radius: CR12, stroke: Stroke::new(1.0, BORDER), ..Default::default() }.show(ui, |ui| {
                ui.horizontal(|ui| { ui.add_space(16.0);
                    ui.label(egui::RichText::new(">").size(14.0).color(TEAL).strong()); ui.add_space(8.0);
                    ui.add(egui::TextEdit::singleline(&mut "theme".to_owned()).text_color(TEXT).background_color(Color32::TRANSPARENT).desired_width(400.0).hint_text("Type a command or file name…"));
                });
                ui.separator();
                let items: [(&str, &str, Option<&str>); 6] = [
                    ("🎨", "Preferences: Color Theme", Some("Ctrl+K Ctrl+T")),
                    ("🎨", "Preferences: File Icon Theme", Some("Ctrl+K Ctrl+I")),
                    ("🎨", "Preferences: Product Icon Theme", None),
                    ("📦", "Extensions: Install Extensions", Some("Ctrl+Shift+X")),
                    ("⚙️", "Preferences: Open Settings (UI)", Some("Ctrl+,")),
                    ("⌨️", "Preferences: Keyboard Shortcuts", Some("Ctrl+K Ctrl+S")),
                ];
                for (i, (icon, label, shortcut)) in items.iter().enumerate() {
                    Frame { fill: if i == 0 { TEAL_DIM } else { Color32::TRANSPARENT }, corner_radius: CornerRadius::same(4), ..Default::default() }.show(ui, |ui| {
                        ui.horizontal(|ui| { ui.add_space(12.0);
                            ui.label(egui::RichText::new(*icon).size(14.0).color(TEXT_SEC)); ui.add_space(10.0);
                            ui.label(egui::RichText::new(*label).size(13.0).color(if i == 0 { TEXT_HEADING } else { TEXT }));
                            if let Some(kb) = shortcut {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    Frame { fill: BG_RAISED, corner_radius: CornerRadius::same(4), ..Default::default() }.show(ui, |ui| {
                                        ui.label(egui::RichText::new(*kb).size(11.0).color(TEXT_SEC).family(egui::FontFamily::Monospace));
                                    });
                                });
                            }
                        });
                    }); ui.add_space(2.0);
                }
                ui.add_space(4.0);
            });
        });
    });
}

// ═══  ABOUT  ═══
fn show_about(ctx: &egui::Context) {
    egui::CentralPanel::default().frame(Frame { fill: BG_DEEP, ..Default::default() }).show(ctx, |ui| {
        let a = ui.available_size();
        egui::Area::new(egui::Id::new("abt-overlay")).fixed_pos(egui::pos2(0.0, 0.0)).show(ctx, |ui| {
            ui.set_min_size(a); ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), a), CornerRadius::ZERO, Color32::from_black_alpha(80));
        });
        egui::Area::new(egui::Id::new("about")).fixed_pos(egui::pos2((a.x - 340.0) * 0.5, (a.y - 360.0) * 0.5)).show(ctx, |ui| {
            ui.set_min_width(340.0);
            Frame { fill: BG_PANEL, corner_radius: CR8, stroke: Stroke::new(1.0, BORDER), inner_margin: Margin::symmetric(32, 32), ..Default::default() }.show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    logo(ui, 72.0, 34.0, CornerRadius::same(18));
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("COD Editor").size(20.0).strong().color(TEXT_HEADING));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Version 1.127.0").size(12.0).color(TEXT_SEC).family(egui::FontFamily::Monospace));
                    ui.label(egui::RichText::new("Build 2026-06-28 · Commit aee423cf724").size(11.0).color(TEXT_SEC));
                    ui.add_space(16.0); ui.separator(); ui.add_space(8.0);
                    ui.label(egui::RichText::new("© COD Contributors. All rights reserved.\nBuilt on Visual Studio Code — MIT License.").size(12.0).color(TEXT_SEC));
                    ui.add_space(20.0);
                    if ui.add(tb("Check for Updates").min_size(Vec2::new(276.0, 0.0))).clicked() {}
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("View License").size(12.0).color(TEAL));
                    ui.label(egui::RichText::new("COD on GitHub").size(12.0).color(TEAL));
                });
            });
        });
    });
}

fn main() -> eframe::Result {
    eframe::run_native("COD UI", eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 680.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("COD Editor — UI Preview"),
        ..Default::default()
    }, Box::new(|_cc| Ok(Box::new(CodApp::default()))))
}
