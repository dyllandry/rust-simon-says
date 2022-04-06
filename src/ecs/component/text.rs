pub struct TextComponent {
    pub text: String,
    pub alignment: TextAlignment,
    /// The maximum width of a text component. The width of the actual text may be less if the
    /// glyphs do not fill the width.
    pub width: f32,
}

pub enum TextAlignment {
    Left,
    Center,
}
