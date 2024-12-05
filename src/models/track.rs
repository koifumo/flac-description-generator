use lofty::file::{AudioFile, TaggedFile};
use lofty::prelude::Accessor;
use crate::HasVorbisTag;

#[derive(Debug)]
pub struct Track {
    pub duration_in_millis: u32,
    pub disc: u32,
    pub track: u32,
    pub artists: Vec<String>,
    pub title: String,
}
impl From<&TaggedFile> for Track {
    fn from(tagged_file: &TaggedFile) -> Self {
        let duration_in_millis = tagged_file.properties().duration().as_millis() as u32;
        let Ok(tag) = tagged_file.to_vorbis_tag() else {
            panic!("Missing vorbis tags on track.");
        };
        let disc = tag.disk().unwrap_or(1);
        let Some(track) = tag.track() else {
            panic!("Missing track number on track.")
        };
        let Some(title) = tag.title() else {
            panic!("Missing title tag on track.")
        };
        let title = title.to_string();
        let Some(artist_string) = tag.artist() else {
            panic!("Missing artist tag on track.")
        };
        let artists: Vec<String> = artist_string.split([',', '&']).map(str::trim).map(str::to_string).collect();

        Track {
            duration_in_millis,
            disc,
            track,
            artists,
            title,
        }
    }
}