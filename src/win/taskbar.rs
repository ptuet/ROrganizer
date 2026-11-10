//! Reorders Dofus buttons in the Windows taskbar.
//!
//! Windows exposes no API to move a taskbar button. What it does guarantee is
//! that a button re-added with `ITaskbarList::AddTab` lands at the end, so
//! removing and re-adding every window in the wanted order yields that order.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{ITaskbarList, TaskbarList};

/// Pause after each `DeleteTab` and each `AddTab`. Explorer handles these
/// asynchronously: a Delete immediately followed by its Add can be merged
/// into a no-op, leaving that button in place. Measured on a grouped icon
/// with 4 windows, random target orders: no pause → 3/30 wrong, 20 ms →
/// 1/30 wrong, 50 ms → 30/30 OK, 100 ms → 80/80 OK including under full
/// CPU load. 100 ms keeps a margin.
const STEP_DELAY: Duration = Duration::from_millis(100);

/// Set while a reorder runs. Two overlapping passes interleave their
/// Delete/Add calls and produce a wrong order, so a second request is
/// dropped instead of queued.
static SORTING: AtomicBool = AtomicBool::new(false);

/// True while a reorder is in progress (the UI greys its button).
pub fn is_sorting() -> bool {
    SORTING.load(Ordering::Acquire)
}

/// Re-adds each window's taskbar button in `hwnds` order. Runs on a
/// short-lived thread with its own COM apartment, leaving the UI thread's
/// COM state untouched. Ignored while a previous reorder is still running.
/// `on_done` runs at the end (the UI uses it to repaint the button).
/// Best-effort: failures are silently ignored.
pub fn reorder_taskbar(hwnds: Vec<isize>, on_done: impl FnOnce() + Send + 'static) {
    if hwnds.len() < 2 {
        return;
    }
    if SORTING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    thread::spawn(move || {
        unsafe { reorder(&hwnds) };
        SORTING.store(false, Ordering::Release);
        on_done();
    });
}

unsafe fn reorder(hwnds: &[isize]) {
    unsafe {
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            return;
        }
        if let Ok(list) = CoCreateInstance::<_, ITaskbarList>(&TaskbarList, None, CLSCTX_INPROC_SERVER)
            && list.HrInit().is_ok()
        {
            for &h in hwnds {
                let hwnd = HWND(h as *mut _);
                let _ = list.DeleteTab(hwnd);
                thread::sleep(STEP_DELAY);
                let _ = list.AddTab(hwnd);
                thread::sleep(STEP_DELAY);
            }
        }
        CoUninitialize();
    }
}
