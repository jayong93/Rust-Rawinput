use super::inner_window::*;
use tokio::sync::mpsc;
use winapi::shared::windef::HWND;
use std::future::Future;

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
    input_recevier: mpsc::UnboundedReceiver<(Input, KeyState)>,
    runtime: tokio::runtime::current_thread::Runtime,
}

impl Receiver {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        std::thread::spawn(Self::message_loop_fn(sender));

        Self {
            input_recevier: receiver,
            runtime: tokio::runtime::current_thread::Runtime::new().unwrap(),
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

    #[inline]
    pub fn get(&mut self) -> Option<(Input, KeyState)> {
        self.runtime.block_on(self.input_recevier.recv())
    }

    #[inline]
    pub fn get_async<'a>(&'a mut self) -> impl Future<Output=Option<(Input, KeyState)>> + 'a {
        self.input_recevier.recv()
    }
}
