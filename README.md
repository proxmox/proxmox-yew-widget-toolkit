# Motivation

The motivation for writing this toolkit was:

1. Problems with the currently used JavaScript based framework
2. We want to reduce the number of programming languages

The company writes more and more code in rust, so 'yew' was a natural
choice. Most code can be written in rust, but it is still possible to
use JavaScript. This allows to use existing JavaScript libraries when
necessary.

The [Whitepaper](Whitepaper.md) contains more details on this.

# Usage

We provide a separate repository with usage examples:

https://git.proxmox.com/rust/proxmox-yew-wiget-toolkit-examples.git


# Contributions

All contributions to this project must accept the Developer Certificate of Origin (DCO) https://developercertificate.org/

See our Developer Documentations for how to communicate with other developers and for how
to sent patches https://pve.proxmox.com/wiki/Developer_Documentation


# I18N

All translatable strings are marked using the "tr!" macro.

Extract gettext messages with the "xtr" binary, which is part of the
"tr" crate.

See the demo in the examples crate on how to use this.


# Notes

We try to use the builder pattern instead of the html macro.

Conventions:

- The Yew Component is prefixed with "Pwt" (i.e. "PwtColumn")
- Corresponding Component Properties without prefix (i.e. "Column")
- Builder is implemented on the Component Properties.
- Component Properties implements Into&lt;Html&gt;.


## Focus traversal

Focus handling should be like:

https://developer.mozilla.org/en-US/docs/Web/Accessibility/Keyboard-navigable_JavaScript_widgets


## Dialog

We use the new html &lt;dialog&gt; tag (mainly to simplify focus handling).

This should work in major browsers now (2022). Anyways, a polyfill is also available:

https://github.com/GoogleChrome/dialog-polyfill

You can enable it manually in older versions of firefox in "about:config" (dom.dialog_element.enabled)


## Debugging

Simplest way to debug is using 'printf' debugging using log::{info,warning,error} in the code itself.

Another way is to use the DWARF info from wasm in the browser (chrome/chromium only):

* compile wasm file but keep debug info:
    * trunk: `<link data-trunk rel="rust" data-keep-debug data-no-demangle>` in index.html
    * others: `--keep-debug` in wasm-bindgen
* install debugging extension in chrome:
    * https://goo.gle/wasm-debugging-extension
    * restart browser after (else it won't work)
* open page with wasm with DWARF info
    * there should be a message in the console like: `[C/C++ DevTools Support (DWARF)] Loading debug symbols for ...`
* debugger should show a `file://` entry for rust files on disk
    * if the paths match with your local install, nothing else is to be done
    * otherwise use the extension options to map the source paths correctly
