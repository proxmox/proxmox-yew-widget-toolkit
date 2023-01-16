# White paper: Writing GUIs with Rust and Yew?

We at [Proxmox](https://www.proxmox.com) started using
[Rust](https://www.rust-lang.org) in 2017, and managed to release our
first [Product](https://www.proxmox.com/en/proxmox-backup-server)
entirely written in Rust in 2020.

Well, I need to correct myself: Only the server side is written in
Rust. The product also includes a GUI, which is browser based and is
written in Javascript.

But everyone here is exited about the Rust language, so we asked
ourselves if it's possible to write GUIs in Rust. It would give us
the following advantages:

- Reduce the number of programming languages used inside the company
  (long term goal).

- Improve code reuse (share code between back-end and front-end)

- Of cause, we get all Rust language feature (type safety, ...)


We finally decided to give it a try by writing a prototype GUI for the
backup server.


## Technology selection.

First, let me know you that we currently ship web based GUIs for all
our products. Some products additionally have native GUIs on Linux,
and we ship GUIs for mobile devices (i.e. using
[Flutter](https://flutter.dev)). So we are lokkink for something
Cross-platform.

I started testing various Rust GUI libraries, and finally decided to
use [Yew](https://yew.rs) as base, because

- It is still possible to use/run JavaScript code, i.e. things like
  [noVNC](https://novnc.com/info.html).

- React/Elm like programming experience (but using Rust).

- Quite stable so far.

- Possible to ship native apps using [Tauri](https://tauri.app). So it
  is kind of cross-platform.

- Future prove: Sure, I'm no clairvoyant, but I'm pretty confident
  that used technologies (browser, web-assembly, rust) will last for
  the next 30 years.

- Well known environment (browser), with great debugging options.

- [ARIA](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA) support.

Of cause, I also tested non-browser based libraries like
[Druid](https://github.com/linebender/druid), but none of them really
convinced me.

## Using Yew directly.

I started by writing small test apps like
[7GUIs](https://eugenkiss.github.io/7guis/), and this worked quit
well. Turns out that you can easily translate a
[React](https://reactjs.org) application into a Yew functional
component. So far so good.

Things got worse when I started to rebuild the backup server GUI. It
uses countless of complex UI elements like:

- Data Tables with virtual scroll, sorting, filtering and more ...
- Operating system like menus.
- Modal dialogs.
- Complex forms.
- Window splitters.
- Complex charts.

And all other stuff supported by the currently used framework (about
11Mb JavaScript code!).

So the next step was quite clear. I need to write reusable widget
library providing all that functionality.

## Writing complex components/widgets.

It's no secret that sometimes you have to build your Rust code from
scratch because the compiler won't let you do it the way you
want. Another reason is when the produced HTML doesn't behave as
expected. A third reason is that ARIA roles are not flexible enough to
describe your component, so accessibility is not good enough.

I short, the current library is called [Proxmox Yew Widget
Toolkit](https://git.proxmox.com) and is the result of multiple
rewrites. It is flexible enought to implement larger parts of the
backup server GUI, but there is still room for improvements:

- provide more components
- improve patterns for code sharing
- improve CSS styling

The good think is that Rust is a strongly typed language. It allows
you to restructure large parts of your code, and if it still compiles,
it still works.

### Use Rust (instead of `html!{}` macro) whenever possible.

Yew provide a ways to create components using the `html!{}` macro, i.e:

```
html!{<div class="primary" onclick=|_| { ... }>{"Click me!"}</div>}
```

Instead, our components provides builder function, so you can produce
the same result with:

```
Container::new()
   .class("primary")
   .onclick(|_| { ...})
   .with_child("Click me!)
```

All components implement `Into<Html>`, and `with_child` accepts
anything implementing it.

We think that this style is much easier to read/format and understand,
especially when you configure many propertyies. We still use the html
macro for short html fragments.


### Basic Layout Containers

I added some very simple layout containers called `Row` and `Column`,
but they simply create a HTML flexbox (`<div style="display:
flex;...`>). Everything else can be controlled by adding class
attributes to either the container or its children. We ship a set of
CSS classes which allows to control all flexbox properties.

```
Column::new()
   // CSS: width: 100%; height: 100%; overflow: auto
   .class("pwt-fit")
   // CSS: justify-content: center
   .class("pwt-justify-content-center")
   // CSS: align-items: center
   .class("pwt-align-items-center")
   ...
```

Such CSS class utilities got famous with
[bootstrap](https://getbootstrap.com), but today many frameworks use a
similar approach.

To summarize, we simply use HTML layout, either CSS flexbox or CSS
grid. This is extremely flexible and well known.
