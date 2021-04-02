use std::{fmt::Display, fs, path::Path, process};

use rand::{rngs::StdRng, Rng, SeedableRng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use raytracing::{
    camera::{Camera, CameraBuilder},
    color::{Rgb, RgbAccumulator, BLACK, LIGHTBLUE, WHITE},
    hittable::Sphere,
    image_builder::{ImageBuilder, PNGBuilder},
    material::{Dielectric, Lambertian, Metal},
    ray::Ray,
    world::{HitEvent, World},
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
    let image_width = 600;
    let image_height = (image_width as f64 / aspect_ratio) as u32;

    let camera = CameraBuilder::new().build();

    // geometries
    let world = {
        let mut world = World::new();

        // ground
        world.add(
            Sphere {
                center: Vec3::new(0.0, -100.5, -1.0),
                radius: 100.0,
            },
            Lambertian::new(Rgb::new(0.8, 0.8, 0.0)),
        );

        // central lambertian sphere
        world.add(
            Sphere {
                center: Vec3::new(0.0, 0.0, -1.0),
                radius: 0.5,
            },
            Lambertian::new(Rgb::new(0.1, 0.2, 0.5)),
        );

        let left_material = Dielectric::new(1.5);

        // left dielectric sphere
        world.add(
            Sphere {
                center: Vec3::new(-1.0, 0.0, -1.0),
                radius: 0.5,
            },
            left_material,
        );

        // hollow core of left dielectric sphere
        world.add(
            Sphere {
                center: Vec3::new(-1.0, 0.0, -1.0),
                radius: -0.4,
            },
            left_material,
        );

        // right metal sphere
        world.add(
            Sphere {
                center: Vec3::new(1.0, 0.0, -1.0),
                radius: 0.5,
            },
            Metal::new(Rgb::new(0.8, 0.6, 0.2), 0.0),
        );

        world
    };

    let mut image_builder = PNGBuilder::with_dimensions(image_width, image_height);

    const SAMPLE_PER_PIXEL: u32 = 100;

    let instant = std::time::Instant::now();

    let samplers: Vec<_> = camera.cast(image_width, image_height).collect();
    let pixels: Vec<_> = samplers
        .par_iter()
        .map(|sampler| {
            let mut acc = RgbAccumulator::new();
            let mut rng = StdRng::from_entropy();

            for _ in 0..SAMPLE_PER_PIXEL {
                let ray = sampler.sample(&mut rng);
                let pixel = ray_color(&mut rng, &ray, &world);
                acc.feed(pixel);
            }

            acc.sample()
        })
        .collect();

    for pixel in pixels {
        image_builder.put(pixel)?;
    }

    if !Path::new("output").is_dir() {
        fs::create_dir("output")?;
    }

    image_builder.output_to_file("output/raytrace.png")?;

    println!("{:?}", instant.elapsed());
    Ok(())
}

fn error_exit<T: Display>(err: T) {
    eprintln!("{:#}", err);
    process::exit(1);
}

fn ray_color<R: Rng>(rng: &mut R, ray: &Ray, world: &World) -> Rgb {
    const MAXIMUM_REFLECTION: usize = 64;
    let mut attenuations = Vec::with_capacity(MAXIMUM_REFLECTION);

    let mut reflect_cnt = 0;
    let mut ray = ray.clone();

    loop {
        if reflect_cnt >= MAXIMUM_REFLECTION {
            attenuations.push(BLACK);
            break;
        }

        if let Some(event) = world.hit(rng, &ray, 0.001, f64::INFINITY) {
            let HitEvent { record, scatter } = event;
            let attenuation = if let Some(scatter) = scatter {
                ray = Ray::new(record.hit_at, scatter.direction);
                scatter.attenuation
            } else {
                attenuations.push(BLACK);
                break;
            };

            attenuations.push(attenuation);
            reflect_cnt += 1;
        } else {
            attenuations.push(background(&ray));
            break;
        }
    }

    attenuations.into_iter().fold(WHITE, |p, rgb| p * rgb)
}

fn background(ray: &Ray) -> Rgb {
    let unit_dir = ray.direction().normalized();
    let t = 0.5 * (unit_dir.y() + 1.0);
    (1.0 - t) * WHITE + t * LIGHTBLUE
}
