use rust_rawinput::Receiver;
fn main() {
    let receiver = Receiver::new();
    while let Ok(input) = receiver.get() {
        println!("{:?}", input);
    }
}
