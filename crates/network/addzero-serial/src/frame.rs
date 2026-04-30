//! Frame decoding for serial data streams.
//!
//! Supports common industrial protocols: fixed-length, delimiter-based,
//! and length-prefixed frames.

use serde::{Deserialize, Serialize};

/// The format used to delimit frames in a byte stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameFormat {
    /// Fixed-length frames (every frame is exactly N bytes).
    FixedLength(usize),
    /// Delimiter-terminated frames (e.g., `\r\n` or `0xAA 0x55`).
    Delimiter(Vec<u8>),
    /// Length-prefixed frames: a header of N bytes encodes the payload length.
    LengthPrefixed {
        /// Number of bytes in the length field (1, 2, or 4).
        length_bytes: usize,
        /// Whether the length includes the header itself.
        length_includes_header: bool,
    },
}

/// Events produced by the frame decoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameEvent {
    /// A complete frame was decoded.
    Frame(Vec<u8>),
    /// The internal buffer overflowed — frame discarded.
    Overflow,
}

/// Incremental frame decoder.
///
/// Feed bytes in, get complete frames out. Useful for streaming serial data.
///
/// # Examples
///
/// ```
/// use addzero_serial::{FrameDecoder, FrameFormat, FrameEvent};
///
/// let mut decoder = FrameDecoder::new(FrameFormat::Delimiter(vec![0x0A]));
/// decoder.push(b"Hello");
/// assert!(decoder.poll().is_none());
/// decoder.push(b"\n");
/// assert_eq!(decoder.poll(), Some(FrameEvent::Frame(b"Hello".to_vec())));
/// ```
pub struct FrameDecoder {
    format: FrameFormat,
    buffer: Vec<u8>,
    max_buffer_size: usize,
}

impl FrameDecoder {
    /// Create a new frame decoder with the given format.
    pub fn new(format: FrameFormat) -> Self {
        Self {
            format,
            buffer: Vec::with_capacity(1024),
            max_buffer_size: 8192,
        }
    }

    /// Create a new decoder with a custom maximum buffer size.
    pub fn with_max_buffer(mut self, max: usize) -> Self {
        self.max_buffer_size = max;
        self
    }

    /// Push incoming bytes into the decoder.
    pub fn push(&mut self, data: &[u8]) {
        if self.buffer.len() + data.len() > self.max_buffer_size {
            self.buffer.clear();
            return;
        }
        self.buffer.extend_from_slice(data);
    }

    /// Try to extract the next complete frame from the buffer.
    ///
    /// Returns `Some(FrameEvent::Frame(data))` if a complete frame is available,
    /// `Some(FrameEvent::Overflow)` if the buffer was too large,
    /// or `None` if no complete frame is available yet.
    pub fn poll(&mut self) -> Option<FrameEvent> {
        if self.buffer.len() > self.max_buffer_size {
            self.buffer.clear();
            return Some(FrameEvent::Overflow);
        }

        match &self.format {
            FrameFormat::FixedLength(len) => {
                if self.buffer.len() >= *len {
                    Some(FrameEvent::Frame(self.buffer.drain(..*len).collect()))
                } else {
                    None
                }
            }
            FrameFormat::Delimiter(delim) => {
                if delim.is_empty() {
                    return None;
                }
                let pos = find_subsequence(&self.buffer, delim)?;
                let frame: Vec<u8> = self.buffer.drain(..pos).collect();
                // Remove the delimiter
                self.buffer.drain(..delim.len());
                Some(FrameEvent::Frame(frame))
            }
            FrameFormat::LengthPrefixed {
                length_bytes,
                length_includes_header,
            } => {
                if self.buffer.len() < *length_bytes {
                    return None;
                }
                let len_val = match *length_bytes {
                    1 => self.buffer[0] as usize,
                    2 => u16::from_be_bytes([self.buffer[0], self.buffer[1]]) as usize,
                    4 => u32::from_be_bytes([
                        self.buffer[0],
                        self.buffer[1],
                        self.buffer[2],
                        self.buffer[3],
                    ]) as usize,
                    _ => return None,
                };
                let payload_len = if *length_includes_header {
                    len_val.saturating_sub(*length_bytes)
                } else {
                    len_val
                };
                let total = *length_bytes + payload_len;
                if self.buffer.len() >= total {
                    let payload: Vec<u8> = self.buffer.drain(*length_bytes..total).collect();
                    Some(FrameEvent::Frame(payload))
                } else {
                    None
                }
            }
        }
    }

    /// Clear the internal buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the current number of bytes in the buffer.
    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }
}

/// Find the first occurrence of `needle` in `haystack`.
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_length_frame() {
        let mut dec = FrameDecoder::new(FrameFormat::FixedLength(3));
        dec.push(&[0x01, 0x02]);
        assert!(dec.poll().is_none());
        dec.push(&[0x03]);
        assert_eq!(dec.poll(), Some(FrameEvent::Frame(vec![1, 2, 3])));
        assert!(dec.poll().is_none());
    }

    #[test]
    fn delimiter_frame() {
        let mut dec = FrameDecoder::new(FrameFormat::Delimiter(vec![0x0A]));
        dec.push(b"Hello");
        assert!(dec.poll().is_none());
        dec.push(b"\nWorld\n");
        assert_eq!(dec.poll(), Some(FrameEvent::Frame(b"Hello".to_vec())));
        assert_eq!(dec.poll(), Some(FrameEvent::Frame(b"World".to_vec())));
        assert!(dec.poll().is_none());
    }

    #[test]
    fn length_prefixed_frame() {
        let mut dec = FrameDecoder::new(FrameFormat::LengthPrefixed {
            length_bytes: 2,
            length_includes_header: false,
        });
        // Length = 3, payload = [0xAA, 0xBB, 0xCC]
        dec.push(&[0x00, 0x03, 0xAA, 0xBB]);
        assert!(dec.poll().is_none());
        dec.push(&[0xCC]);
        assert_eq!(dec.poll(), Some(FrameEvent::Frame(vec![0xAA, 0xBB, 0xCC])));
    }

    #[test]
    fn length_prefixed_with_header_included() {
        let mut dec = FrameDecoder::new(FrameFormat::LengthPrefixed {
            length_bytes: 1,
            length_includes_header: true,
        });
        // Length byte = 3 (1 header + 2 payload)
        dec.push(&[0x03, 0xAA, 0xBB]);
        assert_eq!(dec.poll(), Some(FrameEvent::Frame(vec![0xAA, 0xBB])));
    }

    #[test]
    fn clear_buffer() {
        let mut dec = FrameDecoder::new(FrameFormat::FixedLength(10));
        dec.push(&[1, 2, 3, 4, 5]);
        assert_eq!(dec.buffered_len(), 5);
        dec.clear();
        assert_eq!(dec.buffered_len(), 0);
    }

    #[test]
    fn find_subsequence_basic() {
        assert_eq!(find_subsequence(b"Hello\nWorld", b"\n"), Some(5));
        assert_eq!(find_subsequence(b"abc", b"def"), None);
        assert_eq!(find_subsequence(b"abc", b""), None);
    }

    #[test]
    fn empty_delimiter_returns_none() {
        let mut dec = FrameDecoder::new(FrameFormat::Delimiter(vec![]));
        dec.push(b"data");
        assert!(dec.poll().is_none());
    }

    #[test]
    fn multiple_frames_single_push() {
        let mut dec = FrameDecoder::new(FrameFormat::FixedLength(2));
        dec.push(&[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(dec.poll(), Some(FrameEvent::Frame(vec![1, 2])));
        assert_eq!(dec.poll(), Some(FrameEvent::Frame(vec![3, 4])));
        assert!(dec.poll().is_none());
    }
}
