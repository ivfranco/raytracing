use crate::color::Rgb;

/// An interface to put pixels on image one by one.
pub trait ImagePrinter {
    /// Initialize an image with given width and height.
    fn with_dimensions(width: u32, height: u32) -> Self;

    /// Put a pixel onto the image, from top to bottom, left to right.
    fn put(&mut self, rgb: Rgb);
}
