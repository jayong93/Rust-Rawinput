use super::inner_window::*;
use std::sync::mpsc;
use winapi::shared::windef::HWND;
use winapi::um::winuser::CloseWindow;

#[derive(Debug, Clone)]
pub enum KeyState {
    Down,
    Up,
}

// Each input has a Virtual Key of Windows
#[derive(Debug, Clone)]
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

    pub fn try_get(&self) -> Option<(Input, KeyState)> {
        self.input_recevier.try_recv().ok()
    }
    pub fn get(&self) -> Result<(Input, KeyState), String> {
        self.input_recevier.recv().map_err(|e| format!("{}", e))
    }
}
