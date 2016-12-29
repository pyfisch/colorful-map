use protobuf::{ProtobufError, ProtobufResult};

/// Chains of commands form paths and polygons.
///
/// (0, 0) is in the upper left corner of the tile.
/// All coordinates are abssolute.
pub enum Command {
    /// Move the cursor to point (x, y)
    MoveTo(i32, i32),
    /// Draw a line to the given point.
    LineTo(i32, i32),
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
}

impl<'a> Cursor<'a> {
    /// Get a cursor for the geometry.
    pub fn new(geometry: &'a[u32]) -> Cursor<'a> {
        Cursor {
            geometry: geometry,
            id: 0,
            count: 0,
            x: 0,
            y: 0,
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
        use Command::*;
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
            1 => {
                self.x += de_zigzag(self.geometry[0]);
                self.y += de_zigzag(self.geometry[1]);
                self.geometry = &self.geometry[2..];
                Some(Ok(MoveTo(self.x, self.y)))
            },
            2 => {
                self.x += de_zigzag(self.geometry[0]);
                self.y += de_zigzag(self.geometry[1]);
                self.geometry = &self.geometry[2..];
                Some(Ok(LineTo(self.x, self.y)))
            }
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
