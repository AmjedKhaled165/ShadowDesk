use crate::native::WindowInfo;
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::info;
use windows::Win32::{
    Foundation::HWND,
    System::Threading::{AttachThreadInput, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::{
            MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, RegisterHotKey,
            UnregisterHotKey, VK_F7, VK_F8,
        },
        WindowsAndMessaging::{
            BringWindowToTop, GetForegroundWindow,
            GetWindowDisplayAffinity, GetWindowThreadProcessId, IsIconic, SetForegroundWindow,
            ShowWindow, MSG, SW_MINIMIZE, SW_RESTORE, WDA_EXCLUDEFROMCAPTURE, WM_HOTKEY,
        },
    },
};

pub const HOTKEY_ID_CTRL_SHIFT_H: i32 = 1001;
pub const HOTKEY_ID_F8: i32 = 1002;
pub const HOTKEY_ID_ALT_H: i32 = 1003;
pub const HOTKEY_ID_F7: i32 = 1004;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyPreset {
    Both,
    F8Only,
    CtrlShiftHOnly,
    AltHOnly,
    Disabled,
}

impl HotkeyPreset {
    pub fn name(&self) -> &'static str {
        match self {
            HotkeyPreset::Both => "F8 & Ctrl+Shift+H (Recommended)",
            HotkeyPreset::F8Only => "F8 (Single key)",
            HotkeyPreset::CtrlShiftHOnly => "Ctrl + Shift + H",
            HotkeyPreset::AltHOnly => "Alt + H",
            HotkeyPreset::Disabled => "Disabled",
        }
    }
}

pub fn toggle_hidden_windows(windows: &Arc<Mutex<Vec<WindowInfo>>>) {
    let my_pid = std::process::id();
    let hidden_windows: Vec<(u32, u32)> = {
        let list = windows.lock().unwrap();
        list.iter()
            .filter(|w| w.hidden && w.pid != my_pid)
            .map(|w| (w.pid, w.hwnd))
            .collect()
    };

    if hidden_windows.is_empty() {
        info!("Hotkey pressed: No windows are currently marked as hidden.");
        return;
    }

    // Check if any of the hidden windows is currently minimized
    let any_minimized = hidden_windows.iter().any(|&(_pid, hwnd)| {
        unsafe { IsIconic(HWND(hwnd as *mut _)).as_bool() }
    });

    let fg_hwnd = unsafe { GetForegroundWindow() };

    if any_minimized {
        // State 1: Hidden windows are minimized -> Restore & bring to front!
        info!("Hotkey toggle: Restoring hidden windows to front");
        for &(pid, hwnd_raw) in &hidden_windows {
            let hwnd = HWND(hwnd_raw as *mut _);
            unsafe {
                let _ = ShowWindow(hwnd, SW_RESTORE);

                // Double check and ensure WDA_EXCLUDEFROMCAPTURE stays active
                let mut affinity = 0u32;
                let _ = GetWindowDisplayAffinity(hwnd, &mut affinity);
                if affinity != WDA_EXCLUDEFROMCAPTURE.0 {
                    let _ = crate::native::Injector::set_window_props(pid, &[hwnd_raw], true, None);
                }
            }
        }

        // Set foreground to the first hidden window
        if let Some(&(_pid, first_hwnd_raw)) = hidden_windows.first() {
            let target_hwnd = HWND(first_hwnd_raw as *mut _);
            unsafe {
                let mut fg_pid = 0u32;
                let fg_thread = GetWindowThreadProcessId(fg_hwnd, Some(&mut fg_pid));
                let current_thread = GetCurrentThreadId();

                if fg_thread != 0 && fg_thread != current_thread {
                    let _ = AttachThreadInput(fg_thread, current_thread, true);
                    let _ = SetForegroundWindow(target_hwnd);
                    let _ = BringWindowToTop(target_hwnd);
                    let _ = AttachThreadInput(fg_thread, current_thread, false);
                } else {
                    let _ = SetForegroundWindow(target_hwnd);
                    let _ = BringWindowToTop(target_hwnd);
                }
            }
        }
    } else {
        // State 2: Hidden windows are currently visible on user screen -> Minimize them!
        info!("Hotkey toggle: Minimizing hidden windows away from monitor");
        for &(_pid, hwnd_raw) in &hidden_windows {
            let hwnd = HWND(hwnd_raw as *mut _);
            unsafe {
                let _ = ShowWindow(hwnd, SW_MINIMIZE);
            }
        }
    }
}

pub fn toggle_shadowdesk_window() {
    if let Some(hwnd) = crate::native::get_own_hwnd() {
        unsafe {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = SetForegroundWindow(hwnd);
                let _ = BringWindowToTop(hwnd);
            } else {
                let fg = GetForegroundWindow();
                if fg == hwnd {
                    let _ = ShowWindow(hwnd, SW_MINIMIZE);
                } else {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                    let _ = BringWindowToTop(hwnd);
                }
            }
        }
    }
}

pub fn start_hotkey_listener(
    windows: Arc<Mutex<Vec<WindowInfo>>>,
    preset: Arc<Mutex<HotkeyPreset>>,
) {
    thread::spawn(move || {
        let register = |p: HotkeyPreset| {
            unsafe {
                // Clear existing
                let _ = UnregisterHotKey(None, HOTKEY_ID_CTRL_SHIFT_H);
                let _ = UnregisterHotKey(None, HOTKEY_ID_F8);
                let _ = UnregisterHotKey(None, HOTKEY_ID_ALT_H);
                let _ = UnregisterHotKey(None, HOTKEY_ID_F7);

                // F7 is always registered for toggling ShadowDesk itself
                let _ = RegisterHotKey(
                    None,
                    HOTKEY_ID_F7,
                    MOD_NOREPEAT,
                    VK_F7.0 as u32,
                );

                match p {
                    HotkeyPreset::Both => {
                        let _ = RegisterHotKey(
                            None,
                            HOTKEY_ID_CTRL_SHIFT_H,
                            MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT,
                            b'H' as u32,
                        );
                        let _ = RegisterHotKey(
                            None,
                            HOTKEY_ID_F8,
                            MOD_NOREPEAT,
                            VK_F8.0 as u32,
                        );
                    }
                    HotkeyPreset::F8Only => {
                        let _ = RegisterHotKey(
                            None,
                            HOTKEY_ID_F8,
                            MOD_NOREPEAT,
                            VK_F8.0 as u32,
                        );
                    }
                    HotkeyPreset::CtrlShiftHOnly => {
                        let _ = RegisterHotKey(
                            None,
                            HOTKEY_ID_CTRL_SHIFT_H,
                            MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT,
                            b'H' as u32,
                        );
                    }
                    HotkeyPreset::AltHOnly => {
                        let _ = RegisterHotKey(
                            None,
                            HOTKEY_ID_ALT_H,
                            MOD_ALT | MOD_NOREPEAT,
                            b'H' as u32,
                        );
                    }
                    HotkeyPreset::Disabled => {}
                }
            }
        };

        let initial = *preset.lock().unwrap();
        let mut registered_preset = initial;
        register(initial);

        let mut msg = MSG::default();
        loop {
            // Check if user changed preset in GUI
            let current_preset = *preset.lock().unwrap();
            if current_preset != registered_preset {
                register(current_preset);
                registered_preset = current_preset;
            }

            let has_msg = unsafe {
                windows::Win32::UI::WindowsAndMessaging::PeekMessageW(
                    &mut msg,
                    None,
                    0,
                    0,
                    windows::Win32::UI::WindowsAndMessaging::PM_REMOVE,
                )
            };

            if has_msg.as_bool() {
                if msg.message == WM_HOTKEY {
                    info!("WM_HOTKEY received (ID: {})", msg.wParam.0);
                    if msg.wParam.0 == HOTKEY_ID_F7 as usize {
                        toggle_shadowdesk_window();
                    } else {
                        toggle_hidden_windows(&windows);
                    }
                }
            } else {
                thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    });
}
