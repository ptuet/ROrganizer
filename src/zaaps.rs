//! Zaaps of Dofus 3, for the "send every account to a zaap" button.
//!
//! Source: DofusDB (community database extracted from the game files), map
//! hints named "Zaap" + their subarea ("Territoire" in game) and area.
//! Generated on 2026-11-10 by `scripts/gen_zaaps.py`; checked against the
//! in-game zaap list. Alliance prisms are not included (they change all the
//! time). Incarnam (Cimetière, Pâturages, Route des âmes) and three zaaps of
//! other worlds (Village de la Canopée, Entrée du château de Harebourg,
//! Crocuzko) are kept, though `/zaap` may not reach another world.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zaap {
    /// Territory, as shown in the game's zaap list.
    pub name: &'static str,
    /// Region, only used to widen the search ("cania", "frigost"…).
    pub area: &'static str,
    pub x: i32,
    pub y: i32,
}

impl Zaap {
    /// Chat command that teleports to this zaap.
    pub fn command(&self) -> String {
        format!("/zaap {},{}", self.x, self.y)
    }
}

/// Zaaps whose territory or region contains every word of `query`, ignoring
/// case and accents. An empty query returns the whole list.
pub fn search(query: &str) -> Vec<&'static Zaap> {
    let words: Vec<String> = query.split_whitespace().map(fold).collect();
    ZAAPS
        .iter()
        .filter(|z| {
            let haystack = fold(&format!("{} {}", z.name, z.area));
            words.iter().all(|w| haystack.contains(w.as_str()))
        })
        .collect()
}

/// Lowercase without accents: "Château" → "chateau".
fn fold(s: &str) -> String {
    s.chars()
        .map(crate::class_icon::strip_accent)
        .flat_map(char::to_lowercase)
        .collect()
}

#[rustfmt::skip]
pub const ZAAPS: &[Zaap] = &[
    Zaap { name: "Arche de Vili", area: "Profondeurs d'Astrub", x: 15, y: -20 },
    Zaap { name: "Bord de la forêt maléfique", area: "Amakna", x: -1, y: 13 },
    Zaap { name: "Champs de Cania", area: "Plaines de Cania", x: -27, y: -36 },
    Zaap { name: "Château d'Amakna", area: "Amakna", x: 3, y: -5 },
    Zaap { name: "Cimetière", area: "Incarnam", x: 3, y: 0 },
    Zaap { name: "Cimetière primitif", area: "Montagne des Koalaks", x: -12, y: 19 },
    Zaap { name: "Cité d'Astrub", area: "Astrub", x: 5, y: -18 },
    Zaap { name: "Coin des Bouftous", area: "Amakna", x: 5, y: 7 },
    Zaap { name: "Crocuzko", area: "Archipel des Écailles", x: -83, y: -15 },
    Zaap { name: "Cœur immaculé", area: "Bonta", x: -31, y: -56 },
    Zaap { name: "Dunes des ossements", area: "Saharach", x: 15, y: -58 },
    Zaap { name: "Entrée du château de Harebourg", area: "Île de Frigost", x: -67, y: -77 },
    Zaap { name: "Foire du Trool", area: "Foire du Trool", x: -11, y: -36 },
    Zaap { name: "Futaie enneigée", area: "Archipel de Valonia", x: 39, y: -82 },
    Zaap { name: "La Bourgade", area: "Île de Frigost", x: -78, y: -41 },
    Zaap { name: "La Cuirasse", area: "Brâkmar", x: -26, y: 37 },
    Zaap { name: "Laboratoires abandonnés", area: "Île des Wabbits", x: 27, y: -14 },
    Zaap { name: "Lac de Cania", area: "Plaines de Cania", x: -3, y: -42 },
    Zaap { name: "Massif de Cania", area: "Plaines de Cania", x: -13, y: -28 },
    Zaap { name: "Mont des Tombeaux", area: "Île de Grobe", x: 40, y: -44 },
    Zaap { name: "Montagne des Craqueleurs", area: "Amakna", x: -5, y: -8 },
    Zaap { name: "Nimotopia", area: "Nimotopia", x: -67, y: 29 },
    Zaap { name: "Plage de la Tortue", area: "Île de Moon", x: 35, y: 12 },
    Zaap { name: "Plaine des Porkass", area: "Plaines de Cania", x: -5, y: -23 },
    Zaap { name: "Plaine des Scarafeuilles", area: "Amakna", x: -1, y: 24 },
    Zaap { name: "Plaines Rocheuses", area: "Plaines de Cania", x: -17, y: -47 },
    Zaap { name: "Port de Madrestam", area: "Amakna", x: 7, y: -4 },
    Zaap { name: "Pâturages", area: "Incarnam", x: 2, y: -5 },
    Zaap { name: "Rivage sufokien", area: "Baie de Sufokia", x: 10, y: 22 },
    Zaap { name: "Route des Roulottes", area: "Landes de Sidimote", x: -25, y: 12 },
    Zaap { name: "Route des âmes", area: "Incarnam", x: -1, y: -3 },
    Zaap { name: "Routes Rocailleuses", area: "Plaines de Cania", x: -20, y: -20 },
    Zaap { name: "Sufokia", area: "Baie de Sufokia", x: 13, y: 26 },
    Zaap { name: "Tainéla", area: "Astrub", x: 1, y: -32 },
    Zaap { name: "Temple des alliances", area: "Baie de Sufokia", x: 13, y: 35 },
    Zaap { name: "Terres Désacrées", area: "Landes de Sidimote", x: -15, y: 25 },
    Zaap { name: "Village côtier", area: "Île d'Otomaï", x: -46, y: 18 },
    Zaap { name: "Village d'Amakna", area: "Amakna", x: -2, y: 0 },
    Zaap { name: "Village de la Canopée", area: "Île d'Otomaï", x: -54, y: 16 },
    Zaap { name: "Village de Pandala", area: "Île de Pandala", x: 20, y: -29 },
    Zaap { name: "Village des Dopeuls", area: "Plaines de Cania", x: -34, y: -8 },
    Zaap { name: "Village des Kanigs", area: "Plaines de Cania", x: 0, y: -56 },
    Zaap { name: "Village des Éleveurs", area: "Montagne des Koalaks", x: -16, y: 1 },
    Zaap { name: "Village enseveli", area: "Île de Frigost", x: -77, y: -73 },
    Zaap { name: "Île de la Cawotte", area: "Île des Wabbits", x: 25, y: -4 },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn names(query: &str) -> Vec<&'static str> {
        search(query).into_iter().map(|z| z.name).collect()
    }

    #[test]
    fn empty_query_lists_every_zaap() {
        assert_eq!(search("").len(), ZAAPS.len());
        assert_eq!(search("   ").len(), ZAAPS.len());
    }

    #[test]
    fn search_ignores_case_and_accents() {
        assert_eq!(
            names("chateau"),
            vec!["Château d'Amakna", "Entrée du château de Harebourg"]
        );
        assert_eq!(names("CHÂTEAU d'amakna"), vec!["Château d'Amakna"]);
    }

    #[test]
    fn search_matches_the_region_too() {
        let cania = names("cania");
        assert!(cania.contains(&"Plaine des Porkass"));
        assert!(cania.contains(&"Lac de Cania"));
    }

    #[test]
    fn every_word_must_match() {
        assert_eq!(names("village amakna"), vec!["Village d'Amakna"]);
        assert!(names("porkass frigost").is_empty());
    }

    #[test]
    fn command_uses_the_game_syntax() {
        let porkass = search("porkass")[0];
        assert_eq!(porkass.command(), "/zaap -5,-23");
    }

    #[test]
    fn table_matches_the_in_game_list() {
        // Checked by hand against a screenshot of the in-game zaap window.
        for (name, x, y) in [
            ("Plaine des Porkass", -5, -23),
            ("Coin des Bouftous", 5, 7),
            ("Route des Roulottes", -25, 12),
            ("Lac de Cania", -3, -42),
            ("Village d'Amakna", -2, 0),
            ("Dunes des ossements", 15, -58),
            ("Futaie enneigée", 39, -82),
            ("Foire du Trool", -11, -36),
        ] {
            let z = ZAAPS.iter().find(|z| z.name == name).expect(name);
            assert_eq!((z.x, z.y), (x, y), "{name}");
        }
    }
}
