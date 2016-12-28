use std::collections::BTreeMap;

/// Stores the visualization of a map tile.
///
/// A map tile has many different 'sort_ranks'. These ranks consist
/// of geographic features like points, lines and polygons and all
/// features on the same rank should be painted at the same time.
/// So features with a low rank are painted first and later higher
/// ranking features are painted and partially cover the ones painted
/// before.
/// This storage stores a string for each rank used. One can push
/// new data to each rank. In the end the data is serialized in the
/// correct order.
#[derive(Debug)]
pub struct Storage {
    // Note: BTreeMap is used because the storage needs to be
    // iterated in correct order when "painting".
    data: BTreeMap<u16, String>,
    size: usize,
}

impl Storage {
    /// Creates a new, empty storage.
    pub fn new() -> Storage {
        Storage {
            data: BTreeMap::new(),
            size: 0,
        }
    }

    /// Selects a 'sort_rank' and returns it for editing.
    ///
    /// If there is no string for the given rank it is created.
    pub fn select(&mut self, sort_rank: u16) -> Rank {
        if !self.data.contains_key(&sort_rank) {
            self.data.insert(sort_rank, String::new());
        }
        Rank {
            selected: self.data.get_mut(&sort_rank)
                               .expect("The map is not empty for the given rank."),
            size: &mut self.size
        }
    }
}

// Serialize ("Paint") the storage.
impl From<Storage> for String {
    fn from(storage: Storage) -> String {
        let mut s = String::with_capacity(storage.size);
        for (_, value) in storage.data.iter() {
            s.push_str(value.as_str());
        }
        s
    }
}

/// A rank represents one one sort_rank on the map.
///
/// Add data to the rank. At any time only one rank can be edited.
#[derive(Debug)]
pub struct Rank<'a> {
    selected: &'a mut String,
    size: &'a mut usize,
}

impl<'a> Rank<'a> {
    /// Push a string to the end of the ranks text.
    pub fn push_str(&mut self, s: &str) {
        (*self.size) += s.len();
        (*self.selected).push_str(s);
    }

    /// Push a single char to the end of the ranks text.
    pub fn push(&mut self, c: char) {
        (*self.size) += 1;
        (*self.selected).push(c);
    }
}
