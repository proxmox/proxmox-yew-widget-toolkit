// This is similarg to the "tr" macro from crate "tr", but uses the single global catalog

#[cfg(doc)]
use crate::gettext_runtime_format;

#[cfg(doc)]
use crate::gettext_wrapper::{gettext, pgettext, ngettext, npgettext};

#[macro_export]
macro_rules! tr {
    ($msgid:tt, $($tail:tt)* ) => {
        $crate::gettext_runtime_format!($crate::gettext_wrapper::gettext($msgid), $($tail)*)
    };
    ($msgid:tt) => {
        $crate::gettext_runtime_format!($crate::gettext_wrapper::gettext($msgid))
    };
    ($msgctx:tt => $msgid:tt, $($tail:tt)* ) => {
        $crate::gettext_runtime_format!($crate::gettext_wrapper::pgettext($msgctx, $msgid), $($tail)*)
    };
    ($msgctx:tt => $msgid:tt) => {
        $crate::gettext_runtime_format!($crate::gettext_wrapper::pgettext($msgctx, $msgid))
    };
    ($msgid:tt | $plur:tt % $n:expr, $($tail:tt)* ) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::gettext_wrapper::ngettext($msgid, $plur, n), $($tail)*, n=n)
    }};
    ($msgid:tt | $plur:tt % $n:expr) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::gettext_wrapper::ngettext($msgid, $plur, n), n=n)
    }};
    ($msgctx:tt => $msgid:tt | $plur:tt % $n:expr, $($tail:tt)* ) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::gettext_wrapper::npgettext($msgctx, $msgid, $plur, n), $($tail)*, n=n)
    }};
    ($msgctx:tt => $msgid:tt | $plur:tt % $n:expr) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::gettext_wrapper::npgettext($msgctzx, $msgid, $plur, n), n=n)
    }};
}

/// Like [gettext()], but format arguments using [gettext_runtime_format!].
#[macro_export]
macro_rules! gettext {
    ($msgid:tt, $($tail:tt)* ) => {
        $crate::gettext_runtime_format!($crate::gettext_wrapper::gettext($msgid), $($tail)*)
    };
    ($msgid:tt) => {
        $crate::gettext_runtime_format!($crate::gettext_wrapper::gettext($msgid))
    };
}

/// Like [ngettext()], but format arguments using [gettext_runtime_format!].
///
/// Note: You can use `{n}` to reference the passed count.
#[macro_export]
macro_rules! ngettext {
    ($msgid:tt , $plur:tt , $n:expr, $($tail:tt)* ) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::gettext_wrapper::ngettext($msgid, $plur, n), $($tail)*, n=n)
    }};
    ($msgid:tt , $plur:tt , $n:expr) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::gettext_wrapper::ngettext($msgid, $plur, n), n=n)
    }};
}

/// Like [pgettext()], but format arguments using [gettext_runtime_format!].
#[macro_export]
macro_rules! pgettext {
    ($msgid:tt, $msgctx:tt,  $($tail:tt)* ) => {
        $crate::gettext_runtime_format!($crate::gettext_wrapper::pgettext($msgctx, $msgid), $($tail)*)
    };
    ($msgid:tt, $msgctx:tt) => {
        $crate::gettext_runtime_format!($crate::gettext_wrapper::pgettext($msgctx, $msgid))
    };
}
/// Like [npgettext()], but format arguments using [gettext_runtime_format!].
///
/// Note: You can use `{n}` to reference the passed count.
#[macro_export]
macro_rules! npgettext {
    ($msgctx:tt ,  $msgid:tt , $plur:tt , $n:expr, $($tail:tt)* ) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::gettext_wrapper::npgettext($msgctx, $msgid, $plur, n), $($tail)*, n=n)
    }};
    ($msgctx:tt , $msgid:tt , $plur:tt , $n:expr) => {{
        let n = $n;
        $crate::gettext_runtime_format!($crate::gettext_wrapper::npgettext($msgctzx, $msgid, $plur, n), n=n)
    }};
}
