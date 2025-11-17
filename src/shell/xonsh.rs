#![allow(dead_code)]
use std::{borrow::Cow, fmt::Display};

use indoc::formatdoc;

use crate::shell::Shell;

pub struct Xonsh;

impl Xonsh {
    fn escape_sq(input: &str) -> Cow<'_, str> {
        for (i, ch) in input.char_indices() {
            if Self::escape_char(ch).is_some() {
                let mut escaped_string = String::with_capacity(input.len());

                escaped_string.push_str(&input[..i]);
                for ch in input[i..].chars() {
                    match Self::escape_char(ch) {
                        Some(escaped_char) => escaped_string.push_str(escaped_char),
                        None => escaped_string.push(ch),
                    };
                }
                return Cow::Owned(escaped_string);
            }
        }
        Cow::Borrowed(input)
    }

    fn escape_char(ch: char) -> Option<&'static str> {
        match ch {
            // escape ' \ ␤ (docs.python.org/3/reference/lexical_analysis.html#strings)
            '\'' => Some("\\'"),
            '\\' => Some("\\\\"),
            '\n' => Some("\\n"),
            _ => None,
        }
    }
}

impl Shell for Xonsh {
    fn set_env(&self, k: &str, v: &str) -> String {
        formatdoc!(
            r#"
            from xonsh.built_ins import XSH
            XSH.env['{k}'] = '{v}'
        "#,
            k = shell_escape::unix::escape(k.into()), // todo: drop illegal chars, not escape?
            v = Self::escape_sq(v)
        )
    }

    fn prepend_env(&self, k: &str, v: &str) -> String {
        formatdoc!(
            r#"
            from xonsh.built_ins import XSH
            XSH.env['{k}'].add('{v}', front=True)
        "#,
            k = shell_escape::unix::escape(k.into()),
            v = Self::escape_sq(v)
        )
    }

    fn unset_env(&self, k: &str) -> String {
        formatdoc!(
            r#"
            from xonsh.built_ins import XSH
            XSH.env.pop('{k}',None)
        "#,
            k = shell_escape::unix::escape(k.into()) // todo: drop illegal chars, not escape?
        )
    }
}

impl Display for Xonsh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "xonsh")
    }
}
