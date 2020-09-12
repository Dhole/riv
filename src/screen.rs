//! Screen contains the Screen struct which contains all SDL initialised data required
//! for building the window and rendering to screen.
use sdl2::image::LoadTexture;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::{FullscreenType, WindowContext};
use sdl2::Sdl;
use std::collections::HashMap;
use std::path::PathBuf;
use FullscreenType::*;

/// TextureCache caches past and future textures.
pub struct TextureCache<'a> {
    /// last_texture is the last image texture rendered
    pub last_texture: Option<sdl2::render::Texture<'a>>,
    /// cache TODO
    pub cache: HashMap<usize, sdl2::render::Texture<'a>>,
}

impl<'a> TextureCache<'a> {
    /// new TODO
    pub fn new() -> Self {
        Self {
            last_texture: None,
            cache: HashMap::new(),
        }
    }
    /// load TODO
    pub fn load(&mut self, screen: &Screen<'a>, current_imagepath: &PathBuf) -> Result<(), String> {
        match screen.texture_creator.load_texture(current_imagepath) {
            Ok(t) => {
                self.last_texture = Some(t);
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        };
    }
}

/// Screen contains all SDL related data required for running the screen rendering.
pub struct Screen<'a> {
    /// sdl_context is required for running SDL
    pub sdl_context: Sdl,
    /// canvas is where images and text will be rendered
    pub canvas: WindowCanvas,
    /// texture_creator is used for loading images
    pub texture_creator: &'a TextureCreator<WindowContext>,
    /// font is used for printing text
    pub font: Font<'a, 'static>,
    /// mono_font is used for printing mono spaced text
    pub mono_font: Font<'a, 'static>,
    /// last_index is the index of the last texture rendered
    pub last_index: Option<usize>,
    /// cache keeps past and future tectures.
    pub cache: TextureCache<'a>,
    /// dirty, if true indicates that last texture must be discarded
    pub dirty: bool,
}

impl Screen<'_> {
    /// Updates window for fullscreen state
    pub fn update_fullscreen(&mut self, fullscreen: bool) -> Result<(), String> {
        let fullscreen_type = if fullscreen { Off } else { True };
        if self.canvas.window().fullscreen_state() == fullscreen_type {
            return Ok(());
        }

        if let Err(e) = self.canvas.window_mut().set_fullscreen(fullscreen_type) {
            return Err(format!("failed to update display: {:?}", e).to_string());
        }
        match fullscreen_type {
            Off => {
                let window = self.canvas.window_mut();
                window.set_fullscreen(Off).unwrap();
                window.set_bordered(true);
            }
            Desktop | True => {
                let window = self.canvas.window_mut();
                window.set_bordered(false);
                window.set_fullscreen(True).unwrap();
            }
        };
        Ok(())
    }

    /// load TODO
    pub fn load_texture(
        &mut self,
        current_imagepath: &PathBuf,
        index: Option<usize>,
    ) -> Result<(), String> {
        match self.cache.last_texture.take() {
            Some(t) => {
                self.cache.cache.insert(self.last_index.unwrap(), t);
            }
            None => {}
        };
        let i = index.unwrap();
        match self.cache.cache.remove(&i) {
            Some(t) => {
                self.cache.last_texture = Some(t);
                self.last_index = index;
                return Ok(());
            }
            None => {}
        }
        match self.texture_creator.load_texture(current_imagepath) {
            Ok(t) => {
                self.cache.last_texture = Some(t);
                self.last_index = index;
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        };
    }
}
