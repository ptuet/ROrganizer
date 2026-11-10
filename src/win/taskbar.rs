//! Reorders Dofus buttons in the Windows taskbar.
//!
//! Windows exposes no API to move a taskbar button. What it does guarantee is
//! that a button re-added with `ITaskbarList::AddTab` lands at the end, so
//! removing and re-adding every window in the wanted order yields that order.

use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{ITaskbarList, TaskbarList};

/// Lets Explorer process each re-add before the next one, so the buttons
/// keep the requested order.
const STEP_DELAY: Duration = Duration::from_millis(30);

/// Re-adds each window's taskbar button in `hwnds` order. Runs on a
/// short-lived thread with its own COM apartment, leaving the UI thread's
/// COM state untouched. Best-effort: failures are silently ignored.
pub fn reorder_taskbar(hwnds: Vec<isize>) {
    if hwnds.len() < 2 {
        return;
    }
    thread::spawn(move || unsafe {
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            return;
        }
        if let Ok(list) = CoCreateInstance::<_, ITaskbarList>(&TaskbarList, None, CLSCTX_INPROC_SERVER)
            && list.HrInit().is_ok()
        {
            for h in hwnds {
                let hwnd = HWND(h as *mut _);
                let _ = list.DeleteTab(hwnd);
                let _ = list.AddTab(hwnd);
                thread::sleep(STEP_DELAY);
            }
        }
        CoUninitialize();
    });
}
