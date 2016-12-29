importScripts('colorful-map.js');

// `base` and `api_key` describe the API endpoint.
const base = 'https://tile.mapzen.com/mapzen/vector/v1/';
const api_key = 'mapzen-j16kH4C'

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

// Gets the URL from where to load the vector tile.
// By default it loads all layers, but you can also give it a layer
// name or a list of layers to load.
function getURL(coords, layers='all') {
  if (Array.isArray(layers)) {
    layers = layers.join();
  }
  return `${ base }${ layers }/${ coords.z }/${ coords.x }/${ coords.y }.mvt?api_key=${ api_key }`;
}

self.addEventListener('message',
  e => fetch(getURL(e.data.coords), {cache: "force-cache"})
    .then(res => res.arrayBuffer())
    .then(buffer => self.postMessage({id: e.data.id, tile: process(buffer)})));