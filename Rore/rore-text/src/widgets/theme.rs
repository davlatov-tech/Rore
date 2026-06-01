use rore_types::Color;

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_click: Color,
    pub text: Color,
    pub text_muted: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: Color::hex("#0f172a"),    // slate-900
            surface: Color::hex("#1e293b"),       // slate-800
            primary: Color::hex("#3b82f6"),       // blue-500
            primary_hover: Color::hex("#60a5fa"), // blue-400
            primary_click: Color::hex("#2563eb"), // blue-600
            text: Color::WHITE,
            text_muted: Color::hex("#94a3b8"), // slate-400
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::hex("#f8fafc"), // slate-50
            surface: Color::WHITE,
            primary: Color::hex("#2563eb"),       // blue-600
            primary_hover: Color::hex("#3b82f6"), // blue-500
            primary_click: Color::hex("#1d4ed8"), // blue-700
            text: Color::hex("#0f172a"),          // slate-900
            text_muted: Color::hex("#64748b"),    // slate-500
        }
    }
}
