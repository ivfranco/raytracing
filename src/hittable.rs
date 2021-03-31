use crate::{ray::Ray, Vec3};

/// Describles when, where and how a ray hit an object.
pub struct HitRecord {
    /// Where did the ray hit the object.
    pub hit_at: Vec3,
    /// The normal of the object at the hit point.
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

/// A sphere described by its center and radius.
pub struct Sphere {
    /// Center of the sphere.
    pub center: Vec3,
    /// Radius of the sphere.
    pub radius: f64,
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
