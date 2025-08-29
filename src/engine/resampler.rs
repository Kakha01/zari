use rubato::{
    Resampler as RubatoResampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};

use crate::engine::{AudioError, utils::Utils};

pub struct Resampler;

impl Resampler {
    const PARAMETERS: SincInterpolationParameters = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    pub fn resample(
        samples: Vec<f64>,
        new_sample_rate: u32,
        old_sample_rate: u32,
        num_channels: u16,
    ) -> Result<Vec<f64>, AudioError> {
        if num_channels == 0 || samples.is_empty() {
            return Ok(Vec::new());
        }

        let num_channels = num_channels as usize;

        let deinterleaved_samples: Vec<Vec<f64>> =
            Utils::deinterleave_samples(&samples, num_channels);

        let mut resampler = SincFixedIn::<f64>::new(
            new_sample_rate as f64 / old_sample_rate as f64,
            2.0,
            Resampler::PARAMETERS,
            samples.len() / num_channels,
            num_channels,
        )?;

        let samples: Vec<Vec<f64>> = resampler.process(&deinterleaved_samples, None)?;

        let interleaved_output = Utils::interleave_samples(&samples);
        Ok(interleaved_output)
    }
}
