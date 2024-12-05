use crate::models::track::Track;

#[derive(Debug)]
pub struct Disc {
    pub duration_in_millis: u32,
    pub disc: u32,
    pub tracks: Vec<Track>,
}

impl Disc {
    pub fn add_track(&mut self, track: Track) -> &mut Self {
        self.duration_in_millis += track.duration_in_millis;
        self.tracks.push(track);
        self
    }
}