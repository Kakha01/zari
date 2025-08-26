use rubato::{
    self, Resampler as RubatoResampler, SincFixedIn, SincInterpolationParameters,
    SincInterpolationType, WindowFunction,
};
use std::fmt::Display;
use std::ops::Add;
use std::path::Path;

#[derive(Clone, Copy, PartialEq)]
pub struct TrackId(u32);

impl Add for TrackId {
    type Output = TrackId;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Display for TrackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct Resampler;

impl Resampler {
    const PARAMETERS: SincInterpolationParameters = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    fn resample(
        samples: Vec<f32>,
        new_sample_rate: u32,
        old_sample_rate: u32,
        num_channels: u16,
    ) -> Result<Vec<f32>, anyhow::Error> {
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
        let num_frames = if let Some(wave) = waves_out.get(0) {
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

pub struct Clip {
    data: Vec<f32>,
    channel: u16,
    start_time_in_samples: u64,
}

#[allow(non_camel_case_types)]
struct i24;
impl i24 {
    const MAX: i32 = 8_388_607;
}

const I8_SCALE: f32 = 1.0 / (i8::MAX as f32 + 1.0);
const I16_SCALE: f32 = 1.0 / (i16::MAX as f32 + 1.0);
const I24_SCALE: f32 = 1.0 / (i24::MAX as f32 + 1.0);
const I32_SCALE: f32 = 1.0 / (i32::MAX as f32 + 1.0);

impl Clip {
    pub fn from_path(
        path: &Path,
        start_time_in_samples: u64,
        timeline_sample_rate: u32,
    ) -> Result<Self, hound::Error> {
        let reader = hound::WavReader::open(path)?;
        let spec: hound::WavSpec = reader.spec();
        dbg!(spec);
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => match spec.bits_per_sample {
                8 => reader
                    .into_samples::<i8>()
                    .map(|sample| sample.map_or(0.0, |s| s as f32 * I8_SCALE))
                    .collect(),
                16 => reader
                    .into_samples::<i16>()
                    .map(|sample| sample.map_or(0.0, |s| s as f32 * I16_SCALE))
                    .collect(),
                24 => reader
                    .into_samples::<i32>()
                    .map(|sample| sample.map_or(0.0, |s| s as f32 * I24_SCALE))
                    .collect(),
                32 => reader
                    .into_samples::<i32>()
                    .map(|sample| sample.map_or(0.0, |s| s as f32 * I32_SCALE))
                    .collect(),
                _ => return Err(hound::Error::Unsupported),
            },
            hound::SampleFormat::Float => match spec.bits_per_sample {
                32 => reader
                    .into_samples::<f32>()
                    .map(|sample| sample.unwrap_or(0.0))
                    .collect(),
                _ => return Err(hound::Error::Unsupported),
            },
        };

        let data = if dbg!(spec.sample_rate) != timeline_sample_rate {
            Resampler::resample(
                samples,
                timeline_sample_rate,
                spec.sample_rate,
                spec.channels,
            )
            .unwrap()
        } else {
            samples
        };

        Ok(Clip {
            data,
            channel: spec.channels,
            start_time_in_samples,
        })
    }
}

pub struct Track {
    id: TrackId,
    name: String,
    volume: f32,
    clips: Vec<Clip>,
    is_muted: bool,
    is_soloed: bool,
}

impl Track {
    pub fn new() -> Self {
        Track {
            id: TrackId(0),
            name: String::from("Track"),
            volume: 1.0,
            clips: Vec::new(),
            is_muted: false,
            is_soloed: false,
        }
    }

    pub fn add_clip(&mut self, path: &Path, timeline_sample_rate: u32) -> Result<(), hound::Error> {
        let start_time_in_samples: u64 = if let Some(c) = self.clips.last() {
            c.start_time_in_samples + c.data.len() as u64 + 1
        } else {
            0
        };

        let clip = Clip::from_path(path, start_time_in_samples, timeline_sample_rate)?;

        self.clips.push(clip);
        Ok(())
    }
}

impl Default for Track {
    fn default() -> Self {
        Self {
            id: TrackId(1),
            volume: 1.0,
            clips: Vec::new(),
            is_muted: false,
            is_soloed: false,
            name: "Default Track".into(),
        }
    }
}

pub struct Timeline {
    tracks: Vec<Track>,
    sample_rate: u32,
    playhead_position: u64,
}

impl Timeline {
    pub fn new(sample_rate: u32) -> Self {
        Timeline {
            tracks: Vec::new(),
            sample_rate,
            playhead_position: 0,
        }
    }

    pub fn new_track(&mut self) -> TrackId {
        let track_id = if let Some(track) = self.tracks.last() {
            track.id + TrackId(1)
        } else {
            TrackId(1)
        };

        let track = Track {
            id: track_id,
            name: format!("Track {}", track_id).into(),
            ..Track::default()
        };

        self.tracks.push(track);

        track_id
    }

    pub fn add_clip(&mut self, track_id: TrackId, path: &Path) -> Result<(), hound::Error> {
        let timeline_sample_rate = self.sample_rate;

        if let Some(track) = self.find_track(track_id) {
            track.add_clip(path, timeline_sample_rate)?;
        }

        Ok(())
    }

    pub fn set_volume(&mut self, track_id: TrackId, volume: f32) {
        if volume < 0.0 && volume > 1.0 {
            return;
        }

        if let Some(track) = self.find_track(track_id) {
            track.volume = volume;
        }
    }

    pub fn set_track_name(&mut self, track_id: TrackId, new_name: String) {
        if let Some(track) = self.find_track(track_id) {
            track.name = new_name;
        }
    }

    pub fn toggle_mute(&mut self, track_id: TrackId) {
        if let Some(track) = self.find_track(track_id) {
            track.is_muted = !track.is_muted;
            track.is_soloed = false;
        }
    }

    pub fn toggle_solo(&mut self, track_id: TrackId) {
        if let Some(track) = self.find_track(track_id) {
            track.is_soloed = !track.is_soloed;
            track.is_muted = false;
        }
    }

    pub fn find_track(&mut self, track_id: TrackId) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == track_id)
    }

    pub fn process(&mut self, num_samples: usize) -> Vec<[f32; 2]> {
        let mut output_buffer: Vec<[f32; 2]> = Vec::new();

        for _ in 0..num_samples {
            let mut mix_bus: [f32; 2] = [0.0, 0.0];

            for track in self.tracks.iter() {
                if track.is_muted {
                    continue;
                }

                for clip in track.clips.iter() {
                    let clip_end_time =
                        clip.start_time_in_samples + clip.data.len() as u64 / clip.channel as u64;

                    if self.playhead_position >= clip.start_time_in_samples
                        && self.playhead_position < clip_end_time
                    {
                        let frame_within_clip = self.playhead_position - clip.start_time_in_samples;

                        match clip.channel {
                            1 => {
                                let sample_index = frame_within_clip as usize;
                                let sample = clip.data[sample_index];

                                mix_bus[0] += sample * track.volume;
                                mix_bus[1] += sample * track.volume;
                            }
                            2 => {
                                let sample_index = (frame_within_clip * 2) as usize;
                                let sample1 = clip.data[sample_index];
                                let sample2 = clip.data[sample_index + 1];

                                mix_bus[0] += sample1 * track.volume;
                                mix_bus[1] += sample2 * track.volume;
                            }
                            _ => {}
                        }
                    }
                }
            }

            output_buffer.push(mix_bus);
            self.playhead_position += 1;
        }

        output_buffer
    }
}
