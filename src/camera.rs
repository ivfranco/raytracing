use crate::{ray::Ray, Vec3};

const VIEWPORT_HEIGHT: f64 = 2.0;

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
    pub fn new(aspect_ratio: f64, focal_length: f64) -> Self {
        let camera_origin = Vec3::origin();
        let viewport_height = VIEWPORT_HEIGHT;
        let viewport_width = viewport_height * aspect_ratio;

        let viewport_origin =
            camera_origin - Vec3::new(viewport_width / 2.0, viewport_height / 2.0, focal_length);

        Self {
            camera_origin,
            viewport_origin,
            viewport_width,
            viewport_height,
        }
    }

    /// Cast a ray pointing to the given horizontal and vertical ratio of the viewport.
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
}
