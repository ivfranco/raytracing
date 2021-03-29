use std::ops::Mul;

use derive_more::{Add, Deref};

use crate::Vec3;

/// A pixel with 3 color channels red, green and blue.
#[derive(Deref, Clone, Copy, Add)]
pub struct Rgb(Vec3);

impl Rgb {
    /// Initialize the color with red, green and blue channels.
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Vec3::new(r, g, b))
    }

    /// Value of the red channel.
    pub fn r(self) -> f64 {
        self[0]
    }

    /// Value of the green channel.
    pub fn g(self) -> f64 {
        self[1]
    }

    /// Value of the blue channel.
    pub fn b(self) -> f64 {
        self[2]
    }
}

impl Mul<Rgb> for f64 {
    type Output = Rgb;

    fn mul(self, rhs: Rgb) -> Self::Output {
        Rgb(self * rhs.0)
    }
}

/// hex value: #
pub const LIGHTBLUE: Rgb = Rgb::new(0.5, 0.7, 1.0);
/// hex value: #FFFFFF
pub const WHITE: Rgb = Rgb::new(1.0, 1.0, 1.0);
