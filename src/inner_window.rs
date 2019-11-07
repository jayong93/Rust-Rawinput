use super::input_receiver::{Input, KeyState};
use std::ffi::OsStr;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use tokio::sync::mpsc::UnboundedSender as Sender;
use winapi::shared::minwindef::{BOOL, LPARAM, LRESULT, WPARAM};
use winapi::shared::ntdef::NULL;
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;

static mut SENDER: Option<Sender<(Input, KeyState)>> = None;

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            assert!(SENDER.is_some());
            DefWindowProcW(hwnd, msg, w_param, l_param)
        }
        WM_INPUT => {
            let mut input_size = 0u32;
            GetRawInputData(
                l_param as _,
                RID_INPUT,
                NULL,
                &mut input_size as *mut _,
                size_of::<RAWINPUTHEADER>() as u32,
            );
            let mut input_bytes = vec![0u8; input_size as usize];
            GetRawInputData(
                l_param as _,
                RID_INPUT,
                input_bytes.as_mut_ptr() as _,
                &mut input_size as _,
                size_of::<RAWINPUTHEADER>() as u32,
            );

            let sender = SENDER.as_mut().unwrap();
            let raw_input = &*(input_bytes.as_ptr() as *const RAWINPUT);
            match raw_input.header.dwType {
                RIM_TYPEKEYBOARD => {
                    let raw_keyboard_input = raw_input.data.keyboard();
                    match raw_keyboard_input.Flags as u32 {
                        RI_KEY_MAKE => {
                            // if the key has pressed
                            sender
                                .try_send((
                                    Input::KeyBoard(raw_keyboard_input.VKey as i32),
                                    KeyState::Down,
                                ))
                                .ok();
                        }
                        RI_KEY_BREAK => {
                            sender
                                .try_send((
                                    Input::KeyBoard(raw_keyboard_input.VKey as i32),
                                    KeyState::Up,
                                ))
                                .ok();
                        }
                        _ => {}
                    }
                }
                RIM_TYPEMOUSE => {
                    let raw_mouse_input = raw_input.data.mouse();
                    match raw_mouse_input.usButtonFlags {
                        RI_MOUSE_LEFT_BUTTON_DOWN => {
                            sender.try_send((Input::Mouse(VK_LBUTTON), KeyState::Down)).ok();
                        }
                        RI_MOUSE_LEFT_BUTTON_UP => {
                            sender.try_send((Input::Mouse(VK_LBUTTON), KeyState::Up)).ok();
                        }
                        RI_MOUSE_RIGHT_BUTTON_DOWN => {
                            sender.try_send((Input::Mouse(VK_RBUTTON), KeyState::Down)).ok();
                        }
                        RI_MOUSE_RIGHT_BUTTON_UP => {
                            sender.try_send((Input::Mouse(VK_RBUTTON), KeyState::Up)).ok();
                        }
                        RI_MOUSE_MIDDLE_BUTTON_DOWN => {
                            sender.try_send((Input::Mouse(VK_MBUTTON), KeyState::Down)).ok();
                        }
                        RI_MOUSE_MIDDLE_BUTTON_UP => {
                            sender.try_send((Input::Mouse(VK_MBUTTON), KeyState::Up)).ok();
                        }
                        RI_MOUSE_BUTTON_4_DOWN => {
                            sender
                                .try_send((Input::Mouse(VK_XBUTTON1), KeyState::Down))
                                .ok();
                        }
                        RI_MOUSE_BUTTON_4_UP => {
                            sender.try_send((Input::Mouse(VK_XBUTTON1), KeyState::Up)).ok();
                        }
                        RI_MOUSE_BUTTON_5_DOWN => {
                            sender
                                .try_send((Input::Mouse(VK_XBUTTON2), KeyState::Down))
                                .ok();
                        }
                        RI_MOUSE_BUTTON_5_UP => {
                            sender.try_send((Input::Mouse(VK_XBUTTON2), KeyState::Up)).ok();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param) 
    }
}

pub fn make_blank_window(sender: Sender<(Input, KeyState)>) -> HWND {
    unsafe {
        SENDER = Some(sender);
        let hinstance = GetModuleHandleW(null_mut());
        let mut wclass = WNDCLASSW::default();
        let class_name_vec = OsStr::new("MyMsgClass\0").encode_wide().collect::<Vec<_>>();
        let win_name_vec = OsStr::new("Win\0").encode_wide().collect::<Vec<_>>();
        wclass.lpszClassName = class_name_vec.as_ptr();
        wclass.lpfnWndProc = Some(wnd_proc);
        RegisterClassW(&wclass as *const _);
        let hwnd = CreateWindowExW(
            0,
            class_name_vec.as_ptr(),
            win_name_vec.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            null_mut(),
            hinstance,
            null_mut(),
        );
        // set_keyboard_hook();
        register_raw_devices(hwnd);
        hwnd
    }
}

// unsafe extern "system" fn keyboard_hook(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
//     let sender = SENDER.as_ref().unwrap();
//     let key_struct = &*(l_param as *const KBDLLHOOKSTRUCT);
//     match w_param as u32 {
//         WM_KEYDOWN => {
//             sender.try_send((Input::KeyBoard(key_struct.vkCode as i32), KeyState::Down)).ok();
//         },
//         WM_KEYUP => {
//             sender.try_send((Input::KeyBoard(key_struct.vkCode as i32), KeyState::Up)).ok();
//         },
//         _ => {}
//     }
//     CallNextHookEx(null_mut(), code, w_param, l_param)
// }

// unsafe fn set_keyboard_hook() {
//     SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), GetModuleHandleW(null_mut()), 0);
// }

unsafe fn register_raw_devices(hwnd: HWND) -> BOOL {
    let mut device_vec = vec![RAWINPUTDEVICE::default(); 2];
    let mouse_dev = &mut device_vec[0];
    mouse_dev.usUsagePage = 1;
    mouse_dev.usUsage = 2;
    mouse_dev.dwFlags = RIDEV_INPUTSINK;
    mouse_dev.hwndTarget = hwnd;

    let keyboard_dev = &mut device_vec[1];
    keyboard_dev.usUsagePage = 1;
    keyboard_dev.usUsage = 6;
    keyboard_dev.dwFlags = RIDEV_INPUTSINK;
    keyboard_dev.hwndTarget = hwnd;

    RegisterRawInputDevices(
        device_vec.as_ptr(),
        device_vec.len() as u32,
        std::mem::size_of::<RAWINPUTDEVICE>() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use tokio::sync::mpsc;
    use winapi::shared::minwindef::DWORD;
    use winapi::shared::ntdef::{LANG_NEUTRAL, MAKELANGID, SUBLANG_DEFAULT};
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::winbase::{
        FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
    };

    unsafe fn error_to_string(error_id: DWORD) -> OsString {
        let mut buf = vec![0u16; 256];
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            std::ptr::null_mut(),
            error_id,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT).into(),
            buf.as_mut_ptr(),
            buf.len() as u32,
            std::ptr::null_mut(),
        );

        OsString::from_wide(buf.as_slice())
    }

    #[test]
    fn check_window_created() {
        let (sender, _) = mpsc::unbounded_channel();
        let hwnd = make_blank_window(sender);
        if hwnd == null_mut() {
            unsafe { panic!("{:?}", error_to_string(GetLastError()).to_str()) }
        }
    }

    #[test]
    fn check_registering() {
        let (sender, _) = mpsc::unbounded_channel();
        let hwnd = make_blank_window(sender);
        unsafe {
            assert_ne!(register_raw_devices(hwnd), 0);
        }
    }
}
