use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use windows::{
    Win32::{
        Foundation::{BOOL, CloseHandle, HWND, LPARAM, WPARAM},
        System::Threading::{
            OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        },
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
                MAPVK_VK_TO_VSC, MapVirtualKeyW, SendInput, VIRTUAL_KEY, VK_A, VK_CONTROL,
                VK_LCONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT, VK_V,
            },
            WindowsAndMessaging::{
                EnumWindows, GW_OWNER, GetForegroundWindow, GetWindow, GetWindowThreadProcessId,
                IsWindow, IsWindowVisible, PostMessageW, WM_KEYDOWN, WM_KEYUP,
            },
        },
    },
    core::PWSTR,
};

const INPUT_DELAY: Duration = Duration::from_millis(20);

#[derive(Debug, Clone, Copy)]
enum InputMethod {
    SendInput,
    Background,
}

// Capture the VRChat window before conversion; background input always targets
// that HWND and never activates it. Only Ctrl is injected globally; A and V
// remain window messages rather than being injected into the foreground app.
pub struct ChatboxTarget {
    window: HWND,
    process_id: u32,
    method: InputMethod,
}

impl ChatboxTarget {
    pub fn capture() -> Result<Self> {
        let window = unsafe { GetForegroundWindow() };
        if is_vrchat_window(window) {
            return Self::from_window(window, InputMethod::SendInput);
        }

        let mut windows = Vec::<HWND>::new();
        unsafe {
            EnumWindows(
                Some(collect_vrchat_windows),
                LPARAM(&mut windows as *mut Vec<HWND> as isize),
            )
        }
        .context("Cannot enumerate VRChat windows")?;
        ensure!(
            windows.len() == 1,
            "Expected one VRChat window for background input, found {}",
            windows.len()
        );
        Self::from_window(windows[0], InputMethod::Background)
    }

    fn from_window(window: HWND, method: InputMethod) -> Result<Self> {
        let mut process_id = 0;
        let thread_id = unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
        ensure!(
            thread_id != 0 && process_id != 0,
            "VRChat window is no longer available"
        );
        Ok(Self {
            window,
            process_id,
            method,
        })
    }

    fn ensure_window(&self) -> Result<()> {
        ensure!(
            unsafe { IsWindow(self.window) }.as_bool(),
            "VRChat window closed during conversion"
        );
        let mut process_id = 0;
        unsafe { GetWindowThreadProcessId(self.window, Some(&mut process_id)) };
        ensure!(
            process_id == self.process_id,
            "VRChat window changed during conversion"
        );
        Ok(())
    }

    fn ensure_input_target(&self) -> Result<()> {
        self.ensure_window()?;
        if matches!(self.method, InputMethod::SendInput) {
            ensure!(
                unsafe { GetForegroundWindow() } == self.window,
                "Foreground window changed during conversion"
            );
        }
        Ok(())
    }

    pub fn method_name(&self) -> &'static str {
        match self.method {
            InputMethod::SendInput => "SendInput (foreground)",
            InputMethod::Background => "SendInput Ctrl + PostMessage A/V (background)",
        }
    }

    pub fn wait_for_keys_released(&self) -> Result<()> {
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            self.ensure_input_target()?;
            if !keys_pressed() {
                return Ok(());
            }
            ensure!(
                Instant::now() < deadline,
                "Keys are still held; automatic paste cancelled"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn select_all_and_paste(&self) -> Result<()> {
        if matches!(self.method, InputMethod::Background) {
            self.ensure_window()?;
            ensure!(!keys_pressed(), "Keys are held; automatic paste cancelled");
            return background_select_all_and_paste(&mut WindowKeyboard { target: self });
        }
        self.send_shortcut(VK_A)?;
        // Let VRChat process selection in a separate frame before pasting.
        thread::sleep(INPUT_DELAY);
        self.send_shortcut(VK_V)
    }

    fn send_shortcut(&self, key: VIRTUAL_KEY) -> Result<()> {
        self.ensure_input_target()?;
        ensure!(!keys_pressed(), "Keys are held; automatic paste cancelled");
        self.inject_shortcut(key)
    }

    fn post_key(&self, key: VIRTUAL_KEY, released: bool) -> Result<()> {
        self.ensure_window()?;
        let scan_code = unsafe { MapVirtualKeyW(key.0 as u32, MAPVK_VK_TO_VSC) };
        let mut bits = 1u32 | ((scan_code & 0xff) << 16);
        if released {
            // Previous-state and transition bits required for WM_KEYUP.
            bits |= (1 << 30) | (1 << 31);
        }
        unsafe {
            PostMessageW(
                self.window,
                if released { WM_KEYUP } else { WM_KEYDOWN },
                WPARAM(key.0 as usize),
                LPARAM(bits as isize),
            )
        }
        .context("Cannot post keyboard message to VRChat (check privilege levels)")
    }

    fn inject_shortcut(&self, key: VIRTUAL_KEY) -> Result<()> {
        let inputs = [key_input(VK_CONTROL, false), key_input(key, false)];
        let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
        let release = [key_input(key, true), key_input(VK_CONTROL, true)];
        if sent != inputs.len() as u32 {
            // Best effort cleanup if only part of the shortcut was injected.
            unsafe { SendInput(&release, size_of::<INPUT>() as i32) };
            bail!(
                "SendInput inserted {sent}/{} events; check application privilege levels",
                inputs.len()
            );
        }
        // Keep the shortcut held for a frame instead of pressing and releasing
        // between VRChat's input polls. Always release even if focus changes.
        thread::sleep(INPUT_DELAY);
        let released = unsafe { SendInput(&release, size_of::<INPUT>() as i32) };
        if released != release.len() as u32 {
            unsafe { SendInput(&release, size_of::<INPUT>() as i32) };
            bail!("SendInput could not release all shortcut keys");
        }
        self.ensure_input_target()?;
        Ok(())
    }
}

// PostMessage does not change the asynchronous keyboard state which game
// engines can poll for modifiers. Hold real left Ctrl for the entire operation
// and wait until Windows reports it down before posting either letter.
trait BackgroundKeyboard {
    fn control(&mut self, released: bool) -> Result<()>;
    fn wait_for_control(&mut self) -> Result<()>;
    fn post(&mut self, key: VIRTUAL_KEY, released: bool) -> Result<()>;
    fn pause(&mut self);
}

struct WindowKeyboard<'a> {
    target: &'a ChatboxTarget,
}

impl BackgroundKeyboard for WindowKeyboard<'_> {
    fn control(&mut self, released: bool) -> Result<()> {
        // Deliberately inject only the modifier, never the A/V keys.
        let inputs = [key_input(VK_LCONTROL, released)];
        ensure!(
            unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) } == 1,
            "Cannot {} background Ctrl; check application privilege levels",
            if released { "release" } else { "press" }
        );
        Ok(())
    }

    fn wait_for_control(&mut self) -> Result<()> {
        let deadline = Instant::now() + Duration::from_millis(50);
        loop {
            self.target.ensure_window()?;
            if unsafe { GetAsyncKeyState(VK_LCONTROL.0 as i32) } < 0 {
                return Ok(());
            }
            ensure!(
                Instant::now() < deadline,
                "Background Ctrl did not become pressed; A/V cancelled"
            );
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn post(&mut self, key: VIRTUAL_KEY, released: bool) -> Result<()> {
        if !released && key != VK_CONTROL {
            ensure!(
                unsafe { GetAsyncKeyState(VK_LCONTROL.0 as i32) } < 0,
                "Background Ctrl was released; A/V cancelled"
            );
        }
        self.target.post_key(key, released)
    }

    fn pause(&mut self) {
        thread::sleep(INPUT_DELAY);
    }
}

struct HeldControl<'a, T: BackgroundKeyboard> {
    keyboard: &'a mut T,
    pressed: bool,
}

impl<T: BackgroundKeyboard> HeldControl<'_, T> {
    fn release(&mut self) -> Result<()> {
        if self.pressed {
            self.keyboard.control(true)?;
            self.pressed = false;
        }
        Ok(())
    }
}

impl<T: BackgroundKeyboard> Drop for HeldControl<'_, T> {
    fn drop(&mut self) {
        if let Err(error) = self.release() {
            // Retry a failed release, including during unwinding.
            if let Err(retry) = self.release() {
                tracing::error!(%error, %retry, "Could not release injected background Ctrl");
            }
        }
    }
}

fn background_select_all_and_paste(keyboard: &mut impl BackgroundKeyboard) -> Result<()> {
    let mut held = HeldControl {
        keyboard,
        pressed: false,
    };
    held.keyboard.control(false)?;
    held.pressed = true;
    let result: Result<()> = (|| {
        held.keyboard.wait_for_control()?;
        // Give the receiver a frame to observe the modifier before its first
        // letter. Ctrl remains held between selection and paste (80 ms total).
        held.keyboard.pause();
        held.keyboard.post(VK_CONTROL, false)?;
        for key in [VK_A, VK_V] {
            let down = held.keyboard.post(key, false);
            if down.is_ok() {
                held.keyboard.pause();
            }
            let up = held.keyboard.post(key, true);
            down?;
            up?;
            if key == VK_A {
                held.keyboard.pause();
            }
        }
        Ok(())
    })();
    // Both releases are attempted regardless of window/letter/sync errors.
    let message_release = held.keyboard.post(VK_CONTROL, true);
    let global_release = held.release();
    result?;
    message_release?;
    global_release
}

pub fn is_vrchat_window(window: HWND) -> bool {
    if window.0 == 0 {
        return false;
    }
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
    let Ok(process) =
        (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) })
    else {
        return false;
    };
    let mut path = vec![0u16; 32768];
    let mut length = path.len() as u32;
    let query = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(path.as_mut_ptr()),
            &mut length,
        )
    };
    let _ = unsafe { CloseHandle(process) };
    if query.is_err() {
        return false;
    }
    let path = String::from_utf16_lossy(&path[..length as usize]);
    Path::new(&path)
        .file_name()
        .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("VRChat.exe"))
}

unsafe extern "system" fn collect_vrchat_windows(window: HWND, context: LPARAM) -> BOOL {
    if unsafe { IsWindowVisible(window) }.as_bool()
        && unsafe { GetWindow(window, GW_OWNER) }.0 == 0
        && is_vrchat_window(window)
    {
        let windows = unsafe { &mut *(context.0 as *mut Vec<HWND>) };
        windows.push(window);
    }
    BOOL(1)
}

fn keys_pressed() -> bool {
    [VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN, VK_A, VK_V]
        .iter()
        .any(|key| unsafe { GetAsyncKeyState(key.0 as i32) } < 0)
}

fn key_input(key: VIRTUAL_KEY, released: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: if released {
                    KEYEVENTF_KEYUP
                } else {
                    Default::default()
                },
                ..Default::default()
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::{
        Win32::{
            Foundation::HINSTANCE,
            UI::WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, HMENU, HWND_MESSAGE, MSG, PM_REMOVE, PeekMessageW,
                WINDOW_EX_STYLE, WINDOW_STYLE,
            },
        },
        core::w,
    };

    struct MessageWindow(HWND);

    impl Drop for MessageWindow {
        fn drop(&mut self) {
            let _ = unsafe { DestroyWindow(self.0) };
        }
    }

    #[derive(Debug, PartialEq)]
    enum Action {
        Control(bool),
        Wait,
        Post(u16, bool),
        Pause,
    }

    #[derive(Default)]
    struct FakeKeyboard {
        actions: Vec<Action>,
        fail_press: bool,
        fail_wait: bool,
        fail_key: Option<VIRTUAL_KEY>,
        fail_release_once: bool,
        panic_on_pause: bool,
    }

    impl BackgroundKeyboard for FakeKeyboard {
        fn control(&mut self, released: bool) -> Result<()> {
            self.actions.push(Action::Control(released));
            if !released && self.fail_press {
                bail!("press failed");
            }
            if released && self.fail_release_once {
                self.fail_release_once = false;
                bail!("release failed");
            }
            Ok(())
        }

        fn wait_for_control(&mut self) -> Result<()> {
            self.actions.push(Action::Wait);
            ensure!(!self.fail_wait, "modifier state unavailable");
            Ok(())
        }

        fn post(&mut self, key: VIRTUAL_KEY, released: bool) -> Result<()> {
            self.actions.push(Action::Post(key.0, released));
            ensure!(released || self.fail_key != Some(key), "post failed");
            Ok(())
        }

        fn pause(&mut self) {
            self.actions.push(Action::Pause);
            assert!(
                !self.panic_on_pause,
                "simulated failure while modifier is held"
            );
        }
    }

    #[test]
    fn background_holds_real_ctrl_across_selection_and_paste() {
        let mut keyboard = FakeKeyboard::default();
        background_select_all_and_paste(&mut keyboard).unwrap();
        assert_eq!(
            keyboard.actions,
            vec![
                Action::Control(false),
                Action::Wait,
                Action::Pause,
                Action::Post(VK_CONTROL.0, false),
                Action::Post(VK_A.0, false),
                Action::Pause,
                Action::Post(VK_A.0, true),
                Action::Pause,
                Action::Post(VK_V.0, false),
                Action::Pause,
                Action::Post(VK_V.0, true),
                Action::Post(VK_CONTROL.0, true),
                Action::Control(true),
            ]
        );
    }

    #[test]
    fn background_never_posts_letters_without_confirmed_ctrl() {
        for fail_press in [true, false] {
            let mut keyboard = FakeKeyboard {
                fail_press,
                fail_wait: true,
                ..Default::default()
            };
            assert!(background_select_all_and_paste(&mut keyboard).is_err());
            assert!(!keyboard.actions.iter().any(|action|
                matches!(action, Action::Post(key, false) if *key == VK_A.0 || *key == VK_V.0)));
            if !fail_press {
                assert_eq!(keyboard.actions.last(), Some(&Action::Control(true)));
            }
        }
    }

    #[test]
    fn background_releases_ctrl_on_letter_failure_and_retries_failed_release() {
        let mut keyboard = FakeKeyboard {
            fail_key: Some(VK_A),
            fail_release_once: true,
            ..Default::default()
        };
        assert!(background_select_all_and_paste(&mut keyboard).is_err());
        assert!(keyboard.actions.contains(&Action::Post(VK_A.0, true)));
        assert!(!keyboard.actions.contains(&Action::Post(VK_V.0, false)));
        assert_eq!(
            &keyboard.actions[keyboard.actions.len() - 2..],
            &[Action::Control(true), Action::Control(true)]
        );
    }

    #[test]
    fn background_releases_ctrl_during_unwinding() {
        let mut keyboard = FakeKeyboard {
            panic_on_pause: true,
            ..Default::default()
        };
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                background_select_all_and_paste(&mut keyboard)
            }))
            .is_err()
        );
        assert_eq!(keyboard.actions.last(), Some(&Action::Control(true)));
    }

    #[test]
    fn raw_keyboard_messages_are_queued_only_to_the_target_window() {
        // A message-only receiver cannot become foreground. No real application
        // receives input, and this test does not use global SendInput.
        let window = MessageWindow(unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Background input test"),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                HMENU::default(),
                HINSTANCE::default(),
                None,
            )
        });
        assert_ne!(window.0.0, 0);
        let target = ChatboxTarget::from_window(window.0, InputMethod::Background).unwrap();
        for key in [VK_A, VK_V] {
            for (key, released) in [
                (VK_CONTROL, false),
                (key, false),
                (key, true),
                (VK_CONTROL, true),
            ] {
                target.post_key(key, released).unwrap();
            }
        }

        let mut received = Vec::new();
        let mut message = MSG::default();
        while unsafe { PeekMessageW(&mut message, window.0, WM_KEYDOWN, WM_KEYUP, PM_REMOVE) }
            .as_bool()
        {
            assert_eq!(message.hwnd, window.0);
            // Key-up messages must have the previous-state and transition bits.
            assert_eq!(
                (message.lParam.0 as u32) >> 30,
                if message.message == WM_KEYUP { 3 } else { 0 }
            );
            received.push((message.message, message.wParam.0 as u16));
        }
        assert_eq!(
            received,
            vec![
                (WM_KEYDOWN, VK_CONTROL.0),
                (WM_KEYDOWN, VK_A.0),
                (WM_KEYUP, VK_A.0),
                (WM_KEYUP, VK_CONTROL.0),
                (WM_KEYDOWN, VK_CONTROL.0),
                (WM_KEYDOWN, VK_V.0),
                (WM_KEYUP, VK_V.0),
                (WM_KEYUP, VK_CONTROL.0),
            ]
        );
        drop(window);
        assert!(
            target.ensure_window().is_err(),
            "Closed targets must be rejected"
        );
    }
}
