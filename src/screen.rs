//! Screen contains the Screen struct which contains all SDL initialised data required
//! for building the window and rendering to screen.
use sdl2::image::{LoadSurface, LoadTexture};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{TextureAccess, TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::video::{FullscreenType, WindowContext};
use sdl2::Sdl;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use FullscreenType::*;

use std::time::Instant;

/// TextureCache caches past and future textures.
pub struct TextureCache<'a> {
    /// last_texture is the last image texture rendered
    pub last_texture: Option<sdl2::render::Texture<'a>>,
    /// cache TODO
    // pub cache: HashMap<usize, sdl2::render::Texture<'a>>,
    pub cache: HashMap<usize, Surface<'a>>,
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
        //match self.cache.last_texture.take() {
        //    Some(t) => {
        //        self.cache.cache.insert(self.last_index.unwrap(), t);
        //    }
        //    None => {}
        //};

        let i = index.unwrap();
        let surface = match self.cache.cache.get(&i) {
            Some(s) => {
                s
                // self.cache.last_texture = Some(t);
                // self.last_index = index;
                // return Ok(());
            }
            None => {
                let start = Instant::now();
                let s = match Surface::from_file(current_imagepath) {
                    Ok(s) => s,
                    Err(e) => return Err(e),
                };
                println!("Surface::from_file(): {} ms", start.elapsed().as_millis());
                let start = Instant::now();
                let s = s.convert_format(PixelFormatEnum::RGB888).unwrap();
                println!(
                    "Surface.convert_format(): {} ms",
                    start.elapsed().as_millis()
                );

                self.cache.cache.insert(i, s);
                self.cache.cache.get(&i).unwrap()
            }
        };
        let (width, height) = surface.size();
        let pitch = surface.pitch();
        let pixels = surface.without_lock().unwrap();
        let mut v = Vec::new();
        for _ in 0..8 {
            let start = Instant::now();
            let mut texture = self
                .texture_creator
                // .create_texture(None, TextureAccess::Static, width, height)
                .create_texture(
                    Some(PixelFormatEnum::RGB888),
                    TextureAccess::Streaming,
                    width,
                    height,
                )
                .unwrap();
            // println!(
            //     "\n> TextureCreator.create_texture(): {} ms",
            //     start.elapsed().as_millis()
            // );
            // println!("texture query {:?}", texture.query());
            // println!("{} x {}", width, height);
            // let start = Instant::now();
            // println!("empty: {} ms", start.elapsed().as_millis());

            let start = Instant::now();
            texture
                .update(Rect::new(0, 0, 0, 0), pixels, 1 as usize)
                .unwrap();
            // texture.with_lock(Rect::new(0, 0, 0, 0), |_, _| {}).unwrap();
            println!(
                "> Texture.update(): >>>>>> {} ms <<<<<<<",
                start.elapsed().as_millis()
            );
            v.push(texture);
        }
        let mut texture = v.pop().unwrap();
        let pixels_len = width * height;
        const TRANS_LEN: u32 = 1024 * 1024;
        let chunks = std::cmp::max(1, pixels_len / TRANS_LEN);
        let lines = height / chunks;
        for i in 0..chunks {
            let start = Instant::now();
            let rect = Rect::new(0, (lines * i) as i32, width, lines);
            let pixels = &pixels[((lines * i) * width * 4) as usize
                ..std::cmp::min(pixels.len() - 1, ((lines * (i + 1)) * width * 4) as usize)];
            match texture.update(rect, pixels, pitch as usize) {
                Ok(_) => {
                    println!(
                        "Texture.update(): {} ms (rect: {:?}, len: {})",
                        start.elapsed().as_millis(),
                        rect,
                        pixels.len()
                    );
                }
                Err(e) => {
                    return Err(format!("{}", e));
                }
            }
        }
        // println!("");
        // let mut texture = self
        //     .texture_creator
        //     // .create_texture(None, TextureAccess::Static, width, height)
        //     .create_texture(
        //         Some(PixelFormatEnum::RGB888),
        //         TextureAccess::Static,
        //         width,
        //         height,
        //     )
        //     .unwrap();
        // for i in 0..chunks {
        //     let start = Instant::now();
        //     let rect = Rect::new(0, (lines * i) as i32, width, lines);
        //     let pixels = &pixels[((lines * i) * width * 4) as usize
        //         ..std::cmp::min(pixels.len() - 1, ((lines * (i + 1)) * width * 4) as usize)];
        //     match texture.update(rect, pixels, pitch as usize) {
        //         Ok(_) => {
        //             println!(
        //                 "Texture.update(): {} ms (rect: {:?}, len: {})",
        //                 start.elapsed().as_millis(),
        //                 rect,
        //                 pixels.len()
        //             );
        //         }
        //         Err(e) => {
        //             return Err(format!("{}", e));
        //         }
        //     }
        // }
        self.cache.last_texture = Some(texture);
        self.last_index = index;
        return Ok(());
    }
}
