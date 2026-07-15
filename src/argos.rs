use std::time::{Duration, Instant};

use tokio::time::{MissedTickBehavior, interval as tokio_interval};
use xgo::{Motor, XgoDog};

use crate::model::Model;

pub struct Argos {
    xgo: XgoDog,
    model: Model,
}

impl Argos {
    pub fn new() -> Self {
        Argos {
            xgo: XgoDog::builder().port_name("/dev/ttyAMA0").build().unwrap(),
            model: Model::new(),
        }
    }

    pub async fn run_ms_async(&mut self) {
        let motors = Motor::ALL;
        let mut timer = tokio_interval(Duration::from_micros(2500));
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut i = 0;
        let mut diff = Instant::now();
        loop {
            timer.tick().await;
            self.xgo
                .motor(motors[i], self.model.motor(motors[i]))
                .unwrap();
            i = (i + 1) % 12;
            let duration = diff.elapsed();
            println!("{duration:?}");
            diff = Instant::now();
        }
    }
}
