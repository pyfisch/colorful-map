use std::collections::BTreeMap;
use std::fmt::{self, Write};

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
    // Note: Keep track of the total data size to avoid reallocations
    // when painting.
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
    /// Push a single char to the end of the ranks text.
    pub fn push(&mut self, c: char) {
        (*self.size) += 1;
        (*self.selected).push(c);
    }

    /// Push a string to the end of the ranks text.
    pub fn push_str(&mut self, s: &str) {
        (*self.size) += s.len();
        (*self.selected).push_str(s);
    }

    /// Add a formatted string to the end of the text.
    pub fn push_format(&mut self, args: fmt::Arguments) {
        let len = self.selected.len();
        (*self.selected).write_fmt(args)
            .expect("writing a string never fails");
        (*self.size) += self.selected.len() - len;
    }
}

#[test]
fn test_storage() {
    let mut storage = Storage::new();
    {
        let mut rank = storage.select(42);
        rank.push_str("middle rank, ");
    }
    {
        let mut rank = storage.select(17);
        rank.push_str("low ");
        rank.push_str("rank, ");
    }
    {
        let mut rank = storage.select(123);
        rank.push_str("upper rank");
    }
    assert_eq!(String::from(storage).as_str(), "low rank, middle rank, upper rank");
}
