//! Zaap picker: clicking a zaap (or Enter for the first match) sends every
//! launched account there. Built on the shared `picker` modal.

use eframe::egui;
use rust_i18n::t;

use super::picker::{self, Outcome, Row, Texts};
use crate::app::App;
use crate::{favorites, zaaps};

pub fn draw(ctx: &egui::Context, app: &mut App) {
    let Some(query) = app.zaap_query_mut() else {
        return;
    };
    let mut query = query.clone();
    let theme = app.theme();
    let favs = app.favorite_zaaps().to_vec();
    let (results, starred) =
        favorites::favorites_first(zaaps::search(&query), &favs, |z| z.favorite_key());
    let rows: Vec<Row<'_>> = results
        .iter()
        .enumerate()
        .map(|(i, z)| Row { label: z.name, detail: "", coords: (z.x, z.y), favorite: i < starred })
        .collect();
    let texts = Texts {
        title: t!("zaap.title").to_string(),
        search_hint: t!("zaap.search_hint").to_string(),
        no_result: t!("zaap.no_result").to_string(),
        hint: t!("zaap.hint").to_string(),
        star_hint: t!("favorites.hint", max = favorites::MAX).to_string(),
    };

    let full = favs.len() >= favorites::MAX;
    match picker::show(ctx, theme, "zaap_overlay", &texts, &mut query, &rows, full) {
        Outcome::Close => app.close_zaap(),
        Outcome::Picked(i) => {
            app.close_zaap();
            app.zaap_all(ctx, results[i]);
        }
        Outcome::ToggleFavorite(i) => {
            app.toggle_favorite_zaap(results[i].favorite_key());
            if let Some(q) = app.zaap_query_mut() {
                *q = query;
            }
        }
        Outcome::Open => {
            if let Some(q) = app.zaap_query_mut() {
                *q = query;
            }
        }
    }
}
