use rand::Rng;

use crate::{ray::Ray, Vec3};

/// Describles when, where and how a ray hit an object.
pub struct HitRecord {
    /// Where did the ray hit the object.
    pub hit_at: Vec3,
    /// The normal of the object at the hit point. Always on the same side as the ray origin with
    /// respect to the object surface.
    pub normal: Vec3,
    /// The ray parameter when the hit occurred.
    pub t: f64,
    /// Where the normal points to.
    pub pointing: Pointing,
}

/// Where the normal points to.
#[derive(Clone, Copy, PartialEq)]
pub enum Pointing {
    /// The normal points towards the inside of the object.
    Inward,
    /// The normal points towards the outside of the object.
    Outward,
}

impl HitRecord {
    fn new(ray: &Ray, t: f64, outward_normal: Vec3) -> Self {
        let pointing = if ray.direction().dot(outward_normal) < 0.0 {
            Pointing::Outward
        } else {
            Pointing::Inward
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
    Object(Box<dyn Hittable>),
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
    /// A random point on this sphere.
    pub fn random_point_on_surface<R: Rng>(&self, rng: &mut R) -> Vec3 {
        let p = loop {
            let p = Vec3(rng.gen());
            if p.norm_squared() < 1.0 {
                break p;
            }
        };

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
            None
        } else {
            let sqrt_d = discriminant.sqrt();
            let mut root = (-half_b - sqrt_d) / a;
            if root < t_min || t_max < root {
                root = (-half_b + sqrt_d) / a;
                if root < t_min || t_max < root {
                    return None;
                }
            }

            let normal = ray.at(root) - self.center;
            Some(HitRecord::new(&ray, root, normal))
        }
    }
}

/// A collection of hittable objects.
#[derive(Default)]
pub struct World {
    objects: Vec<HittableObject>,
}

impl World {
    /// Initialize a new empty world.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an hittable object to the world.
    pub fn add<O: Into<HittableObject>>(&mut self, obj: O) {
        self.objects.push(obj.into());
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        self.objects
            .iter()
            .filter_map(|obj| obj.hit(ray, t_min, t_max))
            .min_by(|rec0, rec1| f64_cmp(rec0.t, rec1.t))
    }
}

fn f64_cmp(a: f64, b: f64) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;

    assert!(!a.is_nan());
    assert!(!b.is_nan());

    if a < b {
        Less
    } else if a > b {
        Greater
    } else {
        Equal
    }
}
