White paper: Writing GUIs with Rust and Yew?
===========================================

We at [Proxmox](https://www.proxmox.com) started using
[Rust](https://www.rust-lang.org) in 2017, and managed to release our
first [Product](https://www.proxmox.com/en/proxmox-backup-server)
entirely written in Rust in 2020.

Well, I need to correct myself: Only the server side is written in
Rust. The product also includes a GUI, which is browser based and is
written in Javascript. 

But everyone here is exited about the Rust language, so we asked
ourselves if it is possible to write GUIs in Rust. It would give us
the following advantages:

- Reduce the number of programming languages used inside the company
  (long term goal). Yes, I want to get rid of Javascript.

- Improve code reuse (share code between back-end and front-end)

- Of cause, we get all Rust language feature (type safety, ...)


We finally decided to give it a try by writing a prototype GUI for the
backup server.

I started testing various Rust GUI libraries, and finally decided to
use [Yew](https://yew.rs) as base, because

- It is still possible to use/run JavaScript code, i.e. things like
  [noVNC](https://novnc.com/info.html).

- React/Elm like programming experience (but using Rust).

- Quite stable so far.

- Possible to ship native apps using [Tauri](https://tauri.app). So it
  is kind of cross-platform.

- Future prove: Sure, I'm no clairvoyant, but I'm pretty confident that
  this technology will last for the next 30 years.

- Great debug environment (browser).

Of cause, I also tested non-browser based libraries like
[Druid](https://github.com/linebender/druid), but none of them really
convinced me.
