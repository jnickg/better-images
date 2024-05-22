use crate::color_space::ColorSpace;
use crate::pixel::PixelComponent;

pub trait ImageFactory: PixelComponent {
    fn create(data: ColorSpace<Self>) -> Image;
}

impl ImageFactory for u8 {
    fn create(data: ColorSpace<Self>) -> Image {
        Image::new_u8(data)
    }
}

impl ImageFactory for u16 {
    fn create(data: ColorSpace<Self>) -> Image {
        Image::new_u16(data)
    }
}

impl ImageFactory for u32 {
    fn create(data: ColorSpace<Self>) -> Image {
        Image::new_u32(data)
    }
}

impl ImageFactory for f32 {
    fn create(data: ColorSpace<Self>) -> Image {
        Image::new_f32(data)
    }
}

impl ImageFactory for f64 {
    fn create(data: ColorSpace<Self>) -> Image {
        Image::new_f64(data)
    }
}

pub struct ImageImpl<T: PixelComponent> {
    pub(crate) data: ColorSpace<T>,
}

impl<T: PixelComponent> ImageImpl<T> {
    pub fn width(&self) -> usize {
        match &self.data {
            ColorSpace::Rgba(buf) => buf.width,
            ColorSpace::Rgb(buf) => buf.width,
            ColorSpace::Hsv(buf) => buf.width,
            ColorSpace::Cielab(buf) => buf.width,
        }
    }

    pub fn height(&self) -> usize {
        match &self.data {
            ColorSpace::Rgba(buf) => buf.height,
            ColorSpace::Rgb(buf) => buf.height,
            ColorSpace::Hsv(buf) => buf.height,
            ColorSpace::Cielab(buf) => buf.height,
        }
    }
}

pub enum Implementation {
    U8(ImageImpl<u8>),
    U16(ImageImpl<u16>),
    U32(ImageImpl<u32>),
    F32(ImageImpl<f32>),
    F64(ImageImpl<f64>),
}

impl Implementation {
    pub fn width(&self) -> usize {
        match self {
            Implementation::U8(imp) => imp.width(),
            Implementation::U16(imp) => imp.width(),
            Implementation::U32(imp) => imp.width(),
            Implementation::F32(imp) => imp.width(),
            Implementation::F64(imp) => imp.width(),
        }
    }

    pub fn height(&self) -> usize {
        match self {
            Implementation::U8(imp) => imp.height(),
            Implementation::U16(imp) => imp.height(),
            Implementation::U32(imp) => imp.height(),
            Implementation::F32(imp) => imp.height(),
            Implementation::F64(imp) => imp.height(),
        }
    }

}
pub struct Image {
    pub(crate) imp: Implementation
}


impl Image {
    pub fn new<T: ImageFactory>(data: ColorSpace<T>) -> Self {
        <T as ImageFactory>::create(data)
    }

    pub fn new_u8(data: ColorSpace<u8>) -> Self {
        Self {
            imp: Implementation::U8(ImageImpl { data })
        }
    }
    pub fn new_u16(data: ColorSpace<u16>) -> Self {
        Self {
            imp: Implementation::U16(ImageImpl { data })
        }
    }
    pub fn new_u32(data: ColorSpace<u32>) -> Self {
        Self {
            imp: Implementation::U32(ImageImpl { data })
        }
    }
    pub fn new_f32(data: ColorSpace<f32>) -> Self {
        Self {
            imp: Implementation::F32(ImageImpl { data })
        }
    }
    pub fn new_f64(data: ColorSpace<f64>) -> Self {
        Self {
            imp: Implementation::F64(ImageImpl { data })
        }
    }

    pub fn width(&self) -> usize {
        self.imp.width()
    }

    pub fn height(&self) -> usize {
        self.imp.height()
    }
}

#[cfg(test)]
mod tests {

  use crate::image_buffer::ImageBuffer;

use super::*;

  #[test]
  fn new_rgba_u8() {
    let img = Image::new::<u8>(ColorSpace::Rgba(ImageBuffer::empty(4, 4)));
    match img.imp {
      Implementation::U8(cs) => {
        match cs {
            ImageImpl { data: ColorSpace::Rgba(buf) } => {
                assert_eq!(buf.width, 4);
                assert_eq!(buf.height, 4);
            },
            _ => panic!("Wrong type"),
        }
      }
      _ => panic!("Wrong type"),
    }
  }

}