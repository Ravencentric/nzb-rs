use chrono::{DateTime, Utc};

use crate::segment::Segment;
use crate::subject;

/// Represents a single file, consisting of segments that make up a file.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct File {
    poster: String,
    posted_at: DateTime<Utc>,
    subject: String,
    groups: Vec<String>,
    segments: Vec<Segment>,
}

impl File {
    /// Creates a new [`File`] instance.
    #[must_use]
    pub fn new(
        poster: impl Into<String>,
        posted_at: impl Into<DateTime<Utc>>,
        subject: impl Into<String>,
        groups: impl IntoIterator<Item = impl Into<String>>,
        segments: impl IntoIterator<Item = Segment>,
    ) -> Self {
        Self {
            poster: poster.into(),
            posted_at: posted_at.into(),
            subject: subject.into(),
            groups: groups.into_iter().map(Into::into).collect(),
            segments: segments.into_iter().collect(),
        }
    }

    /// Poster of the file.
    #[must_use]
    pub fn poster(&self) -> &str {
        &self.poster
    }

    /// Date and time the file was posted, in UTC.
    #[must_use]
    pub fn posted_at(&self) -> &DateTime<Utc> {
        &self.posted_at
    }

    /// Subject associated with the file.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Usenet groups listed for the file.
    #[must_use]
    pub fn groups(&self) -> &[String] {
        &self.groups
    }

    /// Segments that make up the file.
    #[must_use]
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Size of the file calculated from the sum of segment sizes.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.segments.iter().map(|x| u64::from(x.size())).sum::<u64>()
    }

    /// Complete name of the file with it's extension extracted from the subject.
    /// May return [`None`] if it fails to extract the name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        subject::file_name(&self.subject)
    }

    /// Base name of the file without it's extension extracted from the [`File::name`].
    /// May return [`None`] if it fails to extract the stem.
    #[must_use]
    pub fn stem(&self) -> Option<&str> {
        self.name().map(subject::file_stem)
    }

    ///  Extension of the file extracted from the [`File::name`].
    /// May return [`None`] if it fails to extract the extension.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.name().and_then(subject::file_extension)
    }

    /// Return [`true`] if the file has the specified extension, [`false`] otherwise.
    ///
    /// This method ensures consistent extension comparison
    /// by normalizing the extension (removing any leading dot) and handling case-folding.
    pub fn has_extension(&self, ext: impl AsRef<str>) -> bool {
        let ext = ext.as_ref().strip_prefix('.').unwrap_or_else(|| ext.as_ref()).trim();
        self.extension()
            .is_some_and(|file_ext| file_ext.eq_ignore_ascii_case(ext))
    }

    /// Return [`true`] if the file is a `.par2` file, [`false`] otherwise.
    #[must_use]
    pub fn is_par2(&self) -> bool {
        self.name().is_some_and(subject::is_par2)
    }

    /// Return [`true`] if the file is a `.rar` file, [`false`] otherwise.
    #[must_use]
    pub fn is_rar(&self) -> bool {
        self.name().is_some_and(subject::is_rar)
    }

    /// Return [`true`] if the file is obfuscated, [`false`] otherwise.
    pub fn is_obfuscated(&self) -> bool {
        self.stem().is_none_or(subject::is_obfuscated)
    }
}
