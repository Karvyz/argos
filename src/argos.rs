use std::{thread::sleep, time::Duration};

use xgo::{Motor, XgoDog};

const SHOULDER_LENGTH: f32 = 2.86;
const UPPER_LENGTH: f32 = 5.5;
const LOWER_LENGTH: f32 = 6.68;

pub struct Coord {
    x: f32,
    y: f32,
    z: f32,
}

impl Coord {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Coord { x, y, z }
    }
}

pub struct Argos {
    xgo: XgoDog,
}

impl Argos {
    pub fn new() -> Self {
        Argos {
            xgo: XgoDog::builder().port_name("/dev/ttyAMA0").build().unwrap(),
        }
    }

    // pub async fn run_ms_async(&mut self) {
    //     let motors = Motor::ALL;
    //     let mut timer = tokio_interval(Duration::from_micros(2500));
    //     timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    //
    //     let mut i = 0;
    //     let mut diff = Instant::now();
    //     loop {
    //         timer.tick().await;
    //         self.xgo
    //             .motor(motors[i], self.model.motor(motors[i]))
    //             .unwrap();
    //         i = (i + 1) % 12;
    //         let duration = diff.elapsed();
    //         println!("{duration:?}");
    //         diff = Instant::now();
    //     }
    // }

    pub fn single(&mut self) {
        let origin = Coord::new(0., 0., 0.);
        let objective = Coord::new(0., -10., 0.);
        self.front_right(&origin, &objective);
        self.front_left(&origin, &objective);
        self.back_right(&origin, &objective);
        self.back_left(&origin, &objective);
        sleep(Duration::from_secs(2));
        self.xgo.reset().unwrap();
    }

    fn front_right(&mut self, origin: &Coord, objective: &Coord) {
        let (x, y, z) = self.leg(origin, objective);
        self.xgo.motor(Motor::ShoulderFR, x as f64).unwrap();
        self.xgo.motor(Motor::UpperLegFR, y as f64).unwrap();
        self.xgo.motor(Motor::LowerLegFR, z as f64).unwrap();
    }

    fn front_left(&mut self, origin: &Coord, objective: &Coord) {
        let (x, y, z) = self.leg(origin, objective);
        self.xgo.motor(Motor::ShoulderFL, x as f64).unwrap();
        self.xgo.motor(Motor::UpperLegFL, y as f64).unwrap();
        self.xgo.motor(Motor::LowerLegFL, z as f64).unwrap();
    }

    fn back_right(&mut self, origin: &Coord, objective: &Coord) {
        let (x, y, z) = self.leg(origin, objective);
        self.xgo.motor(Motor::ShoulderBR, x as f64).unwrap();
        self.xgo.motor(Motor::UpperLegBR, y as f64).unwrap();
        self.xgo.motor(Motor::LowerLegBR, z as f64).unwrap();
    }

    fn back_left(&mut self, origin: &Coord, objective: &Coord) {
        let (x, y, z) = self.leg(origin, objective);
        self.xgo.motor(Motor::ShoulderBL, x as f64).unwrap();
        self.xgo.motor(Motor::UpperLegBL, y as f64).unwrap();
        self.xgo.motor(Motor::LowerLegBL, z as f64).unwrap();
    }

    pub fn leg(&self, origin: &Coord, objective: &Coord) -> (f32, f32, f32) {
        let dx = objective.x - origin.x;
        let dy = origin.y - objective.y;
        let dz = objective.z - origin.z;

        let d = (dy * dy + dz * dz).sqrt();
        let a = ((d * d + dy * dy - dz * dz) / (2. * d * dy)).acos();
        let b = ((SHOULDER_LENGTH * 1.) / d).asin(); // sin(90 deg) = 1
        let e =
            (SHOULDER_LENGTH * SHOULDER_LENGTH + d * d - 2. * d * SHOULDER_LENGTH * b.cos()).sqrt();
        let dd = (e * e + dx * dx).sqrt(); //2*dp*e*cos(90 deg) = 0
        let j = ((e * e + dd * dd - dx * dx) / (2. * dd * e)).acos();
        let k = ((dd * dd + UPPER_LENGTH * UPPER_LENGTH - LOWER_LENGTH * LOWER_LENGTH)
            / (2. * UPPER_LENGTH * dd))
            .acos();

        let l = ((UPPER_LENGTH * UPPER_LENGTH + LOWER_LENGTH * LOWER_LENGTH - dd * dd)
            / (2. * UPPER_LENGTH * LOWER_LENGTH))
            .acos()
            .to_degrees();
        let kj = k.to_degrees() + j.to_degrees();
        (-(a.to_degrees() + b.to_degrees()), kj, l - 90.)
        // (0., 0., 0.)
    }
}
