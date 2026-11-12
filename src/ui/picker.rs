//! Search modal shared by the zaap and NPC pickers: a search field always
//! focused, the filtered list below, click (or Enter for the first match)
//! to pick, Esc to close.
//!
//! Same backdrop + centred card pattern as `capture::draw` / `about::draw`.
//! The list is virtualised (`ScrollArea::show_rows`): only visible rows are
//! laid out, the NPC list has ~1 800 entries.

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, Margin, Sense, Stroke, Vec2};

use crate::theme::{self, Theme};

pub const CARD_SIZE: Vec2 = Vec2::new(320.0, 400.0);
const ROW_HEIGHT: f32 = 26.0;
const ROW_HEIGHT_WITH_DETAIL: f32 = 36.0;

/// Texts of one picker, already translated.
pub struct Texts {
    pub title: String,
    pub search_hint: String,
    pub no_result: String,
    pub hint: String,
}

/// One line: label on the left (detail under it, if any), coordinates on
/// the right.
pub struct Row<'a> {
    pub label: &'a str,
    pub detail: &'a str,
    pub coords: (i32, i32),
}

pub enum Outcome {
    Open,
    Close,
    Picked(usize),
}

/// Draws the picker over the whole window. `query` is edited in place.
pub fn show(
    ctx: &egui::Context,
    theme: Theme,
    id: &str,
    texts: &Texts,
    query: &mut String,
    rows: &[Row<'_>],
) -> Outcome {
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return Outcome::Close;
    }
    let mut outcome = if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !rows.is_empty() {
        Outcome::Picked(0)
    } else {
        Outcome::Open
    };

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
    let row_height = if rows.iter().any(|r| !r.detail.is_empty()) {
        ROW_HEIGHT_WITH_DETAIL
    } else {
        ROW_HEIGHT
    };

    egui::Area::new(egui::Id::new(id))
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
                            egui::RichText::new(texts.title.to_uppercase())
                                .size(11.0)
                                .color(theme::text_tertiary(theme)),
                        );
                        ui.add_space(8.0);

                        let field = ui.add(
                            egui::TextEdit::singleline(query)
                                .hint_text(texts.search_hint.as_str())
                                .desired_width(f32::INFINITY),
                        );
                        if !field.has_focus() {
                            field.request_focus();
                        }
                        ui.add_space(8.0);

                        if rows.is_empty() {
                            ui.label(
                                egui::RichText::new(texts.no_result.as_str())
                                    .size(12.0)
                                    .color(theme::text_tertiary(theme)),
                            );
                            ui.add_space(CARD_SIZE.y - 150.0);
                        } else {
                            egui::ScrollArea::vertical()
                                .max_height(CARD_SIZE.y - 130.0)
                                .auto_shrink([false, false])
                                .show_rows(ui, row_height, rows.len(), |ui, range| {
                                    for i in range {
                                        if row(ui, theme, &rows[i], row_height).clicked() {
                                            outcome = Outcome::Picked(i);
                                        }
                                    }
                                });
                        }

                        ui.add_space(6.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(texts.hint.as_str())
                                    .size(10.0)
                                    .color(theme::text_tertiary(theme)),
                            );
                        });
                    });
            });
        });
    outcome
}

fn row(ui: &mut egui::Ui, theme: Theme, r: &Row<'_>, height: f32) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 4.0, theme::hover_bg(theme));
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let painter = ui.painter();
    let left = rect.left() + 8.0;
    if r.detail.is_empty() {
        painter.text(
            egui::pos2(left, rect.center().y),
            egui::Align2::LEFT_CENTER,
            r.label,
            egui::FontId::proportional(13.0),
            theme::text_primary(theme),
        );
    } else {
        painter.text(
            egui::pos2(left, rect.center().y - 1.0),
            egui::Align2::LEFT_BOTTOM,
            r.label,
            egui::FontId::proportional(13.0),
            theme::text_primary(theme),
        );
        painter.text(
            egui::pos2(left, rect.center().y + 1.0),
            egui::Align2::LEFT_TOP,
            r.detail,
            egui::FontId::proportional(10.5),
            theme::text_tertiary(theme),
        );
    }
    painter.text(
        rect.right_center() - Vec2::new(8.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        format!("[{},{}]", r.coords.0, r.coords.1),
        egui::FontId::monospace(11.0),
        theme::text_tertiary(theme),
    );
    response
}
