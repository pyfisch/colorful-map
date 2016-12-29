let worker = new Worker('worker.js');
worker.addEventListener('message',
  function(e) {
    let elem = document.getElementById(e.data.id);
    // Sometimes after rendering the tile is no longer needed.
    if (elem !== null) {
      elem.innerHTML = e.data.tile
    }
});

// The vector tile layer displays vector tiles in the Leaflet window.
//
// Tiles must be 256x256 pixels in size.
const VectorTileLayer = L.GridLayer.extend({
    createTile: function(coords){
        let tile = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        tile.setAttribute('width', 256);
        tile.setAttribute('height', 256);
        tile.id = `${ coords.z }-${ coords.x }-${ coords.y }`;
        worker.postMessage({id: tile.id, coords: coords});
        return tile;
    }
});

// Creates the map using the div with the id `map`
let map = L.map('main-map').setView([50, 8], 11);
new VectorTileLayer().addTo(map);
