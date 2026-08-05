use std::time::Duration;

use comms::{Comms, MotorCommand, Publisher, topics::MotorCommands};
use tokio::{
    sync::mpsc,
    time::{MissedTickBehavior, interval},
};

use crate::model::Model;

pub enum Action {
    Circle,
    Exit,
}

pub struct Argos {
    motors: Publisher<MotorCommands>,
    model: Model,
    rx: mpsc::Receiver<Action>,
}

impl Argos {
    pub async fn new(rx: mpsc::Receiver<Action>, comms: Comms) -> Self {
        let motors = comms.publisher::<MotorCommands>().await.unwrap();

        Argos {
            motors,
            model: Model::new(),
            rx,
        }
    }

    pub async fn run_ms_async(&mut self) {
        let mut timer = interval(Duration::from_millis(200));
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        // let mut i: f32 = 0.;
        // let mut k = 0;

        self.model.update();
        // println!("{:?}", self.model);
        // let position = self.model.position;
        // let rot = self.model.rotation;

        // let feetpos = self.model.feets[0];

        loop {
            timer.tick().await;

            let angles = self.model.angles.map(|(_, angle)| angle);
            self.motors.send(MotorCommand { angles }).await.unwrap();
            // println!("{imu:?}");
            // let offset = Vec3::X * (i/*  + PI / 2. */).cos() + Vec3::Z * i.sin();
            // println!("offset: {offset}");
            // self.model.position = position + offset;
            //

            self.model.update();
            // println!("{:?}", self.model);
            // i += 0.1;
            // k = (k + 1) % 4;
            match self.rx.try_recv() {
                Ok(a) => match a {
                    Action::Circle => (),
                    Action::Exit => break,
                },
                Err(_) => (),
            }
            // let duration = instant.elapsed();
            // println!("{duration:?}");
        }
    }
}
