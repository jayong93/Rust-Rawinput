use rust_rawinput::Receiver;
fn main() {
    let mut receiver = Receiver::new();
    while let Ok(input) = receiver.get() {
        println!("{:?}", input);
    }
}
