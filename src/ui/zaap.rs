//! Zaap picker modal: a search field over the zaap list. Clicking a zaap (or
//! Enter for the first match) sends every launched account there.
//!
//! Same backdrop + centred card pattern as `capture::draw` / `about::draw`.

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, Margin, Sense, Stroke, Vec2};
use rust_i18n::t;

use crate::app::App;
use crate::theme::{self, Theme};
use crate::zaaps::{self, Zaap};

const CARD_SIZE: Vec2 = Vec2::new(320.0, 400.0);
const ROW_HEIGHT: f32 = 26.0;

pub fn draw(ctx: &egui::Context, app: &mut App) {
    let Some(query) = app.zaap_query_mut() else {
        return;
    };
    let mut query = query.clone();
    let theme = app.theme();

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.close_zaap();
        return;
    }

    let results = zaaps::search(&query);
    let mut picked: Option<&'static Zaap> = None;
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        picked = results.first().copied();
    }

    let screen_rect = ctx.content_rect();
    let (backdrop, card_bg, card_border) = match theme {
        Theme::Dark => (
            Color32::from_rgba_unmultiplied(0x10, 0x10, 0x0E, 0xCC),
            theme::BG_ROW_DARK,
            theme::BORDER_STRONG_DARK,
        ),
        Theme::Light => (
            Color32::from_rgba_unmultiplied(0xFA, 0xFA, 0xF7, 0xCC),
            theme::BG_ROW_LIGHT,
            theme::BORDER_STRONG_LIGHT,
        ),
    };

    egui::Area::new(egui::Id::new("zaap_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen_rect.left_top())
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(screen_rect, CornerRadius::ZERO, backdrop);
            let card_rect = egui::Rect::from_center_size(screen_rect.center(), CARD_SIZE);
            ui.scope_builder(egui::UiBuilder::new().max_rect(card_rect), |ui| {
                Frame::new()
                    .fill(card_bg)
                    .corner_radius(CornerRadius::same(10))
                    .stroke(Stroke::new(1.0, card_border))
                    .inner_margin(Margin::symmetric(14, 14))
                    .show(ui, |ui| {
                        ui.set_min_size(CARD_SIZE - Vec2::splat(28.0));
                        ui.label(
                            egui::RichText::new(t!("zaap.title").to_uppercase())
                                .size(11.0)
                                .color(theme::text_tertiary(theme)),
                        );
                        ui.add_space(8.0);

                        let field = ui.add(
                            egui::TextEdit::singleline(&mut query)
                                .hint_text(t!("zaap.search_hint").to_string())
                                .desired_width(f32::INFINITY),
                        );
                        if !field.has_focus() {
                            field.request_focus();
                        }
                        ui.add_space(8.0);

                        egui::ScrollArea::vertical()
                            .max_height(CARD_SIZE.y - 130.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if results.is_empty() {
                                    ui.label(
                                        egui::RichText::new(t!("zaap.no_result").to_string())
                                            .size(12.0)
                                            .color(theme::text_tertiary(theme)),
                                    );
                                }
                                for zaap in &results {
                                    if zaap_row(ui, theme, zaap).clicked() {
                                        picked = Some(zaap);
                                    }
                                }
                            });

                        ui.add_space(6.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(t!("zaap.hint").to_string())
                                    .size(10.0)
                                    .color(theme::text_tertiary(theme)),
                            );
                        });
                    });
            });
        });

    match picked {
        Some(zaap) => {
            app.close_zaap();
            app.zaap_all(ctx, zaap);
        }
        None => {
            if let Some(q) = app.zaap_query_mut() {
                *q = query;
            }
        }
    }
}

/// One line of the list: territory on the left, coordinates on the right.
fn zaap_row(ui: &mut egui::Ui, theme: Theme, zaap: &Zaap) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 4.0, theme::hover_bg(theme));
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let painter = ui.painter();
    painter.text(
        rect.left_center() + Vec2::new(8.0, 0.0),
        egui::Align2::LEFT_CENTER,
        zaap.name,
        egui::FontId::proportional(13.0),
        theme::text_primary(theme),
    );
    painter.text(
        rect.right_center() - Vec2::new(8.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        format!("[{},{}]", zaap.x, zaap.y),
        egui::FontId::monospace(11.0),
        theme::text_tertiary(theme),
    );
    response
}
