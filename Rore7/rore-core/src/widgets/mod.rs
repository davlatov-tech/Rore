// rore-core/src/widgets/mod.rs

pub mod base;
pub mod text;
pub mod view;
pub mod button;
pub mod input;

// Hammasini bitta joydan eksport qilamiz
pub use base::{Widget, RenderOutput, BuildContext};
pub use text::Text;
pub use view::View;
pub use button::Button;
pub use input::TextInput;