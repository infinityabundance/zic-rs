//! Lexical records: the output of the lexer, before any semantic interpretation.
//!
//! A tzdata source file is line-oriented. After comment stripping and field splitting,
//! each non-blank line becomes a [`Line`] holding its 1-based number and the [`Field`]s on
//! it. Keeping the column of each field lets diagnostics point precisely at the offending
//! token rather than just the line.

/// One whitespace-delimited (or quoted) field, with the byte column it started at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// The field's content with surrounding quotes removed and escapes resolved.
    pub text: String,
    /// 0-based byte column of the field's first character on the original line.
    pub col: usize,
}

impl Field {
    pub fn new(text: impl Into<String>, col: usize) -> Self {
        Field {
            text: text.into(),
            col,
        }
    }
}

/// A single logical source line that contained at least one field.
#[derive(Debug, Clone)]
pub struct Line {
    /// 1-based line number in the source file.
    pub number: usize,
    pub fields: Vec<Field>,
}

impl Line {
    /// The first field's text, if any — the record keyword for a fresh line (`Rule`,
    /// `Zone`, `Link`), or the first column of a zone continuation line.
    pub fn keyword(&self) -> Option<&str> {
        self.fields.first().map(|f| f.text.as_str())
    }
}
