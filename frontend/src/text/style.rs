pub enum Weight {
    Normal,
    Bold,
    Italic,
    BoldItalic,
}

pub enum Style {
    Color(String),
    Size(String),
    Weight(Weight),
}
