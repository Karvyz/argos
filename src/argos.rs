use std::time::Duration;

use glam::Vec3;
use tokio::time::{Instant, MissedTickBehavior, interval};
use xgo::{Motor, XgoDog};

const SHOULDER_LENGTH: f32 = 2.86;
const UPPER_LENGTH: f32 = 5.5;
const LOWER_LENGTH: f32 = 6.68;
const BODY_WIDTH: f32 = 5.;
const BODY_LENGTH: f32 = 20.;
const BASE_LEG_OBJ: Vec3 = Vec3::new(0., -10., SHOULDER_LENGTH);
const ORIGIN_FR: Vec3 = Vec3::new(BODY_LENGTH / 2., 0., BODY_WIDTH / 2.);
const ORIGIN_FL: Vec3 = Vec3::new(BODY_LENGTH / 2., 0., -BODY_WIDTH / 2.);
const ORIGIN_BR: Vec3 = Vec3::new(-BODY_LENGTH / 2., 0., BODY_WIDTH / 2.);
const ORIGIN_BL: Vec3 = Vec3::new(-BODY_LENGTH / 2., 0., -BODY_WIDTH / 2.);

pub struct Argos {
    xgo: XgoDog,
    origin_pos: Vec3,
    legs_obj: [Vec3; 4], // Relative position from base
}

impl Argos {
    pub async fn new() -> Self {
        let mut xgo = XgoDog::builder()
            .port_name("/dev/ttyAMA0")
            .build()
            .await
            .unwrap();
        xgo.load_all_motors().await.unwrap();

        Argos {
            xgo,
            origin_pos: Vec3::ZERO,
            legs_obj: [Vec3::ZERO; 4],
        }
    }

    pub async fn run_ms_async(&mut self) {
        let mut timer = interval(Duration::from_millis(200));
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            timer.tick().await;

            let instant = Instant::now();
            let imu = self.xgo.read_imu().await.unwrap();
            self.update_model().await;
            let duration = instant.elapsed();
            println!("{duration:?}");
        }
    }

    fn step1(origin: Vec3, objective: Vec3, avancement: f32) -> Vec3 {
        let diff = objective - origin;
        match avancement > 0.5 {
            true => origin + (diff * avancement),
            false => objective - (diff * avancement),
        }
    }

    async fn front_left(&mut self) {
        let origin = self.origin_pos + ORIGIN_FL;
        let objective = self.legs_obj[0] + ORIGIN_FL + BASE_LEG_OBJ;
        let (x, y, z) = self.leg(&origin, &objective);
        self.xgo.motor(Motor::ShoulderFL, x).await.unwrap();
        self.xgo.motor(Motor::UpperLegFL, y).await.unwrap();
        self.xgo.motor(Motor::LowerLegFL, z).await.unwrap();
    }

    async fn front_right(&mut self) {
        let origin = self.origin_pos + ORIGIN_FR;
        let objective = self.legs_obj[1] + ORIGIN_FR + BASE_LEG_OBJ;
        let (x, y, z) = self.leg(&origin, &objective);
        self.xgo.motor(Motor::ShoulderFR, x).await.unwrap();
        self.xgo.motor(Motor::UpperLegFR, y).await.unwrap();
        self.xgo.motor(Motor::LowerLegFR, z).await.unwrap();
    }

    async fn back_left(&mut self) {
        let origin = self.origin_pos + ORIGIN_BL;
        let objective = self.legs_obj[2] + ORIGIN_BL + BASE_LEG_OBJ;
        let (x, y, z) = self.leg(&origin, &objective);
        self.xgo.motor(Motor::ShoulderBL, x).await.unwrap();
        self.xgo.motor(Motor::UpperLegBL, y).await.unwrap();
        self.xgo.motor(Motor::LowerLegBL, z).await.unwrap();
    }

    async fn back_right(&mut self) {
        let origin = self.origin_pos + ORIGIN_BR;
        let objective = self.legs_obj[2] + ORIGIN_BR + BASE_LEG_OBJ;
        let (x, y, z) = self.leg(&origin, &objective);
        self.xgo.motor(Motor::ShoulderBR, x).await.unwrap();
        self.xgo.motor(Motor::UpperLegBR, y).await.unwrap();
        self.xgo.motor(Motor::LowerLegBR, z).await.unwrap();
    }

    async fn update_model(&mut self) {
        self.front_left().await;
        self.front_right().await;
        self.back_left().await;
        self.back_right().await;
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
