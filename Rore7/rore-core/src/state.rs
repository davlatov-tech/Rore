use glam::Vec2;
use rore_layout::Node;
use std::collections::HashMap;
use std::time::Instant; 
use rore_types::{Easing, Lerp, RoreConfig};
use arboard::Clipboard;

#[derive(Clone)]
pub struct AnimState<T> {
    pub start_val: T,
    pub end_val: T,
    pub start_time: Instant,
    pub duration: f32,
    pub easing: Easing,
}

impl<T: Lerp + Clone> AnimState<T> {
    pub fn get_value(&self, now: Instant) -> (T, bool) { 
        let elapsed = now.duration_since(self.start_time).as_secs_f32();
        if elapsed >= self.duration {
            (self.end_val.clone(), true) 
        } else {
            let t = elapsed / self.duration;
            let curved_t = self.easing.apply(t);
            (self.start_val.lerp(&self.end_val, curved_t), false) 
        }
    }
}

pub struct FrameworkState {
    pub cursor_pos: Vec2,
    pub hovered_node: Option<Node>,
    pub focused_node: Option<Node>, 
    pub active_node: Option<Node>,  
    pub scroll_offsets: HashMap<String, f32>,
    
    // --- INPUT STATE ---
    pub focused_input_id: Option<String>,
    pub input_cursor_idx: usize,
    pub input_selection: Option<(usize, usize)>, // (start, end)
    pub drag_start_idx: Option<usize>,
    
    // --- YANGI: Tizim resurslari ---
    pub clipboard: Option<std::sync::Mutex<Clipboard>>, // Clipboard
    pub needs_redraw: bool,                             // Optimizatsiya uchun flag
    
    // Animatsiya tizimi
    pub color_animations: HashMap<String, AnimState<[f32; 4]>>,
    pub last_colors: HashMap<String, [f32; 4]>,

    pub config: RoreConfig,
}

impl FrameworkState {
    pub fn new(config: RoreConfig) -> Self {
        // Clipboardni xavfsiz yaratish (xato bersa ham ilova o'chmasligi kerak)
        let clipboard = Clipboard::new().ok().map(|c| std::sync::Mutex::new(c));

        Self {
            cursor_pos: Vec2::ZERO,
            hovered_node: None,
            focused_node: None,
            active_node: None,
            scroll_offsets: HashMap::new(),
            
            focused_input_id: None,
            input_cursor_idx: 0,
            input_selection: None,
            drag_start_idx: None,

            // Yangi maydonlar
            clipboard,
            needs_redraw: true, // Dastur ochilganda bir marta chizish shart

            color_animations: HashMap::new(),
            last_colors: HashMap::new(),
            config,
        }
    }

    // --- YANGI: Optimizatsiya signali ---
    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    // --- YANGI: Selection normalizatsiyasi (Chap/O'ng farqi yo'q) ---
    pub fn get_normalized_selection(&self) -> Option<(usize, usize)> {
        if let Some((start, end)) = self.input_selection {
            if start < end { 
                Some((start, end)) 
            } else { 
                Some((end, start)) // Orqaga tortilganda swap qilish
            }
        } else {
            None
        }
    }

    pub fn update_cursor(&mut self, x: f32, y: f32) {
        self.cursor_pos = Vec2::new(x, y);
        // Kursor qimirlaganda redraw so'ramaymiz (agar hover o'zgarmasa), 
        // buni lib.rs dagi mantiq hal qiladi.
    }

    pub fn handle_scroll(&mut self, delta_y: f32, target_id: &str) {
        let current = *self.scroll_offsets.get(target_id).unwrap_or(&0.0);
        let new_val = current - delta_y; 
        self.scroll_offsets.insert(target_id.to_string(), new_val);
        
        // Scroll bo'lsa ekran o'zgaradi -> Redraw
        self.request_redraw();
    }

    pub fn is_animating(&self) -> bool {
        if !self.config.animations {
            return false;
        }
        !self.color_animations.is_empty()
    }

    pub fn get_animated_color(&mut self, id: &str, target_val: [f32; 4], duration: f32) -> [f32; 4] {
        if !self.config.animations {
            return target_val;
        }

        let now = Instant::now();
        
        if let Some(anim) = self.color_animations.get(id) {
            // Maqsad o'zgardi (masalan hover chiqib ketdi)
            if anim.end_val != target_val {
                let (current, _) = anim.get_value(now);
                self.color_animations.insert(id.to_string(), AnimState {
                    start_val: current,
                    end_val: target_val,
                    start_time: now,
                    duration,
                    easing: Easing::EaseOutCubic,
                });
                self.request_redraw(); // Animatsiya yangilandi
                return current;
            }
            
            let (val, finished) = anim.get_value(now);
            if finished {
                self.color_animations.remove(id);
                self.last_colors.insert(id.to_string(), val);
                self.request_redraw(); // Oxirgi kadrni chizish uchun
            } else {
                self.request_redraw(); // Animatsiya davom etyapti -> Redraw
            }
            return val;
        }

        let last = *self.last_colors.get(id).unwrap_or(&target_val);
        if last != target_val {
            // Yangi animatsiya boshlandi
            self.color_animations.insert(id.to_string(), AnimState {
                start_val: last,
                end_val: target_val,
                start_time: now,
                duration,
                easing: Easing::EaseOutCubic,
            });
            self.request_redraw(); // Boshlash uchun
            return last;
        }

        self.last_colors.insert(id.to_string(), target_val);
        target_val
    }
}