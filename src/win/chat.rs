//! Sends chat commands to Dofus windows (group invite, zaap for everyone).
//!
//! This is the only place where ROrganizer sends input to the game. The
//! sequence mirrors what the Zaapy project validated on Dofus 3:
//!
//! - commands are **pasted** (clipboard + Ctrl+V), not typed letter by letter;
//! - every injected key carries its **scan code**: Dofus reads the physical
//!   key, a virtual key code alone is ignored;
//! - every key is **held** a few ms, in separate batches: the client polls
//!   once per frame and would miss a press and release sent together;
//! - after a command the chat **keeps the keyboard**, Escape gives it back.
//!
//! In each window the chat is opened once, every command is pasted and sent
//! in a row, and Escape closes it at the end:
//! open chat (Space by default) → (Ctrl+V → Enter) × N → Escape. The keys of
//! the game (open chat, follow the leader) are settings: `game_keys`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC, VIRTUAL_KEY, VK_CONTROL, VK_ESCAPE, VK_RETURN,
};

use super::focus::{focus_window, foreground_window};
use crate::game_keys::{GameKey, GameKeys};

const CF_UNICODETEXT: u32 = 13;
const VK_V: VIRTUAL_KEY = VIRTUAL_KEY(0x56);
const VK_SHIFT: VIRTUAL_KEY = VIRTUAL_KEY(0x10);
const VK_ALT: VIRTUAL_KEY = VIRTUAL_KEY(0x12);

/// How long each key stays down.
const KEY_HOLD: Duration = Duration::from_millis(30);
/// After opening the chat, and between paste and send: the input needs a
/// few frames to take the text.
const STEP_DELAY: Duration = Duration::from_millis(120);
/// After a command is sent: lets the game process it and clear the input
/// before the next paste (or the final Escape).
const AFTER_SEND: Duration = Duration::from_millis(150);
/// Maximum wait for the target window to come to the foreground.
const FOCUS_TIMEOUT: Duration = Duration::from_millis(800);
/// Once in front, lets the client settle before the first key.
const FOCUS_SETTLE: Duration = Duration::from_millis(200);

/// Set while commands are being sent: a second request is dropped, two
/// sequences would interleave their keys.
static SENDING: AtomicBool = AtomicBool::new(false);

/// True while commands are being sent (the UI greys its button).
pub fn is_busy() -> bool {
    SENDING.load(Ordering::Acquire)
}

/// One step of a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Commands typed in the chat of this window.
    Type(isize, Vec<String>),
    /// "Follow the group leader" shortcut (Ctrl+W, the game's default)
    /// pressed in this window.
    FollowLeader(isize),
    /// Wait, e.g. for maps to load after a teleport.
    Pause(Duration),
}

/// Runs the actions in order on a background thread, then brings
/// `finish_on` to the front. The global shortcuts are paused meanwhile, so
/// the injected keys can't trigger a binding. Stops everything if a window
/// won't come to the front or the user brings another one forward. The
/// clipboard's text is restored at the end. `on_done` runs once finished
/// (the UI uses it to repaint).
pub fn send_plan(
    plan: Vec<Action>,
    keys: GameKeys,
    finish_on: Option<isize>,
    on_done: impl FnOnce() + Send + 'static,
) {
    let has_input = plan.iter().any(|a| match a {
        Action::Type(_, cmds) => !cmds.is_empty(),
        Action::FollowLeader(_) => true,
        Action::Pause(_) => false,
    });
    if !has_input
        || SENDING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
    {
        return;
    }
    thread::spawn(move || {
        crate::hooks::set_paused(true);
        let saved = read_clipboard_text();
        let completed = plan.iter().all(|action| match action {
            Action::Type(hwnd, cmds) => run(*hwnd, cmds, keys.open_chat),
            Action::FollowLeader(hwnd) => follow_leader(*hwnd, keys.follow_leader),
            Action::Pause(d) => {
                thread::sleep(*d);
                true
            }
        });
        if completed && let Some(hwnd) = finish_on {
            focus_window(hwnd);
        }
        match saved {
            Some(text) => {
                set_clipboard_text(&text);
            }
            None => clear_clipboard(),
        }
        crate::hooks::set_paused(false);
        SENDING.store(false, Ordering::Release);
        on_done();
    });
}

/// Types `commands` in `target`. False if it had to stop (window not in
/// front), so the caller doesn't go on with the next windows.
fn run(target: isize, commands: &[String], open_chat: GameKey) -> bool {
    if commands.is_empty() {
        return true;
    }
    if !bring_to_front(target) {
        return false;
    }

    tap_key(open_chat);
    thread::sleep(STEP_DELAY);
    for cmd in commands {
        // The user switched window: stop rather than type into it.
        if foreground_window() != Some(target) || !set_clipboard_text(cmd) {
            return false;
        }
        press(VK_CONTROL);
        thread::sleep(KEY_HOLD);
        tap(VK_V);
        release(VK_CONTROL);
        thread::sleep(STEP_DELAY);
        tap(VK_RETURN);
        thread::sleep(AFTER_SEND);
    }
    if foreground_window() != Some(target) {
        return false;
    }
    tap(VK_ESCAPE);
    true
}

/// Focuses `target` and waits until it is really in front. False if it
/// won't come.
fn bring_to_front(target: isize) -> bool {
    focus_window(target);
    let start = Instant::now();
    while foreground_window() != Some(target) {
        if start.elapsed() > FOCUS_TIMEOUT {
            return false;
        }
        thread::sleep(Duration::from_millis(25));
    }
    thread::sleep(FOCUS_SETTLE);
    true
}

/// Presses the "follow the group leader" shortcut (Ctrl+W by default) in
/// `target`.
fn follow_leader(target: isize, key: GameKey) -> bool {
    if !bring_to_front(target) {
        return false;
    }
    tap_key(key);
    thread::sleep(AFTER_SEND);
    true
}

/// Presses a game key with its modifiers: modifiers down, key tapped,
/// modifiers up in reverse order. Each key goes with its scan code on the
/// user's layout, as if pressed by hand (the key labelled W, on AZERTY too).
fn tap_key(key: GameKey) {
    let mods: Vec<VIRTUAL_KEY> = [(key.ctrl, VK_CONTROL), (key.shift, VK_SHIFT), (key.alt, VK_ALT)]
        .into_iter()
        .filter_map(|(on, vk)| on.then_some(vk))
        .collect();
    for &m in &mods {
        press(m);
    }
    if !mods.is_empty() {
        thread::sleep(KEY_HOLD);
    }
    tap(VIRTUAL_KEY(key.vk as u16));
    for &m in mods.iter().rev() {
        release(m);
    }
}

fn key_event(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    let scan = unsafe { MapVirtualKeyW(u32::from(vk.0), MAPVK_VK_TO_VSC) } as u16;
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn press(vk: VIRTUAL_KEY) {
    unsafe { SendInput(&[key_event(vk, false)], size_of::<INPUT>() as i32) };
}

fn release(vk: VIRTUAL_KEY) {
    unsafe { SendInput(&[key_event(vk, true)], size_of::<INPUT>() as i32) };
}

fn tap(vk: VIRTUAL_KEY) {
    press(vk);
    thread::sleep(KEY_HOLD);
    release(vk);
}

/// Runs `body` with the clipboard open, retrying briefly while another
/// process holds it.
fn with_clipboard<T>(body: impl FnOnce() -> T) -> Option<T> {
    for _ in 0..5 {
        if unsafe { OpenClipboard(None) }.is_ok() {
            let out = body();
            unsafe {
                let _ = CloseClipboard();
            }
            return Some(out);
        }
        thread::sleep(Duration::from_millis(10));
    }
    None
}

fn read_clipboard_text() -> Option<String> {
    with_clipboard(|| unsafe {
        let handle = GetClipboardData(CF_UNICODETEXT).ok()?;
        let mem = HGLOBAL(handle.0);
        let ptr = GlobalLock(mem) as *const u16;
        if ptr.is_null() {
            return None;
        }
        let max = GlobalSize(mem) / 2;
        let len = (0..max).take_while(|&i| *ptr.add(i) != 0).count();
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
        let _ = GlobalUnlock(mem);
        Some(text)
    })
    .flatten()
}

fn set_clipboard_text(text: &str) -> bool {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    with_clipboard(|| unsafe {
        if EmptyClipboard().is_err() {
            return false;
        }
        let Ok(mem) = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2) else {
            return false;
        };
        let ptr = GlobalLock(mem) as *mut u16;
        if ptr.is_null() {
            let _ = GlobalFree(Some(mem));
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        let _ = GlobalUnlock(mem);
        // On success the system owns `mem`; on failure it is still ours.
        if SetClipboardData(CF_UNICODETEXT, Some(HANDLE(mem.0))).is_err() {
            let _ = GlobalFree(Some(mem));
            return false;
        }
        true
    })
    .unwrap_or(false)
}

fn clear_clipboard() {
    with_clipboard(|| unsafe {
        let _ = EmptyClipboard();
    });
}

/// `/invite` commands sent from the first account to every other one.
pub fn invite_commands<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    names.into_iter().map(|n| format!("/invite {n}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{invite_commands, read_clipboard_text, set_clipboard_text};

    /// Touches the real system clipboard (restored at the end), so it only
    /// runs on demand: `cargo test -- --ignored clipboard`.
    #[test]
    #[ignore]
    fn clipboard_round_trip() {
        let saved = read_clipboard_text();
        assert!(set_clipboard_text("/invite Été-Ñandú"));
        assert_eq!(read_clipboard_text().as_deref(), Some("/invite Été-Ñandú"));
        if let Some(text) = saved {
            set_clipboard_text(&text);
        }
    }

    #[test]
    fn one_invite_per_name_in_order() {
        assert_eq!(
            invite_commands(["Bravo", "Charlie-Farm"]),
            vec!["/invite Bravo", "/invite Charlie-Farm"]
        );
    }

    #[test]
    fn no_names_no_commands() {
        assert!(invite_commands([]).is_empty());
    }
}
