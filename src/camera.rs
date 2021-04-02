use rand::Rng;

use crate::{ray::Ray, Vec3};

/// A camera that may cast rays into the world.
pub struct Camera {
    camera_origin: Vec3,
    viewport_origin: Vec3,
    viewport_width: f64,
    viewport_height: f64,
}

impl Camera {
    /// Construct a camera with the given aspect ratio and focal length. The camera is located at
    /// the origin point (0, 0, 0);
    pub fn new(v_fov: f64, aspect_ratio: f64, focal_length: f64) -> Self {
        let theta = v_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let camera_origin = Vec3::origin();
        let viewport_origin =
            camera_origin - Vec3::new(viewport_width / 2.0, viewport_height / 2.0, focal_length);

        Self {
            camera_origin,
            viewport_origin,
            viewport_width,
            viewport_height,
        }
    }

    /// Cast a ray pointing to the given horizontal and vertical ratio of the viewport, Starting
    /// from the bottom left corner.
    ///
    /// # Examples
    /// ```
    /// # use raytracing::camera::Camera;
    /// let camera = Camera::new(1.0, 1.0);
    /// // a ray pointing to the center of the viewport
    /// let ray = camera.get_ray(0.5, 0.5);
    /// ```
    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let direction = self.viewport_origin
            + Vec3::new(self.viewport_width * u, self.viewport_height * v, 0.0)
            - self.camera_origin;

        Ray::new(self.camera_origin, direction)
    }

    /// Scan the image pixel by pixel, row by row from bottom to top.
    pub fn cast(&self, pixel_width: u32, pixel_height: u32) -> RayCaster {
        RayCaster::new(self, pixel_width, pixel_height)
    }
}

/// An iterator that scans over the pixels of an image from a camera.  
pub struct RayCaster<'a> {
    camera: &'a Camera,
    pixel_width: u32,
    pixel_height: u32,
    x: u32,
    y: u32,
}

impl<'a> RayCaster<'a> {
    fn new(camera: &'a Camera, pixel_width: u32, pixel_height: u32) -> Self {
        assert!(pixel_width > 0);
        assert!(pixel_height > 0);

        Self {
            camera,
            pixel_width,
            pixel_height,
            x: pixel_width,
            y: pixel_height,
        }
    }

    fn advance_pixel(&mut self) -> Option<()> {
        self.x += 1;
        if self.x >= self.pixel_width {
            self.x = 0;
            self.y = self.y.checked_sub(1)?;
        }

        Some(())
    }
}

impl<'a> Iterator for RayCaster<'a> {
    type Item = RaySampler<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance_pixel()?;

        let sampler = RaySampler {
            camera: self.camera,
            x: self.x as f64 / (self.pixel_width - 1) as f64,
            y: self.y as f64 / (self.pixel_height - 1) as f64,
            dx: 1.0 / (self.pixel_width - 1) as f64,
            dy: 1.0 / (self.pixel_height - 1) as f64,
        };

        Some(sampler)
    }
}

/// A sampler that samples rays around a single pixel.
pub struct RaySampler<'a> {
    camera: &'a Camera,
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
}

impl<'a> RaySampler<'a> {
    /// Samples a ray around the pixel.
    pub fn sample<R: Rng>(&self, rng: &mut R) -> Ray {
        self.camera.get_ray(
            self.x + self.dx * rng.gen::<f64>(),
            self.y + self.dy * rng.gen::<f64>(),
        )
    }
}
