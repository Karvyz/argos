use xgo::Motor;

const X: f32 = 10.;
const Y: f32 = 10.;
const Z: f32 = 10.;

pub struct Coord {
    x: f32,
    y: f32,
    z: f32,
}

pub struct Model {}

impl Model {
    pub fn new() -> Self {
        Model {}
    }

    pub fn motor(&self, motor: Motor) -> f64 {
        0.0
    }

    pub fn leg(&self, origin: Coord, objective: Coord) {
        let dv = origin.y - objective.y;
        let dh = origin.z - objective.z;

        let d = f32::sqrt(dv * dv + dh * dh);
        let a = f32::acos((d * d + dv * dv + dh * dh) / 2 * d * dv);
    }
}
