use std::sync::atomic::Ordering;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_SYSKEYDOWN,
};

use super::state::{BindingAction, HOOK_ENABLED, HOOK_PAUSED, HOOK_STATE};
use crate::triggers::Trigger;

pub unsafe extern "system" fn kb_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT { unsafe {
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    // Hot-path early-out: lock-free atomic check before touching the mutex.
    if !HOOK_ENABLED.load(Ordering::Relaxed) || HOOK_PAUSED.load(Ordering::Relaxed) {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let msg = wparam.0 as u32;
    if msg != WM_KEYDOWN && msg != WM_SYSKEYDOWN {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
    let vk = info.vkCode;

    let focus_target = HOOK_STATE.get().and_then(|m| {
        let mut state = m.lock().ok()?;
        resolve_action(
            &mut state,
            Trigger::Key(vk),
            crate::win::foreground_window,
            crate::win::is_dofus_window,
        )
    });

    if let Some(hwnd) = focus_target {
        crate::win::focus_window(hwnd);
        return LRESULT(1);
    }

    CallNextHookEx(None, code, wparam, lparam)
}}

/// Looks up the trigger against the binding table and returns the HWND to
/// focus, advancing the cycler cursor as needed. Caller still holds the lock.
///
/// `foreground` (the window in front) is only queried once a binding matches,
/// and only when needed:
/// - with `dofus_only`, nothing happens unless that window is Dofus — a
///   detected account, or any window of a Dofus executable (login screen,
///   secondary window), checked through `is_dofus_window`. `None` lets the
///   caller pass the key on untouched;
/// - for cycle actions, when that window is part of the cycle, the cursor
///   snaps to it first. The user may have switched account with the mouse
///   (taskbar, click) since our last focus, and Next / Prev must continue
///   from where they actually are.
pub(super) fn resolve_action(
    state: &mut super::state::HookState,
    trigger: Trigger,
    foreground: impl FnOnce() -> Option<isize>,
    is_dofus_window: impl FnOnce(isize) -> bool,
) -> Option<isize> {
    let action = state
        .bindings
        .iter()
        .find(|(t, _)| *t == trigger)
        .map(|(_, a)| *a)?;
    let is_cycle = matches!(action, BindingAction::CycleNext | BindingAction::CyclePrev);
    let fg = if state.dofus_only || is_cycle {
        foreground()
    } else {
        None
    };
    if state.dofus_only
        && !fg.is_some_and(|h| state.cycle_hwnds.contains(&h) || is_dofus_window(h))
    {
        return None;
    }
    if is_cycle
        && let Some(fg) = fg
        && let Some(pos) = state.cycle_hwnds.iter().position(|&x| x == fg)
    {
        state.cycle_index = pos;
    }
    let target = match action {
        BindingAction::Focus(h) => {
            // A direct focus binding aligns the cycler on its target so the
            // next CycleNext from there continues the user's expected flow.
            if let Some(pos) = state.cycle_hwnds.iter().position(|&x| x == h) {
                state.cycle_index = pos;
            }
            Some(h)
        }
        BindingAction::CycleNext => {
            let len = state.cycle_hwnds.len();
            if len == 0 {
                return None;
            }
            state.cycle_index = (state.cycle_index + 1) % len;
            Some(state.cycle_hwnds[state.cycle_index])
        }
        BindingAction::CyclePrev => {
            let len = state.cycle_hwnds.len();
            if len == 0 {
                return None;
            }
            state.cycle_index = (state.cycle_index + len - 1) % len;
            Some(state.cycle_hwnds[state.cycle_index])
        }
    };
    if let Some(h) = target {
        state.cycle_current_hwnd = Some(h);
    }
    target
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triggers::Trigger;

    fn make_state(
        bindings: Vec<(Trigger, BindingAction)>,
        cycle_hwnds: Vec<isize>,
    ) -> super::super::state::HookState {
        super::super::state::HookState {
            bindings,
            cycle_hwnds,
            cycle_index: 0,
            cycle_current_hwnd: None,
            dofus_only: false,
        }
    }

    #[test]
    fn empty_bindings_resolves_to_none() {
        let mut st = make_state(Vec::new(), vec![100, 200]);
        assert!(resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false).is_none());
    }

    #[test]
    fn unmatched_trigger_resolves_to_none() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::Focus(100))],
            vec![100],
        );
        assert!(resolve_action(&mut st, Trigger::Key(0x71), || None, |_| false).is_none());
    }

    #[test]
    fn focus_aligns_cycle_index_to_target() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::Focus(300))],
            vec![100, 200, 300, 400],
        );
        st.cycle_index = 0; // start away from 300
        let r = resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false);
        assert_eq!(r, Some(300));
        assert_eq!(st.cycle_index, 2);
        assert_eq!(st.cycle_current_hwnd, Some(300));
    }

    #[test]
    fn focus_does_not_move_cursor_if_target_not_in_cycle() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::Focus(999))],
            vec![100, 200],
        );
        st.cycle_index = 1;
        resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false);
        assert_eq!(st.cycle_index, 1, "untouched when target outside cycle");
        assert_eq!(st.cycle_current_hwnd, Some(999));
    }

    #[test]
    fn cycle_next_wraps_last_to_first() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::CycleNext)],
            vec![100, 200, 300],
        );
        st.cycle_index = 2;
        let r = resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false);
        assert_eq!(r, Some(100));
        assert_eq!(st.cycle_index, 0);
        assert_eq!(st.cycle_current_hwnd, Some(100));
    }

    #[test]
    fn cycle_next_advances_when_not_at_end() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::CycleNext)],
            vec![100, 200, 300],
        );
        st.cycle_index = 0;
        let r = resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false);
        assert_eq!(r, Some(200));
        assert_eq!(st.cycle_index, 1);
    }

    #[test]
    fn cycle_prev_wraps_first_to_last() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::CyclePrev)],
            vec![100, 200, 300],
        );
        st.cycle_index = 0;
        let r = resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false);
        assert_eq!(r, Some(300));
        assert_eq!(st.cycle_index, 2);
    }

    #[test]
    fn cycle_prev_decrements_when_not_at_start() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::CyclePrev)],
            vec![100, 200, 300],
        );
        st.cycle_index = 2;
        let r = resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false);
        assert_eq!(r, Some(200));
        assert_eq!(st.cycle_index, 1);
    }

    #[test]
    fn cycle_next_on_empty_list_returns_none() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::CycleNext)],
            Vec::new(),
        );
        assert!(resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false).is_none());
        assert_eq!(st.cycle_current_hwnd, None);
    }

    #[test]
    fn cycle_prev_on_empty_list_returns_none() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::CyclePrev)],
            Vec::new(),
        );
        assert!(resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false).is_none());
    }

    #[test]
    fn first_matching_binding_wins_on_duplicate_trigger() {
        // App pushes Account focus actions BEFORE Cycle ones, so Account wins.
        let mut st = make_state(
            vec![
                (Trigger::Key(0x70), BindingAction::Focus(500)),
                (Trigger::Key(0x70), BindingAction::CycleNext),
            ],
            vec![100, 200],
        );
        let r = resolve_action(&mut st, Trigger::Key(0x70), || None, |_| false);
        assert_eq!(r, Some(500));
    }

    #[test]
    fn cycle_next_continues_from_window_picked_with_the_mouse() {
        // Cursor on 100 (last hotkey focus), user then clicked 300 in the
        // taskbar: Next must go to 400, not 200.
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::CycleNext)],
            vec![100, 200, 300, 400],
        );
        st.cycle_index = 0;
        let r = resolve_action(&mut st, Trigger::Key(0x70), || Some(300), |_| false);
        assert_eq!(r, Some(400));
        assert_eq!(st.cycle_index, 3);
    }

    #[test]
    fn cycle_prev_continues_from_window_picked_with_the_mouse() {
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::CyclePrev)],
            vec![100, 200, 300, 400],
        );
        st.cycle_index = 0;
        let r = resolve_action(&mut st, Trigger::Key(0x70), || Some(300), |_| false);
        assert_eq!(r, Some(200));
    }

    #[test]
    fn cycle_keeps_cursor_when_foreground_is_not_in_cycle() {
        // A browser / Discord in front: behave as before.
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::CycleNext)],
            vec![100, 200, 300],
        );
        st.cycle_index = 1;
        let r = resolve_action(&mut st, Trigger::Key(0x70), || Some(999), |_| false);
        assert_eq!(r, Some(300));
    }

    #[test]
    fn focus_binding_never_queries_foreground() {
        // Option off: a direct focus binding needs no foreground lookup.
        let mut st = make_state(
            vec![(Trigger::Key(0x70), BindingAction::Focus(200))],
            vec![100, 200],
        );
        let r = resolve_action(
            &mut st,
            Trigger::Key(0x70),
            || panic!("foreground queried for a focus binding"),
            |_| panic!("process checked for a focus binding"),
        );
        assert_eq!(r, Some(200));
    }

    fn dofus_only_state(binding: BindingAction) -> super::super::state::HookState {
        let mut st = make_state(vec![(Trigger::Key(0x41), binding)], vec![100, 200, 300]);
        st.dofus_only = true;
        st
    }

    #[test]
    fn dofus_only_ignores_the_key_outside_dofus() {
        // "A" typed in a browser (hwnd 999, not a Dofus process): the key
        // must go through, and the cycle cursor must not move.
        for binding in [
            BindingAction::Focus(200),
            BindingAction::CycleNext,
            BindingAction::CyclePrev,
        ] {
            let mut st = dofus_only_state(binding);
            st.cycle_index = 1;
            let r = resolve_action(&mut st, Trigger::Key(0x41), || Some(999), |_| false);
            assert_eq!(r, None, "{binding:?}");
            assert_eq!(st.cycle_index, 1, "{binding:?}");
            assert_eq!(st.cycle_current_hwnd, None, "{binding:?}");
        }
    }

    #[test]
    fn dofus_only_ignores_the_key_without_foreground_window() {
        let mut st = dofus_only_state(BindingAction::Focus(200));
        assert_eq!(resolve_action(&mut st, Trigger::Key(0x41), || None, |_| true), None);
    }

    #[test]
    fn dofus_only_acts_from_a_detected_account() {
        let mut st = dofus_only_state(BindingAction::CycleNext);
        let r = resolve_action(
            &mut st,
            Trigger::Key(0x41),
            || Some(100),
            |_| panic!("no process lookup needed for a detected account"),
        );
        assert_eq!(r, Some(200));
    }

    #[test]
    fn dofus_only_acts_from_another_dofus_window() {
        // Login screen of a client: a Dofus process, not in the account list.
        let mut st = dofus_only_state(BindingAction::Focus(300));
        let r = resolve_action(&mut st, Trigger::Key(0x41), || Some(555), |h| h == 555);
        assert_eq!(r, Some(300));
    }

    #[test]
    fn dofus_only_does_not_touch_unbound_keys() {
        // Unbound key: no foreground lookup at all, whatever the option.
        let mut st = dofus_only_state(BindingAction::Focus(200));
        let r = resolve_action(
            &mut st,
            Trigger::Key(0x42),
            || panic!("foreground queried for an unbound key"),
            |_| panic!("process checked for an unbound key"),
        );
        assert_eq!(r, None);
    }
}
