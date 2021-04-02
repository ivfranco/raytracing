use rand::Rng;

use crate::{
    color::{self, Rgb},
    hittable::{HitRecord, Pointing, Sphere},
    ray::Ray,
    Vec3,
};

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
    /// Dielectric material, always refract light.
    Dielectric(Dielectric),
}

impl Material {
    /// Scatter lights after a hit event on the material.
    pub fn scatter<R: Rng>(&self, rng: &mut R, ray: &Ray, record: &HitRecord) -> Option<Scatter> {
        match self {
            Material::Lambertian(l) => Some(l.scatter(rng, record.normal)),
            Material::Metal(m) => m.scatter(rng, ray, record.normal),
            Material::Dielectric(d) => Some(d.scatter(rng, ray, record)),
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

impl From<Dielectric> for Material {
    fn from(d: Dielectric) -> Self {
        Self::Dielectric(d)
    }
}

/// Lambertian materials, always scatter light randomly in Lambertian distribution.
pub struct Lambertian {
    albedo: Rgb,
}

impl Lambertian {
    /// Construct a Lambertian material with the given color.
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
    /// Construct a metal material with the given color.
    pub fn new(albedo: Rgb, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz: fuzz.min(1.0),
        }
    }

    /// Construct a metal material with the given color.
    pub fn scatter<R: Rng>(&self, rng: &mut R, ray: &Ray, normal: Vec3) -> Option<Scatter> {
        let reflected = reflect(ray.direction().normalized(), normal);
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

fn refract(uv: Vec3, normal: Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = (-uv).dot(normal).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * normal);
    let r_out_parallel = -(1.0 - r_out_perp.norm_squared()).abs().sqrt() * normal;

    r_out_perp + r_out_parallel
}

/// Dielectric material, always refract light.
#[derive(Clone, Copy)]
pub struct Dielectric {
    ir: f64,
}

impl Dielectric {
    /// Construct a dielectric material with the given index of refraction.
    pub fn new(ir: f64) -> Self {
        Self { ir }
    }

    fn scatter<R: Rng>(&self, rng: &mut R, ray: &Ray, record: &HitRecord) -> Scatter {
        let refraction_ratio = match record.pointing {
            Pointing::Inward => self.ir,
            Pointing::Outward => 1.0 / self.ir,
        };

        let unit_direction = ray.direction().normalized();
        let cos_theta = (-unit_direction).dot(record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction = if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng.gen() {
            reflect(unit_direction, record.normal)
        } else {
            refract(unit_direction, record.normal, refraction_ratio)
        };

        Scatter {
            direction,
            attenuation: color::WHITE,
        }
    }
}

fn reflect(direction: Vec3, normal: Vec3) -> Vec3 {
    direction - 2.0 * direction.dot(normal) * normal
}

fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
