use std::{fmt::Display, fs, path::Path, process};

use indicatif::{ParallelProgressIterator, ProgressBar};
use rand::{prelude::SliceRandom, rngs::StdRng, Rng, SeedableRng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use raytracing::{
    camera::CameraBuilder,
    color::{Rgb, RgbAccumulator, BLACK, LIGHTBLUE, WHITE},
    hittable::Sphere,
    image_builder::{ImageBuilder, PNGBuilder},
    material::{Dielectric, Lambertian, Material, Metal},
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
    let aspect_ratio = 3.0 / 2.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as u32;

    const SAMPLE_PER_PIXEL: u32 = 50;

    let camera = CameraBuilder::new()
        .look_from(Vec3::new(13.0, 2.0, 3.0))
        .look_at(Vec3::origin())
        .aspect_ratio(aspect_ratio)
        .focus_dist(10.0)
        .v_fov(20.0)
        .aperture(0.1)
        .build();

    let world = random_world(&mut rand::rngs::StdRng::from_entropy());

    let mut image_builder = PNGBuilder::with_dimensions(image_width, image_height);

    let instant = std::time::Instant::now();

    let progress = ProgressBar::new((image_width * image_height) as u64);
    progress.set_draw_delta(1000);

    let samplers: Vec<_> = camera.cast(image_width, image_height).collect();
    let pixels: Vec<_> = samplers
        .par_iter()
        .progress_with(progress.clone())
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

    progress.finish();

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

fn random_world<R: Rng>(rng: &mut R) -> World {
    let mut world = World::new();

    let ground_material = Lambertian::new(Rgb::new(0.5, 0.5, 0.5));
    let glass_material = Dielectric::new(1.5);

    world.add(
        Sphere {
            center: Vec3::new(0.0, -1000.0, 0.0),
            radius: 1000.0,
        },
        ground_material,
    );

    let empty_spot = Vec3::new(4.0, 0.2, 0.0);
    let small_radius = 0.2;
    let choices = [(0, 80), (1, 15), (2, 5)];

    for (a, b) in (-11..11).flat_map(|a| (-11..11).map(move |b| (a, b))) {
        let center = Vec3::new(
            a as f64 + 0.9 * rng.gen::<f64>(),
            0.2,
            b as f64 + 0.9 * rng.gen::<f64>(),
        );

        if (center - empty_spot).norm() > 0.9 {
            let material: Material = match choices.choose_weighted(rng, |(_, w)| *w).unwrap().0 {
                0 => rng.gen::<Lambertian>().into(),
                1 => rng.gen::<Metal>().into(),
                2 => glass_material.into(),
                _ => unreachable!(),
            };

            world.add(
                Sphere {
                    center,
                    radius: small_radius,
                },
                material,
            );
        }
    }

    let big_radius = 1.0;

    world.add(
        Sphere {
            center: Vec3::new(0.0, 1.0, 0.0),
            radius: big_radius,
        },
        glass_material,
    );

    world.add(
        Sphere {
            center: Vec3::new(-4.0, 1.0, 0.0),
            radius: big_radius,
        },
        Lambertian::new(Rgb::new(0.4, 0.2, 0.1)),
    );

    world.add(
        Sphere {
            center: Vec3::new(4.0, 1.0, 0.0),
            radius: big_radius,
        },
        Metal::new(Rgb::new(0.7, 0.6, 0.5), 0.0),
    );

    world
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
