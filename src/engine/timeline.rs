use std::{collections::HashSet, ops::AddAssign, path::Path};

use crate::engine::{AudioError, FromF64Sample, Track, TrackId};

pub struct Timeline {
    tracks: Vec<Track>,
    active_track_ids: HashSet<TrackId>,
    sample_rate: u32,
    playhead_position: u64,
}

impl Timeline {
    pub fn new(sample_rate: u32) -> Self {
        Timeline {
            tracks: Vec::new(),
            active_track_ids: HashSet::new(),
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

        let track = Track::new(track_id);

        self.active_track_ids.insert(track.id);
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

    pub fn mute(&mut self, track_id: TrackId) -> Result<(), AudioError> {
        let track = self
            .get_mut_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        if !track.is_muted() {
            track.mute();
            track.unsolo();
            self.active_track_ids.remove(&track_id);
        }

        Ok(())
    }

    pub fn unmute(&mut self, track_id: TrackId) -> Result<(), AudioError> {
        let track = self
            .get_mut_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        if track.is_muted() {
            track.unmute();
            self.active_track_ids.insert(track_id);
        }

        Ok(())
    }

    pub fn is_muted(&self, track_id: TrackId) -> Result<bool, AudioError> {
        let track = self
            .get_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        Ok(track.is_muted())
    }

    pub fn solo(&mut self, track_id: TrackId) -> Result<(), AudioError> {
        let track = self
            .get_mut_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        if !track.is_soloed() {
            track.solo();
            self.tracks
                .iter_mut()
                .filter(|t| t.id != track_id)
                .for_each(|track| {
                    track.unsolo();
                    track.mute();
                });
            self.active_track_ids.clear();
            self.active_track_ids.insert(track_id);
        }

        Ok(())
    }

    pub fn unsolo(&mut self, track_id: TrackId) -> Result<(), AudioError> {
        let track = self
            .get_mut_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        if track.is_soloed() {
            track.unsolo();
            if track.is_muted() {
                self.active_track_ids.remove(&track_id);
            }
        }

        Ok(())
    }

    pub fn is_soloed(&self, track_id: TrackId) -> Result<bool, AudioError> {
        let track = self
            .get_track(track_id)
            .ok_or(AudioError::TrackNotFound(track_id))?;

        Ok(track.is_soloed())
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

    pub fn reset_playhead(&mut self) {
        self.playhead_position = 0;
    }

    pub fn get_mut_track(&mut self, track_id: TrackId) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == track_id)
    }

    pub fn get_track(&self, track_id: TrackId) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == track_id)
    }

    pub fn process<T>(&mut self, buffer: &mut [T], output_channels: u16)
    where
        T: FromF64Sample + Default + Clone + AddAssign,
    {
        let samples_per_frame = output_channels as usize;
        let num_frames = buffer.len() / samples_per_frame;

        buffer.fill(T::default());

        for frame_idx in 0..num_frames {
            let soloed_track = self.tracks.iter().find(|t| t.is_soloed());

            self.tracks
                .iter()
                .filter(|track| {
                    if let Some(soloed_track) = soloed_track {
                        track.id == soloed_track.id
                    } else {
                        !track.is_muted()
                    }
                })
                .filter_map(|track| {
                    track
                        .find_clip_at_playhead_position(self.playhead_position)
                        .map(|clip| (clip, track.volume()))
                })
                .for_each(|(clip, volume)| {
                    clip.process_sample(
                        buffer,
                        volume,
                        output_channels,
                        self.playhead_position,
                        frame_idx,
                    )
                });

            self.playhead_position += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_mute() -> Result<(), anyhow::Error> {
        let mut timeline = Timeline::new(44100);
        let track_1 = timeline.new_track();
        let track_2 = timeline.new_track();
        let track_3 = timeline.new_track();

        timeline.mute(track_2)?;

        assert!(!timeline.is_muted(track_1)?);
        assert!(timeline.is_muted(track_2)?);
        assert!(!timeline.is_muted(track_3)?);

        assert!(timeline.active_track_ids.contains(&track_1));
        assert!(!timeline.active_track_ids.contains(&track_2));
        assert!(timeline.active_track_ids.contains(&track_3));

        timeline.unmute(track_2)?;

        assert!(!timeline.is_muted(track_1)?);
        assert!(!timeline.is_muted(track_2)?);
        assert!(!timeline.is_muted(track_3)?);

        assert!(timeline.active_track_ids.contains(&track_1));
        assert!(timeline.active_track_ids.contains(&track_2));
        assert!(timeline.active_track_ids.contains(&track_3));

        timeline.mute(track_1)?;
        timeline.mute(track_3)?;

        assert!(timeline.is_muted(track_1)?);
        assert!(!timeline.is_muted(track_2)?);
        assert!(timeline.is_muted(track_3)?);

        assert!(!timeline.active_track_ids.contains(&track_1));
        assert!(timeline.active_track_ids.contains(&track_2));
        assert!(!timeline.active_track_ids.contains(&track_3));

        Ok(())
    }

    #[test]
    fn test_toggle_solo() -> Result<(), anyhow::Error> {
        let mut timeline = Timeline::new(44100);
        let track_id = timeline.new_track();

        timeline.solo(track_id)?;

        assert!(timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        timeline.unsolo(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        Ok(())
    }

    #[test]
    fn test_mute_solo_unsolo_unmute() -> Result<(), anyhow::Error> {
        let mut timeline = Timeline::new(44100);
        let track_id = timeline.new_track();
        timeline.mute(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(!timeline.active_track_ids.contains(&track_id));

        timeline.solo(track_id)?;

        assert!(timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        timeline.unsolo(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(!timeline.active_track_ids.contains(&track_id));

        timeline.unmute(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        Ok(())
    }

    #[test]
    fn test_mute_solo_unmute_unsolo() -> Result<(), anyhow::Error> {
        let mut timeline = Timeline::new(44100);
        let track_id = timeline.new_track();
        timeline.mute(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(!timeline.active_track_ids.contains(&track_id));

        timeline.solo(track_id)?;

        assert!(timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        timeline.unmute(track_id)?;

        assert!(timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        timeline.unsolo(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        Ok(())
    }

    #[test]
    fn test_solo_mute_unsolo_unmute() -> Result<(), anyhow::Error> {
        let mut timeline = Timeline::new(44100);
        let track_id = timeline.new_track();
        timeline.solo(track_id)?;

        assert!(timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        timeline.mute(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(!timeline.active_track_ids.contains(&track_id));

        timeline.unsolo(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(!timeline.active_track_ids.contains(&track_id));

        timeline.unmute(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        Ok(())
    }

    #[test]
    fn test_solo_mute_unmute_unsolo() -> Result<(), anyhow::Error> {
        let mut timeline = Timeline::new(44100);
        let track_id = timeline.new_track();
        timeline.solo(track_id)?;

        assert!(timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        timeline.mute(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(!timeline.active_track_ids.contains(&track_id));

        timeline.unmute(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        timeline.unsolo(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));

        Ok(())
    }

    #[test]
    fn test_mute_solo_unsolo_unmute_relative_to_other_track() -> Result<(), anyhow::Error> {
        let mut timeline = Timeline::new(44100);
        let track_id = timeline.new_track();
        let other_track_id = timeline.new_track();

        timeline.mute(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(!timeline.active_track_ids.contains(&track_id));
        assert!(timeline.active_track_ids.contains(&other_track_id));

        timeline.solo(track_id)?;

        assert!(timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));
        assert!(!timeline.active_track_ids.contains(&other_track_id));

        timeline.unsolo(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(timeline.is_muted(track_id)?);
        assert!(!timeline.active_track_ids.contains(&track_id));
        assert!(!timeline.active_track_ids.contains(&other_track_id));

        timeline.unmute(track_id)?;

        assert!(!timeline.is_soloed(track_id)?);
        assert!(!timeline.is_muted(track_id)?);
        assert!(timeline.active_track_ids.contains(&track_id));
        assert!(!timeline.active_track_ids.contains(&other_track_id));

        Ok(())
    }

    fn test_with_solo_track_add_new_track() -> Result<(), anyhow::Error> {
        let mut timeline = Timeline::new(44100);
        let track_id = timeline.new_track();
        timeline.solo(track_id)?;

        let new_track_id = timeline.new_track();

        // New track should be muted, as we have soloed track 
        assert!(timeline.is_muted(track_id)?);

        Ok(())
    }
}
