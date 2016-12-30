// `base` and `api_key` describe the API endpoint.
const base = 'https://tile.mapzen.com/mapzen/vector/v1/';
const api_key = 'mapzen-j16kH4C'

// A web worker is used to render the tiles.
let worker = new Worker('worker.js');
// The returned strings are added as innerHTML to the given element.
worker.addEventListener('message',
  function(e) {
    let elem = document.getElementById(e.data.id);
    // Sometimes after rendering the tile is no longer needed.
    if (elem !== null) {
      elem.innerHTML = e.data.tile;
    } else {
      console.log(`Element ${ e.data.id } was removed.`);
    }
});

// Gets the URL from where to load the vector tile.
// By default it loads all layers, but you can also give it a layer
// name or a list of layers to load.
function getURL(coords, layers='all') {
  if (Array.isArray(layers)) {
    layers = layers.join();
  }
  return `${ base }${ layers }/${ coords.z }/${ coords.x }/${ coords.y }.mvt?api_key=${ api_key }`;
}

// The vector tile layer displays vector tiles in the Leaflet window.
//
// Tiles must be 256x256 pixels in size.
const VectorTileLayer = L.GridLayer.extend({
    createTile: function(coords) {
        let tile = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        tile.setAttribute('width', 256);
        tile.setAttribute('height', 256);
        tile.id = `${ coords.z }-${ coords.x }-${ coords.y }`;
        fetch(getURL(coords), {cache: "force-cache"})
          .then(res => res.blob())
          .then(blob => {
            if (!document.body.contains(tile)) {
              console.log(`The tile ${ tile.id } was removed before the request completed.`);
              return;
            }
            worker.postMessage({
            id: tile.id,
            blob: URL.createObjectURL(blob)})});
        return tile;
    }
});

/// Parses the URL fragment and returns position and zoom.
function parseFragment(standard=null) {
  const expr = /map=(\d+)\/([\d\.]+)\/([\d\.]+)/;
  const match = location.hash.match(expr);
  if (match !== null) {
    return [[Number(match[2]), Number(match[3])], Number(match[1])];
  } else {
    return standard;
  }
}

// Creates the map using the div with the id `map`
// The map is set to the position from the fragment
// or is centered on Frankfurt (Main), Germany.
let map = L.map('main-map');
const view = parseFragment([[50.1, 8.6], 12])
map.setView(view[0], view[1]);
new VectorTileLayer().addTo(map);
// After the map finishes a move (or zoom)
// the fragment is updated with the new location.
map.on('moveend', function(e) {
  const center = map.getCenter();
  const lat = center.lat.toFixed(4);
  const lng = center.lng.toFixed(4);
  const zoom = map.getZoom();
  location.hash = `map=${ zoom }/${ lat }/${ lng }`
})
// If the user changes the fragment the map is updated.
window.onhashchange = function(e) {
  const view = parseFragment();
  if (view !== null) {
    const center = map.getCenter();
    const delta = Math.abs(center.lat - view[0][0])
        + Math.abs(center.lng - view[0][1]);
    if (delta > 0.001 || map.getZoom() !== view[1]) {
      map.setView(view[0], view[1]);
    }
  }
}
