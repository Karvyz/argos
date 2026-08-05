use anyhow::Result;
use comms::{Comms, topics::MotorCommands};
use xgo::{Motor, XgoDog};

pub async fn run(comms: Comms) -> Result<()> {
    let subscriber = comms.subscriber::<MotorCommands>().await?;

    let mut dog = XgoDog::builder().port_name("/dev/ttyAMA0").build().await?;
    dog.load_all_motors().await?;

    loop {
        let command = match subscriber.recv().await {
            Ok(command) => command,
            Err(comms::Error::InvalidPayload { .. }) => continue,
            Err(error) => return Err(error.into()),
        };
        let cmds: [(Motor, f32); 15] = std::array::from_fn(|i| (Motor::ALL[i], command.angles[i]));
        dog.motors(&cmds).await?;
    }
}
