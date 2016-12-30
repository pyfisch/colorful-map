importScripts('colorful-map.js');

// Wrapper for Rust functions. Do not use directly.
const _process = Module.cwrap('process_web', 'number', ['number', 'number']);
const _free_cstring = Module.cwrap('free_cstring_web', null, ['number']);

// Parses an `mvt` (Mapbox Vector Tile) file and renders its contents
// as an SVG fragment. The fragment is returned as a string.
//
// Wraps the `process` function written in Rust.
function process(mvt) {
  const array = new Uint8Array(mvt);
  const length = array.length;
  const array_p = Module._malloc(length);
  Module.writeArrayToMemory(array, array_p);
  const string_p = _process(array_p, length);
  const string = UTF8ToString(string_p);
  Module._free(array_p);
  _free_cstring(string_p);
  return string;
}

self.addEventListener('message',
  e => fetch(e.data.blob)
    .then(res => res.arrayBuffer())
    .then(buffer => self.postMessage({id: e.data.id, tile: process(buffer)})));
