use chrono::DateTime;
use roxmltree::Document;

use crate::errors::{FileAttributeKind, ParseNzbError};
use crate::parity::Parity;
use crate::subject;
use crate::{File, Files, Segment};

/// Parse all `<file>` elements from an NZB Document.
///
/// Each `<file>` must contain:
/// - `poster`, `date`, and `subject` attributes
/// - at least one `<group>` inside `<groups>`
/// - at least one `<segment>` inside `<segments>`
///
/// Segments missing required attributes (`bytes`, `number`) or message IDs
/// are skipped rather than causing a hard error.
pub(crate) fn parse_files(nzb: &Document) -> Result<(Files, Parity), ParseNzbError> {
    let mut files = Vec::new();

    for node in nzb.root_element().children().filter(|n| n.has_tag_name("file")) {
        let poster = node
            .attribute("poster")
            .ok_or(ParseNzbError::FileAttribute(FileAttributeKind::Poster))?;

        let posted_at = node
            .attribute("date") // Unix timestamp in seconds
            .and_then(|d| d.parse::<i64>().ok())
            .and_then(|d| DateTime::from_timestamp(d, 0))
            .ok_or(ParseNzbError::FileAttribute(FileAttributeKind::Date))?;

        let subject = node
            .attribute("subject")
            .ok_or(ParseNzbError::FileAttribute(FileAttributeKind::Subject))?;

        let mut groups = Vec::new();
        let mut segments = Vec::new();

        for child in node.children() {
            match child.tag_name().name() {
                "groups" => {
                    for group in child.children().filter(|n| n.has_tag_name("group")) {
                        if let Some(text) = group.text()
                            && !text.is_empty()
                        {
                            groups.push(text.to_owned());
                        }
                    }
                }

                "segments" => {
                    for segment in child.children().filter(|n| n.has_tag_name("segment")) {
                        // Message-ID text is required and must be non-empty.
                        if let Some(message_id) = segment.text()
                            && !message_id.is_empty()
                        {
                            // Article size is typically ~700KB and safely fits in u32.
                            let Some(size) = segment.attribute("bytes").and_then(|bytes| bytes.parse::<u32>().ok())
                            else {
                                continue;
                            };
                            let Some(number) = segment
                                .attribute("number")
                                .and_then(|number| number.parse::<u32>().ok())
                            else {
                                continue;
                            };

                            segments.push(Segment::new(size, number, message_id));
                        }
                    }
                }
                _ => continue,
            }
        }

        // A file must belong to at least one group.
        if groups.is_empty() {
            return Err(ParseNzbError::GroupsElement);
        }

        // A file must contain at least one valid segment.
        if segments.is_empty() {
            return Err(ParseNzbError::SegmentsElement);
        }

        // Sort for consistency
        groups.sort_unstable();
        segments.sort_unstable_by_key(|segment| segment.number());

        files.push(File::new(poster, posted_at, subject, groups, segments));
    }

    files.sort_unstable_by(|a, b| {
        let ka = subject::file_number(a.subject());
        let kb = subject::file_number(b.subject());
        ka.cmp(&kb).then_with(|| a.subject().cmp(b.subject()))
    });

    if files.is_empty() {
        return Err(ParseNzbError::FileElement);
    }

    let (payload, parity): (Vec<_>, Vec<_>) = files.into_iter().partition(|file| !file.is_par2());

    if payload.is_empty() {
        return Err(ParseNzbError::OnlyPar2Files);
    }

    Ok((Files::from_payload_vec(payload)?, Parity::from_vec(parity)))
}
