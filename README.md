# nes-rust

nes-rust is a NES emulator written in Rust.

[Online Singleplay Demo](https://takahirox.github.io/nes-rust/index.html)

[Online Multiplay Demo](https://takahirox.github.io/nes-rust/multiplay.html) / [Video](https://twitter.com/superhoge/status/1205427421010247680)

[Online VR Multiplay Demo](https://takahirox.github.io/nes-rust/vr.html) / [Video](https://twitter.com/superhoge/status/1209685614074875906)

# Screenshots

[nestest](http://wiki.nesdev.com/w/index.php/Emulator_tests)

![nestest](./screenshots/nestest.png)

[Sgt. Helmet Training Day](http://www.mojontwins.com/juegos_mojonos/sgt-helmet-training-day-nes/)

![Sgt. Helmet Training Day](./screenshots/Sgt_Helmet.png)

# Features

- Audio support with SDL2 / WebAudio
- WebAssembly support
- Remote multiplay support with WebRTC

# How to build and run

## Standalone

You need SDL2. Refer to [Rust-SDL2 Readme](https://github.com/Rust-SDL2/rust-sdl2#rust) for the detail.

```
$ git clone https://github.com/takahirox/nes-rust.git
$ cd nes-rust
$ # install or setup SDL2
$ cargo build --release
$ cargo run --release path_to_rom
```

## Web (WebAssembly)

You need wasm-bindgen client. Refer to [the document](https://rustwasm.github.io/docs/wasm-bindgen/) for the detail.

```
$ git clone https://github.com/takahirox/nes-rust.git
$ cd nes-rust
$ # install wasm-bindgen
$ cargo build --release --lib --target wasm32-unknown-unknown
$ wasm-bindgen ./target/wasm32-unknown-unknown/release/nes_rust.wasm --out-dir ./wasm/ --target web --no-typescript
# Boot up local web server and access index.html via the server
```
