use super::inner_window::*;
use winapi::shared::windef::HWND;
use std::cmp::{Eq, PartialEq};
use futures::channel::mpsc;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum KeyState {
    Down,
    Up,
}

// Each input has a Virtual Key of Windows
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Input {
    Mouse(i32),
    KeyBoard(i32),
}

use std::fmt;
use winapi::um::winuser;
impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mouse(vk) => {
                let key_names = ["MOUSE_LBUTTON", "MOUSE_RBUTTON", "CANCEL", "MOUSE_MBUTTON", "MOUSE_XBUTTON1", "MOUSE_XBUTTON2"];
                write!(f, "{}", key_names[*vk as usize - 1])
            }
            Self::KeyBoard(vk) => unsafe {
                let scan_code = winuser::MapVirtualKeyA(*vk as _, winuser::MAPVK_VK_TO_VSC);
                let mut name = [0u8; 16];
                winuser::GetKeyNameTextA((scan_code << 16) as _, name.as_mut_ptr() as _, 16);
                let c_name = std::ffi::CStr::from_ptr(name.as_ptr() as _);
                write!(f, "{}", c_name.to_str().unwrap())
            }
        }
    }
}

pub struct Receiver {
    input_recevier: mpsc::UnboundedReceiver<(Input, KeyState)>,
}

impl Receiver {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded();
        std::thread::spawn(Self::message_loop_fn(sender));

        Self {
            input_recevier: receiver,
        }
    }

    fn message_loop_fn(sender: mpsc::UnboundedSender<(Input, KeyState)>) -> impl FnOnce() {
        use winapi::shared::ntdef::TRUE;
        use winapi::um::winuser::*;
        move || {
            let hwnd = make_blank_window(sender);
            let mut msg = MSG::default();
            unsafe {
                while TRUE as i32 == GetMessageW(&mut msg as _, hwnd as HWND, WM_INPUT, WM_INPUT) {
                    DispatchMessageW(&msg as _);
                }
                CloseWindow(hwnd);
            }
        }
    }

    pub async fn get(&mut self) -> Option<(Input, KeyState)> {
        use futures::stream::StreamExt;
        self.input_recevier.next().await
    }
}
