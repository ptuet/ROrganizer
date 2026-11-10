use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::win::keyboard_layout;

pub type SlotKey = String;

/// What the user is binding a Trigger to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BindingTarget {
    Account(SlotKey),
    CycleNext,
    CyclePrev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trigger {
    /// Win32 Virtual-Key code (VK_F1 = 0x70, VK_A = 0x41, …).
    Key(u32),
    Mouse(MouseBtn),
    Wheel(WheelDir),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseBtn {
    Middle,
    /// XBUTTON1 — "mb4" / side back.
    X1,
    /// XBUTTON2 — "mb5" / side forward.
    X2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WheelDir {
    Up,
    Down,
}

impl Trigger {
    pub fn display_label(&self) -> String {
        match self {
            Trigger::Key(vk) if is_oem_vk(*vk) => keyboard_layout::vk_to_char(*vk)
                .map(|c| c.to_string())
                .unwrap_or_else(|| vk_to_label(*vk)),
            Trigger::Key(vk) => vk_to_label(*vk),
            Trigger::Mouse(MouseBtn::Middle) => "MWheel".into(),
            Trigger::Mouse(MouseBtn::X1) => "Mb4".into(),
            Trigger::Mouse(MouseBtn::X2) => "Mb5".into(),
            Trigger::Wheel(WheelDir::Up) => "MWh\u{2191}".into(),
            Trigger::Wheel(WheelDir::Down) => "MWh\u{2193}".into(),
        }
    }
}

/// VK the global hook will receive for this key press on the active layout.
///
/// Layout-dependent keys are resolved from their physical position, because
/// `key_to_vk` assumes US-QWERTY VKs (AZERTY `²` would be stored as 0xC0,
/// which the hook sees as `ù`). Everything else goes through `key_to_vk`.
pub fn event_to_vk(key: egui::Key, physical_key: Option<egui::Key>) -> Option<u32> {
    physical_key
        .and_then(oem_scan_code)
        .and_then(keyboard_layout::scan_code_to_vk)
        .or_else(|| key_to_vk(key))
}

/// Set-1 scan code of the physical keys whose VK depends on the layout.
/// egui names physical keys after their US-QWERTY legend.
fn oem_scan_code(key: egui::Key) -> Option<u32> {
    use egui::Key;
    Some(match key {
        Key::Backtick => 0x29,
        Key::Minus => 0x0C,
        Key::Equals => 0x0D,
        Key::OpenBracket => 0x1A,
        Key::CloseBracket => 0x1B,
        Key::Backslash => 0x2B,
        Key::Semicolon => 0x27,
        Key::Quote => 0x28,
        Key::Comma => 0x33,
        Key::Period => 0x34,
        Key::Slash => 0x35,
        _ => return None,
    })
}

/// VK_OEM_* range, whose printed character depends on the layout.
fn is_oem_vk(vk: u32) -> bool {
    matches!(vk, 0xBA..=0xC0 | 0xDB..=0xDF | 0xE2)
}

pub fn key_to_vk(key: egui::Key) -> Option<u32> {
    use egui::Key;
    Some(match key {
        Key::F1 => 0x70,
        Key::F2 => 0x71,
        Key::F3 => 0x72,
        Key::F4 => 0x73,
        Key::F5 => 0x74,
        Key::F6 => 0x75,
        Key::F7 => 0x76,
        Key::F8 => 0x77,
        Key::F9 => 0x78,
        Key::F10 => 0x79,
        Key::F11 => 0x7A,
        Key::F12 => 0x7B,
        Key::F13 => 0x7C,
        Key::F14 => 0x7D,
        Key::F15 => 0x7E,
        Key::F16 => 0x7F,
        Key::F17 => 0x80,
        Key::F18 => 0x81,
        Key::F19 => 0x82,
        Key::F20 => 0x83,
        Key::A => 0x41,
        Key::B => 0x42,
        Key::C => 0x43,
        Key::D => 0x44,
        Key::E => 0x45,
        Key::F => 0x46,
        Key::G => 0x47,
        Key::H => 0x48,
        Key::I => 0x49,
        Key::J => 0x4A,
        Key::K => 0x4B,
        Key::L => 0x4C,
        Key::M => 0x4D,
        Key::N => 0x4E,
        Key::O => 0x4F,
        Key::P => 0x50,
        Key::Q => 0x51,
        Key::R => 0x52,
        Key::S => 0x53,
        Key::T => 0x54,
        Key::U => 0x55,
        Key::V => 0x56,
        Key::W => 0x57,
        Key::X => 0x58,
        Key::Y => 0x59,
        Key::Z => 0x5A,
        Key::Num0 => 0x30,
        Key::Num1 => 0x31,
        Key::Num2 => 0x32,
        Key::Num3 => 0x33,
        Key::Num4 => 0x34,
        Key::Num5 => 0x35,
        Key::Num6 => 0x36,
        Key::Num7 => 0x37,
        Key::Num8 => 0x38,
        Key::Num9 => 0x39,
        Key::Space => 0x20,
        Key::Backspace => 0x08,
        Key::Tab => 0x09,
        Key::Enter => 0x0D,
        Key::Insert => 0x2D,
        Key::Delete => 0x2E,
        Key::Home => 0x24,
        Key::End => 0x23,
        Key::PageUp => 0x21,
        Key::PageDown => 0x22,
        Key::ArrowLeft => 0x25,
        Key::ArrowUp => 0x26,
        Key::ArrowRight => 0x27,
        Key::ArrowDown => 0x28,
        Key::Minus => 0xBD,
        Key::Equals => 0xBB,
        Key::OpenBracket => 0xDB,
        Key::CloseBracket => 0xDD,
        Key::Backslash => 0xDC,
        Key::Semicolon => 0xBA,
        Key::Quote => 0xDE,
        Key::Comma => 0xBC,
        Key::Period => 0xBE,
        Key::Slash => 0xBF,
        Key::Backtick => 0xC0,
        _ => return None,
    })
}

pub fn vk_to_label(vk: u32) -> String {
    match vk {
        // Numpad — distinct from top-row digits and operator keys.
        0x60 => "Num0".into(),
        0x61 => "Num1".into(),
        0x62 => "Num2".into(),
        0x63 => "Num3".into(),
        0x64 => "Num4".into(),
        0x65 => "Num5".into(),
        0x66 => "Num6".into(),
        0x67 => "Num7".into(),
        0x68 => "Num8".into(),
        0x69 => "Num9".into(),
        0x6A => "Num*".into(),
        0x6B => "Num+".into(),
        0x6D => "Num-".into(),
        0x6E => "Num.".into(),
        0x6F => "Num/".into(),
        0x70 => "F1".into(),
        0x71 => "F2".into(),
        0x72 => "F3".into(),
        0x73 => "F4".into(),
        0x74 => "F5".into(),
        0x75 => "F6".into(),
        0x76 => "F7".into(),
        0x77 => "F8".into(),
        0x78 => "F9".into(),
        0x79 => "F10".into(),
        0x7A => "F11".into(),
        0x7B => "F12".into(),
        0x7C => "F13".into(),
        0x7D => "F14".into(),
        0x7E => "F15".into(),
        0x7F => "F16".into(),
        0x80 => "F17".into(),
        0x81 => "F18".into(),
        0x82 => "F19".into(),
        0x83 => "F20".into(),
        0x41..=0x5A => ((vk as u8) as char).to_string(),
        0x30..=0x39 => ((vk as u8) as char).to_string(),
        0x20 => "Space".into(),
        0x08 => "Backspace".into(),
        0x09 => "Tab".into(),
        0x0D => "Enter".into(),
        0x2D => "Ins".into(),
        0x2E => "Del".into(),
        0x24 => "Home".into(),
        0x23 => "End".into(),
        0x21 => "PgUp".into(),
        0x22 => "PgDn".into(),
        0x25 => "\u{2190}".into(),
        0x26 => "\u{2191}".into(),
        0x27 => "\u{2192}".into(),
        0x28 => "\u{2193}".into(),
        0xBD => "-".into(),
        0xBB => "=".into(),
        0xDB => "[".into(),
        0xDD => "]".into(),
        0xDC => "\\".into(),
        0xBA => ";".into(),
        0xDE => "'".into(),
        0xBC => ",".into(),
        0xBE => ".".into(),
        0xBF => "/".into(),
        0xC0 => "`".into(),
        _ => format!("VK_0x{vk:02X}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vk_label_function_keys() {
        for (vk, label) in [
            (0x70u32, "F1"),
            (0x77, "F8"),
            (0x7B, "F12"),
            (0x83, "F20"),
        ] {
            assert_eq!(vk_to_label(vk), label);
        }
    }

    #[test]
    fn vk_label_letters_and_digits() {
        assert_eq!(vk_to_label(0x41), "A");
        assert_eq!(vk_to_label(0x5A), "Z");
        assert_eq!(vk_to_label(0x30), "0");
        assert_eq!(vk_to_label(0x39), "9");
    }

    #[test]
    fn vk_label_named_keys() {
        assert_eq!(vk_to_label(0x20), "Space");
        assert_eq!(vk_to_label(0x0D), "Enter");
        assert_eq!(vk_to_label(0x09), "Tab");
        assert_eq!(vk_to_label(0x08), "Backspace");
        assert_eq!(vk_to_label(0x2D), "Ins");
        assert_eq!(vk_to_label(0x2E), "Del");
        assert_eq!(vk_to_label(0x21), "PgUp");
        assert_eq!(vk_to_label(0x22), "PgDn");
    }

    #[test]
    fn vk_label_punctuation() {
        assert_eq!(vk_to_label(0xBD), "-");
        assert_eq!(vk_to_label(0xBB), "=");
        assert_eq!(vk_to_label(0xDB), "[");
        assert_eq!(vk_to_label(0xDD), "]");
        assert_eq!(vk_to_label(0xC0), "`");
    }

    #[test]
    fn vk_label_arrows_use_unicode() {
        assert_eq!(vk_to_label(0x25), "\u{2190}");
        assert_eq!(vk_to_label(0x26), "\u{2191}");
        assert_eq!(vk_to_label(0x27), "\u{2192}");
        assert_eq!(vk_to_label(0x28), "\u{2193}");
    }

    #[test]
    fn vk_label_unknown_falls_back_to_hex() {
        assert_eq!(vk_to_label(0xFE), "VK_0xFE");
        assert_eq!(vk_to_label(0x00), "VK_0x00");
    }

    #[test]
    fn oem_scan_codes_cover_every_punctuation_key() {
        use egui::Key;
        for (key, sc) in [
            (Key::Backtick, 0x29),
            (Key::Minus, 0x0C),
            (Key::Equals, 0x0D),
            (Key::OpenBracket, 0x1A),
            (Key::CloseBracket, 0x1B),
            (Key::Backslash, 0x2B),
            (Key::Semicolon, 0x27),
            (Key::Quote, 0x28),
            (Key::Comma, 0x33),
            (Key::Period, 0x34),
            (Key::Slash, 0x35),
        ] {
            assert_eq!(oem_scan_code(key), Some(sc), "{key:?}");
            assert!(is_oem_vk(key_to_vk(key).unwrap()), "{key:?}");
        }
    }

    #[test]
    fn non_oem_keys_skip_the_layout_lookup() {
        use egui::Key;
        assert_eq!(oem_scan_code(Key::A), None);
        assert_eq!(oem_scan_code(Key::F1), None);
        assert_eq!(event_to_vk(Key::F1, Some(Key::F1)), Some(0x70));
        assert_eq!(event_to_vk(Key::Space, None), Some(0x20));
        assert_eq!(event_to_vk(Key::Escape, Some(Key::Escape)), None);
    }

    #[test]
    fn oem_vk_range() {
        assert!(is_oem_vk(0xDE));
        assert!(is_oem_vk(0xC0));
        assert!(is_oem_vk(0xE2));
        assert!(!is_oem_vk(0x41));
        assert!(!is_oem_vk(0x70));
    }

    #[test]
    fn trigger_display_label() {
        assert_eq!(Trigger::Key(0x70).display_label(), "F1");
        assert_eq!(Trigger::Key(0x41).display_label(), "A");
        assert_eq!(Trigger::Mouse(MouseBtn::Middle).display_label(), "MWheel");
        assert_eq!(Trigger::Mouse(MouseBtn::X1).display_label(), "Mb4");
        assert_eq!(Trigger::Mouse(MouseBtn::X2).display_label(), "Mb5");
        assert_eq!(Trigger::Wheel(WheelDir::Up).display_label(), "MWh\u{2191}");
        assert_eq!(Trigger::Wheel(WheelDir::Down).display_label(), "MWh\u{2193}");
    }
}
