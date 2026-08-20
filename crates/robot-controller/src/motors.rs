use anyhow::Result;
use comms::{Comms, topics::MotorCommands};
use tracing::{debug, error, info};
use xgo::{Motor, XgoDog};

pub async fn run(comms: Comms) -> Result<()> {
    let subscriber = comms
        .subscriber::<MotorCommands>()
        .await
        .inspect_err(|error| error!("failed to create motor subscriber: {:?}", error))?;

    info!("initializing motor controller on port {}", "/dev/ttyAMA0");

    let mut dog = XgoDog::builder()
        .port_name("/dev/ttyAMA0")
        .build()
        .await
        .inspect_err(|error| error!("failed to open XGO port: {:?}", error))?;

    dog.load_all_motors()
        .await
        .inspect_err(|error| error!("failed to load motors: {:?}", error))?;

    info!("motor controller ready");

    loop {
        let command = match subscriber.recv().await {
            Ok(command) => command,
            Err(comms::Error::InvalidPayload { .. }) => {
                debug!("ignoring invalid motor command payload");
                continue;
            }
            Err(error) => {
                error!("failed to receive motor command: {:?}", error);
                return Err(error.into());
            }
        };
        let cmds: [(Motor, f32); 15] = std::array::from_fn(|i| (Motor::ALL[i], command.angles[i]));
        debug!("received motor command for {} motors", cmds.len());
        dog.motors(&cmds)
            .await
            .inspect_err(|error| error!("failed to send motor command: {:?}", error))?;
    }
}
