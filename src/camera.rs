use rand::Rng;

use crate::{ray::Ray, Vec3};

/// Constructor of a camera.
pub struct CameraBuilder {
    look_from: Vec3,
    look_at: Vec3,
    v_up: Vec3,
    v_fov: f64,
    aspect_ratio: f64,
    aperture: f64,
    focus_dist: Option<f64>,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        CameraBuilder {
            look_from: Vec3::origin(),
            look_at: Vec3::new(0.0, 0.0, -1.0),
            v_up: Vec3::new(0.0, 1.0, 0.0),
            v_fov: 90.0,
            aspect_ratio: 16.0 / 9.0,
            aperture: 0.0,
            focus_dist: None,
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

    /// Set the point the camera is looking at.
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

    /// Set the aperture of the camera.
    ///
    /// # Default:
    /// 0, no blur at all
    pub fn aperture(&mut self, aperture: f64) -> &mut Self {
        self.aperture = aperture;
        self
    }

    /// Set the focus distant of the camera.
    ///
    /// # Default:
    /// the distance between look_from and look_at.
    pub fn focus_dist(&mut self, focus_dist: f64) -> &mut Self {
        self.focus_dist.get_or_insert(focus_dist);
        self
    }

    /// Build the camera with the given parameters and defaults.
    pub fn build(&self) -> Camera {
        let theta = self.v_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = self.aspect_ratio * viewport_height;

        let w = (self.look_from - self.look_at).normalized();
        let u = self.v_up.cross(w).normalized();
        let v = w.cross(u);

        let camera_origin = self.look_from;

        let focus_dist = self
            .focus_dist
            .unwrap_or_else(|| (self.look_from - self.look_at).norm());

        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let viewport_origin = camera_origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        let lens_radius = self.aperture / 2.0;

        Camera {
            camera_origin,
            viewport_origin,
            horizontal,
            vertical,
            lens_radius,
            u,
            v,
            w,
        }
    }
}

/// A camera that may cast rays into the world.
pub struct Camera {
    camera_origin: Vec3,
    viewport_origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lens_radius: f64,
    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl Camera {
    /// Cast a ray pointing to the given horizontal and vertical ratio of the viewport, Starting
    /// from the bottom left corner.
    ///
    /// # Examples
    /// ```
    /// # use raytracing::camera::CameraBuilder;
    /// let camera = CameraBuilder::new().build();
    /// // a ray pointing to the center of the viewport
    /// let ray = camera.get_ray(0.5, 0.5);
    /// ```
    pub fn get_ray<R: Rng>(&self, rng: &mut R, s: f64, t: f64) -> Ray {
        let random_look_from = self.lens_radius * random_in_unit_xy_disk(rng);
        let offset = self.u * random_look_from.x() + self.v * random_look_from.y();

        let direction = self.viewport_origin + s * self.horizontal + t * self.vertical
            - self.camera_origin
            - offset;

        Ray::new(self.camera_origin + offset, direction)
    }

    /// Scan the image pixel by pixel, row by row from bottom to top.
    pub fn cast(&self, pixel_width: u32, pixel_height: u32) -> RayCaster {
        RayCaster::new(self, pixel_width, pixel_height)
    }
}

fn random_in_unit_xy_disk<R: Rng>(rng: &mut R) -> Vec3 {
    loop {
        let p = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
        if p.norm_squared() < 1.0 {
            return p;
        }
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
        let rx = self.x + self.dx * rng.gen::<f64>();
        let ry = self.y + self.dy * rng.gen::<f64>();

        self.camera.get_ray(rng, rx, ry)
    }
}
