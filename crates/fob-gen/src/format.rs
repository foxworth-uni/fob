//! Code formatting options for generated JavaScript

/// Quote style for string literals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuoteStyle {
    /// Single quotes: `'hello'`
    Single,
    /// Double quotes: `"hello"`
    #[default]
    Double,
}

/// Indentation style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    /// Tabs
    Tabs,
    /// Spaces with specified width
    Spaces(u8),
}

impl Default for IndentStyle {
    fn default() -> Self {
        IndentStyle::Spaces(2)
    }
}

/// Formatting options for code generation
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Use semicolons at end of statements
    pub use_semicolons: bool,
    /// Quote style for string literals
    pub quote_style: QuoteStyle,
    /// Indentation style
    pub indent: IndentStyle,
    /// Add trailing commas in arrays/objects
    pub trailing_commas: bool,
    /// Line width for formatting (0 = no limit)
    pub line_width: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            use_semicolons: true,
            quote_style: QuoteStyle::default(),
            indent: IndentStyle::default(),
            trailing_commas: false,
            line_width: 80,
        }
    }
}

impl FormatOptions {
    /// Create formatting options matching fob-cli output expectations
    pub fn fob_default() -> Self {
        Self::default()
    }

    /// Create formatting options for minified output
    pub fn minified() -> Self {
        Self {
            use_semicolons: true,
            quote_style: QuoteStyle::Double,
            indent: IndentStyle::Spaces(0),
            trailing_commas: false,
            line_width: 0,
        }
    }
}
