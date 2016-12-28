const base = 'https://tile.mapzen.com/mapzen/vector/v1';
const process_web = Module.cwrap('process_web', 'string', ['number', 'number']);

var mymap = L.map('mapid').setView([50, 8], 11);

var SvgLayer = L.GridLayer.extend({
    createTile: function(coords){
        let tile = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        const size = this.getTileSize();
        tile.setAttribute('width', size.x);
        tile.setAttribute('height', size.y);
        fetch(`${ base }/${ ['all'].join() }/${ coords.z }/${ coords.x }/${ coords.y }.mvt?api_key=mapzen-j16kH4C`)
          .then(function (res) {
            return res.arrayBuffer();
          }).then(function (buffer) {
            let data = new Uint8Array(buffer);
            let p = Module._malloc(data.length);
            Module.writeArrayToMemory(data, p);
            tile.innerHTML = process_web(p, data.length);
          });
        return tile;
    }
});

new SvgLayer().addTo(mymap);
