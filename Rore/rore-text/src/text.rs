use glam::Vec2;
use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextRenderer as GlyphonRenderer,
};
use rore_types::text::{TextMeasurer, TextRenderer};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};
use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration};

pub static GLOBAL_MEASURER: OnceLock<Arc<Mutex<LayoutMeasurer>>> = OnceLock::new();

pub fn get_measurer() -> Arc<Mutex<LayoutMeasurer>> {
    GLOBAL_MEASURER
        .get_or_init(|| Arc::new(Mutex::new(LayoutMeasurer::new())))
        .clone()
}

pub struct LayoutMeasurer {
    pub font_sys: FontSystem,
    pub scratch_buffer: Option<Buffer>,
    pub measure_cache: HashMap<(u64, u32, u32), (f32, f32)>,
}

impl LayoutMeasurer {
    pub fn new() -> Self {
        Self {
            font_sys: FontSystem::new(),
            scratch_buffer: None,
            measure_cache: HashMap::new(),
        }
    }

    fn hash_text(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_cursor_pos(
        &mut self,
        text: &str,
        font_size: f32,
        max_width: Option<f32>,
        cursor_byte_idx: usize,
    ) -> (f32, f32, f32) {
        let line_height = font_size * 1.2;
        if self.scratch_buffer.is_none() {
            self.scratch_buffer = Some(Buffer::new(
                &mut self.font_sys,
                Metrics::new(font_size, line_height),
            ));
        }

        let buffer = self.scratch_buffer.as_mut().unwrap();
        buffer.set_metrics(&mut self.font_sys, Metrics::new(font_size, line_height));
        buffer.set_size(
            &mut self.font_sys,
            max_width.unwrap_or(f32::INFINITY),
            f32::INFINITY,
        );
        buffer.set_text(
            &mut self.font_sys,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_sys);

        let mut caret_x = 0.0;
        let mut caret_y = 0.0;

        for (i, run) in buffer.layout_runs().enumerate() {
            caret_y = i as f32 * line_height;

            if run.glyphs.is_empty() {
                continue;
            }

            let first = run.glyphs.first().unwrap();
            let last = run.glyphs.last().unwrap();

            if cursor_byte_idx > last.start {
                caret_x = last.physical((0.0, 0.0), 1.0).x as f32 + last.w;
                continue;
            }

            match run
                .glyphs
                .binary_search_by_key(&cursor_byte_idx, |g| g.start)
            {
                Ok(idx) => {
                    caret_x = run.glyphs[idx].physical((0.0, 0.0), 1.0).x as f32;
                    return (caret_x, caret_y, line_height);
                }
                Err(idx) => {
                    if idx > 0 {
                        let prev = &run.glyphs[idx - 1];
                        caret_x = prev.physical((0.0, 0.0), 1.0).x as f32 + prev.w;
                        return (caret_x, caret_y, line_height);
                    } else {
                        caret_x = first.physical((0.0, 0.0), 1.0).x as f32;
                        return (caret_x, caret_y, line_height);
                    }
                }
            }
        }

        if cursor_byte_idx == text.len() && text.ends_with('\n') {
            let runs_count = buffer.layout_runs().count();
            caret_y = runs_count as f32 * line_height;
            caret_x = 0.0;
        }

        (caret_x, caret_y, line_height)
    }
}

impl TextMeasurer for LayoutMeasurer {
    fn measure(&mut self, text: &str, font_size: f32, max_width: Option<f32>) -> (f32, f32) {
        let w_bits = max_width.unwrap_or(f32::INFINITY).to_bits();
        let s_bits = font_size.to_bits();
        let key = (LayoutMeasurer::hash_text(text), s_bits, w_bits);

        if let Some(&dim) = self.measure_cache.get(&key) {
            return dim;
        }

        let line_height = font_size * 1.2;
        if self.scratch_buffer.is_none() {
            self.scratch_buffer = Some(Buffer::new(
                &mut self.font_sys,
                Metrics::new(font_size, line_height),
            ));
        }

        let buffer = self.scratch_buffer.as_mut().unwrap();
        buffer.set_metrics(&mut self.font_sys, Metrics::new(font_size, line_height));
        buffer.set_size(
            &mut self.font_sys,
            max_width.unwrap_or(f32::INFINITY),
            f32::INFINITY,
        );
        buffer.set_text(
            &mut self.font_sys,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_sys);

        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;

        for (i, run) in buffer.layout_runs().enumerate() {
            w = w.max(run.line_w);
            h = (i as f32 + 1.0) * line_height;
        }
        if h == 0.0 && !text.is_empty() {
            h = line_height;
        }

        let dim = (w.ceil(), h.ceil());
        self.measure_cache.insert(key, dim);
        dim
    }
}

pub struct CachedBuffer {
    pub buffer: Buffer,
    pub text: String,
    pub size_bits: u32,
    pub width_bits: u32,
    pub last_frame: u64,
}

struct TextDrawCall {
    node_id: u32,
    pos: Vec2,
    clip: Option<[f32; 4]>,
    scale: f32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub struct TextSystem {
    pub font_sys: FontSystem,
    pub swash_cache: SwashCache,
    pub buffers: HashMap<u32, CachedBuffer>,
    pub current_frame: u64,
    pub viewport: Resolution,
    pub atlas: TextAtlas,
    pub text_renderer: GlyphonRenderer,
    current_draw_calls: HashMap<u32, TextDrawCall>,
    trim_timer: u64,
}

impl TextSystem {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        let mut atlas = TextAtlas::new(device, queue, config.format);

        // INQILOB 2: Glyphon uchun Stencil Qafasi!
        let ui_stencil = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal, // Faqat teshik bo'lsa chizadi!
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                read_mask: 0xff,
                write_mask: 0x00,
            },
            bias: wgpu::DepthBiasState::default(),
        };

        let text_renderer =
            GlyphonRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        Self {
            font_sys: FontSystem::new(),
            swash_cache: SwashCache::new(),
            buffers: HashMap::new(),
            current_frame: 0,
            viewport: Resolution {
                width: config.width,
                height: config.height,
            },
            atlas,
            text_renderer,
            current_draw_calls: HashMap::new(),
            trim_timer: 0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport = Resolution { width, height };
    }

    pub fn gc(&mut self) {
        if self.buffers.len() > 200 {
            let mut entries: Vec<_> = self
                .buffers
                .iter()
                .map(|(k, v)| (*k, v.last_frame))
                .collect();
            entries.sort_by_key(|x| x.1);
            let to_remove = self.buffers.len() - 150;
            for i in 0..to_remove {
                self.buffers.remove(&entries[i].0);
            }
        }
    }

    fn get_or_update_buffer(
        &mut self,
        node_id: u32,
        text: &str,
        font_size: f32,
        max_width: Option<f32>,
    ) -> &mut Buffer {
        let width_val = max_width.unwrap_or(f32::INFINITY);
        let size_bits = font_size.to_bits();
        let width_bits = width_val.to_bits();
        let mut needs_shaping = false;

        if let Some(cached) = self.buffers.get_mut(&node_id) {
            cached.last_frame = self.current_frame;
            if cached.text != text
                || cached.size_bits != size_bits
                || cached.width_bits != width_bits
            {
                cached.text = text.to_string();
                cached.size_bits = size_bits;
                cached.width_bits = width_bits;

                cached
                    .buffer
                    .set_metrics(&mut self.font_sys, Metrics::new(font_size, font_size * 1.2));
                cached
                    .buffer
                    .set_size(&mut self.font_sys, width_val, f32::INFINITY);
                cached.buffer.set_text(
                    &mut self.font_sys,
                    text,
                    Attrs::new().family(Family::SansSerif),
                    Shaping::Advanced,
                );
                needs_shaping = true;
            }
        } else {
            let mut buffer =
                Buffer::new(&mut self.font_sys, Metrics::new(font_size, font_size * 1.2));
            buffer.set_size(&mut self.font_sys, width_val, f32::INFINITY);
            buffer.set_text(
                &mut self.font_sys,
                text,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );

            self.buffers.insert(
                node_id,
                CachedBuffer {
                    buffer,
                    text: text.to_string(),
                    size_bits,
                    width_bits,
                    last_frame: self.current_frame,
                },
            );
            needs_shaping = true;
        }

        let cached = self.buffers.get_mut(&node_id).unwrap();
        if needs_shaping {
            cached.buffer.shape_until_scroll(&mut self.font_sys);
        }
        &mut cached.buffer
    }

    pub fn update_texts_sparse(&mut self, sparse_texts: &[rore_types::text::SparseTextItem]) {
        self.current_frame += 1;

        for (node_id, text, color, size, pos, clip, width_limit) in sparse_texts {
            if text.is_empty() {
                self.current_draw_calls.remove(node_id);
                self.buffers.remove(node_id);
                continue;
            }

            let r = (color.r * 255.0) as u8;
            let g = (color.g * 255.0) as u8;
            let b = (color.b * 255.0) as u8;
            let a = (color.a * 255.0) as u8;

            let (render_size, render_scale) = if *size > 48.0 {
                (48.0, *size / 48.0)
            } else {
                (*size, 1.0)
            };
            let w = if *width_limit > 0.0 && *width_limit != f32::INFINITY {
                *width_limit / render_scale
            } else {
                f32::INFINITY
            };

            self.get_or_update_buffer(*node_id, text, render_size, Some(w));

            self.current_draw_calls.insert(
                *node_id,
                TextDrawCall {
                    node_id: *node_id,
                    pos: *pos,
                    clip: *clip,
                    scale: render_scale,
                    r,
                    g,
                    b,
                    a,
                },
            );
        }
        self.gc();
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {
        let mut text_areas = Vec::new();

        for call in self.current_draw_calls.values() {
            if let Some(cached) = self.buffers.get(&call.node_id) {
                let bounds = if let Some(rect) = call.clip {
                    glyphon::TextBounds {
                        left: rect[0] as i32,
                        top: rect[1] as i32,
                        right: (rect[0] + rect[2]) as i32,
                        bottom: (rect[1] + rect[3]) as i32,
                    }
                } else {
                    glyphon::TextBounds {
                        left: 0,
                        top: 0,
                        right: self.viewport.width as i32,
                        bottom: self.viewport.height as i32,
                    }
                };

                text_areas.push(TextArea {
                    buffer: &cached.buffer,
                    left: call.pos.x,
                    top: call.pos.y,
                    scale: call.scale,
                    bounds,
                    default_color: Color::rgba(call.r, call.g, call.b, call.a),
                });
            }
        }

        let _ = self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_sys,
            &mut self.atlas,
            self.viewport,
            text_areas.into_iter(),
            &mut self.swash_cache,
        );

        self.trim_timer += 1;
        if self.trim_timer >= 1800 {
            self.atlas.trim();
            self.trim_timer = 0;
        }
    }

    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>) {
        let _ = self.text_renderer.render(&self.atlas, pass);
    }
}

impl TextRenderer for TextSystem {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize(width, height);
    }
    #[rustfmt::skip]
    fn update_sparse(&mut self, sparse_texts: &[rore_types::text::SparseTextItem]) { self.update_texts_sparse(sparse_texts); }
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.prepare(device, queue);
    }
    fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        self.render(pass);
    }
}
