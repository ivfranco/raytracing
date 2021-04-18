use rand::Rng;

use crate::{
    hittable::{HitRecord, Hittable, HittableObject, AABB},
    material::{Material, Scatter},
    ray::Ray,
    Error, NonNan, Vec3,
};

/// The result of a ray hitting the world.
pub struct HitEvent {
    /// When, where and how a ray hit an object.
    pub record: HitRecord,
    /// Whether and how the ray scattered after the hit.
    pub scatter: Option<Scatter>,
}

/// Builder of [World], a collection of hittable objects.
#[derive(Default)]
pub struct WorldBuilder {
    objects: Vec<(HittableObject, Material)>,
}

impl WorldBuilder {
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

    /// Build a world with efficient hit detection.
    pub fn build(self) -> Result<World, Error> {
        let mut nodes: Vec<_> = self
            .objects
            .into_iter()
            .map(|(object, material)| BVH::Leaf { object, material })
            .collect();
        let mut rng = rand::thread_rng();

        while nodes.len() > 1 {
            let axis = rng.gen_range(0..Vec3::DIMENSIONS);
            nodes.sort_by_key(|node| {
                node.bounding_box()
                    .map(|b| NonNan::new(b.min[axis]).unwrap())
            });

            let mut temp = vec![];

            while let Some(left) = nodes.pop() {
                let right = match nodes.pop() {
                    Some(node) => node,
                    None => {
                        temp.push(left);
                        break;
                    }
                };

                let aabb_left = left.bounding_box().ok_or(Error::ObjectNotBounded)?;
                let aabb_right = right.bounding_box().ok_or(Error::ObjectNotBounded)?;

                let node = BVH::Node {
                    aabb: aabb_left.merge(&aabb_right),
                    left: Box::new(left),
                    right: Box::new(right),
                };

                temp.push(node);
            }

            nodes = temp;
        }

        Ok(World {
            bvh: nodes.swap_remove(0),
        })
    }
}

/// A collection of hittable objects. Support more efficient hit detection than a simple vector of
/// objects and materials.
pub struct World {
    bvh: BVH,
}

impl World {
    /// Hit the world with a ray.
    pub fn hit<R: Rng>(&self, rng: &mut R, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitEvent> {
        self.bvh.hit(rng, ray, t_min, t_max)
    }
}

enum BVH {
    Leaf {
        object: HittableObject,
        material: Material,
    },
    Node {
        aabb: AABB,
        left: Box<BVH>,
        right: Box<BVH>,
    },
}

impl BVH {
    fn bounding_box(&self) -> Option<AABB> {
        match self {
            BVH::Leaf { object, .. } => object.bounding_box(),
            BVH::Node { aabb, .. } => Some(aabb.clone()),
        }
    }

    fn hit<R: Rng>(&self, rng: &mut R, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitEvent> {
        match self {
            BVH::Leaf { object, material } => {
                object.hit(ray, t_min, t_max).map(|record| HitEvent {
                    scatter: material.scatter(rng, ray, &record),
                    record,
                })
            }
            BVH::Node { aabb, left, right } => {
                if !aabb.hit(ray, t_min, t_max) {
                    None
                } else {
                    let hit_left = left.hit(rng, ray, t_min, t_max);
                    let t = hit_left.as_ref().map(|rec| rec.record.t).unwrap_or(t_max);
                    let hit_right = right.hit(rng, ray, t_min, t);

                    // if the ray right subtree, the hit is closer to the source of the ray than the
                    // hit event from the left subtree, the right hit event should be preferred
                    hit_right.or(hit_left)
                }
            }
        }
    }
}
