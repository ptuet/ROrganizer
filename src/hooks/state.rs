use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Instant;
use std::sync::{Mutex, OnceLock};

use crate::triggers::Trigger;

/// What the hook should do when a Trigger matches.
#[derive(Debug, Clone, Copy)]
pub enum BindingAction {
    Focus(isize),
    CycleNext,
    CyclePrev,
}

pub struct HookState {
    /// Trigger → action table, sorted so Account focus actions appear before
    /// Cycle ones (priority on conflict).
    pub bindings: Vec<(Trigger, BindingAction)>,
    /// Live HWNDs of the cycle in user-defined order, with dead slots filtered out.
    pub cycle_hwnds: Vec<isize>,
    /// Current position of the cycler cursor inside `cycle_hwnds`.
    pub cycle_index: usize,
    /// Last HWND the cycler / a focus binding actually focused. Tracked here
    /// so we can re-locate it inside `cycle_hwnds` after a reorder, keeping
    /// the user "on the same account" across drags.
    pub cycle_current_hwnd: Option<isize>,
    /// Bindings only fire while a Dofus window is in front; elsewhere the
    /// key / button goes through untouched (an "A" still types in a browser).
    pub dofus_only: bool,
    /// ROrganizer's own window: counts as Dofus for `dofus_only`, so a
    /// shortcut still works right after a click on the app (mini window
    /// kept on top of the game).
    pub own_hwnd: Option<isize>,
}

pub static HOOK_STATE: OnceLock<Mutex<HookState>> = OnceLock::new();

/// Master switch on the hook callbacks. Read on every keystroke / mouse
/// event, so we keep it as an atomic to short-circuit *before* taking
/// the `HOOK_STATE` mutex when the user has the app inactive.
pub static HOOK_ENABLED: AtomicBool = AtomicBool::new(false);

/// Set while ROrganizer itself injects keys (chat commands), so they can't
/// fire a binding. Separate from `HOOK_ENABLED`, which belongs to the user.
pub static HOOK_PAUSED: AtomicBool = AtomicBool::new(false);


/// Last key the keyboard hook acted on: `vk << 32 | now_ms()`. The app
/// handles keys itself while its window is in front (the hook gets nothing
/// then); this keeps it from acting a second time if both ever see one.
pub static LAST_HOOK_KEY: AtomicU64 = AtomicU64::new(0);

/// Milliseconds since the first call, truncated to 32 bits (compared with
/// `wrapping_sub`).
pub fn now_ms() -> u32 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u32
}
