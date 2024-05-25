use std::slice::{ArrayChunks, ArrayChunksMut};

use num_traits::NumCast;

use crate::pixel::{PixelComponent, PixelContainer};

#[derive(Clone, Debug, Default)]
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
  type OnePlane = ImageBuffer<Component, 1, false>;
  type PixelBuffer = Vec<Component>;

  const ALPHA_IDX: Option<usize> = if HAS_ALPHA {
    Some(COMPONENTS_PER_PEL - 1)
  } else {
    None
  };
  const HAS_ALPHA: bool = HAS_ALPHA;
  const NUM_COMPONENTS: usize = COMPONENTS_PER_PEL;
  const NUM_NONALPHA_COMPONENTS: usize = if HAS_ALPHA {
    COMPONENTS_PER_PEL - 1
  } else {
    COMPONENTS_PER_PEL
  };

  fn pixels(&self) -> &Self::PixelBuffer { &self.data }

  fn pixels_mut(&mut self) -> &mut Self::PixelBuffer { &mut self.data }

  fn width(&self) -> usize { self.width }

  fn height(&self) -> usize { self.height }
}

impl<
    Component: PixelComponent,
    const COMPONENTS_PER_PEL: usize,
    const HAS_ALPHA: bool,
  > ImageBuffer<Component, COMPONENTS_PER_PEL, HAS_ALPHA>
{
  pub fn with_data(
    data: <Self as PixelContainer>::PixelBuffer,
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
    one_pel: &<Self as PixelContainer>::OnePixel,
    width: usize,
    height: usize,
  ) -> Self {
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

    result
  }

  pub fn as_other<
    NewComponent: PixelComponent,
    const NEW_COMPONENTS_PER_PEL: usize,
    const NEW_HAS_ALPHA: bool,
  >(
    &self,
  ) -> ImageBuffer<NewComponent, NEW_COMPONENTS_PER_PEL, NEW_HAS_ALPHA> {
    let mut result = ImageBuffer::<
      NewComponent,
      NEW_COMPONENTS_PER_PEL,
      NEW_HAS_ALPHA,
    >::empty(self.width, self.height);

    for (pel, new_pel) in self.iter().zip(result.iter_mut()) {
      for (c1, c2) in pel.iter().zip(new_pel.iter_mut()) {
        *c2 = <NewComponent as NumCast>::from(*c1).unwrap_or_default();
      }
    }

    result
  }

  /// Applies the given pixel mapping function and returns a new image buffer of
  /// the same type, with the result.
  ///
  /// This
  ///
  /// ```F``` is a function that operates on all channels of one pixel at a
  /// time.
  pub fn map<F>(&self, map_fn: &mut F) -> Self
  where F: FnMut(
      &<Self as PixelContainer>::OnePixel,
    ) -> <Self as PixelContainer>::OnePixel {
    let mut result = self.clone();

    for (pel, new_pel) in self.iter().zip(result.iter_mut()) {
      *new_pel = map_fn(pel);
    }

    result
  }

  /// Applies the given pixel mapping function, which can generate any type of
  /// iamge buffer, and returns a new buffer with the result
  pub fn map_into<
    F,
    NewComponent: PixelComponent,
    const NEW_COMPONENTS_PER_PEL: usize,
    const NEW_HAS_ALPHA: bool,
  >(
    &self,
    map_fn: &mut F,
  ) -> ImageBuffer<NewComponent, NEW_COMPONENTS_PER_PEL, NEW_HAS_ALPHA>
  where
    F: FnMut(
      &<Self as PixelContainer>::OnePixel,
    ) -> <ImageBuffer<
      NewComponent,
      NEW_COMPONENTS_PER_PEL,
      NEW_HAS_ALPHA,
    > as PixelContainer>::OnePixel,
  {
    let mut result = ImageBuffer::<
      NewComponent,
      NEW_COMPONENTS_PER_PEL,
      NEW_HAS_ALPHA,
    >::empty(self.width, self.height);

    for (pel, new_pel) in self.iter().zip(result.iter_mut()) {
      *new_pel = map_fn(pel);
    }

    result
  }

  /// Applies the given pixel mapping function in place, on the current mutable
  /// instance
  ///
  /// ```F``` is a function that operates on all channels of one pixel at a
  /// time.
  pub fn apply<F>(&mut self, map_fn: &mut F)
  where F: FnMut(
      &<Self as PixelContainer>::OnePixel,
    ) -> <Self as PixelContainer>::OnePixel {
    for pel in self.iter_mut() {
      *pel = map_fn(pel);
    }
  }

  pub fn get_plane_const<const I: usize>(
    &self,
  ) -> <Self as PixelContainer>::OnePlane {
    let mut result =
      <Self as PixelContainer>::OnePlane::empty(self.width, self.height);

    for (pel, new_pel) in self.iter().zip(result.iter_mut()) {
      *new_pel = [pel[I]];
    }

    result
  }

  pub fn get_alpha(&self) -> Option<<Self as PixelContainer>::OnePlane> {
    if !HAS_ALPHA {
      return None;
    }

    Some(self.get_plane(COMPONENTS_PER_PEL-1).unwrap_or_default())
  }

  pub fn get_plane(
    &self,
    i: usize,
  ) -> Result<<Self as PixelContainer>::OnePlane, &'static str> {
    if i >= COMPONENTS_PER_PEL {
      return Err("Plane index out of bounds");
    }

    let mut result =
      <Self as PixelContainer>::OnePlane::empty(self.width, self.height);

    for (pel, new_pel) in self.iter().zip(result.iter_mut()) {
      *new_pel = [pel[i]];
    }

    Ok(result)
  }

  pub fn put_plane_const<const I: usize>(
    &mut self,
    plane: &<Self as PixelContainer>::OnePlane,
  ) {
    for (pel, new_pel) in self.iter_mut().zip(plane.iter()) {
      pel[I] = new_pel[0];
    }
  }

  pub fn put_plane(
    &mut self,
    i: usize,
    plane: &<Self as PixelContainer>::OnePlane,
  ) {
    for (pel, new_pel) in self.iter_mut().zip(plane.iter()) {
      pel[i] = new_pel[0];
    }
  }
}

pub struct ImageBufferIterator<
  'a,
  Component: PixelComponent,
  const COMPONENT_STRIDE: usize,
  const HAS_ALPHA: bool,
  const SKIP_ALPHA: bool,
> {
  iterator: ArrayChunks<'a, Component, COMPONENT_STRIDE>,
}

pub struct ImagebufferIteratorMut<
  'a,
  Component: PixelComponent,
  const COMPONENT_STRIDE: usize,
  const HAS_ALPHA: bool,
  const SKIP_ALPHA: bool,
> {
  iterator: ArrayChunksMut<'a, Component, COMPONENT_STRIDE>,
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
      iterator: self.data.array_chunks::<COMPONENTS_PER_PEL>(),
    }
  }

  pub fn iter_no_alpha_mut(
    &'a mut self,
  ) -> ImagebufferIteratorMut<'a, Component, COMPONENTS_PER_PEL, HAS_ALPHA, true>
  {
    ImagebufferIteratorMut {
      iterator: self.data.array_chunks_mut::<COMPONENTS_PER_PEL>(),
    }
  }

  pub fn iter_with_alpha(
    &'a self,
  ) -> ImageBufferIterator<'a, Component, COMPONENTS_PER_PEL, HAS_ALPHA, false>
  {
    ImageBufferIterator {
      iterator: self.data.array_chunks::<COMPONENTS_PER_PEL>(),
    }
  }

  pub fn iter_with_alpha_mut(
    &'a mut self,
  ) -> ImagebufferIteratorMut<'a, Component, COMPONENTS_PER_PEL, HAS_ALPHA, false>
  {
    ImagebufferIteratorMut {
      iterator: self.data.array_chunks_mut::<COMPONENTS_PER_PEL>(),
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
  type Item = &'a [Component; COMPONENTS_PER_PEL];

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    // TODO: this should return the right number of components per pixel (known
    // at compile time) depending on whether we HAVE alpha AND whether we want
    // to SKIP it.
    self.iterator.next()
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
  type Item = &'a mut [Component; COMPONENTS_PER_PEL];

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    // TODO: this should return the right number of components per pixel (known
    // at compile time) depending on whether we HAVE alpha AND whether we want
    // to SKIP it.
    self.iterator.next()
  }
}

#[cfg(test)]
mod tests {

  use std::hint::black_box;
  use image::{DynamicImage, GenericImage};
  use test::Bencher;

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
    assert_eq!(image.data.len(), 4 * 4 * 4);
    for pel in image.iter_with_alpha() {
      assert_eq!(pel, &[1u8, 2u8, 3u8, 255]);
    }
  }

  #[test]
  fn new_rgb_u8_with_data() {
    const WIDTH: usize = 4;
    const HEIGHT: usize = 4;
    const RGB_CPP: usize = 3;
    let data = vec![0u8; WIDTH * HEIGHT * RGB_CPP];
    let image = ImageBuffer::<u8, 3, false>::with_data(data, WIDTH, HEIGHT);
    assert!(image.is_ok());
    assert_eq!(image.unwrap().data.len(), 4 * 4 * 3);
  }

  #[test]
  fn new_rgb_u8_empty() {
    const WIDTH: usize = 4;
    const HEIGHT: usize = 4;
    let image = ImageBuffer::<u8, 3, false>::empty(WIDTH, HEIGHT);
    assert_eq!(image.data.len(), 4 * 4 * 3);
    for pel in image.iter_no_alpha() {
      assert_eq!(pel, &[0u8, 0u8, 0u8]);
    }
  }

  #[test]
  fn new_rgb_u8_with_val() {
    const WIDTH: usize = 4;
    const HEIGHT: usize = 4;
    let one_pel = [1u8, 2u8, 3u8];
    let image = ImageBuffer::<u8, 3, false>::with_val(&one_pel, WIDTH, HEIGHT);
    assert_eq!(image.data.len(), 4 * 4 * 3);
    for pel in image.iter_no_alpha() {
      assert_eq!(pel, &[1u8, 2u8, 3u8]);
    }
  }

  #[bench]
  fn bench_new_rgba_u8_with_data(b: &mut Bencher) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    const RGBA_CPP: usize = 4;
    let data = vec![0u8; WIDTH * HEIGHT * RGBA_CPP];
    b.iter(|| {
      black_box(ImageBuffer::<u8, 4, true>::with_data(data.clone(), WIDTH, HEIGHT)
        .unwrap());
    });
  }

  #[bench]
  fn bench_new_rgba_u8_empty(b: &mut Bencher) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    b.iter(|| {
      black_box(ImageBuffer::<u8, 4, true>::empty(WIDTH, HEIGHT));
    });
  }

  #[bench]
  fn bench_new_rgba_u8_with_val(b: &mut Bencher) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    let one_pel = [0u8, 0u8, 0u8, 255];
    b.iter(|| {
      black_box(ImageBuffer::<u8, 4, true>::with_val(&one_pel, WIDTH, HEIGHT));
    });
  }

  #[bench]
  fn bench_new_rgba_u8_dynamic_image_empty(b: &mut Bencher) {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;
    b.iter(|| {
      black_box(DynamicImage::new_rgba8(WIDTH, HEIGHT));
    });
  }

  #[bench]
  fn bench_new_rgba_u8_dynamic_image_with_data(b: &mut Bencher) {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;
    const RGBA_CPP: u32 = 4;
    let data = vec![0u8; WIDTH as usize * HEIGHT as usize * RGBA_CPP as usize];
    let buf = image::ImageBuffer::from_vec(WIDTH, HEIGHT, data).unwrap();
    b.iter(|| black_box(DynamicImage::ImageRgba8(buf.clone())));
  }

  #[bench]
  fn bench_new_rgba_u8_dynamic_image_with_val(b: &mut Bencher) {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;
    let one_pel = image::Rgba([0u8, 0u8, 0u8, 255]);
    let buf = image::ImageBuffer::from_pixel(WIDTH, HEIGHT, one_pel);
    b.iter(|| black_box(DynamicImage::ImageRgba8(buf.clone())));
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
