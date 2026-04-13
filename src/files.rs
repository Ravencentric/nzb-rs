use std::num::NonZeroUsize;
use std::slice;

use crate::{File, ParseNzbError};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Read-only collection of [`File`] entries contained in an NZB.
///
/// This collection is guaranteed to contain at least one non-`.par2` file,
/// which makes [`Files::primary`] infallible.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Files {
    files: Vec<File>,
    primary_index: usize,
}

impl Files {
    pub(crate) fn from_payload_vec(files: Vec<File>) -> Result<Self, ParseNzbError> {
        debug_assert!(files.iter().all(|file| !file.is_par2()));

        if files.is_empty() {
            return Err(ParseNzbError::FileElement);
        }

        let Some((primary_index, _)) = files.iter().enumerate().max_by_key(|(_, file)| file.size()) else {
            return Err(ParseNzbError::FileElement);
        };

        Ok(Self { files, primary_index })
    }

    /// Returns the primary content file (episode, movie, etc.) in the NZB.
    ///
    /// This is determined by finding the largest non-`.par2` file in the
    /// collection and may not always be accurate.
    #[must_use]
    pub fn primary(&self) -> &File {
        &self.files[self.primary_index]
    }

    /// Returns an iterator over the files in the collection.
    pub fn iter(&self) -> slice::Iter<'_, File> {
        self.files.iter()
    }

    /// Returns the number of files in the collection.
    #[must_use]
    pub fn count(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.files.len())
            .expect("Files invariant violated: collection must contain at least one file")
    }

    #[must_use]
    pub(crate) fn size(&self) -> u64 {
        self.iter().map(File::size).sum()
    }
}

/// Read-only parity file view for an NZB.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parity<'a> {
    files: &'a [File],
    total_size: u64,
}

impl<'a> Parity<'a> {
    pub(crate) fn new(files: &'a ParityFiles, total_size: u64) -> Self {
        Self {
            files: &files.files,
            total_size,
        }
    }

    /// Returns an iterator over parity files in the collection.
    pub fn iter(&self) -> slice::Iter<'a, File> {
        self.files.iter()
    }

    /// Returns [`true`] if the collection contains any parity files.
    #[must_use]
    pub fn any(&self) -> bool {
        !self.files.is_empty()
    }

    /// Returns the number of parity files.
    #[must_use]
    pub fn count(&self) -> usize {
        self.files.len()
    }

    /// Returns the total size of all parity files.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.iter().map(File::size).sum()
    }

    /// Returns the percentage of parity bytes relative to the full NZB size.
    #[must_use]
    pub fn percentage(&self) -> f64 {
        if self.total_size == 0 {
            return 0.0;
        }

        (self.size() as f64 / self.total_size as f64) * 100.0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ParityFiles {
    files: Vec<File>,
}

impl ParityFiles {
    pub(crate) fn from_vec(files: Vec<File>) -> Self {
        debug_assert!(files.iter().all(File::is_par2));
        Self { files }
    }

    pub(crate) fn iter(&self) -> slice::Iter<'_, File> {
        self.files.iter()
    }

    pub(crate) fn size(&self) -> u64 {
        self.iter().map(File::size).sum()
    }
}

impl<'a> IntoIterator for &'a Files {
    type Item = &'a File;
    type IntoIter = slice::Iter<'a, File>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(feature = "serde")]
impl Serialize for Files {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.files.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Files {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let files = Vec::<File>::deserialize(deserializer)?;
        if files.is_empty() {
            return Err(serde::de::Error::custom(ParseNzbError::FileElement));
        }
        if files.iter().any(File::is_par2) {
            return Err(serde::de::Error::custom(
                "Files must contain payload files only and cannot include `.par2` entries",
            ));
        }

        Self::from_payload_vec(files).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl Serialize for ParityFiles {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.files.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ParityFiles {
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
    fn test_count_is_non_zero() {
        let files = Files::from_payload_vec(vec![make_file(r#""example.mkv""#)]).unwrap();
        assert_eq!(files.count().get(), 1);
    }

    #[test]
    fn test_rejects_empty_payload_files() {
        assert_eq!(Files::from_payload_vec(vec![]).unwrap_err(), ParseNzbError::FileElement);
    }

    #[test]
    fn test_primary_returns_largest_payload_file() {
        let files = Files::from_payload_vec(vec![
            File::new(
                "poster@example",
                DateTime::from_timestamp(1_706_440_708, 0).unwrap(),
                r#""small.mkv""#,
                ["alt.binaries.example"],
                [Segment::new(10, 1, "small@example")],
            ),
            File::new(
                "poster@example",
                DateTime::from_timestamp(1_706_440_708, 0).unwrap(),
                r#""large.mkv""#,
                ["alt.binaries.example"],
                [Segment::new(20, 1, "large@example")],
            ),
        ])
        .unwrap();

        assert_eq!(files.primary().name(), Some("large.mkv"));
    }

    #[test]
    fn test_parity_collection() {
        let stored = ParityFiles::from_vec(vec![
            make_file(r#""example.mkv.par2""#),
            File::new(
                "poster@example",
                DateTime::from_timestamp(1_706_440_708, 0).unwrap(),
                r#""example.mkv.vol00+01.par2""#,
                ["alt.binaries.example"],
                [Segment::new(20, 1, "large@example")],
            ),
        ]);
        let parity = Parity::new(&stored, 100);

        assert!(parity.any());
        assert_eq!(parity.count(), 2);
        assert_eq!(parity.size(), 30);
        assert_eq!(parity.percentage(), 30.0);
    }
}
