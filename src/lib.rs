//! A CPU-based ray tracer.

#![deny(missing_docs)]
#![allow(clippy::clippy::upper_case_acronyms)]

/// A trait generalizing image file types.
pub mod image_builder;

/// Color types and color constants.
pub mod color;

/// Ray in 3-dimensional space.
pub mod ray;

/// Objects that may be hit and reflect a ray.
pub mod hittable;

/// A camera from where all rays originate.
pub mod camera;

/// Materials with different optical properties.
pub mod material;

/// A collection of hittable objects and their materials.
pub mod world;

use derive_more::{Index, IndexMut};

use std::{
    fmt::{Debug, Display},
    ops::{Div, Mul, Neg},
};

/// The error type for image output operations. Error from std / third party crates is handled by
/// `anyhow` thus not listed here.
#[derive(thiserror::Error)]
pub enum Error {
    /// The number of pixels exceeded the capacity of the image builder.
    #[error("Putting more pixels than the image builder can handle")]
    ImageBufferOverflow,

    /// Value of color channel is not in the range 0.0 .. 1.0.
    #[error("Value of color channel not in range")]
    ColorOutOfRange,
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

/// A vector with 3 components.
#[derive(Debug, Default, Clone, Copy, Index, IndexMut, PartialEq)]
pub struct Vec3([f64; 3]);

impl Vec3 {
    /// Initialize the vector with 3 components.
    pub const fn new(e0: f64, e1: f64, e2: f64) -> Self {
        Self([e0, e1, e2])
    }

    /// The origin point (0, 0, 0).
    pub const fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Return whether the vector is very close to zero in all dimensions.
    pub fn near_zero(self) -> bool {
        const EPSILON: f64 = 1e-8;
        self.0.iter().all(|&e| e < EPSILON)
    }

    /// Get the value of the first component of the vector.
    pub fn x(self) -> f64 {
        self[0]
    }

    /// Get the value of the second component of the vector.
    pub fn y(self) -> f64 {
        self[1]
    }

    /// Get the value of the third component of the vector.
    pub fn z(self) -> f64 {
        self[2]
    }

    /// Square norm of the vector.
    pub fn norm(self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Square of the square norm of the vector.
    pub fn norm_squared(self) -> f64 {
        self[0] * self[0] + self[1] * self[1] + self[2] * self[2]
    }

    /// Dot product of vectors.
    pub fn dot(self, rhs: Self) -> f64 {
        let [e0, e1, e2] = (self * rhs).0;
        e0 + e1 + e2
    }

    /// Cross product of vectors.
    pub fn cross(self, rhs: Self) -> Self {
        Self([
            self[1] * rhs[2] - self[2] * rhs[1],
            self[2] * rhs[0] - self[0] * rhs[2],
            self[0] * rhs[1] - self[1] * rhs[0],
        ])
    }

    /// The unit vector pointing to the same direction.
    pub fn normalized(self) -> Self {
        self.stretch(1.0)
    }

    /// Stretch the vector to a given norm.
    pub fn stretch(self, norm: f64) -> Self {
        assert_ne!(self, Vec3::origin());
        self * (norm / self.norm())
    }

    /// Return whether the angle between the two vectors is smaller than 90 degree.
    pub fn same_direction(self, other: Self) -> bool {
        self.dot(other) > 0.0
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self([-self[0], -self[1], -self[2]])
    }
}

macro_rules! impl_bin_op {
    ($tre: ident, $func: ident, $op: tt) => {
        impl std::ops::$tre for Vec3 {
            type Output = Self;

            fn $func(self, rhs: Self) -> Self::Output {
                Self([self[0] $op rhs[0], self[1] $op rhs[1], self[2] $op rhs[2]])
            }
        }
    }
}

impl_bin_op!(Add, add, +);
impl_bin_op!(Sub, sub, -);
impl_bin_op!(Mul, mul, *);
impl_bin_op!(Div, div, /);

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self([self[0] * rhs, self[1] * rhs, self[2] * rhs])
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        assert!(rhs.abs() >= f64::EPSILON);
        Self([self[0] / rhs, self[1] / rhs, self[2] / rhs])
    }
}

// generates assembly equivalent to a direct implementation:
// https://godbolt.org/z/6zdbve7bd
macro_rules! impl_assign_op {
    ($tre: ident, $func: ident, $op: tt) => {
        impl std::ops::$tre for Vec3 {
            fn $func(&mut self, rhs: Self) {
                *self = *self $op rhs;
            }
        }
    }
}

impl_assign_op!(AddAssign, add_assign, +);
impl_assign_op!(SubAssign, sub_assign, -);
impl_assign_op!(DivAssign, div_assign, /);
