use crate::widgets::Widget;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Click(String),              // ID li tugma bosildi
    Input(String, String),      // ID li inputga yozildi
    Tick(f32),                  // <--- YANGI: Vaqt o'tishi (dt)
    Init,                       // Ilova ishga tushdi
}

pub trait App {
    fn view(&self) -> Box<dyn Widget>;
    fn update(&mut self, event: AppEvent);
}