use derive_more::Deref;

use crate::Vec3;

/// A pixel with 3 color channels red, green and blue.
#[derive(Deref)]
pub struct Rgb(Vec3);

impl Rgb {
    pub fn r(self) -> f64 {
        self[0]
    }

    pub fn g(self) -> f64 {
        self[1]
    }

    pub fn b(self) -> f64 {
        self[2]
    }
}
