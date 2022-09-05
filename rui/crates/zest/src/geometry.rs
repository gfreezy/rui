use std::ops::{Add, Mul, Neg, Sub};

use cgmath::{SquareMatrix, Transform};
use druid_shell::kurbo::Point;

use crate::arithmatic::default_near_equal;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
    pub const ZERO: Self = Size {
        width: 0.0,
        height: 0.0,
    };

    pub fn contains(&self, position: Offset) -> bool {
        position.dx >= 0.0
            && position.dx < self.width
            && position.dy >= 0.0
            && position.dy < self.height
    }
}

impl From<druid_shell::kurbo::Size> for Size {
    fn from(size: druid_shell::kurbo::Size) -> Self {
        Size {
            width: size.width,
            height: size.height,
        }
    }
}

#[derive(Clone, Copy, PartialOrd, Debug, Default)]
pub struct Offset {
    pub dx: f64,
    pub dy: f64,
}

impl PartialEq for Offset {
    fn eq(&self, other: &Self) -> bool {
        default_near_equal(self.dx, other.dx) && default_near_equal(self.dy, other.dy)
    }
}

impl Offset {
    pub const ZERO: Offset = Offset { dx: 0.0, dy: 0.0 };

    pub(crate) fn new(dx: f64, dy: f64) -> Offset {
        Offset { dx, dy }
    }
}

impl Neg for Offset {
    type Output = Offset;

    fn neg(self) -> Offset {
        Offset {
            dx: -self.dx,
            dy: -self.dy,
        }
    }
}

impl From<Offset> for Point {
    fn from(offset: Offset) -> Self {
        Point::new(offset.dx, offset.dy)
    }
}

impl From<Point> for Offset {
    fn from(p: Point) -> Self {
        Offset::new(p.x, p.y)
    }
}

impl Sub<Offset> for Offset {
    type Output = Offset;

    fn sub(self, rhs: Offset) -> Self::Output {
        Offset {
            dx: self.dx - rhs.dx,
            dy: self.dy - rhs.dy,
        }
    }
}

impl Add<Offset> for Offset {
    type Output = Offset;

    fn add(self, rhs: Offset) -> Self::Output {
        Offset {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x, y, z }
    }

    pub(crate) fn dot(&self, other: Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Mul<f64> for Vector3 {
    type Output = Vector3;

    fn mul(self, s: f64) -> Vector3 {
        Vector3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

impl Sub<Vector3> for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Vector3) -> Self::Output {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
pub struct Matrix4(cgmath::Matrix4<f64>);

impl Matrix4 {
    pub fn identity() -> Matrix4 {
        Matrix4(cgmath::Matrix4::identity())
    }

    pub(crate) fn translate(&self, dx: f64, dy: f64) {
        self.0.transform_point(cgmath::Point3::new(dx, dy, 0.));
    }

    pub(crate) fn invert(&self) -> f64 {
        todo!()
    }

    pub(crate) fn perspective_transform(&mut self, _point: Vector3) -> Vector3 {
        todo!()
    }

    pub(crate) fn from_translation(dx: f64, dy: f64) -> Matrix4 {
        Matrix4(cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            dx, dy, 0.,
        )))
    }
}
pub struct Rect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Rect {
    pub fn from_size(size: Size) -> Self {
        Rect {
            x0: 0.,
            y0: 0.,
            x1: size.width,
            y1: size.height,
        }
    }

    pub(crate) fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    pub(crate) fn height(&self) -> f64 {
        self.y1 - self.y0
    }

    pub(crate) fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }
}
