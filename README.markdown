colorful-map
============

Online demo: https://pyfisch.org/stuff/colorful-map/

First clone the repository. Emscripten needs to be installed.

Building and running colerful-map:

```
$ cargo build --target asmjs-unknown-emscripten --release
$ cp target/asmjs-unknown-emscripten/release/colorful-map.js web/
$ cd web
$ python3 -m http.server
```

Go to `http://localhost:8000` in your favourite browser and try the map!

Links
-----

* [Mapzen Vector Tiles Service](https://mapzen.com/documentation/vector-tiles/)
  is the data source used for this crate.
* [Leaflet.js](http://leafletjs.com) provides the UI for the vector tiles.
  It supports moving, zooming, etc.
* [The Rust FFI omnibus](http://jakegoulding.com/rust-ffi-omnibus/) helped
  me to write the functions callled from JavaScript.
  (Mainly articles slice arguments and string return values)
* [Emscripten](https://kripken.github.io/emscripten-site/) used to compile
  Rust for the browser.
* [Compiling Rust to your Browser](http://www.hellorust.com/emscripten/)
  Introduction into Rust and Emscripten from Rust Belt Rust 2016 by badboy
