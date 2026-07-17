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
        let (x, y, z) = self.leg(
            Coord::new(0., 0., 0.),
            Coord::new(0., -10., SHOULDER_LENGTH),
        );
        self.xgo.motor(Motor::ShoulderFR, x as f64).unwrap();
        self.xgo.motor(Motor::UpperLegFR, y as f64).unwrap();
        self.xgo.motor(Motor::LowerLegFR, z as f64).unwrap();
        sleep(Duration::from_secs(2));
        self.xgo.reset().unwrap();
    }

    pub fn leg(&self, origin: Coord, objective: Coord) -> (f32, f32, f32) {
        let dx = objective.x - origin.x;
        let dy = origin.y - objective.y;
        let dz = objective.z - origin.z;

        println!("=== leg() debug ===");
        println!("origin: ({}, {}, {})", origin.x, origin.y, origin.z);
        println!(
            "objective: ({}, {}, {})",
            objective.x, objective.y, objective.z
        );
        println!("dx: {dx}");
        println!("dy: {dy}");
        println!("dz: {dz}");

        let d = (dy * dy + dz * dz).sqrt();
        println!("d: {d}");

        let a = ((d * d + dy * dy - dz * dz) / (2. * d * dy)).acos();
        println!("a: {a} rad = {}°", a.to_degrees());

        let b = f32::asin((SHOULDER_LENGTH * 1.) / d); // sin(90 deg) = 1
        println!("b = asin(X / d): {b} rad = {}°", b.to_degrees());

        let c = 90. - a.to_degrees() - b.to_degrees();
        println!("c = 90° - a° - b°: {c}°");

        let e =
            (SHOULDER_LENGTH * SHOULDER_LENGTH + d * d - 2. * d * SHOULDER_LENGTH * b.cos()).sqrt();
        println!("e = sqrt(X² + d² - 2*d*X*cos(b)): {e}");

        let dd = (e * e + dx * dx).sqrt(); //2*dp*e*cos(90 deg) = 0
        println!("dd = sqrt(e² + dp²): {dd}");

        let j = ((e * e + dd * dd - dx * dx) / (2. * dd * e)).acos();
        println!(
            "j = acos((e² + dd² - dp²) / (2 * dd * e)): {j} rad = {}°",
            j.to_degrees()
        );

        let k = ((dd * dd + UPPER_LENGTH * UPPER_LENGTH - LOWER_LENGTH * LOWER_LENGTH) / 2.
            * UPPER_LENGTH
            * dd)
            .acos();
        println!(
            "k = acos((dd² + Y² - Z²) / (2 * Y * dd)): {k} rad = {}°",
            k.to_degrees()
        );

        let l = ((UPPER_LENGTH * UPPER_LENGTH + LOWER_LENGTH * LOWER_LENGTH - dd * dd) / 2.
            * UPPER_LENGTH
            * LOWER_LENGTH)
            .acos()
            .to_degrees();
        println!("l = acos((Y² + Z² - dd²) / (2 * Y * Z)): {l}°");

        let kj = k.to_degrees() + j.to_degrees();
        println!("kj = k° + j°: {kj}°");
        println!("=== leg() result: shoulder={c}°, upper_leg={kj}°, lower_leg={l}° ===");

        // (c, kj, l)
        (0., 0., 0.)
    }
}
