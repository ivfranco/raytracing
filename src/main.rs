use std::{fmt::Display, fs, path::Path, process};

use raytracing::{
    builder::{ImageBuilder, PPMBuilder},
    color::{Rgb, LIGHTBLUE, WHITE},
    hittable::{Hittable, Sphere},
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
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as u32;

    // viewport dimensions
    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    // camera position
    let origin = Vec3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    // geometries
    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, -1.0),
        radius: 0.5,
    };

    let mut ppm_builder = PPMBuilder::with_dimensions(image_width, image_height);

    for j in (0..image_height).rev() {
        for i in 0..image_width {
            let direction = lower_left_corner
                + (i as f64 / (image_width - 1) as f64) * horizontal
                + (j as f64 / (image_height - 1) as f64) * vertical;

            let ray = Ray::new(origin, direction);

            let pixel = if let Some(record) = sphere.hit(&ray, 0.0, 2.0) {
                let normal = record.normal.normalized();
                0.5 * Rgb::new(normal.x() + 1.0, normal.y() + 1.0, normal.z() + 1.0)
            } else {
                background(&ray)
            };

            ppm_builder.put(pixel)?;
        }
    }

    if !Path::new("output").is_dir() {
        fs::create_dir("output")?;
    }

    ppm_builder.output_to_file("output/raytrace.ppm")?;
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
