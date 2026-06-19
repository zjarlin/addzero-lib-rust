//! 串口字节流帧解码工具。
//!
//! 支持常见工业协议中的固定长度帧、分隔符帧和长度前缀帧。调用方持续推入字节，
//! 解码器在缓冲区中累积数据，并在帧完整时返回 [`FrameEvent`]。


/// 字节流中的帧边界格式。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrameFormat {
    /// 固定长度帧，每一帧都恰好为指定字节数。
    FixedLength(usize),
    /// 以分隔符结尾的帧，例如 `\r\n` 或 `0xAA 0x55`。
    Delimiter(Vec<u8>),
    /// 长度前缀帧，帧头中的 N 个字节编码 payload 长度。
    LengthPrefixed {
        /// 长度字段字节数，支持 1、2 或 4。
        length_bytes: usize,
        /// 长度值是否包含长度字段本身。
        length_includes_header: bool,
    },
}

/// 帧解码器产生的事件。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrameEvent {
    /// 已成功解出一帧完整数据。
    Frame(Vec<u8>),
    /// 内部缓冲区溢出，当前未完成帧已被丢弃。
    Overflow,
}

/// 增量式串口帧解码器。
///
/// 调用方可以多次推入字节，再通过 [`FrameDecoder::poll`] 取出完整帧。
/// 该类型适合串口、TCP 透传或其他边读边拆包的场景。
///
/// # Examples
///
/// ```
/// use az_serial::frame::{FrameDecoder, FrameEvent, FrameFormat};
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
    /// 使用指定帧格式创建解码器。
    pub fn new(format: FrameFormat) -> Self {
        Self {
            format,
            buffer: Vec::with_capacity(1024),
            max_buffer_size: 8192,
        }
    }

    /// 设置自定义最大缓冲区大小。
    pub fn with_max_buffer(mut self, max: usize) -> Self {
        self.max_buffer_size = max;
        self
    }

    /// 向解码器追加新收到的字节。
    pub fn push(&mut self, data: &[u8]) {
        if self.buffer.len() + data.len() > self.max_buffer_size {
            self.buffer.clear();
            return;
        }
        self.buffer.extend_from_slice(data);
    }

    /// 尝试从内部缓冲区取出下一帧完整数据。
    ///
    /// 有完整帧时返回 `Some(FrameEvent::Frame(data))`；缓冲区过大时返回
    /// `Some(FrameEvent::Overflow)`；暂时没有完整帧时返回 `None`。
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
                // 返回帧体后丢弃分隔符，避免下一次 poll 重复命中同一边界。
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

    /// 清空内部缓冲区。
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// 返回当前缓冲区中的字节数。
    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }
}

/// 查找 `needle` 在 `haystack` 中第一次出现的位置。
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
    use crate::frame::{FrameDecoder, FrameEvent, FrameFormat, find_subsequence};

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
