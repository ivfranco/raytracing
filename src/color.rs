use std::ops::Mul;

use derive_more::{Add, AddAssign, Deref, Div, Mul};
use rand::{distributions::Standard, prelude::Distribution};

use crate::Vec3;

/// A pixel with 3 color channels red, green and blue.
#[derive(Default, Deref, Clone, Copy, Add, AddAssign, Div, Mul)]
#[mul(forward)]
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

    /// Return a color whose channels all have been clamped to the valid range.
    pub fn clamp(self) -> Self {
        Self::new(
            self.r().clamp(0.0, 1.0),
            self.g().clamp(0.0, 1.0),
            self.b().clamp(0.0, 1.0),
        )
    }
}

impl Mul<Rgb> for f64 {
    type Output = Rgb;

    fn mul(self, rhs: Rgb) -> Self::Output {
        Rgb(self * rhs.0)
    }
}

impl Distribution<Rgb> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Rgb {
        Rgb(Vec3(rng.gen()))
    }
}

/// Accumulates information about feeded colors for later use.
#[derive(Default)]
pub struct RgbAccumulator {
    sum: Rgb,
    len: u32,
}

impl RgbAccumulator {
    /// Initialize an empty accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed a color sample to the accumulator,
    pub fn feed(&mut self, rgb: Rgb) {
        self.sum += rgb;
        self.len += 1;
    }

    /// Sample a reasonably representative color based on all the feeded colors.
    pub fn sample(&self) -> Rgb {
        let rgb = self.sum / (self.len as f64);
        Rgb::new(rgb.r().sqrt(), rgb.g().sqrt(), rgb.b().sqrt()).clamp()
    }
}

/// hex value: #7FB2FF
pub const LIGHTBLUE: Rgb = Rgb::new(0.5, 0.7, 1.0);
/// hex value: #FFFFFF
pub const WHITE: Rgb = Rgb::new(1.0, 1.0, 1.0);
/// hex value: #000000
pub const BLACK: Rgb = Rgb::new(0.0, 0.0, 0.0);
