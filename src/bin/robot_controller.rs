use zenoh::Config;
use zenoh_ext::z_deserialize;

#[tokio::main]
async fn main() {
    // Initiate logging
    zenoh::init_log_from_env_or("error");

    let config = Config::default();
    let key = "robot/motors";
    println!("config: {config}");

    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    println!("Declaring Subscriber on '{}'...", key);
    let subscriber = session.declare_subscriber(key).await.unwrap();

    println!("Press CTRL-C to quit...");
    while let Ok(sample) = subscriber.recv_async().await {
        // Refer to z_bytes.rs to see how to deserialize different types of message
        println!("Received payload");
        let payload = sample.payload();
        let Ok(data) = z_deserialize::<[f32; 15]>(payload) else {
            continue;
        };

        println!(
            ">> [Subscriber] Received {} ('{}': '{:?}')",
            sample.kind(),
            sample.key_expr().as_str(),
            data
        );
    }
}
