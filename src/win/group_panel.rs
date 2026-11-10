//! Reads the initiative of every group member from the Dofus group panel
//! (experimental).
//!
//! The panel only shows portraits; name and initiative appear in the tooltip
//! of a hovered portrait. For each calibrated portrait position this module
//! moves the mouse over it, waits for the tooltip, captures the area around
//! the cursor and reads it with the OCR built into Windows
//! (`Windows.Media.Ocr`). No click, nothing sent to the game but mouse moves.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Security::Cryptography::CryptographicBuffer;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
    GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_MOVE, MOUSEINPUT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetCursorPos, GetWindowRect, SetCursorPos, WindowFromPoint, GA_ROOT,
};

use super::focus::{focus_window, foreground_window};
use crate::initiative::{parse_tooltip, Member, PanelCalibration};

/// A group has at most 8 members.
const MAX_MEMBERS: i32 = 8;
/// Time for the tooltip to appear (and the previous one to go away).
const HOVER_WAIT: Duration = Duration::from_millis(450);
/// Area captured around the cursor, which must hold the whole tooltip
/// whatever side of the portrait it opens on.
const CAPTURE_LEFT: i32 = 120;
const CAPTURE_TOP: i32 = 220;
const CAPTURE_W: i32 = 640;
const CAPTURE_H: i32 = 520;
const FOCUS_TIMEOUT: Duration = Duration::from_millis(800);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Members read, in group panel order.
    Read(Vec<Member>),
    /// No tooltip on the first portrait: panel moved, or not in a group.
    PanelNotFound,
    /// The calibrated window could not be brought to the front.
    WindowUnavailable,
}

static BUSY: AtomicBool = AtomicBool::new(false);
static OUTCOME: Mutex<Option<Outcome>> = Mutex::new(None);

pub fn is_busy() -> bool {
    BUSY.load(Ordering::Acquire)
}

/// Result of the last finished run, once.
pub fn take_outcome() -> Option<Outcome> {
    OUTCOME.lock().ok()?.take()
}

/// Hovers every portrait of the calibrated window on a background thread.
/// `on_done` runs when the outcome is ready to take.
pub fn read_group(
    hwnd: isize,
    calib: PanelCalibration,
    on_done: impl FnOnce() + Send + 'static,
) {
    if BUSY
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    thread::spawn(move || {
        let back_to = cursor_pos();
        let outcome = run(hwnd, calib);
        if let Some((x, y)) = back_to {
            unsafe {
                let _ = SetCursorPos(x, y);
            }
        }
        if let Ok(mut slot) = OUTCOME.lock() {
            *slot = Some(outcome);
        }
        BUSY.store(false, Ordering::Release);
        on_done();
    });
}

fn run(hwnd: isize, calib: PanelCalibration) -> Outcome {
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
    focus_window(hwnd);
    let start = Instant::now();
    while foreground_window() != Some(hwnd) {
        if start.elapsed() > FOCUS_TIMEOUT {
            return Outcome::WindowUnavailable;
        }
        thread::sleep(Duration::from_millis(25));
    }
    let Some(origin) = window_origin(hwnd) else {
        return Outcome::WindowUnavailable;
    };

    let mut members = Vec::new();
    for i in 0..MAX_MEMBERS {
        let (dx, dy) = calib.portrait(i);
        let (x, y) = (origin.0 + dx, origin.1 + dy);
        hover(x, y);
        thread::sleep(HOVER_WAIT);
        let lines = capture(x - CAPTURE_LEFT, y - CAPTURE_TOP, CAPTURE_W, CAPTURE_H)
            .and_then(|(px, w, h)| ocr(&px, w, h))
            .unwrap_or_default();
        match parse_tooltip(&lines) {
            // The same member twice: the portrait list has ended and the
            // tooltip of the last one is still on screen.
            Some(m) if members.iter().any(|p: &Member| p.name == m.name) => break,
            Some(m) => members.push(m),
            None => break,
        }
    }
    if members.is_empty() {
        Outcome::PanelNotFound
    } else {
        Outcome::Read(members)
    }
}

/// Top-level window under the cursor and the cursor's offset from that
/// window's top-left corner. Used by the calibration.
pub fn cursor_in_window() -> Option<(isize, (i32, i32))> {
    let (x, y) = cursor_pos()?;
    let hwnd = unsafe { GetAncestor(WindowFromPoint(POINT { x, y }), GA_ROOT) };
    if hwnd.0.is_null() {
        return None;
    }
    let (ox, oy) = window_origin(hwnd.0 as isize)?;
    Some((hwnd.0 as isize, (x - ox, y - oy)))
}

fn cursor_pos() -> Option<(i32, i32)> {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p) }.ok()?;
    Some((p.x, p.y))
}

fn window_origin(hwnd: isize) -> Option<(i32, i32)> {
    let mut r = RECT::default();
    unsafe { GetWindowRect(HWND(hwnd as *mut _), &mut r) }.ok()?;
    Some((r.left, r.top))
}

/// Puts the cursor on (x, y) then nudges it with real move events: a bare
/// SetCursorPos does not make the game show a hover tooltip.
fn hover(x: i32, y: i32) {
    unsafe {
        let _ = SetCursorPos(x - 2, y - 2);
    }
    for _ in 0..2 {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT { dx: 1, dy: 1, dwFlags: MOUSEEVENTF_MOVE, ..Default::default() },
            },
        };
        unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
        thread::sleep(Duration::from_millis(15));
    }
}

/// Screen pixels of the rectangle, BGRA top-down, alpha forced opaque.
fn capture(x: i32, y: i32, w: i32, h: i32) -> Option<(Vec<u8>, i32, i32)> {
    unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(mem, bmp.into());
        let copied = BitBlt(mem, 0, 0, w, h, Some(screen), x, y, SRCCOPY).is_ok();
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut px = vec![0u8; (w * h * 4) as usize];
        let lines = GetDIBits(
            mem,
            bmp,
            0,
            h as u32,
            Some(px.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        SelectObject(mem, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        if !copied || lines != h {
            return None;
        }
        px.chunks_exact_mut(4).for_each(|p| p[3] = 255);
        Some((px, w, h))
    }
}

/// OCR lines of a BGRA image, read at 2× size: the game font at 1× loses
/// labels such as "Initiative".
fn ocr(bgra: &[u8], w: i32, h: i32) -> Option<Vec<String>> {
    let img = image::RgbaImage::from_raw(w as u32, h as u32, bgra.to_vec())?;
    let big = image::imageops::resize(
        &img,
        img.width() * 2,
        img.height() * 2,
        image::imageops::FilterType::CatmullRom,
    );
    let buffer = CryptographicBuffer::CreateFromByteArray(big.as_raw()).ok()?;
    let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &buffer,
        BitmapPixelFormat::Bgra8,
        big.width() as i32,
        big.height() as i32,
    )
    .ok()?;
    let engine = OcrEngine::TryCreateFromUserProfileLanguages().ok()?;
    let result = engine.RecognizeAsync(&bitmap).ok()?.join().ok()?;
    let lines = result.Lines().ok()?;
    Some(
        (&lines)
            .into_iter()
            .filter_map(|l| l.Text().ok().map(|t| t.to_string()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs the real OCR on a screenshot of a group tooltip. On demand only:
    /// `ROR_OCR_SAMPLE=<png> cargo test -- --ignored ocr_reads`.
    #[test]
    #[ignore]
    fn ocr_reads_a_group_tooltip_screenshot() {
        let path = std::env::var("ROR_OCR_SAMPLE").expect("set ROR_OCR_SAMPLE");
        let rgba = image::open(path).expect("png").to_rgba8();
        let (w, h) = (rgba.width() as i32, rgba.height() as i32);
        let bgra: Vec<u8> = rgba
            .chunks_exact(4)
            .flat_map(|p| [p[2], p[1], p[0], 255])
            .collect();
        unsafe {
            let _ = RoInitialize(RO_INIT_MULTITHREADED);
        }
        let lines = ocr(&bgra, w, h).expect("ocr");
        println!("{lines:#?}");
        println!("=> {:?}", parse_tooltip(&lines));
    }
}
