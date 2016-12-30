//! Contains the Feature structure.
//!
//! Features are the basic building block of maps and
//! describe all visible and invisible objects.

use protobuf::{ProtobufError, ProtobufResult};

use cursor::Cursor;
use tag::{TagMap, Value};
use storage::Rank;
use vector_tile::{Tile_Feature, Tile_GeomType as GeomType};

/// A feature consists of a geometry and tagging.
///
/// Additionally it contains information about layer, sort_rank and scale.
#[derive(Debug)]
pub struct Feature<'a, 'k, 'v> {
    id: Option<i64>,
    geom_type: GeomType,
    tags: TagMap<'k, 'v>,
    geometry: &'a [u32],
    layer: &'a str,
    scale: f32,
    /// The rank this feature should be drawn at.
    pub sort_rank: u16,
}

impl<'a, 'k, 'v> Feature<'a, 'k, 'v> {
    /// Creates a new feature.
    pub fn new(raw_feature: &'a Tile_Feature, tags: TagMap<'k, 'v>, layer: &'a str, scale: f32)
            -> ProtobufResult<Feature<'a, 'k, 'v>> {
        let id = (&tags).get("id").and_then(Value::i64);
        // features without sort_rank are usally labels and
        // are displayed above all other content.
        let sort_rank = (&tags).get("sort_rank").and_then(Value::u16).unwrap_or(500);
        Ok(Feature {
            id: id,
            geom_type: raw_feature.get_field_type(),
            tags: tags,
            geometry: raw_feature.get_geometry(),
            layer: layer,
            scale: scale,
            sort_rank: sort_rank})
    }

    /// Compute an SVG fragment for the feature.
    pub fn paint(&mut self, rank: &mut Rank) -> ProtobufResult<()> {
        use vector_tile::Tile_GeomType::*;
        match self.geom_type {
            // Points usually carry texts and need more work.
            POINT => Ok(()),
            LINESTRING => {
                rank.push_str("<path");
                self.paint_metadata(rank)?;
                self.paint_description(rank, false)?;
                rank.push_str("></path>\n");
                Ok(())
            },
            // Note: multi-polygons have holes and are filled with the even-odd rule in SVG.
            POLYGON => {
                rank.push_str("<path");
                self.paint_metadata(rank)?;
                self.paint_description(rank, true)?;
                rank.push_str("></path>\n");
                Ok(())
            },
            // Ignore unknown features.
            UNKNOWN => Ok(()),
        }
    }

    fn paint_metadata(&self, rank: &mut Rank)
            -> ProtobufResult<()> {
        // class="kind-{} (boundary)? min-zoom-{}" (data-id="{}")?
        // FIXME: Malicious map tiles can do XSS.
        if self.id == Some(7996457) {
            println!("{:?}", self.tags);
        }
        rank.push_format(format_args!(" class=\"layer-{} kind-{}",
            self.layer,
            self.tags.get("kind").and_then(Value::str)
                .ok_or_else(|| ProtobufError::WireError("kind is required".to_owned()))?));
        if self.tags.get("boundary").map_or(false, Value::yes) {
            rank.push_str(" boundary");
        }
        if self.tags.get("is_tunnel").map_or(false, Value::yes) {
            rank.push_str(" is_tunnel");
        }
        if self.tags.get("is_bridge").map_or(false, Value::yes) {
            rank.push_str(" is_bridge");
        }
        rank.push_format(format_args!(" min-zoom-{}",
            self.tags.get("min_zoom").and_then(Value::f32).unwrap_or(0f32).floor()));
        rank.push('"');
        if let Some(id) = self.id {
            rank.push_format(format_args!(" data-id=\"{}\"", id));
        }
        Ok(())
    }

    fn paint_description(&mut self, rank: &mut Rank, close_path: bool)
                -> ProtobufResult<()> {
        use cursor::Command::*;
        rank.push_str(" d=\"");
        for command in Cursor::new(self.geometry, self.scale) {
            match command {
                Ok(MoveTo(x, y)) => rank.push_format(
                    format_args!("M {} {} ", x, y)),
                Ok(LineTo(x, y)) => rank.push_format(
                    format_args!("L {} {} ", x, y)),
                Ok(ClosePath) => if close_path {
                    rank.push_str("Z ");
                } else {
                    return Err(ProtobufError::WireError("close path not allowed".to_owned()))
                },
                Err(e) => return Err(e),
            }
        }
        rank.push_str("\"");
        Ok(())
    }
}
