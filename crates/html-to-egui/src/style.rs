//! Some CSS Style that maps to egui style.
use crate::Error;
use ahash::HashMap;
use ahash::HashSet;
use ahash::HashSetExt;
use std::ops::Deref;
use std::{
    hash::{Hash, Hasher},
    num::ParseFloatError,
};

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Style {
    /// The number of pixels from the left for this elements.
    Left,
    /// Pixels fromt the top.
    Top,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StyleValue {
    F32(f32),
}

impl Deref for StyleValue {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        match self {
            StyleValue::F32(val) => val,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct F32Wrapper(f32);

impl Eq for F32Wrapper {}

impl Hash for F32Wrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Treat NaN as equal to itself for hashing purposes.
        if self.0.is_nan() {
            state.write_u32(0x7fc00000); // A specific NaN representation.
        } else {
            state.write_u32(self.0.to_bits());
        }
    }
}

impl From<f32> for F32Wrapper {
    fn from(val: f32) -> Self {
        Self(val)
    }
}

impl From<F32Wrapper> for f32 {
    fn from(val: F32Wrapper) -> Self {
        val.0
    }
}

impl Deref for F32Wrapper {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Style {
    /// Constant for the left style.
    const LEFT: &'static str = "left";
    const TOP: &'static str = "top";

    /// Returns the string representation of the style.
    fn as_str(&self) -> &'static str {
        match self {
            Style::Left => Self::LEFT,
            Style::Top => Self::TOP,
        }
    }

    /// Try to parse a style string ("left: 32px; top: 32px;") into a Style enum.
    pub fn parse(s: &str) -> HashMap<Self, StyleValue> {
        // style string is separated by semicolons, so we split it first.
        let styles: Vec<&str> = s.split(';').collect();

        // next, we iterate over the styles and try to parse them.
        let mut parsed_styles = HashMap::default();

        // iterate over the styles and try to parse them.
        // skip over styles that are invalid.
        for style in styles {
            let style: Vec<&str> = style.split(':').collect();

            if style.len() != 2 {
                continue;
            }

            let label = style[0].trim();
            let val = style[1]
                .chars()
                .take_while(|c| c.is_numeric())
                .collect::<String>();

            let Ok(val) = val.parse::<f32>() else {
                continue;
            };

            match label {
                Self::LEFT => {
                    parsed_styles.insert(Style::Left, StyleValue::F32(val));
                }
                Self::TOP => {
                    parsed_styles.insert(Style::Top, StyleValue::F32(val));
                }
                _ => continue,
            };
        }

        parsed_styles
    }
}

impl TryFrom<&str> for Style {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            Style::LEFT => Ok(Style::Left),
            Style::TOP => Ok(Style::Top),
            _ => Err(Error::InvalidConversion(s.to_string())),
        }
    }
}
