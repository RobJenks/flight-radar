use piston_window::Glyphs;

pub const DEFAULT_FONT: &str = "resources/font-fira-sans.ttf";

pub struct TextManager {
    font: String,
    cache: Glyphs
}

impl TextManager {
    pub fn create(font: String, cache: Glyphs) -> Self {
        Self { font, cache }
    }

    pub fn glyph_cache(&mut self) -> &mut Glyphs {
        &mut self.cache
    }

    #[allow(dead_code)]
    pub fn get_font_name(&self) -> String {
        self.font.clone()
    }
}