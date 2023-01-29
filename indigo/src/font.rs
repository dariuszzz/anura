use std::{path::{PathBuf, Path}, fs};

use ahash::AHashMap;
use fontdue::Metrics;
use indigo_wgpu::wgpu::util::RenderEncoder;
use ordered_float::NotNan;

use crate::graphics::IndigoRenderer;


#[derive(Default)]
pub enum Font {
    #[default]
    Default,
    System(String, f32),
    Path(PathBuf, f32)
}


pub struct GlyphData {
    pub uv: (f32, f32, f32, f32),
    pub metrics: Metrics
}

pub struct FontAtlas<R: IndigoRenderer> {
    pub size: f32,
    pub glyph_data: AHashMap<char, GlyphData>,
    pub texture_handle: R::TextureHandle 
}

impl<R: IndigoRenderer> FontAtlas<R> {
    pub fn new(renderer: &mut R, data: &[u8], size: f32) -> Self {
        
        let mut glyph_data = AHashMap::new();

        let settings = fontdue::FontSettings {
            scale: size,
            ..fontdue::FontSettings::default()
        };

        let font = fontdue::Font::from_bytes(data, settings).unwrap();
        
        #[derive(Clone, Debug)]
        struct Space {
            x: u32,
            y: u32,
            w: u32,
            h: u32,
        }

        #[derive(Clone, Debug)]
        struct Quad {
            x: u32,
            y: u32,
            w: u32,
            h: u32,
            bitmap: Vec<u8>,
            glyph: char,
            metrics: Metrics,
        }

        let mut quads = Vec::new();
        let mut total_area = 0;
        let mut max_width = 0;
        let mut max_quad_size = 0;

        let glyphs = font.chars();
        for glyph in glyphs.keys() {
            let (metrics, bitmap) = font.rasterize_subpixel(*glyph, size);
            total_area += metrics.width * metrics.height;
            max_width = max_width.max(metrics.width);

            let mut rgba_bitmap = Vec::new();
            for (i, _) in bitmap.iter().enumerate().step_by(3) {
                rgba_bitmap.push(bitmap[i]);
                rgba_bitmap.push(bitmap[i+1]);
                rgba_bitmap.push(bitmap[i+2]);
                
                let sum = bitmap[i] as u32 + bitmap[i+1] as u32 + bitmap[i+2] as u32;
                
                rgba_bitmap.push((sum/6) as u8);    
            }

            let quad_size = {
                let width_rounded = ((metrics.width as u32) / 16) * 16 + 16;
                let height_rounded = ((metrics.height as u32) / 16) * 16 + 16;
                width_rounded.max(height_rounded)
            };

            max_quad_size = max_quad_size.max(quad_size);

            quads.push(Quad {
                x: 0,
                y: 0,
                w: 0, 
                h: 0,
                bitmap: rgba_bitmap,
                glyph: *glyph,
                metrics
            });
        }

        quads.iter_mut().for_each(|q| {
            q.w = max_quad_size;
            q.h = max_quad_size;
        } );

        quads.sort_by(|a, b| b.y.cmp(&a.x));

        // const startWidth = Math.max(Math.ceil(Math.sqrt(area / 0.95)), maxWidth);
        let start_width = (total_area as f32 / 0.95).sqrt().ceil().max(max_width as f32) as u32;
        println!("{start_width}");

        let mut spaces = vec![Space { x: 0, y: 0, w: start_width, h: 8192 }];
        let mut packed = Vec::new();

        let mut final_width = 0;
        let mut final_height = 0;

        for quad in quads {
            for i in (0..(spaces.len())).rev() {
                let mut space = spaces[i].clone();

                if quad.w > space.w || quad.h > space.h { continue }

                {
                    let mut quad = quad.clone();
                    quad.x = space.x;
                    quad.y = space.y; 
                    
                    final_width = final_width.max(quad.x + quad.w);
                    final_height = final_height.max(quad.y + quad.h);
                    packed.push(quad);
                }

                if quad.w == space.w && quad.h == space.h {
                    if let Some(last) = spaces.pop() {
                        if i < spaces.len() { spaces[i] = last }
                    }
                } else if quad.h == space.h {
                    space.x += quad.w;
                    space.w -= quad.w;
                } else if quad.w == space.w {
                    space.y += quad.h;
                    space.h -= quad.h;
                } else {
                    spaces.push(Space {
                        x: space.x + quad.w,
                        y: space.y,
                        w: space.w - quad.w,
                        h: quad.h
                    });
                    space.y += quad.h;
                    space.h -= quad.h;
                }
                spaces[i] = space;
                break;
            }
        }

        let mut full_atlas_bitmap: Vec<u8> = vec![0; final_width as usize * final_height as usize * 4];

        for quad in &packed {
            let uv = (
                quad.x as f32 / final_width as f32,
                quad.y as f32 / final_height as f32,
                (quad.metrics.width as u32) as f32 / final_width as f32,
                (quad.metrics.height as u32) as f32 / final_height as f32
            );

            glyph_data.insert(quad.glyph, GlyphData {
                uv,
                metrics: quad.metrics
            });

            let subpx_metrics_width = quad.metrics.width as u32 * 4;
            let subpx_final_width = final_width * 4;

            let subpx_x = quad.x * 4;

            for x in 0..subpx_metrics_width {
                for y in 0..quad.metrics.height as u32 {

                    let atlas_index = (y + quad.y) * subpx_final_width + (x + subpx_x);

                    let quad_index = y * subpx_metrics_width + x;

                    full_atlas_bitmap[atlas_index as usize] = quad.bitmap[quad_index as usize];
                }
            } 
        }

        let texture_handle = renderer.new_texture(&full_atlas_bitmap, (final_width, final_height), None);

        Self {
            size,
            glyph_data,
            texture_handle
        }
    }
}

pub struct FontManager<R: IndigoRenderer> {
    pub fonts: Vec<FontAtlas<R>>,
    pub path_map: AHashMap<(PathBuf, NotNan<f32>), usize>,
    pub default_font: Option<usize>,
}

impl<R: IndigoRenderer> FontManager<R> {
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
            path_map: AHashMap::new(),
            default_font: None
        }
    }

    pub fn load_font(&mut self, renderer: &mut R, font: &Font, default: bool) {
        match font {
            Font::Default => return,
            Font::Path(path, size) => {
                let key = (path.to_path_buf(), NotNan::new(*size).unwrap());

                if self.path_map.contains_key(&key) { return }
                
                let bytes = fs::read(path).expect("Font doesnt exist");
                
                let font_atlas = FontAtlas::new(renderer, &bytes, *size);
                self.fonts.push(font_atlas);

                self.path_map.insert(key, self.fonts.len() - 1);
            },
            Font::System(_path, _size) => todo!("implement system fonts"),
        };
        

        if default {
            self.default_font = Some(self.fonts.len() - 1);
        }
    }

    pub fn get_font(&self, font: &Font) -> Option<&FontAtlas<R>> {
        match font {
            Font::Default => self.fonts.get(
                self.default_font.expect("Default font not set")
            ),
            Font::Path(path, size) => {
                let key = (path.to_path_buf(), NotNan::new(*size).unwrap());

                let idx = self.path_map.get(&key).expect("Font not loaded");

                self.fonts.get(*idx)
            },
            Font::System(_path, _size) => todo!("impement system fonts")
        }
    }
}