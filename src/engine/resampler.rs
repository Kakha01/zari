use rubato::{
    Resampler as RubatoResampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};

use crate::engine::AudioError;

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
        samples: Vec<f32>,
        new_sample_rate: u32,
        old_sample_rate: u32,
        num_channels: u16,
    ) -> Result<Vec<f32>, AudioError> {
        if num_channels == 0 || samples.is_empty() {
            return Ok(Vec::new());
        }

        let num_channels = num_channels as usize;
        let num_frames = samples.len() / num_channels;

        let deinterleaved_samples: Vec<Vec<f32>> =
            Resampler::deinterleave_samples(samples, num_frames, num_channels);

        let mut resampler = SincFixedIn::<f32>::new(
            new_sample_rate as f64 / old_sample_rate as f64,
            2.0,
            Resampler::PARAMETERS,
            num_frames,
            num_channels,
        )?;

        let waves_out: Vec<Vec<f32>> = resampler.process(&deinterleaved_samples, None)?;

        let num_channels = waves_out.len();
        let num_frames = if let Some(wave) = waves_out.first() {
            wave.len()
        } else {
            return Ok(Vec::new());
        };

        let interleaved_output = Resampler::interleave_samples(waves_out, num_frames, num_channels);
        Ok(interleaved_output)
    }

    fn deinterleave_samples(
        samples: Vec<f32>,
        num_frames: usize,
        num_channels: usize,
    ) -> Vec<Vec<f32>> {
        let mut deinterleaved_channels: Vec<Vec<f32>> =
            vec![Vec::with_capacity(num_frames); num_channels];

        for frame in samples.chunks_exact(num_channels) {
            for (channel_index, sample) in frame.iter().enumerate() {
                deinterleaved_channels[channel_index].push(*sample);
            }
        }

        deinterleaved_channels
    }

    fn interleave_samples(
        samples: Vec<Vec<f32>>,
        num_frames: usize,
        num_channels: usize,
    ) -> Vec<f32> {
        let mut interleaved_output: Vec<f32> = vec![0.0; num_channels * num_frames];

        for frame_idx in 0..num_frames {
            for chan_idx in 0..num_channels {
                interleaved_output[frame_idx * num_channels + chan_idx] =
                    samples[chan_idx][frame_idx];
            }
        }

        interleaved_output
    }
}
