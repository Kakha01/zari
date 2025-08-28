use std::path::Path;

use crate::engine::{AudioError, Track, TrackId};

pub struct Timeline {
    tracks: Vec<Track>,
    sample_rate: u32,
    playhead_position: u64,
    mix_buffer: Vec<f32>,
}

impl Timeline {
    pub fn new(sample_rate: u32) -> Self {
        Timeline {
            tracks: Vec::new(),
            sample_rate,
            playhead_position: 0,
            mix_buffer: Vec::new(),
        }
    }

    pub fn new_track(&mut self) -> TrackId {
        let track_id = if let Some(track) = self.tracks.last() {
            track.id + TrackId(1)
        } else {
            TrackId(1)
        };

        let track = Track::new(track_id);

        self.tracks.push(track);

        track_id
    }

    pub fn add_clip<P: AsRef<Path>>(
        &mut self,
        track_id: TrackId,
        path: P,
    ) -> Result<(), AudioError> {
        let timeline_sample_rate = self.sample_rate;

        let track = self
            .get_mut_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        track.add_clip(path, timeline_sample_rate)?;

        Ok(())
    }

    pub fn set_volume(&mut self, track_id: TrackId, volume_percent: f32) -> Result<(), AudioError> {
        let track = self
            .get_mut_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        track.set_volume_percent(volume_percent)?;

        Ok(())
    }

    pub fn set_track_name(
        &mut self,
        track_id: TrackId,
        new_name: String,
    ) -> Result<(), AudioError> {
        let track = self
            .get_mut_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        track.name = new_name;

        Ok(())
    }

    pub fn toggle_mute(&mut self, track_id: TrackId) -> Result<(), AudioError> {
        let track = self
            .get_mut_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        track.toggle_mute();

        Ok(())
    }

    pub fn toggle_solo(&mut self, track_id: TrackId) -> Result<(), AudioError> {
        let track = self
            .get_mut_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        if track.is_soloed() {
            track.unsolo();
        } else {
            track.solo();
            self.tracks
                .iter_mut()
                .filter(|t| t.id != track_id)
                .for_each(|track| {
                    track.unsolo();
                    track.mute();
                });
        }

        Ok(())
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    pub fn get_track_ids(&self) -> Vec<TrackId> {
        self.tracks.iter().map(|t| t.id).collect()
    }

    pub fn duration_in_samples(&self) -> u64 {
        self.tracks
            .iter()
            .map(|track| track.duration_in_samples())
            .max()
            .unwrap_or(0)
    }

    pub fn duration_in_seconds(&self) -> f64 {
        self.duration_in_samples() as f64 / self.sample_rate as f64
    }

    pub fn playhead_position_seconds(&self) -> f64 {
        self.playhead_position as f64 / self.sample_rate as f64
    }

    pub fn set_playhead_seconds(&mut self, seconds: f64) {
        self.playhead_position = (seconds * self.sample_rate as f64) as u64;
    }

    pub fn get_mut_track(&mut self, track_id: TrackId) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == track_id)
    }

    pub fn get_track(&self, track_id: TrackId) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == track_id)
    }

    pub fn process(&mut self, num_samples: usize, output_channels: u16) -> Vec<Vec<f32>> {
        let mut output_buffer: Vec<Vec<f32>> = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            self.mix_buffer.clear();
            self.mix_buffer.resize(output_channels as usize, 0.0);

            let soloed_track = self.tracks.iter().find(|t| t.is_soloed());

            for track in self.tracks.iter() {
                let should_play = if let Some(track_id) = soloed_track {
                    track.id == track_id.id
                } else {
                    !track.is_muted()
                };

                if !should_play {
                    continue;
                }

                if let Some(clip) = track.find_clip_at_playhead_position(self.playhead_position) {
                    clip.process_sample(
                        &mut self.mix_buffer,
                        track.volume(),
                        output_channels,
                        self.playhead_position,
                    );
                }
            }

            output_buffer.push(self.mix_buffer.clone());
            self.playhead_position += 1;
        }

        output_buffer
    }
}
