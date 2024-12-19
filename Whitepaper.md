# White paper: Writing GUIs with Rust and Yew?

We at [Proxmox](https://www.proxmox.com) started using
[Rust](https://www.rust-lang.org) in 2017, and managed to release our first
[Product](https://www.proxmox.com/en/proxmox-backup-server) entirely written in
Rust in 2020.

Well, I need to correct myself: Only the server side is written in Rust. The
product also includes a GUI, which is browser based and written in JavaScript.

But everyone here is excited about the Rust language, so we asked ourselves if
it's possible to write GUIs in Rust. It would give us the following advantages:

- Reduce the number of programming languages used inside the company (long term
  goal).

- Improve code reuse (share code between back-end and front-end)

- Of cause, we get all Rust language feature (type safety, ...)

We finally decided to give it a try by writing a prototype GUI for the backup
server.

## Technology selection.

First, let me inform you that we currently ship web based GUIs for all our
products. Some products additionally have native GUIs on Linux, and we ship
GUIs for mobile devices (i.e. using [Flutter](https://flutter.dev)). So we are
looking for something cross-platform.

I started testing various Rust GUI libraries, and finally decided to use
[Yew](https://yew.rs) as base, because

- It is still possible to use/run JavaScript code, i.e. things like
  [noVNC](https://novnc.com/info.html).

- React/Elm like programming experience (but using Rust).

- Quite stable so far.

- Possible to ship native apps using [Tauri](https://tauri.app). So it is kind
  of cross-platform.

- Future proof: Sure, I'm no clairvoyant, but I'm pretty confident that those
  technologies (browser, web-assembly, rust) will last for the next 30 years.

- Well known environment (browser), with great debugging options.

- [ARIA](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA)
  support.

- We can use [SASS](https://sass-lang.com) to write CSS style sheets.


## Using Yew directly.

I started by writing small test apps like
[7GUIs](https://eugenkiss.github.io/7guis/), and this worked quit well. Turns
out that you can easily translate a [React](https://reactjs.org) application
into a Yew functional component. So far so good.

Things got worse when I started to rebuild the backup server GUI. It uses
countless of complex UI elements like:

- Data Tables with virtual scroll, sorting, filtering and more ...
- Operating system like menus.
- Modal dialogs.
- Complex forms.
- Window splitters.
- Complex charts.

And all other stuff supported by the currently used framework, which is about
11Mb JavaScript code!.

So the next step was quite clear. I need to write a reusable widget library
providing all that functionality.

## Writing complex components/widgets.

The set of required widget was quite clear, so this was more of a trial and
error phase to find convenient APIs and data structures for Rust.

The current library is called [Proxmox Yew Widget
Toolkit](https://git.proxmox.com) and is the result of multiple rewrites. It is
flexible enough to implement larger parts of the backup server GUI, but there is
still room for improvements:

- provide more components
- improve patterns for code sharing
- improve CSS styling

The good thing is that Rust is a strongly typed language. It allows you to
restructure large parts of your code, and if it still compiles, it still works.
We will try to gradually improve the library while keeping the API stable.

### Use Rust (instead of `html!{}` macro) whenever possible.

Yew provides a way to create components using the `html!{}` macro, i.e:

```
html!{<div class="pwt-color-scheme-primary" onclick=|_| { ... }>{"Click me!"}</div>}
```

Instead, our components provides builder function, so you can produce the same
result with:

```
Container::new()
   .class(TextAlign::Justify)
   .class(ColorScheme::Primary)
   .onclick(|_| { ...})
   .with_child("Click me!")
```

All components implement `Into<Html>`, and the container `with_child` method
accepts anything implementing `Into<Html>`. So you can simply pass text, or use
other components:

```
Row::new()
  .with_child(Button::new("Button1"))
  .with_child("Just some text.")
  .with_child(Button::new("Button2"))
```

The `class()` method also deserves some attention. You can pass text strings to
specify CSS classes.  But we also provide Rust types for common classes which
implements `Into<Classes>`. The class method directly accepts those types, so
you can specify css classes in a type safe way.

All base widgets implements a common builder API to specify classes, attributes
and event callbacks.

To further simplify layout, we also defined functions to set padding, margin and
style on those widgets:

```
Container::new()
   .style("color", "red")
   .style("background-color", "white)
   .margin(2)
   .padding(4)
```

We think that this style is much easier to read/format and understand,
especially when you configure many properties. We still use the html macro for
short html fragments, because it's sometimes simpler in that case.


### Layout Containers

We started with some very simple layout containers called `Row` and `Column`.
They simply create a HTML flexbox (`<div style="display: flex;...`>) with the
corresponding direction. Everything else can be controlled by adding class
attributes to either the container or its children.

We ship a set of CSS classes which allows you to control all flexbox properties.
Such CSS class utilities got famous with [bootstrap](https://getbootstrap.com),
but today many frameworks use a similar approach.

```
Column::new()
   // CSS: flex: 1 1 auto;
   .class(Flex::Fill)
   // CSS: justify-content: center
   .class(JustifyContent::Center)
   // CSS: align-items: center
   .class(AlignItems::Center)
   ...
```

Other layout types like our `SplitPane` require programmatic control, but also
use flexbox for the layout.

To summarize, we simply use HTML layout, either CSS flexbox or CSS grid. This is
extremely flexible and well known.


### Accessible Rich Internet Applications ([ARIA](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA))

Without ARIA certain functionality used in Web sites is not available to people
who rely on screen readers and people who cannot use a mouse.

The number of those affected is in the single-digit percentage range. This looks
relatively low at first, but would account to a large number of unhappy users,
which can't use our products. So good ARIA support is a must have for us (Note:
Your eyes don't get better as you get older).

We tired to make all widget fully accessible by following the [ARIA Authoring
Practices Guide](https://www.w3.org/WAI/ARIA/apg).


### Support for right-to-left scripts

Languages like Arabic, Hebrew and Persian are left-to-right, and people expect
that row-layouts also change direction when you use such language. Fortunately,
CSS already support switching between LTR/RTL direction, and a flexbox row
automatically changes the direction.

There are still things you need to do programmatically, and it is sometimes
required to consider the script direction. Especially when you navigate through
flexbox children using arrow keys, or if you resize flexbox children using the
mouse.

When we first tried RTL mode, basically everything was somehow broken. But once
you get aware of the problem, this is relatively easy to fix, and we managed to
make all widgets working with RTL mode.


## Rust specific Problems

Rust is a great language, but it also has its quirks. Worst of all, we're still
used to having things like class inheritance that just aren't available with
Rust. Of cause, this isn't necessarily a bad thing as long as you find another
solution.

Our widgets share a great amount of methods, and we don't want to duplicated the
code for each widget. Inheritance is not available, so we ended up using traits
having default method implementations. For example, we want to attach html event
listeners to widget. Let us use this example to show how we implemented code
sharing.

Our trait is called `EventSubscriber`, and has one method which provides mutable
access to a `Listeners` object, where we can store the callbacks. This is the
only method without a default implementation, and once you implement it for your
widget, you get all the other methods from the default trait implementation.
This is also a good way to split functionality and documentations into smaller
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

For further convenience, we provide a `widget` macro that automatically add
required properties and methods. For example, the following code implements the
builder function for a full featured container widget:

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

## Themes, Colors and Design

As noted previously, our first goal was to replace existing GUIs, so we started
with the same design and colors. The resulting theme is quite dense, allowing
you to display much information on a single page.

In parallel, we tried to write a second theme suitable for touch devices. Touch
devices requires much larger buttons and inputs. Else it is too difficult to tap
them accurately, leaving users frustrated and dissatisfied after making
mistakes.

We used the [material design](https://m3.material.io) guidelines from Google as
baseline. The resulting theme wastes a lot of space on the desktop, but is a
requirement for mobile devices.

In the end, we added another theme with spacing optimized for common desktop
application.

All themes are written with [SASS](https://sass-lang.com), so it is relatively
easy to modify them.


## Mobile Devices

Small mobile devices, such as smartphones or tablets, come with unique
characteristics and constraints that lead to specific widget requirements.

Mobile devices have much smaller screens compared to desktop computers. This
means that GUI widgets need to be carefully designed and arranged to make the
best use of the available space without cluttering the interface.

Most mobile devices primarily use touch interfaces, which means that GUI widgets
need to be large enough to be easily selected with a finger. This is a
significant shift from desktop interfaces that primarily use a mouse cursor,
which can select much smaller targets.

Also, modern user interface on mobile devices use animations, much more than one
would expect on the desktop.

Fortunately, The material design guidelines from Google describes many widgets
used by modern mobile applications, so we just needed to implement them.

We ended up with a set of special widget for mobile devices, making it really
easy write a mobile app.

```
/// Define your routes (uses yew_router library)
#[derive(Clone, Copy, PartialEq, Routable)]
enum Route {
   #[at("/")]
   Home,
   #[at("/config")]
   Config,
   #[at("/config/network")]
   Network,
}

/// Map routes to a stack of pages (automatically adds page animations)
fn switch(route: &Route) -> Vec<Html> {
   match route {
       Route::Home => vec![
            Scaffold::with_title("Home").into(),
       ],
       Route::Config => vec![
            Scaffold::with_title("Config").into(),
       ],
       Route::Network => vec![
            Scaffold::with_title("Config").into(),
            Scaffold::with_title("Network").into(),
       ],
   }
}

/// Finally, implement the application component.
#[function_component]
fn YourApp() -> Html {
    MaterialApp::new(switch)
        .into()
}
```

## Conclusion

After more than one year, I start feeling confident that we can write high
quality user interfaces using Rust. We managed to implement quite complex
widgets within reasonable time.

The next plan is to make the code available to the public in hopes that we can
get outside people to try the code and share their ideas on how we can further
improve the framework.
