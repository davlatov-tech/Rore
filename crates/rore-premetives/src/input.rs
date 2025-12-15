#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Email,
    Number,
    Checkbox,
    Radio,
    File,
    Date,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct InputState {
    pub disabled: bool,
    pub readonly: bool,
    pub required: bool,
    pub checked: bool, 
}