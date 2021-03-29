//! A CPU-based ray tracer.

#![deny(missing_docs)]

/// A trait generalizing image file types.
pub mod printer;

pub(crate) mod color;

use derive_more::{Index, IndexMut};

use std::ops::{Div, Mul, Neg};

/// A vector with three components.
#[derive(Clone, Copy, Index, IndexMut)]
pub struct Vec3([f64; 3]);

impl Vec3 {
    /// Initialize the vector with elements.
    pub fn new(e0: f64, e1: f64, e2: f64) -> Self {
        Self([e0, e1, e2])
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
        self / self.norm()
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

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
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
