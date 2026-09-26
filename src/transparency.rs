#[cfg(windows)]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub(crate) fn hold_transparency(frame: &eframe::Frame) {
    // A screenshot clears the DWM blur region, and the plate turns solid.
    // Putting the empty region back keeps the unpainted pixels transparent.
    #[cfg(windows)]
    {
        let Ok(handle) = frame.window_handle() else {
            return;
        };
        let RawWindowHandle::Win32(window) = handle.as_raw() else {
            return;
        };
        keep_client_transparent(window.hwnd.get());
    }
    #[cfg(not(windows))]
    {
        let _ = frame;
    }
}

#[cfg(windows)]
fn keep_client_transparent(hwnd: isize) {
    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmEnableBlurBehindWindow(hwnd: isize, blur: *const DwmBlurBehind) -> i32;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn CreateRectRgn(left: i32, top: i32, right: i32, bottom: i32) -> isize;
    }

    #[repr(C)]
    struct DwmBlurBehind {
        flags: u32,
        enable: i32,
        region: isize,
        transition_on_maximized: i32,
    }

    const ENABLE: u32 = 0x1;
    const BLUR_REGION: u32 = 0x2;

    static REGION: std::sync::OnceLock<isize> = std::sync::OnceLock::new();
    let region = *REGION.get_or_init(|| unsafe { CreateRectRgn(0, 0, -1, -1) });
    if region == 0 {
        return;
    }
    let blur = DwmBlurBehind {
        flags: ENABLE | BLUR_REGION,
        enable: 1,
        region,
        transition_on_maximized: 0,
    };
    unsafe {
        DwmEnableBlurBehindWindow(hwnd, &blur);
    }
}
