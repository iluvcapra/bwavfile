pub use dasp_sample::I24;

use dasp_sample::Duplex;

pub trait Sample:
    dasp_sample::Sample + Duplex<u8> + Duplex<i16> + Duplex<I24> + Duplex<i32> + Duplex<f32>
{
}

impl Sample for u8 {}
impl Sample for i16 {}
impl Sample for I24 {}
impl Sample for i32 {}
impl Sample for f32 {}
