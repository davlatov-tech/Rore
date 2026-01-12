// rore-types/src/lib.rs

pub mod base;
pub mod ui;
pub mod events;

// Qulaylik uchun barchasini asosiy namespace'ga chiqaramiz
pub use base::*;
pub use ui::*;
pub use events::*;

// ==================== CONFIGURATION (YANGI) ====================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlatformMode {
    Auto,    // Tizim o'zi aniqlaydi (OS API orqali)
    Desktop, // Sichqoncha + Fizik Klaviatura
    Mobile,  // Sensor + Virtual Klaviatura
    Web,     // Browser cheklovlari
}

#[derive(Debug, Clone, Copy)]
pub struct RoreConfig {
    pub mode: PlatformMode,
    
    // Aniq imkoniyatlar (Feature Flags)
    pub touch_support: bool,      // Sensor hodisalarini tinglash
    pub mouse_support: bool,      // Sichqoncha va Hover effektlari
    pub virtual_keyboard: bool,   // Input bosilganda ekran klaviaturasi
    pub text_selection: bool,     // Matnni belgilash (Selection)
    pub animations: bool,         // Global animatsiya o'chirgich
    pub scaling: f32,             // Majburiy masshtab (Zoom)
}

impl Default for RoreConfig {
    fn default() -> Self {
        Self::desktop() // Standart holatda Desktop
    }
}

impl RoreConfig {
    // 1. DESKTOP PRESET (Kompyuterlar uchun)
    pub fn desktop() -> Self {
        Self {
            mode: PlatformMode::Desktop,
            touch_support: false, 
            mouse_support: true,
            virtual_keyboard: false,
            text_selection: true, // Drag orqali
            animations: true,
            scaling: 1.0,
        }
    }

    // 2. MOBILE PRESET (Telefonlar uchun)
    pub fn mobile() -> Self {
        Self {
            mode: PlatformMode::Mobile,
            touch_support: true,
            mouse_support: false, // Kursor yo'q
            virtual_keyboard: true,
            text_selection: true, // Long press orqali
            animations: true,
            scaling: 1.0,
        }
    }

    // 3. LOW POWER (Eski qurilmalar uchun)
    pub fn low_power() -> Self {
        let mut cfg = Self::mobile();
        cfg.animations = false;
        cfg
    }

    // BUILDER PATTERN (Moslashtirish uchun)
    pub fn with_touch(mut self, enabled: bool) -> Self {
        self.touch_support = enabled;
        self
    }
    
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scaling = scale;
        self
    }

    pub fn disable_animations(mut self) -> Self {
        self.animations = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_structure() {
        let _color = Color::hex("#aabbcc");
        let _config = RoreConfig::desktop();
        println!("Rore-Types muvaffaqiyatli ishlayapti!");
    }
}