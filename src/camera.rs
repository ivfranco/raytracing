use rand::Rng;

use crate::{ray::Ray, Vec3};

/// Constructor of a camera.
pub struct CameraBuilder {
    look_from: Vec3,
    look_at: Vec3,
    v_up: Vec3,
    v_fov: f64,
    aspect_ratio: f64,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        CameraBuilder {
            look_from: Vec3::origin(),
            look_at: Vec3::new(0.0, 0.0, -1.0),
            v_up: Vec3::new(0.0, 1.0, 0.0),
            v_fov: 90.0,
            aspect_ratio: 16.0 / 9.0,
        }
    }
}

impl CameraBuilder {
    /// Initialize a new builder for a camera.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the position of the camera.
    ///
    /// # Default:
    /// (0, 0, 0), the original point
    pub fn look_from(&mut self, look_from: Vec3) -> &mut Self {
        self.look_from = look_from;
        self
    }

    /// Set the direction the camera is looking at.
    ///
    /// # Default:
    /// (0, 0, -1)
    pub fn look_at(&mut self, look_at: Vec3) -> &mut Self {
        self.look_at = look_at;
        self
    }

    /// Set the roll of the camera by a view up vector.
    ///
    /// # Default:
    /// (0, 1, 0)
    pub fn v_up(&mut self, v_up: Vec3) -> &mut Self {
        self.v_up = v_up;
        self
    }

    /// Set the vertical field of view of the camera in degrees. The horizontal field of view will be
    /// calculated from the vertical fov and the aspect ratio.
    ///
    /// # Default:
    /// 90 degrees
    pub fn v_fov(&mut self, v_fov: f64) -> &mut Self {
        self.v_fov = v_fov;
        self
    }

    /// Set the aspect ratio of the camera.
    ///
    /// # Default:
    /// 16 / 9
    pub fn aspect_ratio(&mut self, aspect_ratio: f64) -> &mut Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    /// Build the camera with the given parameters and defaults.
    pub fn build(self) -> Camera {
        let theta = self.v_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = self.aspect_ratio * viewport_height;

        let w = (self.look_from - self.look_at).normalized();
        let u = self.v_up.cross(w).normalized();
        let v = w.cross(u);

        let camera_origin = self.look_from;

        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;
        let viewport_origin = camera_origin - horizontal / 2.0 - vertical / 2.0 - w;

        Camera {
            camera_origin,
            viewport_origin,
            horizontal,
            vertical,
        }
    }
}

/// A camera that may cast rays into the world.
pub struct Camera {
    camera_origin: Vec3,
    viewport_origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
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
    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let direction =
            self.viewport_origin + s * self.horizontal + t * self.vertical - self.camera_origin;

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
