#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Role {
    #[default]
    Generic,
    Button,
    Link,
    Image,
    Heading,
    Textbox,
    Checkbox,
    List,
    ListItem,
}