// This is similarg to the "tr" macro from crate "tr", but uses the single global catalog

#[cfg(doc)]
use crate::gettext_runtime_format;

#[cfg(doc)]
use crate::{gettext, pgettext, ngettext, npgettext};

/// Translate strings using gettext and format arguments using [gettext_runtime_format!].
///
/// ```
/// # use pwt::prelude::*;
/// # fn dummy() -> String {
/// // Outputs "Hello world!", or a translated version depending on the loaded catalog.
/// tr!("Hello world!")
/// # }
/// ```
///
/// The string to translate need to be a string literal, as it has to be extracted by
/// the `xtr` tool. One can add more argument
///
/// ```
/// # use pwt::prelude::*;
/// # fn dummy() -> String {
/// // Outputs "You received 5 messages.", or a translated version.
/// let message_count = 5;
/// tr!("You received {} messages", message_count)
/// # }
/// ```
///
/// The format string may reference arguments using the following syntax:
///
/// - `{}` - use next argument (increments internal position counter).
/// - `{<nth>}` - use nth argument, i.e. `{0}` and `{1}`.
/// - `{<name>}` - use named arguments, i.e. `{n}`.
///
/// Plural are using the `"singular" | "plural" % count` syntax. `{n}` will be replaced
/// by the count.
///
/// ```
/// # use pwt::prelude::*;
/// # fn dummy(message_count: usize) -> String {
/// tr!("You received one message" | "You received {n} messages" % message_count)
/// # }
/// ```
///
/// Please note that it is still possible to add more arguments
///
/// ```
/// # use pwt::prelude::*;
/// # fn dummy(number_of_items: usize, folder_name: &str) -> String {
/// tr!(
///     "There is one item in folder {}" |
///     "There are {n} items in folder {}" % number_of_items,
///     folder_name
/// )
/// # }
/// ```
///
/// If you want to use the same string for different pruposes, you may want to use an
/// disambiguation context, using the `"context" =>` syntax. You simply use this
/// to give more context to the translators.
///
/// ```
/// # use pwt::prelude::*;
/// # fn dummy() -> String {
/// // Outputs "CPU", or a translated version.
/// tr!("Central Procesing Unit" => "CPU")
/// # }
/// ```
///
/// To enable the translation, one must first call the [init_i18n](crate::init_i18n).
/// To translate the strings, one can use the `xtr` utility to extract the string,
/// and use the other GNU gettext tools to translate them.
///
/// # Note
///
/// The [tr!](crate::tr!) macro combines the functionality of [gettext!], [pgettext!],
/// [ngettext!] and [npgettext!].

#[macro_export]
macro_rules! tr {
    ($msgid:tt, $($tail:tt)* ) => {
        $crate::gettext_runtime_format!($crate::gettext($msgid), $($tail)*)
    };
    ($msgid:tt) => {
        $crate::gettext_runtime_format!($crate::gettext($msgid))
    };
    ($msgctx:tt => $msgid:tt, $($tail:tt)* ) => {
        $crate::gettext_runtime_format!($crate::pgettext($msgctx, $msgid), $($tail)*)
    };
    ($msgctx:tt => $msgid:tt) => {
        $crate::gettext_runtime_format!($crate::pgettext($msgctx, $msgid))
    };
    ($msgid:tt | $plur:tt % $n:expr, $($tail:tt)* ) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::ngettext($msgid, $plur, n as u64), $($tail)*, n=n)
    }};
    ($msgid:tt | $plur:tt % $n:expr) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::ngettext($msgid, $plur, n as u64), n=n)
    }};
    ($msgctx:tt => $msgid:tt | $plur:tt % $n:expr, $($tail:tt)* ) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::npgettext($msgctx, $msgid, $plur, n as u64), $($tail)*, n=n)
    }};
    ($msgctx:tt => $msgid:tt | $plur:tt % $n:expr) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::npgettext($msgctx, $msgid, $plur, n as u64), n=n)
    }};
}

/// Like [gettext()], but format arguments using [gettext_runtime_format!].
#[macro_export]
macro_rules! gettext {
    ($msgid:tt, $($tail:tt)* ) => {
        $crate::gettext_runtime_format!($crate::gettext($msgid), $($tail)*)
    };
    ($msgid:tt) => {
        $crate::gettext_runtime_format!($crate::gettext($msgid))
    };
}

/// Like [ngettext()], but format arguments using [gettext_runtime_format!].
///
/// Note: You can use `{n}` to reference the passed count.
#[macro_export]
macro_rules! ngettext {
    ($msgid:tt , $plur:tt , $n:expr, $($tail:tt)* ) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::ngettext($msgid, $plur, n), $($tail)*, n=n)
    }};
    ($msgid:tt , $plur:tt , $n:expr) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::ngettext($msgid, $plur, n), n=n)
    }};
}

/// Like [pgettext()], but format arguments using [gettext_runtime_format!].
#[macro_export]
macro_rules! pgettext {
    ($msgid:tt, $msgctx:tt,  $($tail:tt)* ) => {
        $crate::gettext_runtime_format!($crate::pgettext($msgctx, $msgid), $($tail)*)
    };
    ($msgid:tt, $msgctx:tt) => {
        $crate::gettext_runtime_format!($crate::pgettext($msgctx, $msgid))
    };
}
/// Like [npgettext()], but format arguments using [gettext_runtime_format!].
///
/// Note: You can use `{n}` to reference the passed count.
#[macro_export]
macro_rules! npgettext {
    ($msgctx:tt ,  $msgid:tt , $plur:tt , $n:expr, $($tail:tt)* ) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::npgettext($msgctx, $msgid, $plur, n), $($tail)*, n=n)
    }};
    ($msgctx:tt , $msgid:tt , $plur:tt , $n:expr) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::npgettext($msgctzx, $msgid, $plur, n), n=n)
    }};
}
