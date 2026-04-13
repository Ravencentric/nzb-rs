/// Represents a single segment of a file in an NZB.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Segment {
    size: u32,
    number: u32,
    message_id: String,
}

impl Segment {
    /// Creates a new [`Segment`] instance.
    #[must_use]
    pub fn new(size: u32, number: u32, message_id: impl Into<String>) -> Self {
        Self {
            size,
            number,
            message_id: message_id.into(),
        }
    }

    /// Size of the segment in bytes.
    #[must_use]
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Sequence number of the segment within the file.
    #[must_use]
    pub fn number(&self) -> u32 {
        self.number
    }

    /// `Message-ID` of the segment.
    #[must_use]
    pub fn message_id(&self) -> &str {
        &self.message_id
    }
}
