use rust_rawinput::Receiver;
fn main() {
    let mut receiver = Receiver::new();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        loop {
            let input = receiver.get_async().await;
            println!("{:?}", input);
        }
    });
}
