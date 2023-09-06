Motivation
==========

The motivation for writing this toolkit was:

1. Problems with the currently used JavaScript based framework
2. We want to reduce the number of programming languages

The company writes more and more code in rust, so 'yew' was a natural
choice. Most code can be written in rust, but it is still possible to
use JavaScript. This allows to use existing JavaScript libraries when
necessary.

The [Whitepaper](Whitepaper.md) contains more details on this.

Install
=======

1. Install WebAssembly target

 rustup target add wasm32-unknown-unknown

2. Install Trunk

 cargo install trunk

3. Install SASS (dart-sass is required)

We recommend installing dart-sass from github:

 cd ~
 wget https://github.com/sass/dart-sass/releases/download/1.58.3/dart-sass-1.58.3-linux-x64.tar.gz
 tar xf dart-sass-1.58.3-linux-x64.tar.gz

This should extract the binary to `~/dart-sass/sass`. You need to add the `dart-sass`
directory to the PATH variable:

 export PATH="$HOME/dart-sass:$PATH"

Note: In therory, Trunk downloads and use dart-sass automatically. To use it inside
hooks you still have to put the download directory into your path:

 export PATH="$HOME/.cache/trunk/sass-1.50.0:$PATH"


I18N
====

We will use the "tr" crate for I18N. So all translatable strings are
marked using the "tr!" macro.

Extract gettext messages with the "xtr" binary, which is part of the
"tr" crate.

See example/demo for an usage example.


Notes
=====

We try to use the builder pattern instead of the html macro.

Conventions:

- The Component is prefixed with "Pwt" (i.e. "PwtColumn")
- Corresponding Props without prefix (i.e. "Column")

- Builder is implemented on the Props


Focus traversal
---------------

Focus handling should be like:

https://developer.mozilla.org/en-US/docs/Web/Accessibility/Keyboard-navigable_JavaScript_widgets


Dialog
------

We use the new html <dialog> tag (mainly to simplify focus handling).

This should work in major browsers now (2022). Anyways, a polyfill is also available:

https://github.com/GoogleChrome/dialog-polyfill

You can enable it manually in older versions of firefox in "about:config" (dom.dialog_element.enabled)
