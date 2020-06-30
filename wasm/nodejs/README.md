# nes_rust_wasm

[![npm version](https://badge.fury.io/js/nes_rust_wasm.svg)](https://badge.fury.io/js/nes_rust_wasm)

nes_rust_wasm is a WebAssembly NES emulator based on
[nes-rust](https://github.com/takahirox/nes-rust).

## How to install

```
$ npm install nes_rust_wasm
```

## How to use

```javascript
const nes = require('nes_rust_wasm').WasmNes.new();
nes.set_rom(new Uint8Array(romArrayBuffer));

const pixels = new Uint8Array(256 * 240 * 4);

// Audio example code is T.B.D.
const audioBuffer = new Float32Array(4096);

const runFrame = () => {
  setTimeout(runFrame, 0);
  nes.step_frame();
  nes.update_pixels(pixels);
  // Render pixels
};

nes.bootup();
runFrame();
```

## API

Refer to [the comments in WasmNes](https://github.com/takahirox/nes-rust/blob/master/wasm/src/lib.rs)

## How to build WebAssembly RISC-V emulator locally

Prerequirements
- Install [wasm-bindgen client](https://rustwasm.github.io/docs/wasm-bindgen/)

```sh
$ git clone https://github.com/takahirox/nes-rust.git
$ cd nes-rust/wasm
$ bash build.sh
```
