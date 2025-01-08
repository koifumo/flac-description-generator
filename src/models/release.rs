use std::cmp::max;
use std::fmt::Display;
use lofty::file::TaggedFile;
use crate::models::track::Track;
use crate::models::disc::Disc;
use crate::models::albuminfo::AlbumInfo;

#[derive(Debug)]
pub struct Release {
    pub duration_in_millis: u32,
    pub discs: Vec<Disc>,
    pub mofo: Option<String>,
    album_info: AlbumInfo,
}

impl From<Vec<TaggedFile>> for Release {
    fn from(tagged_files: Vec<TaggedFile>) -> Self {
        let tracks: Vec<Track> = tagged_files.iter().map(Track::from).collect();
        let max_disc = tracks.iter().fold(0, |acc, t| max(acc, t.disc));
        let mut discs = Vec::new();
        for i in 1..=max_disc {
            discs.push(Disc {
                duration_in_millis: 0,
                disc: i,
                tracks: Vec::new(),
            });
        }
        for track in tracks {
            discs[track.disc as usize - 1].add_track(track);
        }
        let album_info = AlbumInfo::from(tagged_files);

        Release {
            duration_in_millis: discs.iter().fold(0, |acc, d| acc + d.duration_in_millis),
            discs,
            album_info,
            mofo: None,
        }
    }
}

fn show_millis(millis: u32) -> String {
    let hours = millis / 3_600_000;
    let minutes = millis / 60000 - hours * 60;
    let seconds = millis / 1000 - hours * 3600 - minutes * 60;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

fn show_artists(artists: &[String]) -> String {
    let result = "[artist]".to_string() + &artists.join("[/artist] & [artist]") + "[/artist]";
    if artists.len() > 2 {
        return result.replacen(" & ", ", ", artists.len() - 2).to_string();
    }
    result
}

impl Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        let album_info = &self.album_info;
        let release_date_type = if self.album_info.release_date.len() > 4 { "Date" } else { "Year" };

        output += &format!("[size=5][b]{} — {}[/b][/size]\n", album_info.artist, album_info.title);
        output += &format!("[b]Release {release_date_type}:[/b] {}\n", album_info.release_date);
        output += &format!("[b]Genre:[/b] {}\n\n", album_info.genres.join(", "));
        output += "[size=4][b]Tracklist[/b][/size]\n";
        for disc in &self.discs {
            if self.discs.len() > 1 {
                output += &format!("[size=3][b]Disc {:?}[/b][/size] [i]({})[/i]\n", disc.disc, show_millis(disc.duration_in_millis));
            }
            for track in &disc.tracks {
                output += &format!("[b]{:?}[/b]. {} — {} [i]({})[/i]\n", track.track, show_artists(&track.artists), track.title, show_millis(track.duration_in_millis));
            }
            output += "\n";
        }
        output += &format!("[b]Total length:[/b] {}\n\n\
        [b]More information:[/b] {}", show_millis(self.duration_in_millis), self.mofo.clone().unwrap_or("N/A".to_string()));

        write!(f, "{output}")
    }
}