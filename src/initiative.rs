//! Sort accounts by in-game initiative (experimental).
//!
//! The initiative is not in the window title: it only shows in the tooltip
//! of a portrait in the group panel. `win::group_panel` hovers each portrait
//! and OCRs the tooltip; this module turns the OCR lines into
//! (name, initiative) and reorders the accounts. Pure logic, no Windows API.
//!
//! Tooltip as read by the Windows OCR (2× upscale), one entry per OCR line:
//!
//! ```text
//! CHEF DU GROUPE : KORVAK      ← leader; a member shows just "LUMINA"
//! Classe : Eliotrope
//! Niveau :
//! 161
//! …
//! Initiative :
//! 5166
//! ```
//!
//! Labels and values sometimes land on separate lines, and the capture can
//! include the panel title ("MON GROUPE") above the name: the name is the
//! line right above "Classe", the initiative the first number after the
//! "Initiative" label.

use serde::{Deserialize, Serialize};

/// Where the group panel portraits are, relative to the top-left corner of
/// the calibrated Dofus window, in physical pixels. `step` goes from one
/// portrait to the next: (0, +y) for a vertical panel, (+x, 0) horizontal.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PanelCalibration {
    pub first: (i32, i32),
    pub step: (i32, i32),
}

impl PanelCalibration {
    /// Offset of portrait `i` (0-based) from the window's top-left corner.
    pub fn portrait(&self, i: i32) -> (i32, i32) {
        (self.first.0 + self.step.0 * i, self.first.1 + self.step.1 * i)
    }
}

/// Saved calibration: the account whose window was used, and where its
/// group panel portraits are.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupPanel {
    pub slot: String,
    pub calib: PanelCalibration,
}

/// Calibration from the two portrait positions picked by the user (offsets
/// from the window corner). `None` if they are too close to tell the
/// direction of the panel.
pub fn calibrate(first: (i32, i32), second: (i32, i32)) -> Option<PanelCalibration> {
    let step = (second.0 - first.0, second.1 - first.1);
    (step.0.abs() + step.1.abs() >= 12).then_some(PanelCalibration { first, step })
}

/// One group member read from its tooltip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub initiative: u32,
}

/// Name and initiative from the OCR lines of one tooltip, if both are there.
pub fn parse_tooltip(lines: &[String]) -> Option<Member> {
    let class_at = lines.iter().position(|l| fold(l).trim_start().starts_with("classe"))?;
    let name_line = lines[..class_at].iter().rev().find(|l| !l.trim().is_empty())?;
    // "CHEF DU GROUPE : KORVAK" → "KORVAK"; a plain member line has no ':'.
    let name = name_line.rsplit(':').next()?.trim().to_string();
    if name.is_empty() {
        return None;
    }

    // OCR may drop or misread the first letter: match on "nitiative".
    let label_at = lines.iter().position(|l| fold(l).contains("nitiative"))?;
    let after_label = lines[label_at].split_once(':').map(|(_, v)| v).unwrap_or("");
    let initiative = std::iter::once(after_label)
        .chain(lines[label_at + 1..].iter().map(String::as_str))
        .find_map(parse_number)?;
    Some(Member { name, initiative })
}

/// First number in `s`; digit groups split by spaces ("10 450") are joined.
fn parse_number(s: &str) -> Option<u32> {
    let digits: String = s
        .trim()
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || matches!(c, ' ' | '\u{a0}' | '\u{202f}'))
        .filter(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// The account `name` (as read by OCR, in capitals) refers to: same letters
/// ignoring case and accents, or else the single closest one with at most
/// one wrong, missing or extra letter.
pub fn match_account<'a>(name: &str, accounts: &[&'a str]) -> Option<&'a str> {
    let wanted = fold(name);
    if let Some(a) = accounts.iter().find(|a| fold(a) == wanted) {
        return Some(a);
    }
    let close: Vec<&&str> = accounts.iter().filter(|a| distance(&fold(a), &wanted) == 1).collect();
    match close.as_slice() {
        [only] => Some(**only),
        _ => None,
    }
}

/// `order` with the accounts found in the group moved to the front, highest
/// initiative first. Accounts not found keep their relative order, after.
pub fn sort_order(order: &[String], found: &[(String, u32)]) -> Vec<String> {
    let mut ranked: Vec<&(String, u32)> = found.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1)); // stable: ties keep group order
    let mut out: Vec<String> = ranked.iter().map(|(slot, _)| slot.clone()).collect();
    let rest: Vec<String> = order.iter().filter(|s| !out.contains(s)).cloned().collect();
    out.extend(rest);
    out
}

/// Lowercase without accents: "Éliotrope" → "eliotrope".
fn fold(s: &str) -> String {
    s.chars()
        .map(crate::class_icon::strip_accent)
        .flat_map(char::to_lowercase)
        .collect()
}

/// Levenshtein distance (short strings, no allocation worth optimizing).
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<String> {
        s.lines().map(str::to_string).collect()
    }

    // OCR output of the three screenshots shared while designing the feature.
    const LEADER: &str = "CHEF DU GROUPE : KORVAK\nClasse : Eliotrope\nNiveau :\n161\n\
        Prospection :\n301\n1978 / 1978\nInitiative :\n5166";
    const MEMBER: &str = "LUMINA\nClasse : Sadida\nNiveau :\n166\nProspection :\n164\nPV :\n\
        1985 / 1985\nInitiative :\n2104";
    const HORIZONTAL: &str = "MON GROUPE\nLUMINA\nClasse : Sadida\nNiveau :\n166\n\
        Prospection : 164\nPV :\n1985 / 1985\nInitiative : 2104";

    #[test]
    fn leader_tooltip() {
        assert_eq!(
            parse_tooltip(&lines(LEADER)),
            Some(Member { name: "KORVAK".into(), initiative: 5166 })
        );
    }

    #[test]
    fn member_tooltip() {
        assert_eq!(
            parse_tooltip(&lines(MEMBER)),
            Some(Member { name: "LUMINA".into(), initiative: 2104 })
        );
    }

    #[test]
    fn horizontal_panel_title_is_not_the_name() {
        assert_eq!(
            parse_tooltip(&lines(HORIZONTAL)),
            Some(Member { name: "LUMINA".into(), initiative: 2104 })
        );
    }

    #[test]
    fn ocr_variants_still_parse() {
        // Lowercase "Du" and a big initiative split by a space.
        let t = "CHEF Du GROUPE : KORVAK\nClasse :\nEliotrope\nInitiative : 10 450";
        assert_eq!(
            parse_tooltip(&lines(t)),
            Some(Member { name: "KORVAK".into(), initiative: 10450 })
        );
    }

    #[test]
    fn incomplete_tooltips_are_rejected() {
        assert_eq!(parse_tooltip(&lines("")), None);
        // Initiative label cut off by the capture edge.
        assert_eq!(parse_tooltip(&lines("LUMINA\nClasse : Sadida\nInitiative :")), None);
        // No "Classe" line: not a group member tooltip.
        assert_eq!(parse_tooltip(&lines("Coffre\nInitiative : 12")), None);
    }

    #[test]
    fn account_match_ignores_case_and_tolerates_one_letter() {
        let accounts = ["Korvak", "Lumina", "Mistral", "Brindille"];
        assert_eq!(match_account("KORVAK", &accounts), Some("Korvak"));
        assert_eq!(match_account("LUMINO", &accounts), Some("Lumina"));
        assert_eq!(match_account("INCONNU", &accounts), None);
    }

    #[test]
    fn ambiguous_close_match_is_refused() {
        // One letter away from both: better to leave it out than to guess.
        assert_eq!(match_account("ABCX", &["ABCD", "ABCE"]), None);
    }

    #[test]
    fn sort_puts_highest_initiative_first_and_keeps_the_rest() {
        let order: Vec<String> =
            ["Brindille", "Offline", "Korvak", "Lumina"].map(String::from).to_vec();
        let found = vec![("Lumina".to_string(), 2104), ("Korvak".to_string(), 5166)];
        assert_eq!(sort_order(&order, &found), ["Korvak", "Lumina", "Brindille", "Offline"]);
    }

    #[test]
    fn calibration_needs_two_distinct_portraits() {
        assert_eq!(
            calibrate((40, 300), (40, 364)),
            Some(PanelCalibration { first: (40, 300), step: (0, 64) })
        );
        assert_eq!(calibrate((40, 300), (42, 303)), None);
    }

    #[test]
    fn calibration_walks_both_directions() {
        let vertical = PanelCalibration { first: (40, 300), step: (0, 64) };
        assert_eq!(vertical.portrait(2), (40, 428));
        let horizontal = PanelCalibration { first: (500, 40), step: (70, 0) };
        assert_eq!(horizontal.portrait(3), (710, 40));
    }
}
