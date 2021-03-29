use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Context;

use crate::{color::Rgb, Error};

/// An interface to put pixels on image one by one.
pub trait ImageBuilder {
    /// Initialize an image with given width and height.
    fn with_dimensions(width: u32, height: u32) -> Self;

    /// Put a pixel onto the image, in each row left to right, top to bottom for rows.
    fn put(&mut self, rgb: Rgb) -> anyhow::Result<()>;

    /// Output the image to the specified writer.
    fn output<W: Write>(&self, writer: &mut W) -> anyhow::Result<()>;

    /// Create a file at the given path then write the image to the file. Truncate the file if it exists.
    fn output_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.output(&mut writer)?;
        writer.flush()?;
        Ok(())
    }
}

/// Netpbm color image format.
pub struct PPMBuilder {
    width: u32,
    height: u32,
    pixels: Vec<[u8; 3]>,
}

impl PPMBuilder {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: Vec::with_capacity((width * height) as usize),
        }
    }
}

impl ImageBuilder for PPMBuilder {
    fn with_dimensions(width: u32, height: u32) -> Self {
        Self::new(width, height)
    }

    fn put(&mut self, rgb: Rgb) -> anyhow::Result<()> {
        fn cast(channel: f64) -> anyhow::Result<u8> {
            if (0.0..=1.0).contains(&channel) {
                Ok((255.999 * channel) as u8)
            } else {
                Err(Error::ColorOutOfRange).context(channel)
            }
        }

        if self.pixels.len() >= (self.width * self.height) as usize {
            return Err(Error::ImageBufferOverflow).with_context(|| {
                format!(
                    "PPM builder initialized with dimensions {} x {}",
                    self.width, self.height
                )
            });
        }

        let ir = cast(rgb.r())?;
        let ig = cast(rgb.g())?;
        let ib = cast(rgb.b())?;

        self.pixels.push([ir, ig, ib]);
        Ok(())
    }

    fn output<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        write!(&mut *writer, "P3\n{} {}\n255\n", self.width, self.height)
            .context("failed to write PPM header")?;

        for [r, g, b] in &self.pixels {
            writeln!(&mut *writer, "{} {} {}", r, g, b).context("failed to write PPM pixels")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn ppm_gradient() {
        const WIDTH: u32 = 256;
        const HEIGHT: u32 = 256;

        let mut builder = PPMBuilder::new(WIDTH, HEIGHT);

        for j in 0..HEIGHT {
            for i in 0..WIDTH {
                let pixel = Rgb::new(i as f64 / WIDTH as f64, j as f64 / HEIGHT as f64, 0.25);
                builder.put(pixel).unwrap();
            }
        }

        if !Path::new("output").is_dir() {
            fs::create_dir("output").unwrap();
        }

        builder.output_to_file("output/gradient.ppm").unwrap();
    }
}
