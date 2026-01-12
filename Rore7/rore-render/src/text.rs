use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextRenderer,
};
use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration};
use rore_types::Color as RoreColor;

#[derive(Hash, Eq, PartialEq, Clone)]
struct CacheKey {
    text: String,
    size_bits: u32,
    width_bits: u32, // Kenglik ham kesh kalitining bir qismi
    color_r: u8, color_g: u8, color_b: u8,
}

// --- TextLayout: Matn Geometriyasi ---
#[derive(Debug, Clone)]
pub struct GlyphRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub index: usize, // Bayt indeksi
}

pub struct TextLayout {
    pub glyphs: Vec<GlyphRect>,
    pub total_size: (f32, f32),
    pub line_height: f32,
}

impl TextLayout {
    // Koordinatadan (x, y) matn indeksini topish
    pub fn hit_test(&self, x: f32, y: f32) -> usize {
        if self.glyphs.is_empty() { return 0; }

        let mut closest_idx = 0;
        let mut min_dist = f32::MAX;

        for glyph in &self.glyphs {
            let center_x = glyph.x + glyph.w / 2.0;
            let center_y = glyph.y + glyph.h / 2.0;
            
            let dx = x - center_x;
            let dy = (y - center_y) * 5.0; // Y o'qi bo'yicha og'ishni jazolash
            let dist = dx*dx + dy*dy;

            if dist < min_dist {
                min_dist = dist;
                closest_idx = glyph.index;
                // Agar harfning o'ng tomoniga bosilsa, keyingi indeksga o'tamiz
                if x > center_x {
                    closest_idx += 1; 
                }
            }
        }
        closest_idx
    }

    // Indeksdan kursor koordinatasini olish
    pub fn get_cursor_pos(&self, index: usize) -> Option<(f32, f32, f32)> {
        if self.glyphs.is_empty() {
            return Some((0.0, 0.0, self.line_height));
        }

        // Oxirgi indeks uchun
        if index >= self.glyphs.len() {
            if let Some(last) = self.glyphs.last() {
                return Some((last.x + last.w, last.y, last.h));
            }
        }

        // Mavjud indeks uchun
        for glyph in &self.glyphs {
            if glyph.index == index {
                return Some((glyph.x, glyph.y, glyph.h));
            }
        }
        
        None
    }
    pub fn get_selection_rects(&self, start: usize, end: usize) -> Vec<(f32, f32, f32, f32)> {
        let mut rects = Vec::new();
        if self.glyphs.is_empty() || start >= end { return rects; }

        // Start va End ni to'g'irlash (swap if needed)
        let (s, e) = (start.min(end), start.max(end));

        // Hozirgi qator ma'lumotlari
        let mut current_line_y = -1.0;
        let mut current_rect: Option<(f32, f32, f32, f32)> = None; // x, y, w, h

        for glyph in &self.glyphs {
            // Agar harf tanlangan oraliqda bo'lsa
            if glyph.index >= s && glyph.index < e {
                
                // Yangi qatorga o'tdikmi?
                if glyph.y != current_line_y {
                    // Eski qatorni yopamiz va saqlaymiz
                    if let Some(r) = current_rect {
                        rects.push(r);
                    }
                    // Yangi qator boshlaymiz
                    current_line_y = glyph.y;
                    current_rect = Some((glyph.x, glyph.y, glyph.w, glyph.h));
                } else {
                    // Shu qatorni davom ettiramiz (kengaytiramiz)
                    if let Some(r) = &mut current_rect {
                        r.2 += glyph.w; // Width ortadi
                    }
                }
            }
        }

        // Oxirgi qatorni qo'shish
        if let Some(r) = current_rect {
            rects.push(r);
        }

        rects
    }
}


pub struct FontManager {
    pub font_system: Arc<Mutex<FontSystem>>,
    pub swash_cache: SwashCache,
}

impl FontManager {
    pub fn new() -> Self {
        Self { font_system: Arc::new(Mutex::new(FontSystem::new())), swash_cache: SwashCache::new() }
    }
    
    // Matn o'lchamini hisoblash (Wrapping bilan)
    pub fn measure(&mut self, text: &str, font_size: f32, max_width: Option<f32>) -> (f32, f32) {
        let mut font_system = self.font_system.lock().unwrap();
        let line_height = font_size * 1.2;
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(font_size, line_height));
        
        let width_constraint = max_width.unwrap_or(f32::INFINITY);
        buffer.set_size(&mut font_system, width_constraint, f32::INFINITY);
        
        buffer.set_text(&mut font_system, text, Attrs::new().family(Family::SansSerif), Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system);
        
        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;
        
        for run in buffer.layout_runs() {
            w = w.max(run.line_w);
            h = run.line_y + line_height; 
        }
        
        if h == 0.0 && !text.is_empty() {
            h = line_height;
        }

        (w.ceil(), h.ceil())
    }

    // Matn geometriyasini hisoblash va TextLayout qaytarish
    pub fn layout_text(&mut self, text: &str, font_size: f32, max_width: Option<f32>) -> TextLayout {
        let mut font_system = self.font_system.lock().unwrap();
        let line_height = font_size * 1.2;
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(font_size, line_height));
        
        let width_constraint = max_width.unwrap_or(f32::INFINITY);
        buffer.set_size(&mut font_system, width_constraint, f32::INFINITY);
        
        buffer.set_text(&mut font_system, text, Attrs::new().family(Family::SansSerif), Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system);
        
        let mut glyphs = Vec::new();
        
        // --- FIX: Turlarni aniq ko'rsatamiz (f32) ---
        let mut max_w: f32 = 0.0;
        let mut total_h: f32 = 0.0;

        for run in buffer.layout_runs() {
            total_h = run.line_y + line_height;
            max_w = max_w.max(run.line_w);

            for glyph in run.glyphs {
                let physical_glyph = glyph.physical((0.0, 0.0), 1.0);
                let w = glyph.w;
                let idx = glyph.start; 
                
                glyphs.push(GlyphRect {
                    x: physical_glyph.x as f32,
                    y: run.line_y, 
                    w,
                    h: line_height,
                    index: idx,
                });
            }
        }

        if total_h == 0.0 { total_h = line_height; }

        TextLayout {
            glyphs,
            total_size: (max_w, total_h),
            line_height,
        }
    }
}

pub struct TextSystem {
    pub font_manager: Arc<Mutex<FontManager>>,
    pub viewport: Resolution,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    // (text, color, size, x, y, clip, width_limit)
    pub pending_texts: Vec<(String, RoreColor, f32, f32, f32, Option<[f32; 4]>, f32)>, 
    cache: HashMap<CacheKey, Buffer>,
}

impl TextSystem {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration, font_manager: Arc<Mutex<FontManager>>) -> Self {
        let mut atlas = TextAtlas::new(device, queue, config.format);
        let text_renderer = TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        Self {
            font_manager: font_manager.clone(),
            viewport: Resolution { width: config.width, height: config.height },
            atlas, text_renderer, pending_texts: Vec::new(), cache: HashMap::new(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport = Resolution { width, height };
        self.cache.clear();
    }

    pub fn queue_text(&mut self, text: &str, color: RoreColor, font_size: f32, x: f32, y: f32, clip: Option<[f32; 4]>, width_limit: f32) {
        self.pending_texts.push((text.to_string(), color, font_size, x, y, clip, width_limit));
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {
        let mut manager = self.font_manager.lock().unwrap();
        let font_system_arc = manager.font_system.clone();
        let swash_cache = &mut manager.swash_cache;
        let mut font_system = font_system_arc.lock().unwrap();
        
        for (text, color, size, _, _, _, width_limit) in &self.pending_texts {
            // width_limit ni kesh kalitiga qo'shamiz
            let width_bits = width_limit.to_bits();
            
            let key = CacheKey {
                text: text.clone(), 
                size_bits: size.to_bits(),
                width_bits,
                color_r: (color.r * 255.0) as u8, color_g: (color.g * 255.0) as u8, color_b: (color.b * 255.0) as u8,
            };
            
            if !self.cache.contains_key(&key) {
                let mut buffer = Buffer::new(&mut font_system, Metrics::new(*size, *size * 1.2));
                
                // Wrapping uchun kenglik
                let w = if *width_limit > 0.0 { *width_limit } else { f32::INFINITY };
                buffer.set_size(&mut font_system, w, f32::INFINITY);
                
                let attrs = Attrs::new().family(Family::SansSerif).color(Color::rgb(key.color_r, key.color_g, key.color_b));
                buffer.set_text(&mut font_system, text, attrs, Shaping::Advanced);
                buffer.shape_until_scroll(&mut font_system);
                self.cache.insert(key, buffer);
            }
        }

        let mut text_areas = Vec::new();
        for (text, color, size, x, y, clip, width_limit) in &self.pending_texts {
            let width_bits = width_limit.to_bits();
            let key = CacheKey {
                text: text.clone(), 
                size_bits: size.to_bits(),
                width_bits,
                color_r: (color.r * 255.0) as u8, color_g: (color.g * 255.0) as u8, color_b: (color.b * 255.0) as u8,
            };

            let bounds = if let Some(rect) = clip {
                glyphon::TextBounds {
                    left: rect[0] as i32,
                    top: rect[1] as i32,
                    right: (rect[0] + rect[2]) as i32,
                    bottom: (rect[1] + rect[3]) as i32,
                }
            } else {
                glyphon::TextBounds {
                    left: 0, top: 0,
                    right: self.viewport.width as i32,
                    bottom: self.viewport.height as i32,
                }
            };

            if let Some(buffer) = self.cache.get(&key) {
                text_areas.push(TextArea {
                    buffer,
                    left: *x, top: *y, scale: 1.0,
                    bounds,
                    default_color: Color::rgb(255, 255, 255),
                });
            }
        }

        let _ = self.text_renderer.prepare(device, queue, &mut font_system, &mut self.atlas, self.viewport, text_areas.into_iter(), swash_cache);
        self.pending_texts.clear();
        if self.cache.len() > 1000 { self.cache.clear(); }
    }

    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>) {
        let _ = self.text_renderer.render(&self.atlas, pass);
    }
}