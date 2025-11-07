// modified from https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/case.rs

use std::fmt::{Debug, Display};

use heck::{ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};

/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Copy, Clone, PartialEq)]
pub enum RenameRule {
    None,
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", RenameRule::LowerCase),
    ("UPPERCASE", RenameRule::UpperCase),
    ("PascalCase", RenameRule::PascalCase),
    ("camelCase", RenameRule::CamelCase),
    ("snake_case", RenameRule::SnakeCase),
    ("SCREAMING_SNAKE_CASE", RenameRule::ScreamingSnakeCase),
    ("kebab-case", RenameRule::KebabCase),
    ("SCREAMING-KEBAB-CASE", RenameRule::ScreamingKebabCase),
];

impl RenameRule {
    pub fn from_str(rename_all_str: &str) -> Result<Self, ParseError<'_>> {
        for (name, rule) in RENAME_RULES {
            if rename_all_str == *name {
                return Ok(*rule);
            }
        }
        Err(ParseError {
            unknown: rename_all_str,
        })
    }

    /// Apply a renaming rule to an enum variant, returning the version expected in the source.
    pub fn apply(self, name: &str) -> String {
        match self {
            Self::None => name.to_owned(),
            Self::LowerCase => name.to_ascii_lowercase(),
            Self::UpperCase => name.to_ascii_uppercase(),
            Self::PascalCase => name.to_upper_camel_case(),
            Self::CamelCase => name.to_lower_camel_case(),
            Self::SnakeCase => name.to_snake_case(),
            Self::ScreamingSnakeCase => name.to_shouty_snake_case(),
            Self::KebabCase => name.to_kebab_case(),
            Self::ScreamingKebabCase => name.to_shouty_kebab_case(),
        }
    }
}

pub struct ParseError<'a> {
    unknown: &'a str,
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("unknown rename rule `rename_all = ")?;
        Debug::fmt(self.unknown, f)?;
        f.write_str("`, expected one of ")?;
        for (i, (name, _rule)) in RENAME_RULES.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            Debug::fmt(name, f)?;
        }
        Ok(())
    }
}
