[nes-rust/wasm/web](https://github.com/takahirox/nes-rust/tree/master/wasm/web) is a directory for WebAssembly NES emulator compiled from [nes-rust](https://github.com/takahirox/nes-rust) and its online demo. You can import the emulator into your web page.

## Online Demos / Videos

- [Online Singleplay Demo](https://takahirox.github.io/nes-rust/wasm/web/index.html)
- [Online Multiplay Demo](https://takahirox.github.io/nes-rust/wasm/web/multiplay.html) / [Video](https://twitter.com/superhoge/status/1205427421010247680)
- [Online VR Multiplay Demo](https://takahirox.github.io/nes-rust/wasm/web/vr.html) / [Video](https://twitter.com/superhoge/status/1209685614074875906)

## How to import in a web page

Download [nes_rust_wasm.js](https://github.com/takahirox/nes-rust/blob/master/wasm/web/nes_rust_wasm.js) and [nes_rust_wasm_bg.wasm](https://github.com/takahirox/nes-rust/blob/master/wasm/web/nes_rust_wasm_bg.wasm), and place them to where a web page can access.

Below is the example code to import and use them.

```javascript
<script type="module">
  import init, { WasmNes } from "./nes_rust_wasm.js";
  init().then(async wasm => {
    // Create NES
    const nes = WasmNes.new();

    // Load Rom
    const romBuffer = await fetch(path_to_rom_image).then(res => res.arrayBuffer());
    nes.set_rom(new Uint8Array(romBuffer));

    // Set up Audio
    const audioContext = AudioContext || webkitAudioContext;
    const bufferLength = 4096;
    const context = new audioContext({sampleRate: 44100});
    const scriptProcessor = context.createScriptProcessor(bufferLength, 0, 1);
    scriptProcessor.onaudioprocess = e => {
      const data = e.outputBuffer.getChannelData(0);
      nes.update_sample_buffer(data);
    };
    scriptProcessor.connect(context.destination);

    // Set up screen resources
    const width = 256;
    const height = 240;
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    const imageData = ctx.createImageData(width, height);
    const pixels = new Uint8Array(imageData.data.buffer);

    // animation frame loop
    const stepFrame = () => {
      requestAnimationFrame(stepFrame);
      // Run emulator until screen is refreshed
      nes.step_frame();
      // Load screen pixels and render to canvas
      nes.update_pixels(pixels);
      ctx.putImageData(imageData, 0, 0);
    };

    // Go!
    nes.bootup();
    stepFrame();
  });
</script>
```

## API

Refer to the comments in [`WasmNes`](https://github.com/takahirox/nes-rust/blob/master/wasm/src/lib.rs)

## How to build WebAssembly NES emulator and run demo in web browser locally

Prerequirements
- Install [wasm-bindgen client](https://rustwasm.github.io/docs/wasm-bindgen/)

```sh
$ git clone https://github.com/takahirox/nes-rust.git
$ cd nes-rust/wasm
$ bash build.sh
$ cd ..
# boot local server and access nes-rust/wasm/web/index.html
```
