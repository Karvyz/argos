use anyhow::Result;
use argos::comms;
use xgo::{Motor, XgoDog};
use zenoh::Session;

pub async fn run(session: Session) -> Result<()> {
    let subscriber = session
        .declare_subscriber(comms::keys::MOTORS)
        .await
        .map_err(anyhow::Error::msg)?;

    let mut dog = XgoDog::builder().port_name("/dev/ttyAMA0").build().await?;
    dog.load_all_motors().await?;

    while let Ok(sample) = subscriber.recv_async().await {
        let Some(angles) = comms::motors::decode(sample.payload()) else {
            continue;
        };
        let cmds: [(Motor, f32); 15] = std::array::from_fn(|i| (Motor::ALL[i], angles[i]));
        dog.motors(&cmds).await?;
    }
    Ok(())
}
