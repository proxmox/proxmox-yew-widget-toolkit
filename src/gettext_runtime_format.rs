// Simple runtime formatter for gettext messages
//
// Ideas found found in rust crate "tr" and "runtime_fmt"

#[doc(hidden)]
pub struct FormatArguments<'a> {
    #[doc(hidden)]
    pub format: &'a str,
    #[doc(hidden)]
    pub args: &'a [(&'static str, &'a dyn (::std::fmt::Display))],
}

/// gettext_runtime_format! macro.
#[macro_export]
macro_rules! gettext_runtime_format {
    ($format:expr) => {{
        // Note we call the formatter to check for missing args (instead of using $format.to_string())
        let format = $format;
        let fa = $crate::gettext_runtime_format::FormatArguments {
            format: AsRef::as_ref(&format),
            args: &[],
        };
        $crate::gettext_runtime_format::gettext_runtime_format_arguments_to_string(fa)
    }};
    ($format:expr,  $($tail:tt)* ) => {{
        let format = $format;
        let fa = $crate::gettext_runtime_format::FormatArguments {
            format: AsRef::as_ref(&format),
            args: $crate::gettext_runtime_format!(@parse_args [] $($tail)*)
        };
        $crate::gettext_runtime_format::gettext_runtime_format_arguments_to_string(fa)
    }};

    (@parse_args [$($args:tt)*]) => { &[ $( $args ),* ]  };
    (@parse_args [$($args:tt)*] $name:ident) => {
        $crate::gettext_runtime_format!(@parse_args [$($args)* (stringify!($name) , &$name)])
    };
    (@parse_args [$($args:tt)*] $name:ident, $($tail:tt)*) => {
        $crate::gettext_runtime_format!(@parse_args [$($args)* (stringify!($name) , &$name)] $($tail)*)
    };
    (@parse_args [$($args:tt)*] $name:ident = $e:expr) => {
        $crate::gettext_runtime_format!(@parse_args [$($args)* (stringify!($name) , &$e)])
    };
    (@parse_args [$($args:tt)*] $name:ident = $e:expr, $($tail:tt)*) => {
        $crate::gettext_runtime_format!(@parse_args [$($args)* (stringify!($name) , &$e)] $($tail)*)
    };
    (@parse_args [$($args:tt)*] $e:expr) => {
        $crate::gettext_runtime_format!(@parse_args [$($args)* ("" , &$e)])
    };
    (@parse_args [$($args:tt)*] $e:expr, $($tail:tt)*) => {
        $crate::gettext_runtime_format!(@parse_args [$($args)* ("" , &$e)] $($tail)*)
    };
}

enum ParserState {
    Text,
    Argument,
}

#[doc(hidden)]
pub fn gettext_runtime_format_arguments_to_string(args: FormatArguments) -> String {
    let mut state = ParserState::Text;
    let mut argument: Option<String> = None;
    let mut argument_index = 0;

    let mut output = String::new();

    let mut iter = args.format.chars().peekable();

    while let Some(c) = iter.next() {
        match state {
            ParserState::Text if c == '{' => {
                if let Some('{') = iter.peek() {
                    // quoted '{'
                    output.push(c);
                    iter.next();
                    continue;
                }
                argument = None;
                state = ParserState::Argument;
            }
            ParserState::Text if c == '}' => {
                if let Some('}') = iter.peek() {
                    // quoted '}'
                    output.push(c);
                    iter.next();
                    continue;
                }
                output.push(c); // unexpected, but allowed
            }
            ParserState::Text => {
                output.push(c);
            }
            ParserState::Argument if c == '}' => {
                let argument = argument.take().map(|arg| arg.trim().to_string());
                let arg = if let Some(argument) = argument {
                    if let Ok(n) = argument.parse::<usize>() {
                        args.args.get(n)
                    } else {
                        args.args.iter().find(|arg| arg.0 == argument)
                    }
                } else {
                    let arg = args.args.get(argument_index);
                    argument_index += 1;
                    arg
                };
                if let Some(arg) = arg {
                    output.push_str(&arg.1.to_string());
                } else {
                    log::error!("gettext_runtime_format error - missing argument in \"{}\"", args.format);
                    return args.format.to_string();
                }
                state = ParserState::Text;
            }
            ParserState::Argument => {
                if !(c.is_ascii_alphanumeric() || c == '_') {
                    log::error!("gettext_runtime_format error - illegal chars in argument name \"{}\"", args.format);
                    return args.format.to_string();
                }
                if let Some(argument) = &mut argument {
                    argument.push(c);
                } else {
                    argument = Some(String::from(c));
                }
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_format() {
        assert_eq!(gettext_runtime_format!("Simple Text"), "Simple Text");
        assert_eq!(gettext_runtime_format!("Quoted {{Braces}}"), "Quoted {Braces}");
        assert_eq!(gettext_runtime_format!("ARG {}", "A0"), "ARG A0");
        assert_eq!(gettext_runtime_format!("ARG {0}", "A0"), "ARG A0");
        assert_eq!(gettext_runtime_format!("ARG {0}", 50), "ARG 50");
        assert_eq!(gettext_runtime_format!("ARG {1} {0}", "A0", "A1"), "ARG A1 A0");
        assert_eq!(gettext_runtime_format!("ARG {n} {0}", "A0", n = 5), "ARG 5 A0");

        // strange messages, but alowed
        assert_eq!(gettext_runtime_format!("Strange brace } (not expected)"), "Strange brace } (not expected)");
    }

    #[test]
    fn test_format_errors() {
        assert_eq!(gettext_runtime_format!("ARG {name}", name = "A0"), "ARG A0");
        assert_eq!(gettext_runtime_format!("ARG {name-xxx}", "A0"), "ARG {name-xxx}");
        assert_eq!(gettext_runtime_format!("ARG {non_existent}", "A0"), "ARG {non_existent}");
        assert_eq!(gettext_runtime_format!("ARG {nam{e}", name = "A0"), "ARG {nam{e}");
    }
}
