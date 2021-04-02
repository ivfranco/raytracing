use rand::Rng;

use crate::{ray::Ray, Vec3};

/// Describles when, where and how a ray hit an object.
pub struct HitRecord {
    /// Where did the ray hit the object.
    pub hit_at: Vec3,
    /// The normal of the object at the hit point that's always
    /// - on the same side as the ray origin with respect to the object surface
    /// - normalized to unit norm
    pub normal: Vec3,
    /// The ray parameter when the hit occurred.
    pub t: f64,
    /// Where the normal points to.
    pub pointing: Pointing,
}

/// Where the normal points to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pointing {
    /// The normal points towards the inside of the object.
    Inward,
    /// The normal points towards the outside of the object.
    Outward,
}

impl HitRecord {
    fn new(ray: &Ray, t: f64, outward_normal: Vec3) -> Self {
        let pointing = if ray.direction().same_direction(outward_normal) {
            Pointing::Inward
        } else {
            Pointing::Outward
        };

        let normal = match pointing {
            Pointing::Inward => -outward_normal,
            Pointing::Outward => outward_normal,
        };

        Self {
            hit_at: ray.at(t),
            normal,
            t,
            pointing,
        }
    }
}

/// An object that may be hit by and reflect a ray.
pub trait Hittable {
    /// Hit the object with a ray, return a hit record if the ray intersects the object within the
    /// given range of ray parameter [t_min, t_max].
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

/// An enum wrapping the boxed Hittable and a few named objects so methods of [Hittable](Hittable)
/// still can be statically dispatched most of the time.
pub enum HittableObject {
    /// An sphere.
    Sphere(Sphere),
    /// A general [Hittable](Hittable) trait object.
    Object(Box<dyn Hittable + Send + Sync>),
}

impl Hittable for HittableObject {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        match self {
            HittableObject::Sphere(sphere) => sphere.hit(ray, t_min, t_max),
            HittableObject::Object(obj) => obj.hit(ray, t_min, t_max),
        }
    }
}

impl From<Sphere> for HittableObject {
    fn from(sphere: Sphere) -> Self {
        Self::Sphere(sphere)
    }
}

/// A sphere described by its center and radius.
pub struct Sphere {
    /// Center of the sphere.
    pub center: Vec3,
    /// Radius of the sphere.
    pub radius: f64,
}

impl Sphere {
    /// An unit radius sphere at the origin point.
    pub const fn unit() -> Self {
        Self {
            center: Vec3::origin(),
            radius: 1.0,
        }
    }

    /// A random point in this sphere.
    pub fn random_point_in_sphere<R: Rng>(&self, rng: &mut R) -> Vec3 {
        let p = random_unit(rng);
        self.radius * p + self.center
    }

    /// A random point on this sphere.
    pub fn random_point_on_surface<R: Rng>(&self, rng: &mut R) -> Vec3 {
        let p = random_unit(rng);
        self.radius * p.normalized() + self.center
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin() - self.center;
        let a = ray.direction().norm_squared();
        let half_b = oc.dot(ray.direction());

        let c = oc.norm_squared() - self.radius.powi(2);
        let discriminant = half_b.powi(2) - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let root = std::array::IntoIter::new([(-half_b - sqrt_d) / a, (-half_b + sqrt_d) / a])
            .find(|&root| t_min <= root && root <= t_max)?;

        // must be normalized here: radius may be negative as a trick to describe the hollow inside
        // of a sphere
        let normal = (ray.at(root) - self.center) / self.radius;
        Some(HitRecord::new(&ray, root, normal))
    }
}

fn random_unit<R: Rng>(rng: &mut R) -> Vec3 {
    loop {
        let p = Vec3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        );

        if p.norm_squared() < 1.0 {
            return p;
        }
    }
}
