use protobuf::{ProtobufError, ProtobufResult};

/// Chains of commands form paths and polygons.
///
/// (0, 0) is in the upper left corner of the tile.
/// All coordinates are relative.
pub enum Command {
    /// Move the cursor (x, y) steps.
    MoveTo(i32, i32),
    /// Draw a line (x, y) steps.
    LineTo(i32, i32),
    /// Return to the first point of the polygon.
    ClosePath,
}

/// A cursor iterates over a geometry (line or polygon).
pub struct Cursor<'a> {
    geometry: &'a [u32],
    pos: usize,
    id: u32,
    count: u32,
}

impl<'a> Cursor<'a> {
    /// Get a cursor for the geometry.
    pub fn new(geometry: &'a[u32]) -> Cursor<'a> {
        Cursor {
            geometry: geometry,
            pos: 0,
            id: 0,
            count: 0,
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
        if self.pos >= self.geometry.len() {
            return None;
        }
        if self.count == 0 {
            self.id = self.geometry[self.pos] & 0x7;
            self.count = self.geometry[self.pos] >> 3;
            self.pos += 1;
        }
        self.count -= 1;
        match self.id {
            1 | 2 if self.pos + 2 > self.geometry.len() => {
                self.pos = ::std::usize::MAX;
                Some(Err(ProtobufError::WireError(
                    format!("mvt: Expected at least two remaining integers in geometry.")
                )))
            }
            1 => {
                let x = de_zigzag(self.geometry[self.pos]);
                let y = de_zigzag(self.geometry[self.pos + 1]);
                self.pos += 2;
                Some(Ok(MoveTo(x, y)))
            },
            2 => {
                let x = de_zigzag(self.geometry[self.pos]);
                let y = de_zigzag(self.geometry[self.pos + 1]);
                self.pos += 2;
                Some(Ok(LineTo(x, y)))
            }
            7 => {
                Some(Ok(ClosePath))
            }
            _ => {
                self.pos = ::std::usize::MAX;
                Some(Err(ProtobufError::WireError(
                    format!("mvt: command integer, expected 1, 2 or 7, found {}", self.id)
                )))
            }
        }
    }
}
