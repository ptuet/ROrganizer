//! Moves our own window: centred on the screen it is on, or back to a
//! saved spot. Done in Win32 rather than with `ViewportCommand::OuterPosition`
//! because egui doesn't tell which monitor the window is on, nor where that
//! monitor starts on the desktop.
//!
//! Positions are physical pixels (top-left corner), as Win32 gives them.

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    MONITOR_DEFAULTTONULL,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
};

/// Top-left corner of the window.
pub fn position(hwnd: isize) -> Option<(i32, i32)> {
    let mut r = RECT::default();
    unsafe { GetWindowRect(HWND(hwnd as *mut _), &mut r) }.ok()?;
    Some((r.left, r.top))
}

/// A window placed at `pos` would show on a connected screen (a point a
/// little inside its corner is tested, so a sliver on screen doesn't count).
pub fn is_on_screen((x, y): (i32, i32)) -> bool {
    let probe = POINT { x: x + 20, y: y + 10 };
    !unsafe { MonitorFromPoint(probe, MONITOR_DEFAULTTONULL) }.is_invalid()
}

/// A window of `size` (physical px) placed at `pos` would show whole: its
/// top-left and bottom-right corners (a little inside) are on a screen.
pub fn fits_on_screen((x, y): (i32, i32), (w, h): (i32, i32)) -> bool {
    is_on_screen((x, y)) && is_on_screen((x + w - 40, y + h - 20))
}

/// Moves the window, size and z-order untouched.
pub fn move_to(hwnd: isize, (x, y): (i32, i32)) {
    unsafe {
        let _ = SetWindowPos(
            HWND(hwnd as *mut _),
            None,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

/// Moves the window so that a window of `size` (logical points) is centred
/// in the work area (screen minus taskbar) of the monitor it is on. The
/// size is passed in because a resize asked to eframe in the same frame is
/// only applied afterwards.
pub fn center(hwnd: isize, size: (f32, f32)) {
    center_on(hwnd, hwnd, size);
}

/// Same as `center`, on the screen of another window (`screen_of`): the
/// picker opened from the mini window goes where the game is.
///
/// The size in pixels is computed with *that* screen's scale: screens can
/// differ (100 % / 125 %), and Windows rescales the window once it lands
/// there, which would leave it off-centre.
pub fn center_on(hwnd: isize, screen_of: isize, (w, h): (f32, f32)) {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let (mut dpi_x, mut dpi_y) = (96, 96);
    let ok = unsafe {
        let monitor = MonitorFromWindow(HWND(screen_of as *mut _), MONITOR_DEFAULTTONEAREST);
        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
        GetMonitorInfoW(monitor, &mut info).as_bool()
    };
    if ok {
        let scale = dpi_x as f32 / 96.0;
        let size = ((w * scale).round() as i32, (h * scale).round() as i32);
        move_to(hwnd, centered(info.rcWork, size));
    }
}

/// Top-left corner that centres `size` in `area`, kept inside it when the
/// window is larger than the area.
fn centered(area: RECT, (w, h): (i32, i32)) -> (i32, i32) {
    let x = area.left + ((area.right - area.left - w) / 2).max(0);
    let y = area.top + ((area.bottom - area.top - h) / 2).max(0);
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT { left, top, right, bottom }
    }

    #[test]
    fn centres_in_the_work_area() {
        assert_eq!(centered(rect(0, 0, 1920, 1040), (340, 420)), (790, 310));
    }

    #[test]
    fn follows_a_monitor_left_of_the_primary() {
        assert_eq!(centered(rect(-1280, 0, 0, 984), (340, 484)), (-810, 250));
    }

    #[test]
    fn larger_window_sticks_to_the_top_left_corner() {
        assert_eq!(centered(rect(100, 50, 400, 350), (340, 484)), (100, 50));
    }
}
