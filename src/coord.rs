use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy)]
pub struct Coord {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Coord {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Coord { x, y, z }
    }
}

impl Add for Coord {
    type Output = Coord;

    fn add(self, rhs: Self) -> Self::Output {
        Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<f32> for Coord {
    type Output = Coord;

    fn add(self, rhs: f32) -> Self::Output {
        Coord {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

impl Sub for Coord {
    type Output = Coord;

    fn sub(self, rhs: Self) -> Self::Output {
        Coord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<f32> for Coord {
    type Output = Coord;

    fn sub(self, rhs: f32) -> Self::Output {
        Coord {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}

impl Mul for Coord {
    type Output = Coord;

    fn mul(self, rhs: Self) -> Self::Output {
        Coord {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<f32> for Coord {
    type Output = Coord;

    fn mul(self, rhs: f32) -> Self::Output {
        Coord {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div for Coord {
    type Output = Coord;

    fn div(self, rhs: Self) -> Self::Output {
        Coord {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Div<f32> for Coord {
    type Output = Coord;

    fn div(self, rhs: f32) -> Self::Output {
        Coord {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}
