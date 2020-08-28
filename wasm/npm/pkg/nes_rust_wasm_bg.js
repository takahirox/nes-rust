import * as wasm from './nes_rust_wasm_bg.wasm';

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

let cachegetFloat32Memory0 = null;
function getFloat32Memory0() {
    if (cachegetFloat32Memory0 === null || cachegetFloat32Memory0.buffer !== wasm.memory.buffer) {
        cachegetFloat32Memory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachegetFloat32Memory0;
}

function passArrayF32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4);
    getFloat32Memory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
/**
*/
export const Button = Object.freeze({ Poweroff:0,"0":"Poweroff",Reset:1,"1":"Reset",Select:2,"2":"Select",Start:3,"3":"Start",Joypad1A:4,"4":"Joypad1A",Joypad1B:5,"5":"Joypad1B",Joypad1Up:6,"6":"Joypad1Up",Joypad1Down:7,"7":"Joypad1Down",Joypad1Left:8,"8":"Joypad1Left",Joypad1Right:9,"9":"Joypad1Right",Joypad2A:10,"10":"Joypad2A",Joypad2B:11,"11":"Joypad2B",Joypad2Up:12,"12":"Joypad2Up",Joypad2Down:13,"13":"Joypad2Down",Joypad2Left:14,"14":"Joypad2Left",Joypad2Right:15,"15":"Joypad2Right", });
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

    static __wrap(ptr) {
        const obj = Object.create(WasmNes.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_wasmnes_free(ptr);
    }
    /**
    * Creates a `WasmNes`
    * @returns {WasmNes}
    */
    static new() {
        var ret = wasm.wasmnes_new();
        return WasmNes.__wrap(ret);
    }
    /**
    * Sets up NES rom
    *
    * # Arguments
    * * `rom` Rom image binary `Uint8Array`
    * @param {Uint8Array} contents
    */
    set_rom(contents) {
        var ptr0 = passArray8ToWasm0(contents, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.wasmnes_set_rom(this.ptr, ptr0, len0);
    }
    /**
    * Boots up
    */
    bootup() {
        wasm.wasmnes_bootup(this.ptr);
    }
    /**
    * Resets
    */
    reset() {
        wasm.wasmnes_reset(this.ptr);
    }
    /**
    * Executes a CPU cycle
    */
    step() {
        wasm.wasmnes_step(this.ptr);
    }
    /**
    * Executes a PPU (screen refresh) frame
    */
    step_frame() {
        wasm.wasmnes_step_frame(this.ptr);
    }
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
    update_pixels(pixels) {
        try {
            var ptr0 = passArray8ToWasm0(pixels, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.wasmnes_update_pixels(this.ptr, ptr0, len0);
        } finally {
            pixels.set(getUint8Memory0().subarray(ptr0 / 1, ptr0 / 1 + len0));
            wasm.__wbindgen_free(ptr0, len0 * 1);
        }
    }
    /**
    * Copies audio buffer to passed `Float32Array` buffer.
    * The length should be 4096.
    *
    * # Arguments
    * * `buffer` Audio buffer `Float32Array`
    * @param {Float32Array} buffer
    */
    update_sample_buffer(buffer) {
        try {
            var ptr0 = passArrayF32ToWasm0(buffer, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.wasmnes_update_sample_buffer(this.ptr, ptr0, len0);
        } finally {
            buffer.set(getFloat32Memory0().subarray(ptr0 / 4, ptr0 / 4 + len0));
            wasm.__wbindgen_free(ptr0, len0 * 4);
        }
    }
    /**
    * Presses a pad button
    *
    * # Arguments
    * * `button`
    * @param {number} button
    */
    press_button(button) {
        wasm.wasmnes_press_button(this.ptr, button);
    }
    /**
    * Releases a pad button
    *
    * # Arguments
    * * `buffer`
    * @param {number} button
    */
    release_button(button) {
        wasm.wasmnes_release_button(this.ptr, button);
    }
}

export const __wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

