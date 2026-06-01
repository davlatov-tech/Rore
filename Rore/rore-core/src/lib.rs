pub mod app;
pub mod reactive;
pub mod state;
pub mod time;
pub mod widgets;
// Barcha kerakli narsalarni freymvorkdan tashqariga eksport qilamiz
pub use crate::app::{run, App, AppEvent};
pub use crate::widgets::base::Widget;
pub mod calculs;
