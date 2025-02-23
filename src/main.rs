#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::struct_field_names)]

use std::fs::{File, ReadDir};
use std::io;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use clap::Parser;
use lofty::tag::{Accessor};
use lofty::file::TaggedFileExt;
use lofty::file::TaggedFile;
use lofty::read_from;
use lofty::tag::{Tag, TagType};
use models::release::Release;
use copypasta::{ClipboardContext, ClipboardProvider};
use regex::Regex;

mod models;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: Option<String>,
}

pub trait HasVorbisTag {
    fn to_vorbis_tag(&self) -> Result<Tag, Error>;
}

impl HasVorbisTag for TaggedFile {
    fn to_vorbis_tag(&self) -> Result<Tag, Error> {
        match self.tag(TagType::VorbisComments) {
            Some(t) => Ok(t.clone()),
            None => Err(Error::new(ErrorKind::NotFound, "FLAC file does not have VorbisComments tags.")),
        }
    }
}

fn flac_files_from_dir(read_dir: ReadDir, max_depth: u8) -> Result<Vec<TaggedFile>, Error> {
    let mut flac_files = Vec::<TaggedFile>::new();
    for file_res in read_dir {
        let file = file_res?;
        let file_type = file.file_type()?;
        let file_path = file.path();
        if file_type.is_dir() {
            if max_depth == 0 { continue; }
            let folder_read = file_path.read_dir()?;
            let mut folder_flacs = flac_files_from_dir(folder_read, max_depth - 1)?;
            flac_files.append(&mut folder_flacs);
        } else if file_type.is_file() {
            let Some(extension) = file_path.extension() else { continue; };
            if extension.to_ascii_lowercase() != "flac" { continue; }
            let Ok(tagged_file) = read_from(&mut File::open(file_path)?) else {
                return Err(Error::new(ErrorKind::Other, "Failed to open file."));
            };
            flac_files.push(tagged_file);
        }
    }

    Ok(flac_files)
}

fn sort_tagged_files(mut tagged_files: Vec<TaggedFile>) -> Vec<TaggedFile> {
    tagged_files.sort_unstable_by_key(|file|
        if let Ok(tag) = file.to_vorbis_tag() {
            tag.track().unwrap_or(0)
        } else { 0 }
    );
    tagged_files.sort_by_key(|file|
        if let Ok(tag) = file.to_vorbis_tag() {
            tag.disk().unwrap_or(0)
        } else { 0 }
    );
    tagged_files
}

fn wait_for_confirmation() {
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let path = match args.path {
        Some(folder_name) => PathBuf::from(folder_name),
        None => std::env::current_dir().unwrap()
    };
    let mut clipboard = ClipboardContext::new().ok();
    if clipboard.is_none() {
        println!("Warning: Failed to create clipboard context. Will not be able to write to clipboard.");
    }
    
    let path = path.read_dir()?;
    let tagged_files = flac_files_from_dir(path, 1)?;
    let sorted_files = sort_tagged_files(tagged_files);
    let mut release = Release::from(sorted_files);

    let apm_regex = Regex::new(r"(?:https?://)?(?:beta.)?music.apple.com/(\w{2})/([^/]+)/(?:[^/]+/)?((?:\d+)|(?:ra.\d+)|(?:pl.\w+))").unwrap();
    if let Some(c) = &mut clipboard {
        c.get_contents().ok().and_then(|content| {
            apm_regex.captures(&content).map(|captures| {
                release.mofo = Some(captures[0].to_string());
            })
        });
    }

    println!("{release}\n\nPress ENTER to copy to clipboard");
    wait_for_confirmation();
    if let Some(mut c) = clipboard {
        if c.set_contents(release.to_string()).is_err() {
            println!("Failed to copy to clipboard");
            wait_for_confirmation();
        }
    }
    Ok(())
}

