use num_traits::NumCast;

use crate::{
  image_buffer::ImageBuffer,
  pixel::{PixelComponent, PixelContainer},
};

pub enum ColorSpace<T: PixelComponent> {
  Rgba(ImageBuffer<T, 4, true>),
  Rgb(ImageBuffer<T, 3, false>),
  Hsv(ImageBuffer<T, 3, false>),
  Cielab(ImageBuffer<T, 3, false>),
}

pub fn rgb_to_cielab<T1: PixelComponent, T2: PixelComponent>(
  rgb: &<ImageBuffer<T1, 3, false> as PixelContainer>::OnePixel,
) -> <ImageBuffer<T2, 3, false> as PixelContainer>::OnePixel {
  // This is not correct :sweaty:
  let r = <f32 as NumCast>::from(rgb[0]).unwrap_or_default() / 255.0;
  let g = <f32 as NumCast>::from(rgb[1]).unwrap_or_default() / 255.0;
  let b = <f32 as NumCast>::from(rgb[2]).unwrap_or_default() / 255.0;
  let x = r * 0.4124564 + g * 0.3575761 + b * 0.1804375;
  let y = r * 0.2126729 + g * 0.7151522 + b * 0.0721750;
  let z = r * 0.0193339 + g * 0.1191920 + b * 0.9503041;
  let x = if x > 0.008856 {
    x.powf(1.0 / 3.0)
  } else {
    7.787 * x + 16.0 / 116.0
  };
  let y = if y > 0.008856 {
    y.powf(1.0 / 3.0)
  } else {
    7.787 * y + 16.0 / 116.0
  };
  let z = if z > 0.008856 {
    z.powf(1.0 / 3.0)
  } else {
    7.787 * z + 16.0 / 116.0
  };
  let l = <T2 as NumCast>::from(116.0 * y - 16.0).unwrap_or_default();
  let a = <T2 as NumCast>::from(500.0 * (x - y)).unwrap_or_default();
  let b = <T2 as NumCast>::from(200.0 * (y - z)).unwrap_or_default();
  [l, a, b]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn example_convert_rgb_u8_to_lab_f32() {
    const WIDTH: usize = 4;
    const HEIGHT: usize = 4;
    let one_pel = [1u8, 2u8, 3u8];
    let image = ImageBuffer::<u8, 3, false>::with_val(&one_pel, WIDTH, HEIGHT);
    let lab_image: ImageBuffer<f32, 3, false> =
      image.map_into(&mut |pel| rgb_to_cielab(pel));
    for pel in lab_image.iter() {
      // TODO this is not right :-)
      assert_eq!(pel, &[6.586956, -2.9099135, -7.0868254])
    }
  }
}
