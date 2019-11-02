use rust_rawinput::Receiver;
fn main() {
    let mut receiver = Receiver::new();
    loop {
        let input = receiver.get();
        println!("{:?}", input);
    }
}