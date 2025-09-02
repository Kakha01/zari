mod audio_engine;
mod clip;
mod error;
mod resampler;
mod timeline;
mod track;
mod utils;

use clip::Clip;
use error::AudioError;
use resampler::Resampler;
use track::Track;
use track::TrackId;

#[derive(Clone, Copy)]
enum Scale {
    I8,
    I16,
    I24,
    I32,
    F32,
}

impl Scale {
    const I8_SCALE: f64 = 1.0 / i8::MAX as f64;
    const I16_SCALE: f64 = 1.0 / i16::MAX as f64;
    const I24_SCALE: f64 = 1.0 / ((1 << 23) - 1) as f64;
    const I32_SCALE: f64 = 1.0 / i32::MAX as f64;

    fn get_f64_scale(&self) -> f64 {
        match self {
            Scale::I8 => Self::I8_SCALE,
            Scale::I16 => Self::I16_SCALE,
            Scale::I24 => Self::I24_SCALE,
            Scale::I32 => Self::I32_SCALE,
            Scale::F32 => 1.0,
        }
    }
}

pub trait FromF64Sample {
    fn from_f64_sample(sample: f64) -> Self;
}

impl FromF64Sample for u8 {
    fn from_f64_sample(sample: f64) -> Self {
        ((sample + 1.0) * 127.5) as u8
    }
}

impl FromF64Sample for i16 {
    fn from_f64_sample(sample: f64) -> Self {
        (sample * i16::MAX as f64) as i16
    }
}

impl FromF64Sample for i32 {
    fn from_f64_sample(sample: f64) -> Self {
        (sample * i32::MAX as f64) as i32
    }
}

impl FromF64Sample for f32 {
    fn from_f64_sample(sample: f64) -> Self {
        sample as f32
    }
}

impl FromF64Sample for f64 {
    fn from_f64_sample(sample: f64) -> Self {
        sample
    }
}

pub use audio_engine::AudioEngine;
pub use timeline::Timeline;
