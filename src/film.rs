use image::{ImageBuffer, Rgba, RgbaImage};
use std::path::Path;

pub struct Film {
    width: u32,
    height: u32,
    pixels: Vec<[u8; 4]>,
}

impl Film {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![[0, 0, 0, 255]; (width * height) as usize],
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let index = self.index(x, y);
        self.pixels[index] = color;
    }
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels[self.index(x, y)]
    }
    pub fn save_png(&self, path: impl AsRef<Path>) -> image::ImageResult<()> {
        let path = path.as_ref();
        let mut image: RgbaImage = ImageBuffer::new(self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                image.put_pixel(x, y, Rgba(self.pixel(x, y)));
            }
        }
        image.save(path)
    }
    fn index(&self, x: u32, y: u32) -> usize {
        assert!(
            x < self.width && y < self.height,
            "pixel is outside the film"
        );
        (y * self.width + x) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sets_and_gets_pixels() {
        let mut film = Film::new(2, 1);
        film.set_pixel(1, 0, [10, 20, 30, 255]);
        assert_eq!(film.pixel(1, 0), [10, 20, 30, 255]);
    }

    #[test]
    fn saves_pixels_as_png() {
        let path = std::env::temp_dir().join(format!("rust_learn_film_{}.png", std::process::id()));
        let mut film = Film::new(2, 1);
        film.set_pixel(0, 0, [255, 0, 0, 255]);
        film.save_png(&path).unwrap();
        let saved = image::open(&path).unwrap().to_rgba8();
        assert_eq!(saved.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(saved.dimensions(), (2, 1));
        fs::remove_file(path).unwrap();
    }
}
