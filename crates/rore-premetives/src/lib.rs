pub mod geometry;
pub mod color;
pub mod layout;
pub mod style;
pub mod border;
pub mod background;
pub mod typography;
pub mod events;
pub mod animation;
pub mod input;
pub mod accessibility;


pub use geometry::{Point, Size, Rect, Transform};
pub use color::{Color, LinearGradient, GradientStop};
pub use layout::{
    Val, Thickness, CornerRadius, Display, FlexDirection, Align, Position, 
    Overflow, GridLength, GridTemplate, BoxSizing, Direction, ZIndex
};
pub use style::{Shadow, CursorIcon, Visibility, Filter};
pub use border::{Border, BorderStyle, BorderSide};
pub use background::{Background, ImageFit};
pub use typography::{TextStyle, TextAlign, FontWeight, TextDecoration, TextTransform};
pub use events::{Event, MouseButton, KeyState};
pub use animation::{Transition, Easing};
pub use input::{InputType, InputState};
pub use accessibility::Role;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_framework_capabilities() {
    
        let avatar_bg = Background::Solid(Color::hex("#667eea"));
        let avatar_radius = CornerRadius::all(50.0); 
        let status_color = Color::hex("#4caf50"); 

  
        let grid_layout = GridTemplate {
            columns: vec![GridLength::Fr(1.0), GridLength::Fr(1.0)],
            rows: vec![GridLength::Auto, GridLength::Px(200.0)],
        };
        let overflow_mode = Overflow::Scroll;
        let box_sizing = BoxSizing::BorderBox; // Yangi qo'shilgan
        let z_index = ZIndex(100); // Yangi qo'shilgan

        let hover_transition = Transition {
            property: "all".to_string(),
            duration: 300,
            easing: Easing::EaseInOut,
            delay: 0,
        };

 
        let text_style = TextStyle {
            decoration: TextDecoration::Underline,
            transform: TextTransform::Uppercase,
            ..Default::default()
        };

        let password_input = InputType::Password;
        let input_state = InputState { required: true, ..Default::default() };

 
        let btn_role = Role::Button;

     
        
        assert_eq!(avatar_bg, Background::Solid(Color { r: 0.4, g: 0.49411765, b: 0.91764706, a: 1.0 }));
        assert!(status_color.g > 0.6); 
        assert_eq!(avatar_radius.top_left, 50.0);

        assert_eq!(grid_layout.columns.len(), 2);
        assert_eq!(overflow_mode, Overflow::Scroll);
        assert_eq!(box_sizing, BoxSizing::BorderBox);
        assert_eq!(z_index.0, 100);
        
        assert_eq!(hover_transition.duration, 300);
        
        assert_eq!(text_style.decoration, TextDecoration::Underline);
        assert_eq!(text_style.transform, TextTransform::Uppercase);

        assert_eq!(password_input, InputType::Password);
        assert!(input_state.required);
        
        assert_eq!(btn_role, Role::Button);

        println!(" MUKAMMAL ishladi!");
    }
}