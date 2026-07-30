use zenoh::Config;
use zenoh_ext::z_serialize;

#[tokio::main]
async fn main() {
    // Initiate logging
    zenoh::init_log_from_env_or("error");

    let config = Config::default();
    let key = "robot/motors";
    println!("config: {config}");

    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    println!("Declaring publisher on '{}'...", key);
    let publisher = session.declare_publisher(key).await.unwrap();

    let data = [0.0f32; 15];
    publisher.put(z_serialize(&data)).await.unwrap();
}
