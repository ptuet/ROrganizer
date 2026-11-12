//! Favourite zaaps and NPCs: at most `MAX` of each, shown first in their
//! picker, in the order they were starred.
//!
//! A favourite is stored in `config.json` by a readable key (name, area,
//! coordinates — see `Zaap::favorite_key` / `Npc::favorite_key`): the
//! DofusDB tables have no stable id. A key the table no longer contains
//! (list regenerated, config edited by hand) is dropped at load.

/// Favourites per picker.
pub const MAX: usize = 5;

/// `items` with its favourites moved to the front, in `favorites` order,
/// and how many there are. The others keep their order.
pub fn favorites_first<T>(
    mut items: Vec<T>,
    favorites: &[String],
    key: impl Fn(&T) -> String,
) -> (Vec<T>, usize) {
    let mut front = Vec::new();
    for fav in favorites {
        if let Some(pos) = items.iter().position(|item| key(item) == *fav) {
            front.push(items.remove(pos));
        }
    }
    let count = front.len();
    front.extend(items);
    (front, count)
}

/// Stars `key`, or unstars it if it already is. False when the list is full
/// and `key` isn't in it (nothing changes).
pub fn toggle(favorites: &mut Vec<String>, key: String) -> bool {
    if let Some(pos) = favorites.iter().position(|f| *f == key) {
        favorites.remove(pos);
        true
    } else if favorites.len() < MAX {
        favorites.push(key);
        true
    } else {
        false
    }
}

/// Keys still in the table, without duplicates, at most `MAX`.
pub fn clean(favorites: Vec<String>, exists: impl Fn(&str) -> bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for key in favorites {
        if out.len() < MAX && exists(&key) && !out.contains(&key) {
            out.push(key);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn favorites_come_first_in_starred_order() {
        let items = vec!["a", "b", "c", "d"];
        let (out, n) = favorites_first(items, &keys(&["d", "b"]), |s| s.to_string());
        assert_eq!(out, ["d", "b", "a", "c"]);
        assert_eq!(n, 2);
    }

    #[test]
    fn favorites_missing_from_the_results_are_skipped() {
        let (out, n) = favorites_first(vec!["a", "b"], &keys(&["z", "b"]), |s| s.to_string());
        assert_eq!(out, ["b", "a"]);
        assert_eq!(n, 1);
    }

    #[test]
    fn toggle_adds_removes_and_stops_at_max() {
        let mut favs = Vec::new();
        for i in 0..MAX {
            assert!(toggle(&mut favs, i.to_string()));
        }
        assert!(!toggle(&mut favs, "extra".into()));
        assert_eq!(favs.len(), MAX);
        assert!(toggle(&mut favs, "0".into()));
        assert!(!favs.contains(&"0".to_string()));
        assert!(toggle(&mut favs, "extra".into()));
    }

    #[test]
    fn clean_drops_unknown_duplicates_and_extras() {
        let favs = keys(&["a", "gone", "a", "b", "c", "d", "e", "f"]);
        let out = clean(favs, |k| k != "gone");
        assert_eq!(out, keys(&["a", "b", "c", "d", "e"]));
    }
}
