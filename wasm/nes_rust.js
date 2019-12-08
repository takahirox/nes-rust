
let wasm;

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory;
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm(arg) {
    const ptr = wasm.__wbindgen_malloc(arg.length * 1);
    getUint8Memory().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

function getStringFromWasm(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
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
    * @param {Uint8Array} contents
    * @returns {WasmNes}
    */
    static new(contents) {
        const ret = wasm.wasmnes_new(passArray8ToWasm(contents), WASM_VECTOR_LEN);
        return WasmNes.__wrap(ret);
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
        const ret = wasm.wasmnes_pixels_ptr(this.ptr);
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
        const ret = wasm.wasmnes_sample_buffer_ptr(this.ptr);
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

function init(module) {
    if (typeof module === 'undefined') {
        module = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    let result;
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm(arg0, arg1));
    };

    if ((typeof URL === 'function' && module instanceof URL) || typeof module === 'string' || (typeof Request === 'function' && module instanceof Request)) {

        const response = fetch(module);
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            result = WebAssembly.instantiateStreaming(response, imports)
            .catch(e => {
                return response
                .then(r => {
                    if (r.headers.get('Content-Type') != 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
                        return r.arrayBuffer();
                    } else {
                        throw e;
                    }
                })
                .then(bytes => WebAssembly.instantiate(bytes, imports));
            });
        } else {
            result = response
            .then(r => r.arrayBuffer())
            .then(bytes => WebAssembly.instantiate(bytes, imports));
        }
    } else {

        result = WebAssembly.instantiate(module, imports)
        .then(result => {
            if (result instanceof WebAssembly.Instance) {
                return { instance: result, module };
            } else {
                return result;
            }
        });
    }
    return result.then(({instance, module}) => {
        wasm = instance.exports;
        init.__wbindgen_wasm_module = module;

        return wasm;
    });
}

export default init;

