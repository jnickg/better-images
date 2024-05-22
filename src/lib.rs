#![feature(array_chunks)]
#![feature(test)]
extern crate test;

pub mod color_space;
pub mod image_buffer;
pub mod image;
pub mod pixel;

pub use image_buffer::ImageBuffer;
pub use pixel::PixelContainer;
pub use image::ImageFactory;
pub use image::Image;
