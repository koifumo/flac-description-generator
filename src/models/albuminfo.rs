use std::io::{Error, ErrorKind};
use lofty::file::TaggedFile;
use lofty::tag::{Accessor, ItemKey, Tag};
use crate::{HasVorbisTag};

#[derive(Debug)]
pub struct AlbumInfo {
    pub artist: String,
    pub title: String,
    pub release_date: String,
    pub genres: Vec<String>,
}

fn get_item_key_as_text(tag: &Tag, item_key: &ItemKey) -> Result<String, Error> {
    let Some(tag_item) = &tag.get(item_key) else {
        return Err(Error::new(ErrorKind::NotFound, "Key not found."))
    };
    let Some(text) = tag_item.value().text() else {
        return Err(Error::new(ErrorKind::NotFound, "Key not found."))
    };
    Ok(text.to_string())
}

fn get_release_date(tag: &Tag) -> Option<String> {
    if let Ok(release_date) = get_item_key_as_text(tag, &ItemKey::RecordingDate) {
        return Some(release_date);
    }
    if let Some(release_year) = tag.year() {
        return Some(release_year.to_string());
    }
    None
}

fn get_genres(tagged_files: &Vec<TaggedFile>) -> Result<Vec<String>, Error> {
    let mut genres = Vec::new();
    for file in tagged_files {
        if let Some(genre) = file.to_vorbis_tag()?.genre() {
            let genre_string = genre.to_string();
            if !genres.contains(&genre_string) {
                genres.push(genre_string);
            }
        }
    }
    Ok(genres)
}

impl From<Vec<TaggedFile>> for AlbumInfo {
    fn from(tagged_files: Vec<TaggedFile>) -> Self {
        let Some(last_track) = tagged_files.last() else {
            panic!("No FLAC files in directory.");
        };
        let Ok(album_tag) = last_track.to_vorbis_tag() else {
            panic!("No Vorbis TAGS on FLAC file.")
        };
        let Ok(artist) = get_item_key_as_text(&album_tag, &ItemKey::AlbumArtist) else {
            panic!("No album artist found.");
        };
        let Some(title) = &album_tag.album() else {
            panic!("No album name found.");
        };
        let title = title.to_string();
        let Some(release_date) = get_release_date(&album_tag) else {
            panic!("No release date found.");
        };
        let Ok(genres) = get_genres(&tagged_files) else {
            panic!("Error finding genres.");
        };

        AlbumInfo {
            artist,
            title,
            release_date,
            genres,
        }
    }
}