use std::{cmp, mem};

use rand::Rng;

use crate::{ray::Ray, Vec3};

/// Describes when, where and how a ray hit an object.
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

    /// Return an Axis-Aligned Bounding Box (AABB) containing the hittable object if the object is
    /// bounded. If a ray hits the object, it must also hit the bounding box.
    fn bounding_box(&self) -> Option<AABB>;
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

    fn bounding_box(&self) -> Option<AABB> {
        match self {
            HittableObject::Sphere(sphere) => sphere.bounding_box(),
            HittableObject::Object(obj) => obj.bounding_box(),
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

    fn bounding_box(&self) -> Option<AABB> {
        let r = self.radius;
        Some(AABB::new(
            self.center - Vec3::new(r, r, r),
            self.center + Vec3::new(r, r, r),
        ))
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

/// An Axis-Aligned Bounding Box (AABB).
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    /// Construct a new AABB from two points in 3-dimensional space. The `min` point must have all
    /// its dimensions smaller or equal to the `max` point.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        for i in 0..3 {
            assert!(min[i] <= max[i]);
        }

        Self { min, max }
    }

    /// Test if a ray hits an AABB.
    pub fn hit(&self, ray: &Ray, mut t_min: f64, mut t_max: f64) -> bool {
        for i in 0..3 {
            let (t0, t1) = {
                // When ray.direction[i] == 0.0, inv_d == infinity (positive or negative), if
                // `self.min[i]` and `ray.origin()[i]` have different signs, the entire ray is in
                // the AABB wrt. to ith dimension (i.e. [t0, t1] = (-infinity, +infinity)),
                // otherwise the entire ray is not in the AABB wrt. to ith dimension (i.e. [t0, t1]
                // = (-infinity, -infinity)).
                let inv_d = 1.0 / ray.direction()[i];
                let mut t0 = (self.min[i] - ray.origin()[i]) * inv_d;
                let mut t1 = (self.max[i] - ray.origin()[i]) * inv_d;
                if inv_d < 0.0 {
                    mem::swap(&mut t0, &mut t1);
                }
                (t0, t1)
            };

            // narrow the possible range of t by the calculated range along ith axis
            t_min = t0.max(t_min);
            t_max = t1.min(t_max);

            if t_max <= t_min {
                // the possible range of t is now empty
                return false;
            }
        }

        true
    }

    /// Merge two AABBs, return a bigger AABB containing the two given AABBs.
    pub fn merge(&self, other: &Self) -> Self {
        let mut min = Vec3::default();
        let mut max = Vec3::default();

        for i in 0..3 {
            min[i] = self.min[i].min(other.min[i]);
            max[i] = self.max[i].max(other.max[i]);
        }

        Self::new(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_hit() {
        let aabb = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let go_through_ray = Ray::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(aabb.hit(&go_through_ray, 0.0, f64::INFINITY));

        let parallel_ray = Ray::new(Vec3::new(0.5, 0.5, -1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(aabb.hit(&parallel_ray, 0.0, f64::INFINITY));
    }

    #[test]
    fn aabb_bounding_sphere() {
        let mut rng = rand::thread_rng();

        let sphere = Sphere {
            center: Vec3::new(1.0, 1.0, 1.0),
            radius: 1.0,
        };

        let aabb = sphere.bounding_box().unwrap();

        for _ in 0..100 {
            let direction = Sphere::unit().random_point_on_surface(&mut rng);
            let ray = Ray::new(Vec3::origin(), direction);

            if sphere.hit(&ray, 0.0, f64::INFINITY).is_some() {
                assert!(aabb.hit(&ray, 0.0, f64::INFINITY));
            }
        }
    }
}
