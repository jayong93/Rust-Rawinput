use rust_rawinput::Receiver;
fn main() {
    let receiver = Receiver::new();
    loop {
        let input = receiver.get();
        println!("{:?}", input);
    }
}