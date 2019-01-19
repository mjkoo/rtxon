use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

pub type Scalar = f32;
pub type Vector3 = nalgebra::Vector3<Scalar>;
pub type Point3 = nalgebra::Point3<Scalar>;
//pub type Matrix4 = nalgebra::Matrix4<Scalar>;

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vector3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vector3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn at(&self, t: Scalar) -> Point3 {
        self.origin + t * self.direction
    }
}

impl From<bvh::ray::Ray> for Ray {
    fn from(ray: bvh::ray::Ray) -> Self {
        Self::new(ray.origin, ray.direction)
    }
}

impl Into<bvh::ray::Ray> for Ray {
    fn into(self) -> bvh::ray::Ray {
        bvh::ray::Ray::new(self.origin, self.direction)
    }
}

fn scalar_to_u8(f: Scalar) -> u8 {
    (f * 255.99) as u8
}

fn u8_to_scalar(b: u8) -> Scalar {
    Scalar::from(b) / 255.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: Scalar,
    pub g: Scalar,
    pub b: Scalar,
    pub a: Scalar,
}

impl Color {
    pub fn new(r: Scalar, g: Scalar, b: Scalar, a: Scalar) -> Self {
        Self { r, g, b, a }
    }
}

impl From<image::Rgba<u8>> for Color {
    fn from(rgba: image::Rgba<u8>) -> Self {
        Self {
            r: u8_to_scalar(rgba.data[0]),
            g: u8_to_scalar(rgba.data[1]),
            b: u8_to_scalar(rgba.data[2]),
            a: u8_to_scalar(rgba.data[3]),
        }
    }
}

impl Into<image::Rgba<u8>> for Color {
    fn into(self) -> image::Rgba<u8> {
        image::Rgba([
            scalar_to_u8(self.r),
            scalar_to_u8(self.g),
            scalar_to_u8(self.b),
            scalar_to_u8(self.a),
        ])
    }
}

impl From<image::Rgb<u8>> for Color {
    fn from(rgba: image::Rgb<u8>) -> Self {
        Self {
            r: u8_to_scalar(rgba.data[0]),
            g: u8_to_scalar(rgba.data[1]),
            b: u8_to_scalar(rgba.data[2]),
            a: 1.0,
        }
    }
}

impl Into<image::Rgb<u8>> for Color {
    fn into(self) -> image::Rgb<u8> {
        image::Rgb([
            scalar_to_u8(self.r),
            scalar_to_u8(self.g),
            scalar_to_u8(self.b),
        ])
    }
}

impl From<Vector3> for Color {
    fn from(v: Vector3) -> Self {
        Self {
            r: v.x,
            g: v.y,
            b: v.z,
            a: 1.0,
        }
    }
}

macro_rules! arith {
    ( $trait:ty, $scalar_trait:ty, $func:ident, $assign_trait:ty, $assign_scalar_trait:ty, $assign_func:ident, $op:tt ) => {
        impl $trait for Color {
            type Output = Color;

            fn $func(self, other: Color) -> Self::Output {
                Self::Output {
                    r: self.r $op other.r,
                    g: self.g $op other.g,
                    b: self.b $op other.b,
                    a: self.a $op other.a,
                }
            }
        }

        impl $scalar_trait for Color {
            type Output = Color;

            fn $func(self, other: Scalar) -> Self::Output {
                Self::Output {
                    r: self.r $op other,
                    g: self.g $op other,
                    b: self.b $op other,
                    a: self.a $op other,
                }
            }
        }

        impl $assign_trait for Color {
            fn $assign_func(&mut self, other: Color) {
                *self = Color {
                    r: self.r $op other.r,
                    g: self.g $op other.g,
                    b: self.b $op other.b,
                    a: self.a $op other.a,
                }
            }
        }

        impl $assign_scalar_trait for Color {
            fn $assign_func(&mut self, other: Scalar) {
                *self = Color {
                    r: self.r $op other,
                    g: self.g $op other,
                    b: self.b $op other,
                    a: self.a $op other,
                }
            }
        }
    }
}

arith!(Add, Add<Scalar>, add, AddAssign, AddAssign<Scalar>, add_assign, +);
arith!(Div, Div<Scalar>, div, DivAssign, DivAssign<Scalar>, div_assign, /);
arith!(Mul, Mul<Scalar>, mul, MulAssign, MulAssign<Scalar>, mul_assign, *);
arith!(Sub, Sub<Scalar>, sub, SubAssign, SubAssign<Scalar>, sub_assign, -);
