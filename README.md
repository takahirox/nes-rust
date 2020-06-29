# nes-rust

nes-rust is a NES emulator written in Rust.

[Online Singleplay Demo](https://takahirox.github.io/nes-rust/index.html)

[Online Multiplay Demo](https://takahirox.github.io/nes-rust/multiplay.html) / [Video](https://twitter.com/superhoge/status/1205427421010247680)

[Online VR Multiplay Demo](https://takahirox.github.io/nes-rust/vr.html) / [Video](https://twitter.com/superhoge/status/1209685614074875906)

## Screenshots

[nestest](http://wiki.nesdev.com/w/index.php/Emulator_tests)

![nestest](./screenshots/nestest.png)

[Sgt. Helmet Training Day](http://www.mojontwins.com/juegos_mojonos/sgt-helmet-training-day-nes/)

![Sgt. Helmet Training Day](./screenshots/Sgt_Helmet.png)

## Features

- Audio support with SDL2 / WebAudio
- WebAssembly support
- Remote multiplay support with WebRTC

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

## How to build WebAssembly NES emulator and run in the browser locally

Prerequirements
- Install [wasm-bindgen client](https://rustwasm.github.io/docs/wasm-bindgen/)

```
$ cd nes-rust/wasm
$ bash build_wasm.sh
$ cd ..
# Boot local web server up and access wasm/web/index.html via the server
```
