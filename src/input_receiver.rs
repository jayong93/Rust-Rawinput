use super::inner_window::*;
use winapi::shared::windef::HWND;
use std::sync::mpsc::{self, TryRecvError, RecvError};
use std::cmp::{Eq, PartialEq};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum KeyState {
    Down,
    Up,
}

// Each input has a Virtual Key of Windows
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Input {
    Mouse(i32),
    KeyBoard(i32),
}

pub struct Receiver {
    input_recevier: mpsc::Receiver<(Input, KeyState)>,
}

impl Receiver {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(Self::message_loop_fn(sender));

        Self {
            input_recevier: receiver,
        }
    }

    fn message_loop_fn(sender: mpsc::Sender<(Input, KeyState)>) -> impl FnOnce() {
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

    pub fn try_get(&mut self) -> Result<(Input, KeyState), TryRecvError> {
        self.input_recevier.try_recv()
    }

    pub fn get(&mut self) -> Result<(Input, KeyState), RecvError> {
        self.input_recevier.recv()
    }
}
