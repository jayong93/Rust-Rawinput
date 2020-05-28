use rust_rawinput::Receiver;
use futures::executor::LocalPool;
use futures::task::LocalSpawnExt;
fn main() {
    let mut local_pool = LocalPool::new();
    local_pool.spawner().spawn_local(
        async {
            let mut receiver = Receiver::new();
            while let Some(input) = receiver.get().await {
                println!("{:?}", input);
            }
        }
    ).unwrap();
    local_pool.run();
}
