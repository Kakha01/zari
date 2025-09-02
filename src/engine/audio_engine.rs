use cpal::{
    BuildStreamError, Device, SampleFormat, SampleRate, Stream, StreamError, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::engine::{Timeline, error::AudioError};
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    timeline: Arc<Mutex<Timeline>>,
    input_device: Option<Device>,
    output_device: Option<Device>,
    output_stream: Option<Stream>,
    input_stream: Option<Stream>,
    config: SupportedStreamConfig,
}

macro_rules! create_output_callback {
    ($type:ty, $timeline:expr, $channels:expr) => {
        move |data: &mut [$type], _| {
            if let Ok(mut timeline) = $timeline.lock() {
                timeline.process(data, $channels);
            }
        }
    };
}

impl AudioEngine {
    pub fn new(
        channels: u16,
        sample_format: SampleFormat,
        sample_rate: SampleRate,
    ) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let output_device = host.default_output_device().unwrap();
        let input_device = host.default_input_device().unwrap();

        let mut supported_output_configs = output_device.supported_output_configs().unwrap();

        let supported_config = supported_output_configs.find(|c| {
            c.channels() == channels
                && c.sample_format() == sample_format
                && c.min_sample_rate() < sample_rate
                && c.max_sample_rate() > sample_rate
        });

        let config = supported_config
            .expect("Config is not supported")
            .with_sample_rate(sample_rate);

        Ok(AudioEngine {
            timeline: Arc::new(Mutex::new(Timeline::new(sample_rate.0))),
            output_device: Some(output_device),
            input_device: Some(input_device),
            output_stream: None,
            input_stream: None,
            config,
        })
    }

    pub fn start_playing(&mut self) -> Result<(), AudioError> {
        let output_device = self
            .output_device
            .as_ref()
            .ok_or(AudioError::OutputDeviceNotFound)?;

        let timeline_clone = self.timeline.clone();
        let config = self.config.config();
        let channels = config.channels;

        let stream = match self.config.sample_format() {
            SampleFormat::U8 => output_device.build_output_stream(
                &config,
                create_output_callback!(u8, timeline_clone, channels),
                Self::error_callback,
                None,
            ),
            SampleFormat::I16 => output_device.build_output_stream(
                &config,
                create_output_callback!(i16, timeline_clone, channels),
                Self::error_callback,
                None,
            ),
            SampleFormat::I32 => output_device.build_output_stream(
                &config,
                create_output_callback!(i32, timeline_clone, channels),
                Self::error_callback,
                None,
            ),
            SampleFormat::F32 => output_device.build_output_stream(
                &config,
                create_output_callback!(f32, timeline_clone, channels),
                Self::error_callback,
                None,
            ),
            SampleFormat::F64 => output_device.build_output_stream(
                &config,
                create_output_callback!(f64, timeline_clone, channels),
                Self::error_callback,
                None,
            ),
            _ => Err(BuildStreamError::StreamConfigNotSupported),
        }
        .map_err(AudioError::StreamConfigNotSupported)?;

        stream.play().map_err(AudioError::PlayStreamError)?;

        self.set_output_stream(stream);

        Ok(())
    }

    pub fn stop_playing(&mut self) {
        self.output_stream = None;
        if let Ok(mut timeline) = self.timeline.lock() {
            timeline.reset_playhead();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.output_stream.is_some()
    }

    pub fn start_recording(&mut self) -> Result<(), AudioError> {
        let input_device = self
            .input_device
            .as_ref()
            .ok_or(AudioError::OutputDeviceNotFound)?;

        let config = self.config.config();

        let stream = match self.config.sample_format() {
            SampleFormat::F32 => input_device.build_input_stream(
                &config,
                move |_data: &[f32], _| {},
                Self::error_callback,
                None,
            ),
            _ => Err(BuildStreamError::StreamConfigNotSupported),
        }
        .map_err(AudioError::StreamConfigNotSupported)?;

        stream.play().map_err(AudioError::PlayStreamError)?;

        self.set_input_stream(stream);

        Ok(())
    }

    pub fn stop_recording(&mut self) {
        self.input_stream = None;
        if let Ok(mut timeline) = self.timeline.lock() {
            timeline.reset_playhead();
        }
    }

    pub fn is_recording(&self) -> bool {
        self.input_stream.is_some()
    }

    pub fn set_output_device() {}

    pub fn set_input_device() {}

    pub fn timeline(&self) -> Arc<Mutex<Timeline>> {
        self.timeline.clone()
    }

    fn set_output_stream(&mut self, stream: Stream) {
        self.output_stream = Some(stream);
    }

    fn set_input_stream(&mut self, stream: Stream) {
        self.input_stream = Some(stream);
    }

    fn error_callback(err: StreamError) {
        eprintln!("{err:?}");
    }
}
