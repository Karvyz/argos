use glam::{Quat, Vec3};
use xgo::Motor;

const SHOULDER_LENGTH: f32 = 2.86;
const UPPER_LENGTH: f32 = 5.5;
const LOWER_LENGTH: f32 = 6.68;
const BODY_WIDTH: f32 = 5.;
const BODY_LENGTH: f32 = 20.;
const BASE_HEIGHT: f32 = 10.;
const LEG_FR: Vec3 = Vec3::new(BODY_LENGTH / 2., 0., BODY_WIDTH / 2.);
const LEG_FL: Vec3 = Vec3::new(BODY_LENGTH / 2., 0., -BODY_WIDTH / 2.);
const LEG_BR: Vec3 = Vec3::new(-BODY_LENGTH / 2., 0., BODY_WIDTH / 2.);
const LEG_BL: Vec3 = Vec3::new(-BODY_LENGTH / 2., 0., -BODY_WIDTH / 2.);
const FEET_FR: Vec3 = Vec3::new(BODY_LENGTH / 2., 0., BODY_WIDTH / 2. + SHOULDER_LENGTH);
const FEET_FL: Vec3 = Vec3::new(BODY_LENGTH / 2., 0., -BODY_WIDTH / 2. - SHOULDER_LENGTH);
const FEET_BR: Vec3 = Vec3::new(-BODY_LENGTH / 2., 0., BODY_WIDTH / 2. + SHOULDER_LENGTH);
const FEET_BL: Vec3 = Vec3::new(-BODY_LENGTH / 2., 0., -BODY_WIDTH / 2. - SHOULDER_LENGTH);

pub struct Model {
    pub position: Vec3,
    rotation: Quat,
    legs: [Vec3; 4],
    pub feets: [Vec3; 4],
    rotated_feets: [Vec3; 4],
    pub angles: [(Motor, f32); 15],
}

impl Model {
    pub fn new() -> Self {
        Model {
            position: Vec3::new(0., BASE_HEIGHT, 0.),
            rotation: Quat::IDENTITY,
            legs: [Vec3::ZERO; 4],
            feets: [FEET_FL, FEET_FR, FEET_BL, FEET_BR],
            rotated_feets: [Vec3::ZERO; 4],
            angles: [(Motor::Claw, 0.); 15],
        }
    }

    pub fn rotation(&mut self, pitch: f32, roll: f32, yaw: f32) {
        let qpitch = Quat::from_rotation_z(-pitch.to_radians());
        let qroll = Quat::from_rotation_x(roll.to_radians());
        let qyaw = Quat::from_rotation_y(yaw.to_radians());
        self.rotation = qyaw * qpitch * qroll;
    }

    pub fn rotate(&mut self, pitch: f32, roll: f32, yaw: f32) {
        let qpitch = Quat::from_rotation_z(-pitch.to_radians());
        let qroll = Quat::from_rotation_x(roll.to_radians());
        let qyaw = Quat::from_rotation_y(yaw.to_radians());
        self.rotation = (self.rotation * qyaw * qpitch * qroll).normalize();
    }

    pub fn tilt_forward(&mut self, degrees: f32) {
        self.rotate(degrees, 0., 0.);
    }

    pub fn tilt_backward(&mut self, degrees: f32) {
        self.rotate(-degrees, 0., 0.);
    }

    pub fn tilt_left(&mut self, degrees: f32) {
        self.rotate(0., -degrees, 0.);
    }

    pub fn tilt_right(&mut self, degrees: f32) {
        self.rotate(0., degrees, 0.);
    }

    pub fn turn_left(&mut self, degrees: f32) {
        self.rotate(0., 0., degrees);
    }

    pub fn turn_right(&mut self, degrees: f32) {
        self.rotate(0., 0., -degrees);
    }

    pub fn level(&mut self) {
        self.rotation = Quat::IDENTITY;
    }

    pub fn update(&mut self) {
        self.legs[0] = self.position + self.rotation * LEG_FL;
        self.legs[1] = self.position + self.rotation * LEG_FR;
        self.legs[2] = self.position + self.rotation * LEG_BL;
        self.legs[3] = self.position + self.rotation * LEG_BR;

        for i in 0..4 {
            self.rotated_feets[i] = self.rotation * self.feets[i];
        }

        self.front_left();
        self.front_right();
        self.back_left();
        self.back_right();
    }

    fn front_left(&mut self) {
        let (x, y, z) = self.leg(&self.legs[0], &self.rotated_feets[0], false);
        self.angles[0] = (Motor::ShoulderFL, x);
        self.angles[1] = (Motor::UpperLegFL, y);
        self.angles[2] = (Motor::LowerLegFL, z);
    }

    fn front_right(&mut self) {
        let (x, y, z) = self.leg(&self.legs[1], &self.rotated_feets[1], false);
        self.angles[3] = (Motor::ShoulderFR, x);
        self.angles[4] = (Motor::UpperLegFR, y);
        self.angles[5] = (Motor::LowerLegFR, z);
    }

    fn back_left(&mut self) {
        let (x, y, z) = self.leg(&self.legs[2], &self.rotated_feets[2], false);
        self.angles[6] = (Motor::ShoulderBL, x);
        self.angles[7] = (Motor::UpperLegBL, y);
        self.angles[8] = (Motor::LowerLegBL, z);
    }

    fn back_right(&mut self) {
        let (x, y, z) = self.leg(&self.legs[3], &self.rotated_feets[3], false);
        self.angles[9] = (Motor::ShoulderBR, x);
        self.angles[10] = (Motor::UpperLegBR, y);
        self.angles[11] = (Motor::LowerLegBR, z);
    }

    fn leg(&self, origin: &Vec3, objective: &Vec3, debug: bool) -> (f32, f32, f32) {
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
        let j = match dx > 0. {
            true => -j,
            false => j,
        };

        let k = ((dd * dd + UPPER_LENGTH * UPPER_LENGTH - LOWER_LENGTH * LOWER_LENGTH)
            / (2. * UPPER_LENGTH * dd))
            .acos();

        let lower = ((UPPER_LENGTH * UPPER_LENGTH + LOWER_LENGTH * LOWER_LENGTH - dd * dd)
            / (2. * UPPER_LENGTH * LOWER_LENGTH))
            .acos()
            .to_degrees()
            - 90.;
        let upper = k.to_degrees() + j.to_degrees();

        if debug {
            println!("--- leg debug ---");
            println!(
                "  origin:    ({:>8.3}, {:>8.3}, {:>8.3})",
                origin.x, origin.y, origin.z
            );
            println!(
                "  objective: ({:>8.3}, {:>8.3}, {:>8.3})",
                objective.x, objective.y, objective.z
            );
            println!("  dx={:.3}  dy={:.3}  dz={:.3}", dx, dy, dz);
            println!("  dyz={:.3}", dyz);
            println!(
                "  a={:.3}°  b={:.3}°  shoulder={:.3}°",
                a.to_degrees(),
                b.to_degrees(),
                shoulder
            );
            println!("  leg_length={:.3}  dd={:.3}", leg_length, dd);
            println!(
                "  j={:.3}°  k={:.3}°  upper={:.3}°",
                j.to_degrees(),
                k.to_degrees(),
                upper
            );
            println!("  lower={:.3}°", lower);
            println!("---");
        }

        (shoulder, upper, lower)
    }
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_vec3(v: &Vec3) -> String {
            format!("({:>8.3}, {:>8.3}, {:>8.3})", v.x, v.y, v.z)
        }

        fn fmt_quat(q: &Quat) -> String {
            format!("({:>8.3}, {:>8.3}, {:>8.3}, {:>8.3})", q.x, q.y, q.z, q.w)
        }

        fn fmt_angle(a: f32) -> String {
            format!("{:>+7.2}°", a)
        }

        write!(f, "Model: \n")?;
        write!(f, "position:\t{}\n", fmt_vec3(&self.position))?;
        write!(f, "rotation:\t{}\n", fmt_quat(&self.rotation))?;
        write!(
            f,
            "legs:\nFL\t{}\nFR\t{}\nBL\t{}\nBR\t{}\n",
            fmt_vec3(&self.legs[0]),
            fmt_vec3(&self.legs[1]),
            fmt_vec3(&self.legs[2]),
            fmt_vec3(&self.legs[3]),
        )?;
        write!(
            f,
            "feets:\nFL\t{}\nFR\t{}\nBL\t{}\nBR\t{}\n",
            fmt_vec3(&self.feets[0]),
            fmt_vec3(&self.feets[1]),
            fmt_vec3(&self.feets[2]),
            fmt_vec3(&self.feets[3]),
        )?;
        write!(
            f,
            "rotated feets:\nFL\t{}\nFR\t{}\nBL\t{}\nBR\t{}\n",
            fmt_vec3(&self.rotated_feets[0]),
            fmt_vec3(&self.rotated_feets[1]),
            fmt_vec3(&self.rotated_feets[2]),
            fmt_vec3(&self.rotated_feets[3]),
        )?;
        write!(
            f,
            "angles:
"
        )?;
        for (i, chunk) in self.angles.chunks(3).enumerate() {
            let leg_label = match i {
                0 => "FL",
                1 => "FR",
                2 => "BL",
                3 => "BR",
                _ => "??",
            };
            write!(
                f,
                "{}\tshoulder={}\tupper={}\tlower={}\n",
                leg_label,
                fmt_angle(chunk[0].1),
                fmt_angle(chunk[1].1),
                fmt_angle(chunk[2].1),
            )?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilt_forward_lowers_front_of_body() {
        let mut model = Model::new();

        model.tilt_forward(10.);
        model.update();

        assert!(model.legs[0].y < model.legs[2].y);
        assert!(model.legs[1].y < model.legs[3].y);
    }

    #[test]
    fn tilt_right_lowers_right_side_of_body() {
        let mut model = Model::new();

        model.tilt_right(10.);
        model.update();

        assert!(model.legs[1].y < model.legs[0].y);
        assert!(model.legs[3].y < model.legs[2].y);
    }

    #[test]
    fn turn_left_rotates_front_toward_left_without_tilting() {
        let mut model = Model::new();

        model.turn_left(10.);
        model.update();

        let front_z = model.legs[0].z + model.legs[1].z;
        let back_z = model.legs[2].z + model.legs[3].z;
        assert!(front_z < back_z);
        assert!(
            model
                .legs
                .iter()
                .all(|leg| (leg.y - BASE_HEIGHT).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn tilt_forward_adds_to_existing_left_turn() {
        let mut model = Model::new();
        model.turn_left(25.);
        let left_turn = model.rotation;

        model.tilt_forward(10.);

        let forward_tilt = Quat::from_rotation_z(-10_f32.to_radians());
        let expected = left_turn * forward_tilt;
        assert!(model.rotation.abs_diff_eq(expected, f32::EPSILON));
    }

    #[test]
    fn level_restores_identity_rotation() {
        let mut model = Model::new();
        model.rotation(10., -5., 0.);

        model.level();

        assert_eq!(model.rotation, Quat::IDENTITY);
    }
}
