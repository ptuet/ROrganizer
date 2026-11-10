//! Queries against the active keyboard layout.
//!
//! The binding modal reads keys through egui, which names them after their
//! position on a US-QWERTY board; the LL hook receives the real VK of the
//! user's layout. For OEM keys the two disagree (AZERTY `²` is VK 0xDE, not
//! the US backtick 0xC0), so both capture and display ask Windows directly.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, MAPVK_VK_TO_CHAR, MAPVK_VSC_TO_VK,
};

/// VK produced by `scan_code` on the calling thread's layout.
pub fn scan_code_to_vk(scan_code: u32) -> Option<u32> {
    let vk = unsafe { MapVirtualKeyW(scan_code, MAPVK_VSC_TO_VK) };
    (vk != 0).then_some(vk)
}

/// Unshifted character printed by `vk` on the calling thread's layout.
pub fn vk_to_char(vk: u32) -> Option<char> {
    let raw = unsafe { MapVirtualKeyW(vk, MAPVK_VK_TO_CHAR) };
    // Dead keys (AZERTY `^`, `¨`) come back with the top bit set.
    let c = char::from_u32(raw & 0x7FFF_FFFF)?;
    (c != '\0' && !c.is_control()).then_some(c)
}
