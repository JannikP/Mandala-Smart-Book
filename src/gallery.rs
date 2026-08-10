use anyhow::{Context, Result, anyhow};
use iced::widget::image::Handle;
use rand::rng;
use rand::seq::SliceRandom;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gallery {
    images: Vec<PathBuf>,
    current: usize,
}

impl Gallery {
    pub fn new() -> Result<Self> {
        let gallery_folder = Path::new("/home/jannik/Pictures");
        let mut images: Vec<PathBuf> = read_dir(gallery_folder)
            .context("Failed to read gallery folder content.")?
            .filter_map(Result::ok)
            .map(|p| p.path())
            //.filter(is_image_file)
            .collect();
        if images.is_empty() {
            return Err(anyhow!("No images found in {}!", gallery_folder.display()));
        }
        images.shuffle(&mut rng());
        Ok(Self { images, current: 0 })
    }

    pub fn image(&self) -> impl Into<Handle> {
        Handle::from_path(self.images[self.current].as_path())
    }

    pub fn next(&mut self) {
        self.current += 1;
        if self.current >= self.images.len() {
            self.current = 0;
            self.images.shuffle(&mut rng());
        }
    }
}

fn is_image_file(p: &PathBuf) -> bool {
    p.is_file() && (p.ends_with(".jpg") || p.ends_with(".JPG"))
}
