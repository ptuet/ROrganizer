//! Settings modal (gear button in the header), in three tabs: options,
//! multi-account commands (leader, NPC follow, keys of the game), about.
//! Every option has a "?" that unfolds its explanation under it.
//!
//! Layout follows the same backdrop + centred card pattern as
//! `capture::draw`. The card is theme-aware (dark/light palettes from
//! `crate::theme`) and stays inside the main viewport — no second OS
//! window, no second tray icon, nothing to clean up on close.

use std::path::Path;

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, Margin, Sense, Stroke, StrokeKind, Vec2};
use rust_i18n::t;

use crate::app::App;
use crate::game_keys::GameKeyTarget;
use crate::theme::{self, Theme};

/// The fork's repository: GitHub link and bug reports go there. The original
/// author stays credited (`about.credit`) and is linked from the README.
const REPO_LABEL: &str = "ptuet/ROrganizer";
const GITHUB_URL: &str = "https://github.com/ptuet/ROrganizer";
const ISSUES_URL: &str = "https://github.com/ptuet/ROrganizer/issues";
const LICENSE_LABEL: &str = "MIT OR Apache-2.0";
/// Card height; `App::apply_dynamic_height` keeps the viewport at least this
/// tall while the modal is open, so the close button is never clipped.
pub const ABOUT_CARD_HEIGHT: f32 = 460.0;

pub fn draw(ctx: &egui::Context, app: &mut App) {
    if !app.is_about_open() {
        return;
    }
    let theme = app.theme();

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.close_about();
        return;
    }

    let screen_rect = ctx.content_rect();
    let backdrop = match theme {
        Theme::Dark => Color32::from_rgba_unmultiplied(0x10, 0x10, 0x0E, 0xCC),
        Theme::Light => Color32::from_rgba_unmultiplied(0xFA, 0xFA, 0xF7, 0xCC),
    };
    let card_bg = match theme {
        Theme::Dark => theme::BG_ROW_DARK,
        Theme::Light => theme::BG_ROW_LIGHT,
    };
    let card_border = match theme {
        Theme::Dark => theme::BORDER_STRONG_DARK,
        Theme::Light => theme::BORDER_STRONG_LIGHT,
    };

    // Width fits inside the 340-wide viewport with a 10 px breathing room
    // on each side. Height is sized to comfortably hold every section.
    let card_size = egui::vec2(320.0, ABOUT_CARD_HEIGHT);
    let card_rect = egui::Rect::from_center_size(screen_rect.center(), card_size);

    let close_clicked = std::cell::Cell::new(false);
    let mut config_dir_clicked = false;
    let mut github_clicked = false;
    let mut bug_clicked = false;
    let mut capture_clicked: Option<GameKeyTarget> = None;

    egui::Area::new(egui::Id::new("about_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen_rect.left_top())
        .show(ctx, |ui| {
            // Click on the backdrop (outside the card) closes the modal.
            // Done before painting the card so the card's own clicks win
            // when they overlap.
            let backdrop_resp = ui.interact(
                screen_rect,
                egui::Id::new("about_backdrop"),
                Sense::click(),
            );
            ui.painter()
                .rect_filled(screen_rect, CornerRadius::ZERO, backdrop);
            if backdrop_resp.clicked()
                && backdrop_resp
                    .interact_pointer_pos()
                    .map(|p| !card_rect.contains(p))
                    .unwrap_or(false)
            {
                close_clicked.set(true);
            }

            ui.scope_builder(egui::UiBuilder::new().max_rect(card_rect), |ui| {
                Frame::new()
                    .fill(card_bg)
                    .corner_radius(CornerRadius::same(10))
                    .stroke(Stroke::new(1.0, card_border))
                    .inner_margin(Margin::symmetric(18, 16))
                    .show(ui, |ui| {
                        ui.set_min_size(card_size - egui::vec2(36.0, 32.0));
                        let mut tab = app.settings_tab();
                        tab_bar(ui, theme, &mut tab);
                        app.set_settings_tab(tab);
                        ui.add_space(12.0);
                        match tab {
                            SettingsTab::Options => options_tab(ui, ctx, app, theme),
                            SettingsTab::Group => {
                                capture_clicked = group_tab(ui, app, theme);
                            }
                            SettingsTab::About => {
                                ui.vertical_centered(|ui| {
                                    // App icon — pre-decoded into a TextureHandle in
                                    // App::new because egui_extras isn't built with
                                    // the "image" feature, so PNGs from `bytes://`
                                    // would render as the red error placeholder.
                                    if let Some(handle) = app.about_icon() {
                                        ui.add(
                                            egui::Image::new(handle)
                                                .fit_to_exact_size(egui::vec2(56.0, 56.0)),
                                        );
                                    }
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new("ROrganizer")
                                            .size(16.0)
                                            .strong()
                                            .color(theme::text_primary(theme)),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "v{}",
                                            env!("CARGO_PKG_VERSION")
                                        ))
                                        .size(11.0)
                                        .color(theme::text_tertiary(theme)),
                                    );
                                    // Takes the place of the author's website link
                                    // (one text line, same height): his site hands
                                    // out his own build, without this fork's changes.
                                    ui.add_space(6.0);
                                    ui.label(
                                        egui::RichText::new(t!("about.fork"))
                                            .size(11.0)
                                            .color(match theme {
                                                Theme::Dark => theme::ACTIVE_DARK,
                                                Theme::Light => theme::ACTIVE_LIGHT,
                                            }),
                                    );
                                    ui.add_space(10.0);
                                    ui.label(
                                        egui::RichText::new(t!("about.credit"))
                                            .size(11.0)
                                            .color(theme::text_secondary(theme)),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(t!("about.ankama_credit"))
                                            .size(10.0)
                                            .color(theme::text_tertiary(theme)),
                                    );
                                });

                                ui.add_space(14.0);
                                if github_link(ui, theme).clicked() {
                                    github_clicked = true;
                                }
                                ui.add_space(8.0);
                                if action_row(
                                    ui,
                                    theme,
                                    &t!("about.report_bug"),
                                    BUG_GLYPH,
                                )
                                .clicked()
                                {
                                    bug_clicked = true;
                                }
                                ui.add_space(8.0);
                                if action_row(
                                    ui,
                                    theme,
                                    &t!("about.open_config_dir"),
                                    FOLDER_GLYPH,
                                )
                                .clicked()
                                {
                                    config_dir_clicked = true;
                                }
                                ui.add_space(8.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new(LICENSE_LABEL)
                                            .size(10.0)
                                            .color(theme::text_tertiary(theme)),
                                    );
                                });
                            }
                        }
                        // Close button at the same place on every tab.
                        let free = ui.available_height() - FOOTER_H;
                        if free > 0.0 {
                            ui.add_space(free);
                        }
                        ui.vertical_centered(|ui| {
                            if close_button(ui, theme).clicked() {
                                close_clicked.set(true);
                            }
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(t!("about.esc_hint"))
                                    .size(9.0)
                                    .color(theme::text_tertiary(theme)),
                            );
                        });
                    });
            });
        });

    // Apply the side-effects after the borrow on `app` from `is_about_open`
    // has been released by going out of scope above.
    if github_clicked {
        let _ = webbrowser::open(GITHUB_URL);
    }
    if bug_clicked {
        let _ = webbrowser::open(ISSUES_URL);
    }
    if config_dir_clicked
        && let Some(dir) = app.config_dir()
    {
        open_folder(dir);
    }
    if let Some(target) = capture_clicked {
        app.start_game_key_capture(target);
    }
    if close_clicked.get() {
        app.close_about();
    }
}

/// Height kept free under the tabs for the close button and its hint.
const FOOTER_H: f32 = 48.0;

/// Window behaviour options.
fn options_tab(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut App, theme: Theme) {
    let mut to_taskbar = app.minimize_to_taskbar();
    option_row(ui, theme, "taskbar", &t!("help.minimize_to_taskbar"), |ui| {
        if checkbox(ui, theme, &mut to_taskbar, &t!("about.minimize_to_taskbar")) {
            app.set_minimize_to_taskbar(to_taskbar);
        }
    });
    let mut dofus_only = app.shortcuts_only_in_dofus();
    option_row(ui, theme, "dofus_only", &t!("help.shortcuts_only_in_dofus"), |ui| {
        if checkbox(ui, theme, &mut dofus_only, &t!("about.shortcuts_only_in_dofus")) {
            app.set_shortcuts_only_in_dofus(dofus_only);
        }
    });
    let mut on_top = app.always_on_top();
    option_row(ui, theme, "on_top", &t!("help.always_on_top"), |ui| {
        if checkbox(ui, theme, &mut on_top, &t!("about.always_on_top")) {
            app.set_always_on_top(ctx, on_top);
        }
    });
    // Pointer to the other tab: its options are about the game, not the
    // window, and are easy to miss.
    ui.add_space(8.0);
    ui.add(
        egui::Label::new(
            egui::RichText::new(t!("settings.more_in_group"))
                .size(11.0)
                .color(theme::text_tertiary(theme)),
        )
        .wrap(),
    );
}

/// Group commands: leader, NPC follow, keys of the game. Returns the
/// game key to capture when one of its rows was clicked.
fn group_tab(ui: &mut egui::Ui, app: &mut App, theme: Theme) -> Option<GameKeyTarget> {
    let mut capture = None;
    let height = (ui.available_height() - FOOTER_H).max(60.0);
    egui::ScrollArea::vertical()
        .max_height(height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            // Group leader.
            option_row(ui, theme, "leader", &t!("help.group_leader"), |ui| {
                section_label(ui, theme, &t!("group.leader"));
            });
            leader_combo(ui, app);
            ui.add_space(10.0);

            // NPC travel: follow the leader, and the pause before it.
            let mut follow = app.npc_follow_leader();
            option_row(ui, theme, "follow", &t!("about.npc_follow_leader_hint"), |ui| {
                if checkbox(ui, theme, &mut follow, &t!("about.npc_follow_leader")) {
                    app.set_npc_follow_leader(follow);
                }
            });
            let mut secs = app.npc_follow_delay_ms() as f32 / 1000.0;
            option_row(ui, theme, "pause", &t!("about.npc_follow_delay_hint"), |ui| {
                let min = crate::npcs::FOLLOW_DELAY_MIN_MS as f32 / 1000.0;
                let max = crate::npcs::FOLLOW_DELAY_MAX_MS as f32 / 1000.0;
                let slider = egui::Slider::new(&mut secs, min..=max)
                    .step_by(0.1)
                    .fixed_decimals(1)
                    .suffix(" s")
                    .text(t!("about.npc_follow_delay").to_string());
                if ui.add_enabled(follow, slider).changed() {
                    app.set_npc_follow_delay_ms((secs * 1000.0).round() as u32);
                }
            });
            ui.add_space(10.0);

            // Keys of the game.
            option_row(ui, theme, "keys", &t!("game_keys.intro"), |ui| {
                section_label(ui, theme, &t!("group.keys"));
            });
            let keys = app.game_keys();
            for (target, label) in [
                (GameKeyTarget::OpenChat, t!("game_keys.open_chat")),
                (GameKeyTarget::FollowLeader, t!("game_keys.follow_leader")),
            ] {
                if key_row(ui, theme, &label, &keys.get(target).label()).clicked() {
                    capture = Some(target);
                }
                ui.add_space(6.0);
            }
        });
    capture
}

/// "Chef de groupe" dropdown: first of the list, or a launched account. A
/// saved leader that is not launched stays listed, marked offline.
fn leader_combo(ui: &mut egui::Ui, app: &mut App) {
    let first = t!("group.leader_first").to_string();
    let current = app.group_leader().map(str::to_string);
    let mut names = app.live_names();
    let offline = current.as_ref().filter(|c| !names.contains(c)).cloned();
    if let Some(c) = &offline {
        names.push(c.clone());
    }
    let shown = |name: &String| {
        if Some(name) == offline.as_ref() {
            t!("group.leader_offline", name = name.as_str()).to_string()
        } else {
            name.clone()
        }
    };
    let mut choice = current.clone();
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        egui::ComboBox::from_id_salt("group_leader")
            .width(ui.available_width())
            .selected_text(choice.as_ref().map_or(first.clone(), &shown))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut choice, None, first.as_str());
                for name in &names {
                    ui.selectable_value(&mut choice, Some(name.clone()), shown(name));
                }
            });
    });
    if choice != current {
        app.set_group_leader(choice);
    }
}

/// An option with its "?" button on the right. Clicking "?" shows the
/// explanation under the option, wrapped to the card width; clicking again
/// hides it. Replaces hover tooltips, which showed late and overflowed the
/// 340 px window.
fn option_row(
    ui: &mut egui::Ui,
    theme: Theme,
    id: &str,
    help: &str,
    add: impl FnOnce(&mut egui::Ui),
) {
    let open_id = ui.make_persistent_id(("help", id));
    let mut open = ui.data(|d| d.get_temp::<bool>(open_id)).unwrap_or(false);
    ui.horizontal_top(|ui| {
        let width = ui.available_width() - 22.0;
        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            add,
        );
        if help_button(ui, theme, open).clicked() {
            open = !open;
        }
    });
    ui.data_mut(|d| d.insert_temp(open_id, open));
    if open {
        ui.add(
            egui::Label::new(
                egui::RichText::new(help)
                    .size(10.5)
                    .color(theme::text_tertiary(theme)),
            )
            .wrap(),
        );
    }
    ui.add_space(4.0);
}

/// Small round "?" button; filled while its explanation is shown.
fn help_button(ui: &mut egui::Ui, theme: Theme, open: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::click());
    let accent = match theme {
        Theme::Dark => theme::ACTIVE_DARK,
        Theme::Light => theme::ACTIVE_LIGHT,
    };
    let color = if open || resp.hovered() { accent } else { theme::text_tertiary(theme) };
    let c = rect.center();
    if open {
        ui.painter().circle_filled(c, 7.5, color);
    } else {
        ui.painter().circle_stroke(c, 7.0, Stroke::new(1.0, color));
    }
    // Centred on the glyph's drawn bounds, not on its line box: the line box
    // keeps room for descenders, which left the "?" off-centre in the circle.
    let text_color = if open { theme::window_bg(theme) } else { color };
    let galley = ui.painter().layout_no_wrap(
        "?".to_string(),
        egui::FontId::proportional(11.0),
        text_color,
    );
    let pos = c - galley.mesh_bounds.center().to_vec2();
    ui.painter().galley(pos, galley, text_color);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Checkbox with a wrapping label. True when toggled.
fn checkbox(ui: &mut egui::Ui, theme: Theme, value: &mut bool, label: &str) -> bool {
    ui.checkbox(
        value,
        egui::RichText::new(label).size(12.0).color(theme::text_secondary(theme)),
    )
    .changed()
}

fn section_label(ui: &mut egui::Ui, theme: Theme, text: &str) {
    ui.label(egui::RichText::new(text).size(12.0).strong().color(theme::text_primary(theme)));
}

/// Tabs of the settings modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Options,
    Group,
    About,
}

/// Three equal-width tabs; the selected one is underlined.
fn tab_bar(ui: &mut egui::Ui, theme: Theme, tab: &mut SettingsTab) {
    let tabs = [
        (SettingsTab::Options, t!("settings.tab_options")),
        (SettingsTab::Group, t!("settings.tab_group")),
        (SettingsTab::About, t!("settings.tab_about")),
    ];
    let width = ui.available_width() / tabs.len() as f32;
    let accent = match theme {
        Theme::Dark => theme::ACTIVE_DARK,
        Theme::Light => theme::ACTIVE_LIGHT,
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (t, label) in tabs {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 26.0), Sense::click());
            let selected = *tab == t;
            let color = if selected || resp.hovered() {
                theme::text_primary(theme)
            } else {
                theme::text_tertiary(theme)
            };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label.as_ref(),
                egui::FontId::proportional(12.0),
                color,
            );
            let line_y = rect.bottom() - 1.0;
            let (line_color, thickness) = if selected {
                (accent, 2.0)
            } else {
                (row_palette(theme, false).1, 1.0)
            };
            ui.painter().line_segment(
                [egui::pos2(rect.left(), line_y), egui::pos2(rect.right(), line_y)],
                Stroke::new(thickness, line_color),
            );
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                *tab = t;
            }
        }
    });
}

/// "Label ……… [key]" row of the keys tab; the whole row is clickable.
fn key_row(ui: &mut egui::Ui, theme: Theme, label: &str, key: &str) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), Sense::click());
    let (bg, border) = row_palette(theme, response.hovered());
    ui.painter().rect_filled(rect, 8.0, bg);
    ui.painter()
        .rect_stroke(rect, 8.0, Stroke::new(1.0, border), StrokeKind::Inside);
    ui.painter().text(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        theme::text_primary(theme),
    );
    ui.painter().text(
        egui::pos2(rect.right() - 12.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        key,
        egui::FontId::monospace(11.0),
        theme::text_secondary(theme),
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

const BUG_GLYPH: &str = "\u{26A0}"; // ⚠
const FOLDER_GLYPH: &str = "\u{1F4C1}"; // 📁 — best-effort, font may fall back

fn github_link(ui: &mut egui::Ui, theme: Theme) -> egui::Response {
    let height = 36.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::click());

    let (bg, border) = row_palette(theme, response.hovered());
    ui.painter().rect_filled(rect, 8.0, bg);
    ui.painter()
        .rect_stroke(rect, 8.0, Stroke::new(1.0, border), StrokeKind::Inside);

    // Layout: [12 px pad] [logo 18×18] [8 px gap] [label] [auto] [→ 12 px pad]
    let logo_size = egui::vec2(18.0, 18.0);
    let logo_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 12.0, rect.center().y - logo_size.y / 2.0),
        logo_size,
    );
    let tint = if response.hovered() {
        theme::text_primary(theme)
    } else {
        theme::text_secondary(theme)
    };
    egui::Image::from_bytes("bytes://github.svg", crate::assets::GITHUB_SVG)
        .fit_to_exact_size(logo_size)
        .tint(tint)
        .paint_at(ui, logo_rect);

    let label_pos = egui::pos2(logo_rect.right() + 8.0, rect.center().y);
    ui.painter().text(
        label_pos,
        egui::Align2::LEFT_CENTER,
        REPO_LABEL,
        egui::FontId::proportional(13.0),
        theme::text_primary(theme),
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// Generic "icon + label, full width, click anywhere" row used for the
/// bug / config-dir actions. Keeps the visual rhythm of the github row.
fn action_row(ui: &mut egui::Ui, theme: Theme, label: &str, glyph: &str) -> egui::Response {
    let height = 32.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::click());

    let (bg, border) = row_palette(theme, response.hovered());
    ui.painter().rect_filled(rect, 8.0, bg);
    ui.painter()
        .rect_stroke(rect, 8.0, Stroke::new(1.0, border), StrokeKind::Inside);

    ui.painter().text(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        glyph,
        egui::FontId::proportional(13.0),
        theme::text_secondary(theme),
    );
    ui.painter().text(
        egui::pos2(rect.left() + 38.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        theme::text_primary(theme),
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

fn row_palette(theme: Theme, hovered: bool) -> (Color32, Color32) {
    match theme {
        Theme::Dark => {
            let base = Color32::from_rgb(0x2C, 0x2C, 0x2A);
            let bg = if hovered {
                Color32::from_rgb(0x36, 0x36, 0x32)
            } else {
                base
            };
            (bg, theme::BORDER_SUBTLE_DARK)
        }
        Theme::Light => {
            let base = Color32::from_rgb(0xF1, 0xEF, 0xE8);
            let bg = if hovered {
                Color32::from_rgb(0xE8, 0xE6, 0xDF)
            } else {
                base
            };
            (bg, theme::BORDER_SUBTLE_LIGHT)
        }
    }
}

fn close_button(ui: &mut egui::Ui, theme: Theme) -> egui::Response {
    let size = Vec2::new(96.0, 28.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let (bg, txt) = theme::button_primary(theme);
    let bg = if response.hovered() {
        // Slightly brighten on hover. A direct `mix` would be cleaner but
        // pulling in `ui::main_view::mix` from another module just for
        // this would over-couple the modules.
        match theme {
            Theme::Dark => Color32::from_rgb(0xF0, 0xEE, 0xE7),
            Theme::Light => Color32::from_rgb(0x3A, 0x3A, 0x37),
        }
    } else {
        bg
    };
    ui.painter().rect_filled(rect, 6.0, bg);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        t!("about.close").as_ref(),
        egui::FontId::proportional(12.0),
        txt,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

#[cfg(windows)]
fn open_folder(path: &Path) {
    // `explorer.exe <path>` is the lightest way to open a folder in
    // Explorer without pulling in `Win32_UI_Shell` just for this.
    // Failure is silent — the user can't act on it anyway.
    let _ = std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn();
}

#[cfg(not(windows))]
fn open_folder(_path: &Path) {}
