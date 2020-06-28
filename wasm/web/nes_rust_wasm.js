
let wasm;

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

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
/**
*/
export const Button = Object.freeze({ Poweroff:0,Reset:1,Select:2,Start:3,Joypad1_A:4,Joypad1_B:5,Joypad1_Up:6,Joypad1_Down:7,Joypad1_Left:8,Joypad1_Right:9,Joypad2_A:10,Joypad2_B:11,Joypad2_Up:12,Joypad2_Down:13,Joypad2_Left:14,Joypad2_Right:15, });
/**
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
    * @returns {WasmNes}
    */
    static new() {
        var ret = wasm.wasmnes_new();
        return WasmNes.__wrap(ret);
    }
    /**
    * @param {Uint8Array} contents
    */
    set_rom(contents) {
        var ptr0 = passArray8ToWasm0(contents, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.wasmnes_set_rom(this.ptr, ptr0, len0);
    }
    /**
    */
    bootup() {
        wasm.wasmnes_bootup(this.ptr);
    }
    /**
    */
    reset() {
        wasm.wasmnes_reset(this.ptr);
    }
    /**
    */
    step_frame() {
        wasm.wasmnes_step_frame(this.ptr);
    }
    /**
    */
    update_pixels() {
        wasm.wasmnes_update_pixels(this.ptr);
    }
    /**
    * @returns {number}
    */
    pixels_ptr() {
        var ret = wasm.wasmnes_pixels_ptr(this.ptr);
        return ret;
    }
    /**
    */
    update_sample_buffer() {
        wasm.wasmnes_update_sample_buffer(this.ptr);
    }
    /**
    * @returns {number}
    */
    sample_buffer_ptr() {
        var ret = wasm.wasmnes_sample_buffer_ptr(this.ptr);
        return ret;
    }
    /**
    * @param {number} button
    */
    press_button(button) {
        wasm.wasmnes_press_button(this.ptr, button);
    }
    /**
    * @param {number} button
    */
    release_button(button) {
        wasm.wasmnes_release_button(this.ptr, button);
    }
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {

        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;

    return wasm;
}

export default init;

