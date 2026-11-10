//! Group panel calibration overlay (experimental initiative sort): asks the
//! user to put the cursor on the first, then the second portrait of the
//! Dofus group panel, with a countdown. `App::tick_calibration` reads the
//! cursor when the countdown ends. Esc cancels.
//!
//! Same backdrop + centred card pattern as `capture::draw`.

use std::time::Instant;

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, Margin, Stroke, Vec2};
use rust_i18n::t;

use crate::app::App;
use crate::theme::{self, Theme};

pub fn draw(ctx: &egui::Context, app: &mut App) {
    let Some(cal) = app.calibration() else { return };
    let step = cal.step;
    let secs = cal
        .deadline
        .saturating_duration_since(Instant::now())
        .as_secs_f32()
        .ceil() as u32;
    let theme = app.theme();

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.cancel_calibration();
        return;
    }

    let screen_rect = ctx.content_rect();
    let (backdrop, card_bg, card_border, accent) = match theme {
        Theme::Dark => (
            Color32::from_rgba_unmultiplied(0x10, 0x10, 0x0E, 0xCC),
            theme::BG_ROW_DARK,
            theme::BORDER_STRONG_DARK,
            theme::ACTIVE_DARK,
        ),
        Theme::Light => (
            Color32::from_rgba_unmultiplied(0xFA, 0xFA, 0xF7, 0xCC),
            theme::BG_ROW_LIGHT,
            theme::BORDER_STRONG_LIGHT,
            theme::ACTIVE_LIGHT,
        ),
    };
    let instruction = if step == 1 {
        t!("initiative.calib_step1")
    } else {
        t!("initiative.calib_step2")
    };

    egui::Area::new(egui::Id::new("calibration_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen_rect.left_top())
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(screen_rect, CornerRadius::ZERO, backdrop);
            let card_size = Vec2::new(300.0, 190.0);
            let card_rect = egui::Rect::from_center_size(screen_rect.center(), card_size);
            ui.scope_builder(egui::UiBuilder::new().max_rect(card_rect), |ui| {
                Frame::new()
                    .fill(card_bg)
                    .corner_radius(CornerRadius::same(10))
                    .stroke(Stroke::new(1.0, card_border))
                    .inner_margin(Margin::symmetric(16, 14))
                    .show(ui, |ui| {
                        ui.set_min_size(card_size - Vec2::new(32.0, 28.0));
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(t!("initiative.calib_title").to_uppercase())
                                    .size(11.0)
                                    .color(theme::text_tertiary(theme)),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!("{step} / 2"))
                                    .size(11.0)
                                    .color(theme::text_tertiary(theme)),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(instruction)
                                    .size(13.0)
                                    .strong()
                                    .color(theme::text_primary(theme)),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(format!("{secs} s"))
                                    .size(22.0)
                                    .color(accent),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(t!("initiative.calib_hint"))
                                    .size(10.0)
                                    .color(theme::text_tertiary(theme)),
                            );
                        });
                    });
            });
        });
}
