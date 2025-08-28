use crate::engine::{AudioError, Clip};
use std::{fmt::Display, ops::Add, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackId(pub u32);

impl TrackId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for TrackId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

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

pub struct Track {
    pub id: TrackId,
    pub name: String,
    volume: f32,
    clips: Vec<Clip>,
    is_muted: bool,
    is_soloed: bool,
}

impl Track {
    pub fn new(id: TrackId) -> Self {
        Track {
            id,
            name: format!("Track {}", id).into(),
            ..Track::default()
        }
    }

    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    pub fn find_clip_at_playhead_position(&self, playhead_position: u64) -> Option<&Clip> {
        self.clips.iter().find(|c| {
            playhead_position >= c.start_time_in_samples()
                && playhead_position < c.end_time_in_samples()
        })
    }

    pub fn duration_in_samples(&self) -> u64 {
        self.clips
            .iter()
            .map(|clip| clip.end_time_in_samples())
            .max()
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn volume_percent(&self) -> f32 {
        self.volume * 100.0
    }

    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    pub fn is_soloed(&self) -> bool {
        self.is_soloed
    }

    pub fn unmute(&mut self) {
        self.is_muted = false;
    }

    pub fn mute(&mut self) {
        self.is_muted = true;
    }

    pub fn solo(&mut self) {
        self.is_soloed = true;
    }

    pub fn unsolo(&mut self) {
        self.is_soloed = false;
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }

    pub fn toggle_solo(&mut self) {
        self.is_soloed = !self.is_soloed;
    }

    pub fn set_volume_percent(&mut self, percent: f32) -> Result<(), AudioError> {
        let volume = percent / 100.0;
        if volume < 0.0 || volume > 1.0 {
            return Err(AudioError::InvalidVolume(volume));
        }
        self.volume = volume;
        Ok(())
    }

    pub fn add_clip(&mut self, path: &Path, timeline_sample_rate: u32) -> Result<(), AudioError> {
        let start_time_in_samples: u64 = if let Some(c) = self.clips.last() {
            c.start_time_in_samples() + c.sample_count() as u64 + 1
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
