# nes-rust

[![Build Status](https://travis-ci.com/takahirox/nes-rust.svg?branch=master)](https://travis-ci.com/takahirox/nes-rust)
[![Crate](https://img.shields.io/crates/v/nes_rust.svg)](https://crates.io/crates/nes_rust)
[![npm version](https://badge.fury.io/js/nes_rust_wasm.svg)](https://badge.fury.io/js/nes_rust_wasm)

nes-rust is a NES emulator written in Rust. It can be compiled to WebAssembly.

## Online Demos / Videos

- [Online Singleplay Demo](https://takahirox.github.io/nes-rust/wasm/web/index.html)
- [Online Multiplay Demo](https://takahirox.github.io/nes-rust/wasm/web/multiplay.html) / [Video](https://twitter.com/superhoge/status/1205427421010247680)
- [Online VR Multiplay Demo](https://takahirox.github.io/nes-rust/wasm/web/vr.html) / [Video](https://twitter.com/superhoge/status/1209685614074875906)

## Screenshots

[nestest](http://wiki.nesdev.com/w/index.php/Emulator_tests)

![nestest](./screenshots/nestest.png)

[Sgt. Helmet Training Day](http://www.mojontwins.com/juegos_mojonos/sgt-helmet-training-day-nes/)

![Sgt. Helmet Training Day](./screenshots/Sgt_Helmet.png)

## Features

- Audio support with SDL2 / WebAudio
- WebAssembly support
- Remote multiplay support with WebRTC

## How to import into your Rust project

The emulator module is released at [crates.io](https://crates.io/crates/nes_rust
). Add the following line into Cargo.toml of your Rust project.

```
[dependencies]
nes_rust = "0.1.0"
```

Refer to [Document](https://docs.rs/nes_rust/0.1.0/nes_rust/struct.Nes.html) for the API.

## How to build core library locally

```
$ git clone https://github.com/takahirox/nes-rust.git
$ cd nes-rust
$ cargo build --release
```

## How to run as desktop application

Prerequirements
- Install [Rust-SDL2](https://github.com/Rust-SDL2/rust-sdl2#rust)

```
$ cd nes-rust/cli
$ cargo run --release path_to_rom_file
```

## How to import and use WebAssembly NES emulator in a web browser

See [wasm/web](https://github.com/takahirox/nes-rust/tree/master/wasm/web)

## How to install and use WebAssembly NES emulator npm package

See [wasm/npm](https://github.com/takahirox/nes-rust/tree/master/wasm/npm)
