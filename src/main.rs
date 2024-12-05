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

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let path = match args.path {
        Some(folder_name) => PathBuf::from(folder_name),
        None => std::env::current_dir().unwrap()
    };
    let path = path.read_dir()?;
    let tagged_files = flac_files_from_dir(path, 1)?;
    let sorted_files = sort_tagged_files(tagged_files);
    let release = Release::from(sorted_files);

    println!("{release}\n\nPress ENTER to copy to clipboard");
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
    let mut clipboard = clippers::Clipboard::get();
    if clipboard.write_text(release.to_string()).is_err() {
        println!("Failed to copy to clipboard");
        let _ = io::stdin().read_line(&mut buf);
    }
    Ok(())
}
