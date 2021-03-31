use std::{fmt::Display, fs, path::Path, process};

use rand::SeedableRng;
use raytracing::{
    builder::{ImageBuilder, PNGBuilder},
    camera::Camera,
    color::{Rgb, RgbAccumulator, LIGHTBLUE, WHITE},
    hittable::{Hittable, Sphere, World},
    ray::Ray,
    Vec3,
};

fn main() {
    if let Err(e) = exec() {
        error_exit(e);
    }
}

fn exec() -> anyhow::Result<()> {
    // image dimensions
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 720;
    let image_height = (image_width as f64 / aspect_ratio) as u32;

    let focal_length = 1.0;
    let camera = Camera::new(aspect_ratio, focal_length);

    // geometries
    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, -1.0),
        radius: 0.5,
    };

    let ground = Sphere {
        center: Vec3::new(0.0, -100.5, -1.0),
        radius: 100.0,
    };

    let mut world = World::new();
    world.add(sphere);
    world.add(ground);

    let mut image_builder = PNGBuilder::with_dimensions(image_width, image_height);
    let mut rng = rand::rngs::StdRng::from_entropy();

    const SAMPLE_PER_PIXEL: u32 = 100;

    for sampler in camera.cast(image_width, image_height) {
        let mut acc = RgbAccumulator::new();

        for _ in 0..SAMPLE_PER_PIXEL {
            let ray = sampler.sample(&mut rng);
            let pixel = if let Some(record) = world.hit(&ray, 0.0, 2.0) {
                let normal = record.normal.normalized();
                0.5 * Rgb::new(normal.x() + 1.0, normal.y() + 1.0, normal.z() + 1.0)
            } else {
                background(&ray)
            };

            acc.feed(pixel);
        }

        image_builder.put(acc.sample())?;
    }

    if !Path::new("output").is_dir() {
        fs::create_dir("output")?;
    }

    image_builder.output_to_file("output/raytrace.png")?;
    Ok(())
}

fn error_exit<T: Display>(err: T) {
    eprintln!("{:#}", err);
    process::exit(1);
}

fn background(ray: &Ray) -> Rgb {
    let unit_dir = ray.direction().normalized();
    let t = 0.5 * (unit_dir.y() + 1.0);
    (1.0 - t) * WHITE + t * LIGHTBLUE
}
