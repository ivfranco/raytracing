use rand::Rng;

use crate::{color::Rgb, hittable::Sphere, ray::Ray, Vec3};

/// To which direction and with what attenuation did the light scatter.
pub struct Scatter {
    /// The direction of the scattered light.
    pub direction: Vec3,
    /// the attenuation (what does that even mean) of the scattered light.
    pub attenuation: Rgb,
}

/// Materials with different optical properties.
pub enum Material {
    /// Lambertian materials, always scatter light randomly in Lambertian distribution.
    Lambertian(Lambertian),
    /// Metals, reflect light roughly to the opposite direction.
    Metal(Metal),
}

impl Material {
    /// Scatter lights after a hit event on the material.
    pub fn scatter<R: Rng>(&self, rng: &mut R, ray: &Ray, normal: Vec3) -> Option<Scatter> {
        match self {
            Material::Lambertian(l) => Some(l.scatter(rng, normal)),
            Material::Metal(m) => m.scatter(rng, ray, normal),
        }
    }
}

impl From<Lambertian> for Material {
    fn from(l: Lambertian) -> Self {
        Self::Lambertian(l)
    }
}

impl From<Metal> for Material {
    fn from(m: Metal) -> Self {
        Self::Metal(m)
    }
}

/// Lambertian materials, always scatter light randomly in Lambertian distribution.
pub struct Lambertian {
    albedo: Rgb,
}

impl Lambertian {
    /// Construct a Lambertian meterial with the given color.
    pub fn new(albedo: Rgb) -> Self {
        Self { albedo }
    }

    fn scatter<R: Rng>(&self, rng: &mut R, normal: Vec3) -> Scatter {
        let mut direction = normal + Sphere::unit().random_point_on_surface(rng);
        if direction.near_zero() {
            direction = normal;
        }

        Scatter {
            direction,
            attenuation: self.albedo,
        }
    }
}

/// Metals, reflect light roughly to the opposite direction.
pub struct Metal {
    albedo: Rgb,
    fuzz: f64,
}

impl Metal {
    /// Construct a metal meterial with the given color.
    pub fn new(albedo: Rgb, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz: fuzz.min(1.0),
        }
    }

    /// Construct a metal meterial with the given color.
    pub fn scatter<R: Rng>(&self, rng: &mut R, ray: &Ray, normal: Vec3) -> Option<Scatter> {
        let reflected = ray.direction().normalized().reflect_on(normal);
        let direction = reflected + self.fuzz * Sphere::unit().random_point_in_sphere(rng);

        // the surface absorbs all rays fuzzed into it.
        if reflected.same_direction(normal) {
            Some(Scatter {
                direction,
                attenuation: self.albedo,
            })
        } else {
            None
        }
    }
}
