use std::slice::{ChunksExact, ChunksExactMut};

use num_traits::{Num, Zero};

pub trait PixelComponent: Num + Copy + Clone + Zero {
  type Container: Num;
}
impl PixelComponent for u8 {
  type Container = u8;
}
impl PixelComponent for u16 {
  type Container = u16;
}
impl PixelComponent for u32 {
  type Container = u32;
}
impl PixelComponent for u64 {
  type Container = u64;
}
impl PixelComponent for u128 {
  type Container = u128;
}
impl PixelComponent for f32 {
  type Container = f32;
}
impl PixelComponent for f64 {
  type Container = f64;
}

pub trait PixelContainer {
  type OnePixel;
  const HAS_ALPHA: bool;
  const NUM_COMPONENTS: usize;
  const ALPHA_IDX: Option<usize>;
}

pub struct ImageBuffer<
  Component: PixelComponent,
  const COMPONENTS_PER_PEL: usize,
  const HAS_ALPHA: bool,
> {
  data:       Vec<Component>,
  pub width:  usize,
  pub height: usize,
}

impl<
    Component: PixelComponent,
    const COMPONENTS_PER_PEL: usize,
    const HAS_ALPHA: bool,
  > PixelContainer for ImageBuffer<Component, COMPONENTS_PER_PEL, HAS_ALPHA>
{
  type OnePixel = [Component; COMPONENTS_PER_PEL];

  const ALPHA_IDX: Option<usize> = if HAS_ALPHA {
    Some(COMPONENTS_PER_PEL - 1)
  } else {
    None
  };
  const HAS_ALPHA: bool = HAS_ALPHA;
  const NUM_COMPONENTS: usize = COMPONENTS_PER_PEL;
}

impl<
    Component: PixelComponent,
    const COMPONENTS_PER_PEL: usize,
    const HAS_ALPHA: bool,
  > ImageBuffer<Component, COMPONENTS_PER_PEL, HAS_ALPHA>
{
  pub fn with_data(
    data: Vec<Component>,
    width: usize,
    height: usize,
  ) -> Result<Self, &'static str> {
    let expected_vec_elements = width * height * COMPONENTS_PER_PEL;
    if data.len() != expected_vec_elements {
      return Err(
        "Data vector length does not match width, height, and number of \
         components per pixel",
      );
    }
    Ok(ImageBuffer {
      data,
      width,
      height,
    })
  }

  pub fn empty(width: usize, height: usize) -> Self {
    ImageBuffer {
      data: vec![Component::zero(); width * height * COMPONENTS_PER_PEL],
      width,
      height,
    }
  }

  pub fn with_val(
    one_pel: &[Component; COMPONENTS_PER_PEL],
    width: usize,
    height: usize,
  ) -> Result<Self, &'static str> {
    let mut result = ImageBuffer {
      data: vec![Component::zero(); width * height * COMPONENTS_PER_PEL],
      width,
      height,
    };

    for pel in result.iter_with_alpha_mut() {
      for (c1, c2) in pel.iter_mut().zip(one_pel.iter()) {
        *c1 = *c2;
      }
    }

    Ok(result)
  }
}

pub struct ImageBufferIterator<
  'a,
  Component: PixelComponent,
  const COMPONENTS_PER_PEL: usize,
  const HAS_ALPHA: bool,
  const SKIP_ALPHA: bool,
> {
  iterator: ChunksExact<'a, Component>,
}

pub struct ImagebufferIteratorMut<
  'a,
  Component: PixelComponent,
  const COMPONENTS_PER_PEL: usize,
  const HAS_ALPHA: bool,
  const SKIP_ALPHA: bool,
> {
  iterator: ChunksExactMut<'a, Component>,
}

impl<
    'a,
    Component: PixelComponent,
    const COMPONENTS_PER_PEL: usize,
    const HAS_ALPHA: bool,
  > ImageBuffer<Component, COMPONENTS_PER_PEL, HAS_ALPHA>
{
  pub fn iter(
    &'a self,
  ) -> ImageBufferIterator<'a, Component, COMPONENTS_PER_PEL, HAS_ALPHA, true>
  {
    self.iter_no_alpha()
  }

  pub fn iter_mut(
    &'a mut self,
  ) -> ImagebufferIteratorMut<'a, Component, COMPONENTS_PER_PEL, HAS_ALPHA, true>
  {
    self.iter_no_alpha_mut()
  }

  pub fn iter_no_alpha(
    &'a self,
  ) -> ImageBufferIterator<'a, Component, COMPONENTS_PER_PEL, HAS_ALPHA, true>
  {
    ImageBufferIterator {
      iterator: self.data.chunks_exact(COMPONENTS_PER_PEL),
    }
  }

  pub fn iter_no_alpha_mut(
    &'a mut self,
  ) -> ImagebufferIteratorMut<'a, Component, COMPONENTS_PER_PEL, HAS_ALPHA, true>
  {
    ImagebufferIteratorMut {
      iterator: self.data.chunks_exact_mut(COMPONENTS_PER_PEL),
    }
  }

  pub fn iter_with_alpha(
    &'a self,
  ) -> ImageBufferIterator<'a, Component, COMPONENTS_PER_PEL, HAS_ALPHA, false>
  {
    ImageBufferIterator {
      iterator: self.data.chunks_exact(COMPONENTS_PER_PEL),
    }
  }

  pub fn iter_with_alpha_mut(
    &'a mut self,
  ) -> ImagebufferIteratorMut<'a, Component, COMPONENTS_PER_PEL, HAS_ALPHA, false>
  {
    ImagebufferIteratorMut {
      iterator: self.data.chunks_exact_mut(COMPONENTS_PER_PEL),
    }
  }
}

impl<
    'a,
    Component: PixelComponent,
    const COMPONENTS_PER_PEL: usize,
    const HAS_ALPHA: bool,
    const SKIP_ALPHA: bool,
  > Iterator
  for ImageBufferIterator<
    'a,
    Component,
    COMPONENTS_PER_PEL,
    HAS_ALPHA,
    SKIP_ALPHA,
  >
{
  type Item = &'a [Component];

  fn next(&mut self) -> Option<Self::Item> {
    self.iterator.next().map(|pel| {
      if HAS_ALPHA && SKIP_ALPHA {
        &pel[..COMPONENTS_PER_PEL - 1]
      } else {
        pel
      }
    })
  }
}

impl<
    'a,
    Component: PixelComponent,
    const COMPONENTS_PER_PEL: usize,
    const HAS_ALPHA: bool,
    const SKIP_ALPHA: bool,
  > Iterator
  for ImagebufferIteratorMut<
    'a,
    Component,
    COMPONENTS_PER_PEL,
    HAS_ALPHA,
    SKIP_ALPHA,
  >
{
  type Item = &'a mut [Component];

  fn next(&mut self) -> Option<Self::Item> {
    self.iterator.next().map(|pel| {
      if HAS_ALPHA && SKIP_ALPHA {
        &mut pel[..COMPONENTS_PER_PEL - 1]
      } else {
        pel
      }
    })
  }
}

#[cfg(test)]
mod tests {

  use image::{DynamicImage, GenericImage};
  use test::Bencher;
  use test_case::test_case;

  use super::*;

  #[test]
  fn new_rgba_u8_with_data() {
    const WIDTH: usize = 4;
    const HEIGHT: usize = 4;
    const RGBA_CPP: usize = 4;
    let data = vec![0u8; WIDTH * HEIGHT * RGBA_CPP];
    let image = ImageBuffer::<u8, 4, true>::with_data(data, WIDTH, HEIGHT);
    assert!(image.is_ok());
    assert_eq!(image.unwrap().data.len(), 4 * 4 * 4);
  }

  #[test]
  fn new_rgba_u8_empty() {
    const WIDTH: usize = 4;
    const HEIGHT: usize = 4;
    let image = ImageBuffer::<u8, 4, true>::empty(WIDTH, HEIGHT);
    assert_eq!(image.data.len(), 4 * 4 * 4);
    for pel in image.iter_with_alpha() {
      assert_eq!(pel, &[0u8, 0u8, 0u8, 0u8]);
    }
  }

  #[test]
  fn new_rgba_u8_with_val() {
    const WIDTH: usize = 4;
    const HEIGHT: usize = 4;
    let one_pel = [1u8, 2u8, 3u8, 255];
    let image = ImageBuffer::<u8, 4, true>::with_val(&one_pel, WIDTH, HEIGHT);
    assert!(image.is_ok());
    let image = image.unwrap();
    assert_eq!(image.data.len(), 4 * 4 * 4);
    for pel in image.iter_with_alpha() {
      assert_eq!(pel, &[1u8, 2u8, 3u8, 255]);
    }
  }

  #[bench]
  fn bench_new_rgba_u8_with_data(b: &mut Bencher) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    const RGBA_CPP: usize = 4;
    let data = vec![0u8; WIDTH * HEIGHT * RGBA_CPP];
    b.iter(|| {
      ImageBuffer::<u8, 4, true>::with_data(data.clone(), WIDTH, HEIGHT)
        .unwrap();
    });
  }

  #[bench]
  fn bench_new_rgba_u8_empty(b: &mut Bencher) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    b.iter(|| {
      ImageBuffer::<u8, 4, true>::empty(WIDTH, HEIGHT);
    });
  }

  #[bench]
  fn bench_new_rgba_u8_with_val(b: &mut Bencher) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    let one_pel = [0u8, 0u8, 0u8, 255];
    b.iter(|| {
      ImageBuffer::<u8, 4, true>::with_val(&one_pel, WIDTH, HEIGHT).unwrap();
    });
  }

  #[bench]
  fn bench_new_rgba_u8_dynamic_image_empty(b: &mut Bencher) {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;
    b.iter(|| {
      DynamicImage::new_rgba8(WIDTH, HEIGHT);
    });
  }

  #[bench]
  fn bench_new_rgba_u8_dynamic_image_with_data(b: &mut Bencher) {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;
    const RGBA_CPP: u32 = 4;
    let data = vec![0u8; WIDTH as usize * HEIGHT as usize * RGBA_CPP as usize];
    let buf = image::ImageBuffer::from_vec(WIDTH, HEIGHT, data).unwrap();
    b.iter(|| DynamicImage::ImageRgba8(buf.clone()));
  }

  #[bench]
  fn bench_iteration_rgba_u8_assignment_no_alpha(b: &mut Bencher) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    const RGBA_CPP: usize = 4;
    let data = vec![0u8; WIDTH * HEIGHT * RGBA_CPP];
    let mut image =
      ImageBuffer::<u8, 4, true>::with_data(data, WIDTH, HEIGHT).unwrap();
    let mut new_val: u8 = 0;
    b.iter(|| {
      new_val = new_val.wrapping_add(1);
      for pel in image.iter_no_alpha_mut() {
        pel[0] = new_val;
        pel[1] = new_val;
        pel[2] = new_val;
      }
    });
  }

  #[bench]
  fn bench_iteration_rgba_u8_assignment_with_alpha_skip_alpha(b: &mut Bencher) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    const RGBA_CPP: usize = 4;
    let data = vec![0u8; WIDTH * HEIGHT * RGBA_CPP];
    let mut image =
      ImageBuffer::<u8, 4, true>::with_data(data, WIDTH, HEIGHT).unwrap();
    let mut new_val: u8 = 0;
    b.iter(|| {
      new_val = new_val.wrapping_add(1);
      for pel in image.iter_with_alpha_mut() {
        pel[0] = new_val;
        pel[1] = new_val;
        pel[2] = new_val;
      }
    });
  }

  #[bench]
  fn bench_iteration_rgba_u8_assignment_with_alpha_assign_alpha(
    b: &mut Bencher,
  ) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    const RGBA_CPP: usize = 4;
    let data = vec![0u8; WIDTH * HEIGHT * RGBA_CPP];
    let mut image =
      ImageBuffer::<u8, 4, true>::with_data(data, WIDTH, HEIGHT).unwrap();
    let mut new_val: u8 = 0;
    b.iter(|| {
      new_val = new_val.wrapping_add(1);
      for pel in image.iter_with_alpha_mut() {
        pel[0] = new_val;
        pel[1] = new_val;
        pel[2] = new_val;
        pel[3] = 255;
      }
    });
  }

  #[bench]
  fn bench_iteration_rgba_u8_assignment_dynamic_image(b: &mut Bencher) {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;
    let mut image = DynamicImage::new_rgba8(WIDTH, HEIGHT);
    let mut new_val: u8 = 0;
    b.iter(|| {
      new_val = new_val.wrapping_add(1);
      for y in 0..HEIGHT {
        for x in 0..WIDTH {
          image.put_pixel(x, y, image::Rgba([new_val, new_val, new_val, 255]));
        }
      }
    });
  }
}
