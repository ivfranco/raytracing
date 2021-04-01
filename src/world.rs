use rand::Rng;

use crate::{
    hittable::{HitRecord, Hittable, HittableObject},
    material::{Material, Scatter},
    ray::Ray,
};

/// The result of a ray hitting the world.
pub struct HitEvent {
    /// When, where and how a ray hit an object.
    pub record: HitRecord,
    /// Whether and how the ray scattered after the hit.
    pub scatter: Option<Scatter>,
}

/// A collection of hittable objects.
#[derive(Default)]
pub struct World {
    objects: Vec<(HittableObject, Material)>,
}

impl World {
    /// Initialize a new empty world.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an hittable object to the world.
    pub fn add<O, M>(&mut self, obj: O, material: M)
    where
        O: Into<HittableObject>,
        M: Into<Material>,
    {
        self.objects.push((obj.into(), material.into()))
    }

    /// Hit the world with a ray.
    pub fn hit<R: Rng>(&self, rng: &mut R, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitEvent> {
        let (record, material) = self
            .objects
            .iter()
            .filter_map(|(obj, material)| {
                let record = obj.hit(ray, t_min, t_max)?;
                Some((record, material))
            })
            .min_by(|(record0, _), (record1, _)| f64_cmp(record0.t, record1.t))?;

        let scatter = material.scatter(rng, ray, record.normal);

        Some(HitEvent { record, scatter })
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
