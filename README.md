## 🌐 Crawler

A modern asynchronous web crawler written in Rust using the actor model and Tokio, with live dynamic graph visualization and interactive GUI.

<img src="etc/interface.png" alt="Interface screenshot">

## Building
If you are on nix:
```bash
$ nix-shell etc/shell.nix
```

Then build and run:
```bash
$ cargo build
$ cargo run --release
```
