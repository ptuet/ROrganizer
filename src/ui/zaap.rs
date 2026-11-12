//! Zaap picker: clicking a zaap (or Enter for the first match) sends every
//! launched account there. Built on the shared `picker` modal.

use eframe::egui;
use rust_i18n::t;

use super::picker::{self, Outcome, Row, Texts};
use crate::app::App;
use crate::zaaps;

pub fn draw(ctx: &egui::Context, app: &mut App) {
    let Some(query) = app.zaap_query_mut() else {
        return;
    };
    let mut query = query.clone();
    let theme = app.theme();
    let results = zaaps::search(&query);
    let rows: Vec<Row<'_>> = results
        .iter()
        .map(|z| Row { label: z.name, detail: "", coords: (z.x, z.y) })
        .collect();
    let texts = Texts {
        title: t!("zaap.title").to_string(),
        search_hint: t!("zaap.search_hint").to_string(),
        no_result: t!("zaap.no_result").to_string(),
        hint: t!("zaap.hint").to_string(),
    };

    match picker::show(ctx, theme, "zaap_overlay", &texts, &mut query, &rows) {
        Outcome::Close => app.close_zaap(),
        Outcome::Picked(i) => {
            app.close_zaap();
            app.zaap_all(ctx, results[i]);
        }
        Outcome::Open => {
            if let Some(q) = app.zaap_query_mut() {
                *q = query;
            }
        }
    }
}
