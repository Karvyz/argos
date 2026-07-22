use std::{thread::sleep, time::Duration};

use glam::Vec3;
use tokio::time::{Instant, MissedTickBehavior, interval};
use xgo::{Motor, XgoDog};

const SHOULDER_LENGTH: f32 = 2.86;
const UPPER_LENGTH: f32 = 5.5;
const LOWER_LENGTH: f32 = 6.68;
const BODY_WIDTH: f32 = 5.;
const BODY_LENGTH: f32 = 20.;

pub struct Argos {
    xgo: XgoDog,
}

impl Argos {
    pub fn new() -> Self {
        let mut xgo = XgoDog::builder().port_name("/dev/ttyAMA0").build().unwrap();
        xgo.load_all_motors().unwrap();

        Argos { xgo }
    }

    pub async fn run_ms_async(&mut self) {
        let mut timer = interval(Duration::from_millis(200));
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut i = 0.;

        let origin = Vec3::new(0., 0., 0.);
        let ob1 = Vec3::new(0., -10., SHOULDER_LENGTH);
        let ob2 = Vec3::new(0., -10., 0.);
        // let ob1 = Vec3::new(-5., -10., 0.);
        // let ob2 = Vec3::new(5., -10., 0.);
        // self.xgo.motor_speed(50).unwrap();
        // self.xgo.motor(Motor::ShoulderFR, 20.).unwrap(); // UP
        // self.leg(&origin, &ob2);
        // sleep(Duration::from_secs(3));
        // self.xgo.motor(Motor::ShoulderFR, -10.).unwrap(); // DOWN
        self.front_right(&origin, &ob2);

        let mut diff = Instant::now();
        loop {
            timer.tick().await;
            let uwu = Self::step1(ob1, ob2, i);
            i = (i + 0.05) % 1.;
            // self.front_right(&origin, &uwu);
            let duration = diff.elapsed();
            // println!("{i}:{duration:?}");
            diff = Instant::now();
        }
    }

    fn step1(origin: Vec3, objective: Vec3, avancement: f32) -> Vec3 {
        let diff = objective - origin;
        match avancement > 0.5 {
            true => origin + (diff * avancement),
            false => objective - (diff * avancement),
        }
    }

    pub fn single(&mut self) {
        let origin = Vec3::new(0., 0., 0.);
        let objective = Vec3::new(0., -10., 0.);
        self.front_right(&origin, &objective);
        self.front_left(&origin, &objective);
        self.back_right(&origin, &objective);
        self.back_left(&origin, &objective);
        sleep(Duration::from_secs(2));
        self.xgo.reset().unwrap();
    }

    fn front_right(&mut self, origin: &Vec3, objective: &Vec3) {
        let (x, y, z) = self.leg(origin, objective);
        self.xgo.motor(Motor::ShoulderFR, x).unwrap();
        self.xgo.motor(Motor::UpperLegFR, y).unwrap();
        self.xgo.motor(Motor::LowerLegFR, z).unwrap();
    }

    fn front_left(&mut self, origin: &Vec3, objective: &Vec3) {
        let (x, y, z) = self.leg(origin, objective);
        self.xgo.motor(Motor::ShoulderFL, x).unwrap();
        self.xgo.motor(Motor::UpperLegFL, y).unwrap();
        self.xgo.motor(Motor::LowerLegFL, z).unwrap();
    }

    fn back_right(&mut self, origin: &Vec3, objective: &Vec3) {
        let (x, y, z) = self.leg(origin, objective);
        self.xgo.motor(Motor::ShoulderBR, x).unwrap();
        self.xgo.motor(Motor::UpperLegBR, y).unwrap();
        self.xgo.motor(Motor::LowerLegBR, z).unwrap();
    }

    fn back_left(&mut self, origin: &Vec3, objective: &Vec3) {
        let (x, y, z) = self.leg(origin, objective);
        self.xgo.motor(Motor::ShoulderBL, x).unwrap();
        self.xgo.motor(Motor::UpperLegBL, y).unwrap();
        self.xgo.motor(Motor::LowerLegBL, z).unwrap();
    }

    pub fn leg(&self, origin: &Vec3, objective: &Vec3) -> (f32, f32, f32) {
        let dx = objective.x - origin.x;
        let dy = origin.y - objective.y;
        let dz = objective.z - origin.z;

        let dyz = (dy * dy + dz * dz).sqrt();
        let a = ((dyz * dyz + dy * dy - dz * dz) / (2. * dyz * dy)).acos();

        let leg_length = (dyz * dyz - SHOULDER_LENGTH * SHOULDER_LENGTH).sqrt();
        let b = ((SHOULDER_LENGTH * SHOULDER_LENGTH + dyz * dyz - leg_length * leg_length)
            / (2. * SHOULDER_LENGTH * dyz))
            .acos();
        let shoulder = (a.to_degrees() + b.to_degrees()) - 90.;
        let dd = (leg_length * leg_length + dx * dx).sqrt(); //2*dp*e*cos(90 deg) = 0
        let j = ((leg_length * leg_length + dd * dd - dx * dx) / (2. * dd * leg_length)).acos();
        let k = ((dd * dd + UPPER_LENGTH * UPPER_LENGTH - LOWER_LENGTH * LOWER_LENGTH)
            / (2. * UPPER_LENGTH * dd))
            .acos();

        let lower = ((UPPER_LENGTH * UPPER_LENGTH + LOWER_LENGTH * LOWER_LENGTH - dd * dd)
            / (2. * UPPER_LENGTH * LOWER_LENGTH))
            .acos()
            .to_degrees()
            - 90.;
        let upper = k.to_degrees() + j.to_degrees();
        println!("a: {} b: {} c:{}", a.to_degrees(), b.to_degrees(), shoulder);
        (shoulder, upper, lower)
    }
}
