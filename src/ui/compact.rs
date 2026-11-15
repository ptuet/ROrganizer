//! Mini window: a small bar to leave over the game, with only the group
//! buttons (NPC, zaap, invite) and one to go back to the full window. The
//! zaap and NPC pickers still open: `App::apply_dynamic_height` grows the
//! window while one of them is shown.

use eframe::egui;
use egui::{Sense, Stroke, Vec2};
use rust_i18n::t;

use crate::app::App;
use crate::theme;
use crate::ui::main_view::{icon_button, paint_invite_icon, paint_npc_icon, paint_zaap_icon};
use crate::win;

const GRIP: f32 = 10.0;

/// Window size. Row: grip + 4 buttons of 26×22 with gaps of 4. Square
/// (`grid`): grip + 2 rows of 2 buttons.
pub fn size(grid: bool) -> Vec2 {
    if grid { Vec2::new(82.0, 60.0) } else { Vec2::new(144.0, 36.0) }
}

/// The four buttons, in display order.
#[derive(Clone, Copy)]
enum Button {
    Npc,
    Zaap,
    Invite,
    Expand,
}

pub fn draw(ui: &mut egui::Ui, app: &mut App) {
    let theme = app.theme();
    let ctx = ui.ctx().clone();
    let count = app.live_count();
    let busy = win::chat::is_busy();
    let grid = app.compact_grid();
    let mut clicked = None;

    // Same conditions as the full view, but the buttons stay in place
    // (greyed out) so the window never changes size. Tooltips are one
    // word: a tooltip can't leave this small window and was cut off.
    let mut button = |ui: &mut egui::Ui, b: Button| {
        let (off, paint, tip): (bool, fn(&egui::Painter, egui::Rect, Stroke), _) = match b {
            Button::Npc => (busy || count < 1, paint_npc_icon, t!("header.short_npc")),
            Button::Zaap => (busy || count < 1, paint_zaap_icon, t!("header.short_zaap")),
            Button::Invite => (busy || count < 2, paint_invite_icon, t!("header.short_invite")),
            Button::Expand => (false, paint_expand_icon, t!("header.short_expand")),
        };
        if icon_button(ui, theme, off, paint).on_hover_text(tip.to_string()).clicked() && !off {
            clicked = Some(b);
        }
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme::window_bg(theme)))
        .show(ui, |ui| {
            // The picker covers the grown window; the bar would show through
            // its translucent backdrop.
            if app.is_picker_open() {
                return;
            }
            // Registered first: the buttons drawn after win their clicks,
            // the rest of the bar moves the window.
            let rect = ui.max_rect();
            let drag = ui.interact(rect, egui::Id::new("compact_drag"), Sense::click_and_drag());
            if drag.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.add_space(4.0);
                    paint_grip(ui, theme, app.is_active());
                    if grid {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 4.0;
                            // `horizontal_centered` only centres single
                            // widgets; this block starts at the top.
                            const BLOCK: f32 = 22.0 * 2.0 + 4.0;
                            ui.add_space(((ui.available_height() - BLOCK) / 2.0).max(0.0));
                            ui.horizontal(|ui| {
                                button(ui, Button::Npc);
                                button(ui, Button::Zaap);
                            });
                            ui.horizontal(|ui| {
                                button(ui, Button::Invite);
                                button(ui, Button::Expand);
                            });
                        });
                    } else {
                        for b in [Button::Npc, Button::Zaap, Button::Invite, Button::Expand] {
                            button(ui, b);
                        }
                    }
                });
            });
        });

    match clicked {
        Some(Button::Npc) => app.open_npc(),
        Some(Button::Zaap) => app.open_zaap(),
        Some(Button::Invite) => app.invite_group(&ctx),
        Some(Button::Expand) => app.set_compact(false),
        None => {}
    }
}

/// Two columns of dots on the left: shows the bar can be dragged, and
/// whether the shortcuts are on (green) or off (grey) — no room to add
/// anything without growing the window.
fn paint_grip(ui: &mut egui::Ui, theme: theme::Theme, active: bool) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(GRIP, 18.0), Sense::hover());
    let color = match (active, theme) {
        (false, _) => theme::text_tertiary(theme),
        (true, theme::Theme::Dark) => theme::ACTIVE_DARK,
        (true, theme::Theme::Light) => theme::ACTIVE_LIGHT,
    };
    resp.on_hover_text(if active {
        t!("header.short_active").to_string()
    } else {
        t!("header.short_inactive").to_string()
    });
    for col in [-2.0, 2.0] {
        for row in [-5.0, 0.0, 5.0] {
            ui.painter()
                .circle_filled(rect.center() + Vec2::new(col, row), 1.1, color);
        }
    }
}

/// "Expand": a window frame with its title bar.
fn paint_expand_icon(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    let r = egui::Rect::from_center_size(rect.center(), Vec2::new(14.0, 11.0));
    painter.rect_stroke(r, 1.5, stroke, egui::StrokeKind::Middle);
    let y = r.top() + 3.0;
    painter.line_segment([egui::pos2(r.left(), y), egui::pos2(r.right(), y)], stroke);
}

/// "Mini mode": a large frame with a small filled one in its corner. Used
/// by the title bar button that enters the mini window.
pub fn paint_mini_icon(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    let outer = egui::Rect::from_center_size(rect.center(), Vec2::new(14.0, 11.0));
    painter.rect_stroke(outer, 1.5, stroke, egui::StrokeKind::Middle);
    let inner = egui::Rect::from_min_max(
        outer.right_bottom() - Vec2::new(7.0, 5.5),
        outer.right_bottom() - Vec2::new(1.5, 1.5),
    );
    painter.rect_filled(inner, 1.0, stroke.color);
}
