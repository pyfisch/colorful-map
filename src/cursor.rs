//! The cursor interprets a geometry and returns a chain of commands.

use protobuf::{ProtobufError, ProtobufResult};

/// Chains of commands form paths and polygons.
///
/// (0, 0) is in the upper left corner of the tile.
/// All coordinates are abssolute.
pub enum Command {
    /// Move the cursor to point (x, y)
    MoveTo(f32, f32),
    /// Draw a line to the given point.
    LineTo(f32, f32),
    /// Return to the first point of the polygon.
    ClosePath,
}

/// A cursor iterates over a geometry (line or polygon).
pub struct Cursor<'a> {
    geometry: &'a [u32],
    id: u32,
    count: u32,
    x: i32,
    y: i32,
    scale: f32,
}

impl<'a> Cursor<'a> {
    /// Get a cursor for the geometry.
    pub fn new(geometry: &'a[u32], scale: f32) -> Cursor<'a> {
        Cursor {
            geometry: geometry,
            id: 0,
            count: 0,
            x: 0,
            y: 0,
            scale: scale,
        }
    }
}

/// Decode a 32bit integer according to the protobuf zigzag rules.
///
/// Used for map tile parameter integers.
pub fn de_zigzag(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 1) as i32))
}

impl<'a> Iterator for Cursor<'a> {
    type Item = ProtobufResult<Command>;

    fn next(&mut self) -> Option<ProtobufResult<Command>> {
        use self::Command::*;
        if self.geometry.is_empty() {
            return None;
        }
        if self.count == 0 {
            let (command, geometry) = self.geometry.split_first().unwrap();
            self.geometry = geometry;
            self.id = command & 0x7;
            self.count = command >> 3;
        }
        self.count -= 1;
        match self.id {
            1 | 2 if self.geometry.len() < 2 => {
                self.geometry = Default::default();
                Some(Err(ProtobufError::WireError(
                    format!("mvt: Expected at least two remaining integers in geometry.")
                )))
            }
            1 | 2 => {
                self.x += de_zigzag(self.geometry[0]);
                self.y += de_zigzag(self.geometry[1]);
                self.geometry = &self.geometry[2..];
                let x = self.x as f32 * self.scale;
                let y = self.y as f32 * self.scale;
                Some(Ok(if self.id == 1 {
                    MoveTo(x, y)
                } else {
                    LineTo(x, y)
                }))

            },
            7 => Some(Ok(ClosePath)),
            _ => {
                self.geometry = Default::default();
                Some(Err(ProtobufError::WireError(
                    format!("mvt: command integer, expected 1, 2 or 7, found {}", self.id)
                )))
            }
        }
    }
}
