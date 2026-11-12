//! NPC picker: clicking an NPC (or Enter for the first match) makes every
//! launched account teleport to the nearest zaap then walk to the NPC.
//! Built on the shared `picker` modal.

use eframe::egui;
use rust_i18n::t;

use super::picker::{self, Outcome, Row, Texts};
use crate::app::App;
use crate::npcs;

pub fn draw(ctx: &egui::Context, app: &mut App) {
    let Some(query) = app.npc_query_mut() else {
        return;
    };
    let mut query = query.clone();
    let theme = app.theme();
    let results = npcs::search(&query);
    let rows: Vec<Row<'_>> = results
        .iter()
        .map(|n| Row { label: n.name, detail: n.area, coords: (n.x, n.y) })
        .collect();
    let texts = Texts {
        title: t!("npc.title").to_string(),
        search_hint: t!("npc.search_hint").to_string(),
        no_result: t!("npc.no_result").to_string(),
        hint: t!("npc.hint").to_string(),
    };

    match picker::show(ctx, theme, "npc_overlay", &texts, &mut query, &rows) {
        Outcome::Close => app.close_npc(),
        Outcome::Picked(i) => {
            app.close_npc();
            app.npc_travel_all(ctx, results[i]);
        }
        Outcome::Open => {
            if let Some(q) = app.npc_query_mut() {
                *q = query;
            }
        }
    }
}
