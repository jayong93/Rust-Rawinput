use rust_rawinput::Receiver;
use std::thread;
fn main() {
    let mut receiver = Receiver::new();
    loop {
        if let Ok(input) = receiver.get() {
            println!("{:?}", input);
        }
        thread::yield_now();
    }
}
