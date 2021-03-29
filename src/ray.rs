use crate::Vec3;

/// A ray in 3-dimensional coordinate system.
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    /// Construct a ray from an origin point and a direction.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Get the ray's origin.
    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    /// Get the ray's direction.
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Return the position on the ray given the ray parameter.
    /// P(t) = A + tb, where A = origin, b = direction
    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + t * self.direction
    }
}
