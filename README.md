# rcm-reactor

The RCM context-menu engine rebuilt on native Windows UI — an alternative frontend to
[`rcm-tauri`](https://github.com/ahaoboy/rcm-tauri)

- **[`windows-reactor`](https://github.com/microsoft/windows-rs/tree/master/crates/libs/reactor)** — declarative WinUI 3 in Rust
  (replaces the Tauri WebView + React frontend)
- **[`tray-icon`](https://crates.io/crates/tray-icon)** — system tray (replaces `tauri`'s tray)
- **[`windows`](https://crates.io/crates/windows)** — the little Win32 needed to make Reactor
  windows behave like native popup menus

All system behaviour is shared with `rcm-tauri` through `rcm-core`, `rcm-vm`, `rcm-com` and
`rcm-reg` — including the layout engine, so both builds place menus identically and differ only in
how they draw them.

```sh
cargo run -p rcm-reactor
```

|              | [`rcm-tauri`](../rcm-tauri)              | `rcm-reactor`                                 |
| ------------ | ---------------------------------------- | --------------------------------------------- |
| UI           | WebView + React                          | native WinUI 3                                |
| Menu windows | one WebView per level, pooled and reused | one native popup per level, created on demand |
| Tray         | `tauri`                                  | `tray-icon` + `muda`                          |
