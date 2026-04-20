use std::collections::BTreeSet;
use std::slice;

use crate::{File, ParseNzbError};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Read-only collection of non-`.par2` [`File`] entries contained in an NZB.
///
/// This collection is guaranteed to contain at least one file, which makes
/// [`Files::primary`] infallible.
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
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns the total size of all files in the collection.
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

    /// Returns [`true`] if any file in the collection has the specified extension.
    ///
    /// This method ensures consistent extension comparison by normalizing the
    /// extension (removing any leading dot) and handling case-folding.
    pub fn has_extension(&self, ext: impl AsRef<str>) -> bool {
        self.iter().any(|file| file.has_extension(ext.as_ref()))
    }

    /// Returns [`true`] if any file in the collection is a `.rar` file.
    #[must_use]
    pub fn has_rar(&self) -> bool {
        self.iter().any(File::is_rar)
    }

    /// Returns [`true`] if every file in the collection is a `.rar` file.
    #[must_use]
    pub fn is_rar(&self) -> bool {
        self.iter().all(File::is_rar)
    }

    /// Returns [`true`] if any file in the collection appears obfuscated.
    #[must_use]
    pub fn is_obfuscated(&self) -> bool {
        self.iter().any(File::is_obfuscated)
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
            return Err(serde::de::Error::custom("Files must contain non-`.par2` entries only"));
        }

        Self::from_payload_vec(files).map_err(serde::de::Error::custom)
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
    fn test_len_is_non_zero() {
        let files = Files::from_payload_vec(vec![make_file(r#""example.mkv""#)]).unwrap();
        assert_eq!(files.len(), 1);
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
    fn test_files_collection_helpers() {
        let files = Files::from_payload_vec(vec![
            File::new(
                "poster@example",
                DateTime::from_timestamp(1_706_440_708, 0).unwrap(),
                r#""Big Buck Bunny - S01E01.part01.rar""#,
                ["alt.binaries.example", "alt.binaries.backup"],
                [Segment::new(10, 1, "example-1@example")],
            ),
            File::new(
                "poster@example",
                DateTime::from_timestamp(1_706_440_709, 0).unwrap(),
                r#""Big Buck Bunny - S01E01.part02.r00""#,
                ["alt.binaries.example"],
                [Segment::new(20, 2, "example-2@example")],
            ),
        ])
        .unwrap();

        assert_eq!(files.len(), 2);
        assert_eq!(files.size(), 30);
        assert_eq!(
            files.names().collect::<Vec<_>>(),
            vec![
                "Big Buck Bunny - S01E01.part01.rar",
                "Big Buck Bunny - S01E01.part02.r00",
            ]
        );
        assert_eq!(files.posters().collect::<Vec<_>>(), vec!["poster@example"]);
        assert_eq!(
            files.groups().collect::<Vec<_>>(),
            vec!["alt.binaries.backup", "alt.binaries.example"]
        );
        assert!(files.has_extension("rar"));
        assert!(files.has_extension(".R00"));
        assert!(!files.has_extension("par2"));
        assert!(files.has_rar());
        assert!(files.is_rar());
        assert!(!files.is_obfuscated());
    }

    #[test]
    fn test_files_collection_obfuscated_and_not_all_rar() {
        let files = Files::from_payload_vec(vec![
            make_file(r#""7c1f5e5d7f2a4b1aa4d6f91d93c2b8e1.mkv""#),
            make_file(r#""bonus.nfo""#),
        ])
        .unwrap();

        assert!(files.is_obfuscated());
        assert!(!files.has_rar());
        assert!(!files.is_rar());
    }
}
