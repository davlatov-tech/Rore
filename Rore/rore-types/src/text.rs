use crate::Color;
use glam::Vec2;

// MANA SHU LAQAB BARCHA XATOLARNI YO'Q QILADI:
pub type SparseTextItem = (u32, String, Color, f32, Vec2, Option<[f32; 4]>, f32);

pub trait TextMeasurer: Send + Sync {
    fn measure(&mut self, text: &str, font_size: f32, max_width: Option<f32>) -> (f32, f32);
}

pub trait TextRenderer {
    fn resize(&mut self, width: u32, height: u32);

    // Uzun tiplar o'rniga faqat laqabni ishlatamiz
    fn update_sparse(&mut self, sparse_texts: &[SparseTextItem]);

    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
    fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
}
