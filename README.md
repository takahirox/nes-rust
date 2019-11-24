# nes-rust

nes-rust is a NES emulator written in Rust.

# Screenshots

[nestest](http://wiki.nesdev.com/w/index.php/Emulator_tests)

![nestest](./screenshots/nestest.png)

[Sgt. Helmet Training Day](http://www.mojontwins.com/juegos_mojonos/sgt-helmet-training-day-nes/)

![Sgt. Helmet Training Day](./screenshots/Sgt_Helmet.png)

# How to build and run

You need SDL2. Refer to [Rust-SDL2 Readme](https://github.com/Rust-SDL2/rust-sdl2#rust) for the detail.

```
$ git clone https://github.com/takahirox/nes-rust.git
$ cd nes-rust
$ # install or setup SDL2
$ cargo build --release
$ cargo run --release path_to_rom
```