use std::collections::BTreeSet;
use std::slice;

use crate::File;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Read-only collection of `.par2` [`File`] entries contained in an NZB.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parity {
    files: Vec<File>,
}

impl Parity {
    pub(crate) fn from_vec(files: Vec<File>) -> Self {
        debug_assert!(files.iter().all(File::is_par2));
        Self { files }
    }

    /// Returns an iterator over parity files in the collection.
    pub fn iter(&self) -> slice::Iter<'_, File> {
        self.files.iter()
    }

    /// Returns [`true`] if the collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the number of parity files in the collection.
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns the total size of all parity files in the collection.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.iter().map(File::size).sum()
    }

    /// Returns unique file names in ascending sorted order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.iter().filter_map(File::name).collect::<BTreeSet<_>>().into_iter()
    }

    /// Returns unique posters in ascending sorted order.
    pub fn posters(&self) -> impl Iterator<Item = &str> {
        self.iter().map(File::poster).collect::<BTreeSet<_>>().into_iter()
    }

    /// Returns unique groups in ascending sorted order.
    pub fn groups(&self) -> impl Iterator<Item = &str> {
        self.iter()
            .flat_map(|file| file.groups().iter().map(String::as_str))
            .collect::<BTreeSet<_>>()
            .into_iter()
    }
}

impl<'a> IntoIterator for &'a Parity {
    type Item = &'a File;
    type IntoIter = slice::Iter<'a, File>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(feature = "serde")]
impl Serialize for Parity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.files.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Parity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let files = Vec::<File>::deserialize(deserializer)?;
        if files.iter().any(|file| !file.is_par2()) {
            return Err(serde::de::Error::custom("Parity must contain `.par2` files only"));
        }

        Ok(Self::from_vec(files))
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use super::*;
    use crate::Segment;

    fn make_file(subject: &str) -> File {
        File::new(
            "poster@example",
            DateTime::from_timestamp(1_706_440_708, 0).unwrap(),
            subject,
            ["alt.binaries.example"],
            [Segment::new(10, 1, "message@example")],
        )
    }

    #[test]
    fn test_parity_collection() {
        let parity = Parity::from_vec(vec![
            make_file(r#""example.mkv.par2""#),
            File::new(
                "poster@example",
                DateTime::from_timestamp(1_706_440_708, 0).unwrap(),
                r#""example.mkv.vol00+01.par2""#,
                ["alt.binaries.example", "alt.binaries.backup"],
                [Segment::new(20, 1, "large@example")],
            ),
        ]);

        assert!(!parity.is_empty());
        assert_eq!(parity.len(), 2);
        assert_eq!(parity.size(), 30);
        assert_eq!(
            parity.names().collect::<Vec<_>>(),
            vec!["example.mkv.par2", "example.mkv.vol00+01.par2"]
        );
        assert_eq!(parity.posters().collect::<Vec<_>>(), vec!["poster@example"]);
        assert_eq!(
            parity.groups().collect::<Vec<_>>(),
            vec!["alt.binaries.backup", "alt.binaries.example"]
        );
    }
}
