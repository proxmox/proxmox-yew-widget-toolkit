# White paper: Writing GUIs with Rust and Yew?

We at [Proxmox](https://www.proxmox.com) started using
[Rust](https://www.rust-lang.org) in 2017, and managed to release our
first [Product](https://www.proxmox.com/en/proxmox-backup-server)
entirely written in Rust in 2020.

Well, I need to correct myself: Only the server side is written in
Rust. The product also includes a GUI, which is browser based and
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
[Flutter](https://flutter.dev)). So we are looking for something
cross-platform.

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

And all other stuff supported by the currently used framework, which
is about 11Mb JavaScript code!.

So the next step was quite clear. I need to write reusable widget
library providing all that functionality.


## Writing complex components/widgets.

The set of required widget was quite clear, so this was more of a
trial and error phase to find convienient APIs and data structures for
Rust.

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
it still works. We will try to gradually improve the library while
keeping the API stable.


### Use Rust (instead of `html!{}` macro) whenever possible.

Yew provide a ways to create components using the `html!{}` macro, i.e:

```
html!{<div class="pwt-color-scheme-primary" onclick=|_| { ... }>{"Click me!"}</div>}
```

Instead, our components provides builder function, so you can produce
the same result with:

```
Container::new()
   .class(TextAlign::Justify)
   .class(ColorScheme::Primary)
   .onclick(|_| { ...})
   .with_child("Click me!)
```

All components implement `Into<Html>`, and the container `with_child`
method accepts anything implementing `Into<Html>`. So you can simply
pass text, or use other components:

```
Row::new()
  .with_child(Button::new("Button1"))
  .with_child("Just some text.")
  .with_child(Button::new("Button2"))
```

The `class()` method also deserves some attention. You can pass text
strings to specify CSS classes. But we also provide Rust types for
common classes which implements `Into<Classes>`. The class method
directly accepts those types, so you can spefify css classes in a type
safe way.

All base widgets implements a common builder API to specify classes,
attributes and event callbacks.

We think that this style is much easier to read/format and understand,
especially when you configure many propertyies. We still use the html
macro for short html fragments, because it's sometimes simpler in that
case.


### Layout Containers

I started with some very simple layout containers called `Row` and
`Column`. They simply create a HTML flexbox (`<div style="display:
flex;...`>) with the corresponding direction. Everything else can be
controlled by adding class attributes to either the container or its
children.

We ship a set of CSS classes which allows you to control all flexbox
properties. Such CSS class utilities got famous with
[bootstrap](https://getbootstrap.com), but today many frameworks use a
similar approach.

```
Column::new()
   // CSS: width: 100%; height: 100%; overflow: auto
   .class(Fit)
   // CSS: justify-content: center
   .class(JustifyContent::Center)
   // CSS: align-items: center
   .class(AlignItems::Center)
   ...
```

Other layout types like our `SpliPane` require programmatic control,
but also use flexbox for the layout.

To summarize, we simply use HTML layout, either CSS flexbox or CSS
grid. This is extremely flexible and well known.


### Accessible Rich Internet Applications ([ARIA](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA))

Without ARIA certain functionality used in Web sites is not available
to people who rely on screen readers and people who cannot use a
mouse.

The number of those affected is in the single-digit percentage
range. This looks relatively low at first, but accounts to a large
number of unhappy users which can't use your product. So good ARIA
support is a must have for us (Note: Your eyes don't get better as you
get older).

We tired to make all widget fully accessible by following the [ARIA
Authoring Practices Guide](https://www.w3.org/WAI/ARIA/apg).


### Support for right-to-left scripts

Languages like Arabic, Hebrew and Persian are left-to-right, and
people expect that row-layouts also change direction when you use such
language. Fortunatly, CSS already support switching between LTR/RTL
direction, and a flexbox row automatically changes the direction.

There are still things you need to do programatically, and it is
sometimes required to consider the script direction. Especially when
you navigate through flexbox children using arrow keys, or if you
resize flexbox children using the mouse.

When we first tried RTL mode, basically everything was somehow
broken. But once you get aware of the problem, this is relatively easy
to fix, and we managed to make all widgets working with RTL mode.


## Rust specific Problems

Rust is a great language, but it also has its quirks. Worst of all,
we're still used to having things like class inheritance that just
aren't available with Rust. Of cause, this isn't necessarily a bad
thing as long as you find another solution.

Our widgets share a great amount of methods, and we don't want to
duplicated the code for each widget. Inheritance is not available, so
we ended up using traits having default method implementations. For
example, we want to attach html event listeners to widget, so lets use
this to show how we share code.

Our trait is called `EventSubscriber`, and has one method which
provides mutable access to a `Listeners` object, where we can store
the callbacks. This is the only method without a default
implementation, and once you implement it for your widget, you get all
the other methods from the default trait implementation. This is also
a good way to split functionality and documentations into smaller
parts.

```
pub trait EventSubscriber: Static {
    /// Mutable access to something that can store Listeners.
    fn as_listeners_mut(&mut self) -> &mut Listeners;

    /// all other method have default implementations

    fn onclick(mut self, cb: IntoEventCallback<MouseEvent>) -> Self {
        self.as_listeners_mut().add(cb.into_event_callback());
        self
    }

    fn ondblclick(mut self, cb: IntoEventCallback<MouseEvent>) -> Self {
        self.as_listeners_mut().add(cb.into_event_callback());
        self
    }

    ...
}
```

For further convenience, we provide a `widget` macro that
automatically add required properties and methods. For example, the
following code implements the buider function for a full featured
container:

```
#[widget(/* required features */ @element, @container)]
#[derive(Default, Clone)]
pub struct Row {}

impl Row {
    /// Create a new instance.
    pub fn new() -> Self {
        Self::default()
            .class(Display::Flex)
    }
}
```
