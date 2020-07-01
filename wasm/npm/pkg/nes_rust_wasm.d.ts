/* tslint:disable */
/* eslint-disable */
/**
*/
export enum Button {
  Poweroff,
  Reset,
  Select,
  Start,
  Joypad1A,
  Joypad1B,
  Joypad1Up,
  Joypad1Down,
  Joypad1Left,
  Joypad1Right,
  Joypad2A,
  Joypad2B,
  Joypad2Up,
  Joypad2Down,
  Joypad2Left,
  Joypad2Right,
}
/**
* `WasmNes` is an interface between user JavaScript code and
* WebAssembly NES emulator. The following code is example
* JavaScript user code.
*
* ```ignore
* // Create NES
* const nes = WasmNes.new();
*
* // Load Rom
* nes.set_rom(new Uint8Array(romArrayBuffer));
*
* // Set up Audio
* const audioContext = AudioContext || webkitAudioContext;
* const bufferLength = 4096;
* const context = new audioContext({sampleRate: 44100});
* const scriptProcessor = context.createScriptProcessor(bufferLength, 0, 1);
* scriptProcessor.onaudioprocess = e => {
*   const data = e.outputBuffer.getChannelData(0);
*   nes.update_sample_buffer(data);
* };
* scriptProcessor.connect(context.destination);
*
* // Set up screen resources
* const width = 256;
* const height = 240;
* const canvas = document.createElement('canvas');
* const ctx = canvas.getContext('2d');
* const imageData = ctx.createImageData(width, height);
* const pixels = new Uint8Array(imageData.data.buffer);
*
* // animation frame loop
* const stepFrame = () => {
*   requestAnimationFrame(stepFrame);
*   // Run emulator until screen is refreshed
*   nes.step_frame();
*   // Load screen pixels and render to canvas
*   nes.update_pixels(pixels);
*   ctx.putImageData(imageData, 0, 0);
* };
*
* // Go!
* nes.bootup();
* stepFrame();
* ```
*/
export class WasmNes {
  free(): void;
/**
* Creates a `WasmNes`
* @returns {WasmNes}
*/
  static new(): WasmNes;
/**
* Sets up NES rom
*
* # Arguments
* * `rom` Rom image binary `Uint8Array`
* @param {Uint8Array} contents
*/
  set_rom(contents: Uint8Array): void;
/**
* Boots up
*/
  bootup(): void;
/**
* Resets
*/
  reset(): void;
/**
* Executes a CPU cycle
*/
  step(): void;
/**
* Executes a PPU (screen refresh) frame
*/
  step_frame(): void;
/**
* Copies RGB pixels of screen to passed RGBA pixels.
* The RGBA pixels length should be
* 245760 = 256(width) * 240(height) * 4(RGBA).
* A channel will be filled with 255(opaque).
*
* # Arguments
* * `pixels` RGBA pixels `Uint8Array` or `Uint8ClampedArray`
* @param {Uint8Array} pixels
*/
  update_pixels(pixels: Uint8Array): void;
/**
* Copies audio buffer to passed `Float32Array` buffer.
* The length should be 4096.
*
* # Arguments
* * `buffer` Audio buffer `Float32Array`
* @param {Float32Array} buffer
*/
  update_sample_buffer(buffer: Float32Array): void;
/**
* Presses a pad button
*
* # Arguments
* * `button`
* @param {number} button
*/
  press_button(button: number): void;
/**
* Releases a pad button
*
* # Arguments
* * `buffer`
* @param {number} button
*/
  release_button(button: number): void;
}
