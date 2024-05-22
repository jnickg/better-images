use num_traits::{Num, Zero, ToPrimitive, NumCast};

pub trait PixelComponent: Num + Copy + Clone + Zero + Sized + ToPrimitive + NumCast + Default {
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
  type PixelBuffer;
  type OnePlane;
  const HAS_ALPHA: bool;
  const NUM_COMPONENTS: usize;
  const ALPHA_IDX: Option<usize>;
  const NUM_NONALPHA_COMPONENTS: usize;

  fn pixels(&self) -> &Self::PixelBuffer;
  fn pixels_mut(&mut self) -> &mut Self::PixelBuffer;
}
