let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
  if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

function decodeText(ptr, len) {
  return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return decodeText(ptr, len);
}

let heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
  if (heap_next === heap.length) heap.push(heap.length + 1);
  const idx = heap_next;
  heap_next = heap[idx];

  heap[idx] = obj;
  return idx;
}

function getObject(idx) {
  return heap[idx];
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
  cachedTextEncoder.encodeInto = function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
      read: arg.length,
      written: buf.length,
    };
  };
}

function passStringToWasm0(arg, malloc, realloc) {
  if (realloc === undefined) {
    const buf = cachedTextEncoder.encode(arg);
    const ptr = malloc(buf.length, 1) >>> 0;
    getUint8ArrayMemory0()
      .subarray(ptr, ptr + buf.length)
      .set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
  }

  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;

  const mem = getUint8ArrayMemory0();

  let offset = 0;

  for (; offset < len; offset++) {
    const code = arg.charCodeAt(offset);
    if (code > 0x7f) break;
    mem[ptr + offset] = code;
  }

  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset);
    }
    ptr = realloc(ptr, len, (len = offset + arg.length * 3), 1) >>> 0;
    const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
    const ret = cachedTextEncoder.encodeInto(arg, view);

    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }

  WASM_VECTOR_LEN = offset;
  return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
  if (
    cachedDataViewMemory0 === null ||
    cachedDataViewMemory0.buffer.detached === true ||
    (cachedDataViewMemory0.buffer.detached === undefined &&
      cachedDataViewMemory0.buffer !== wasm.memory.buffer)
  ) {
    cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
  }
  return cachedDataViewMemory0;
}

function dropObject(idx) {
  if (idx < 132) return;
  heap[idx] = heap_next;
  heap_next = idx;
}

function takeObject(idx) {
  const ret = getObject(idx);
  dropObject(idx);
  return ret;
}
/**
 * Initialize panic hook for better error messages in console
 */
exports.init = function () {
  wasm.init();
};

function isLikeNone(x) {
  return x === undefined || x === null;
}

function _assertClass(instance, klass) {
  if (!(instance instanceof klass)) {
    throw new Error(`expected instance of ${klass.name}`);
  }
}
/**
 * Compile MDX source to JSX
 *
 * # Arguments
 *
 * * `source` - MDX source code as string (max 10MB)
 * * `options` - Compilation options (optional, uses defaults if None)
 *
 * # Returns
 *
 * * `Ok(WasmMdxResult)` - Compiled JSX and metadata
 * * `Err(JsValue)` - Structured error object with kind, message, location, etc.
 *
 * # Errors
 *
 * Returns structured error objects that can be discriminated by `kind`:
 * - `"validationError"` - Input validation failed (size limit, null bytes)
 * - `"compilationError"` - MDX syntax error (with location and suggestion)
 * - `"serializationError"` - Failed to serialize result to JavaScript
 *
 * # Example
 *
 * ```javascript
 * import { compile_mdx, WasmMdxOptions } from './pkg/bunny_wasm.js';
 *
 * const options = new WasmMdxOptions();
 * options.set_gfm(true);
 *
 * try {
 *   const result = compile_mdx("# Hello **World**", options);
 *   console.log(result.code);
 * } catch (error) {
 *   if (error.kind === "compilationError") {
 *     console.error(error.message);
 *     if (error.location) {
 *       console.error(`At ${error.location.line}:${error.location.column}`);
 *     }
 *     if (error.suggestion) {
 *       console.log(`Suggestion: ${error.suggestion}`);
 *     }
 *   }
 * }
 * ```
 * @param {string} source
 * @param {WasmMdxOptions | null} [options]
 * @returns {any}
 */
exports.compile_mdx = function (source, options) {
  try {
    const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    let ptr1 = 0;
    if (!isLikeNone(options)) {
      _assertClass(options, WasmMdxOptions);
      ptr1 = options.__destroy_into_raw();
    }
    wasm.compile_mdx(retptr, ptr0, len0, ptr1);
    var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
    var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
    var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
    if (r2) {
      throw takeObject(r1);
    }
    return takeObject(r0);
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
};

const WasmMdxOptionsFinalization =
  typeof FinalizationRegistry === 'undefined'
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry((ptr) => wasm.__wbg_wasmmdxoptions_free(ptr >>> 0, 1));
/**
 * WASM-compatible MDX compilation options
 *
 * This is a JS-friendly wrapper around `MdxCompileOptions` that can be
 * constructed and configured from JavaScript.
 */
class WasmMdxOptions {
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    WasmMdxOptionsFinalization.unregister(this);
    return ptr;
  }

  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_wasmmdxoptions_free(ptr, 0);
  }
  /**
   * Get JSX runtime
   * @returns {string}
   */
  get jsx_runtime() {
    let deferred1_0;
    let deferred1_1;
    try {
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      wasm.wasmmdxoptions_jsx_runtime(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      deferred1_0 = r0;
      deferred1_1 = r1;
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_export3(deferred1_0, deferred1_1, 1);
    }
  }
  /**
   * Set the filepath (for error messages)
   * @param {string} filepath
   */
  set_filepath(filepath) {
    const ptr0 = passStringToWasm0(filepath, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    wasm.wasmmdxoptions_set_filepath(this.__wbg_ptr, ptr0, len0);
  }
  /**
   * Get output format
   * @returns {string}
   */
  get output_format() {
    let deferred1_0;
    let deferred1_1;
    try {
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      wasm.wasmmdxoptions_output_format(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      deferred1_0 = r0;
      deferred1_1 = r1;
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_export3(deferred1_0, deferred1_1, 1);
    }
  }
  /**
   * Enable/disable footnotes
   * @param {boolean} enabled
   */
  set_footnotes(enabled) {
    wasm.wasmmdxoptions_set_footnotes(this.__wbg_ptr, enabled);
  }
  /**
   * Set JSX runtime (default: "react/jsx-runtime")
   * @param {string} runtime
   */
  set_jsx_runtime(runtime) {
    const ptr0 = passStringToWasm0(runtime, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    wasm.wasmmdxoptions_set_jsx_runtime(this.__wbg_ptr, ptr0, len0);
  }
  /**
   * Set output format ("program" or "function-body")
   * @param {string} format
   */
  set_output_format(format) {
    const ptr0 = passStringToWasm0(format, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    wasm.wasmmdxoptions_set_output_format(this.__wbg_ptr, ptr0, len0);
  }
  /**
   * Get GFM setting
   * @returns {boolean}
   */
  get gfm() {
    const ret = wasm.wasmmdxoptions_gfm(this.__wbg_ptr);
    return ret !== 0;
  }
  /**
   * Create new options with defaults
   */
  constructor() {
    const ret = wasm.wasmmdxoptions_new();
    this.__wbg_ptr = ret >>> 0;
    WasmMdxOptionsFinalization.register(this, this.__wbg_ptr, this);
    return this;
  }
  /**
   * Get math setting
   * @returns {boolean}
   */
  get math() {
    const ret = wasm.wasmmdxoptions_math(this.__wbg_ptr);
    return ret !== 0;
  }
  /**
   * Enable/disable GFM (GitHub Flavored Markdown)
   * @param {boolean} enabled
   */
  set_gfm(enabled) {
    wasm.wasmmdxoptions_set_gfm(this.__wbg_ptr, enabled);
  }
  /**
   * Get the filepath
   * @returns {string | undefined}
   */
  get filepath() {
    try {
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      wasm.wasmmdxoptions_filepath(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      let v1;
      if (r0 !== 0) {
        v1 = getStringFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export3(r0, r1 * 1, 1);
      }
      return v1;
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
    }
  }
  /**
   * Enable/disable math
   * @param {boolean} enabled
   */
  set_math(enabled) {
    wasm.wasmmdxoptions_set_math(this.__wbg_ptr, enabled);
  }
  /**
   * Get footnotes setting
   * @returns {boolean}
   */
  get footnotes() {
    const ret = wasm.wasmmdxoptions_footnotes(this.__wbg_ptr);
    return ret !== 0;
  }
}
if (Symbol.dispose) WasmMdxOptions.prototype[Symbol.dispose] = WasmMdxOptions.prototype.free;

exports.WasmMdxOptions = WasmMdxOptions;

exports.__wbg_Error_e83987f665cf5504 = function (arg0, arg1) {
  const ret = Error(getStringFromWasm0(arg0, arg1));
  return addHeapObject(ret);
};

exports.__wbg_String_8f0eb39a4a4c2f66 = function (arg0, arg1) {
  const ret = String(getObject(arg1));
  const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
  const len1 = WASM_VECTOR_LEN;
  getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
  getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

exports.__wbg___wbindgen_is_string_fbb76cb2940daafd = function (arg0) {
  const ret = typeof getObject(arg0) === 'string';
  return ret;
};

exports.__wbg___wbindgen_throw_b855445ff6a94295 = function (arg0, arg1) {
  throw new Error(getStringFromWasm0(arg0, arg1));
};

exports.__wbg_new_1acc0b6eea89d040 = function () {
  const ret = new Object();
  return addHeapObject(ret);
};

exports.__wbg_new_68651c719dcda04e = function () {
  const ret = new Map();
  return addHeapObject(ret);
};

exports.__wbg_new_e17d9f43105b08be = function () {
  const ret = new Array();
  return addHeapObject(ret);
};

exports.__wbg_set_3f1d0b984ed272ed = function (arg0, arg1, arg2) {
  getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
};

exports.__wbg_set_907fb406c34a251d = function (arg0, arg1, arg2) {
  const ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
  return addHeapObject(ret);
};

exports.__wbg_set_c213c871859d6500 = function (arg0, arg1, arg2) {
  getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
};

exports.__wbindgen_cast_2241b6af4c4b2941 = function (arg0, arg1) {
  // Cast intrinsic for `Ref(String) -> Externref`.
  const ret = getStringFromWasm0(arg0, arg1);
  return addHeapObject(ret);
};

exports.__wbindgen_cast_4625c577ab2ec9ee = function (arg0) {
  // Cast intrinsic for `U64 -> Externref`.
  const ret = BigInt.asUintN(64, arg0);
  return addHeapObject(ret);
};

exports.__wbindgen_cast_9ae0607507abb057 = function (arg0) {
  // Cast intrinsic for `I64 -> Externref`.
  const ret = arg0;
  return addHeapObject(ret);
};

exports.__wbindgen_cast_d6cd19b81560fd6e = function (arg0) {
  // Cast intrinsic for `F64 -> Externref`.
  const ret = arg0;
  return addHeapObject(ret);
};

exports.__wbindgen_object_clone_ref = function (arg0) {
  const ret = getObject(arg0);
  return addHeapObject(ret);
};

exports.__wbindgen_object_drop_ref = function (arg0) {
  takeObject(arg0);
};

const wasmPath = `${__dirname}/bunny_wasm_bg.wasm`;
const wasmBytes = require('fs').readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBytes);
const wasm = (exports.__wasm = new WebAssembly.Instance(wasmModule, imports).exports);

wasm.__wbindgen_start();
