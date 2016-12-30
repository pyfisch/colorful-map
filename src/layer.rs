//! Layers group features of similiar type and use together.
//!
//! Layers are a storage unit in MVT files.
use protobuf::{ProtobufResult, ProtobufError};

use feature::Feature;
use storage::Storage;
use tag::{TagMap, Value};
use vector_tile::Tile_Layer;

/// Contains a layer and its scale.
#[derive(Debug)]
pub struct Layer<'l> {
    inner: &'l Tile_Layer,
    // scale
    scale: f32,
}

impl<'l> Layer<'l> {
    /// Creates a new layer from a Tile_Layer.
    pub fn new(raw_layer: &'l Tile_Layer) -> Layer<'l> {
        Layer {
            inner: raw_layer,
            scale: 256f32 / raw_layer.get_extent() as f32,
        }
    }

    /// Decodes the tags of a feature using the layers dictionary.
    pub fn get_tags(&self, tags: &[u32])
            -> ProtobufResult<TagMap> {
        if tags.len() % 2 != 0 {
            return Err(ProtobufError::WireError(
                "mvt: A tag list must be an even number of integers.".to_owned()));
        }
        let mut map = TagMap::new();
        let keys = self.inner.get_keys();
        let values = self.inner.get_values();
        for i in 0..(tags.len() / 2) {
            let k = tags[i * 2] as usize;
            let v = tags[i * 2 + 1] as usize;
            if k >= keys.len() || v >= values.len() {
                return Err(ProtobufError::WireError(
                    "mvt: There is no such tag key/value.".to_owned()));
            }
            map.insert(keys[k].as_str(), Value::from_tile_value(&values[v])?);
        }
        Ok(map)
    }

    /// Paints all features in the tile to the given storage.
    pub fn paint(&mut self, storage: &mut Storage) -> ProtobufResult<()> {
        for raw_feature in self.inner.get_features() {
            let mut feature = Feature::new(
                raw_feature,
                self.get_tags(raw_feature.get_tags())?,
                self.inner.get_name(),
                self.scale)?;
            let mut rank = storage.select(feature.sort_rank);
            feature.paint(&mut rank)?;
        }
        Ok(())
    }
}
