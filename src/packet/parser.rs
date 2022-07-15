use bitvec::prelude::*;

pub use super::config::*;
use super::utils::parse_frame_length;

/// Code or type of packet. Primarily dictates the layout of frames inside a packet.
/// And optionally padding data if any (only for Code 3 packets).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Code {
    /// One frame.
    Code0 = 0,
    /// Two frames.
    Code1 = 1,
    /// Two frames (variable length).
    Code2 = 2,
    /// Multiple frames (upto 120 ms total).
    /// 
    /// Static length per frame if CBR else variable length and VBR.
    Code3 = 4,
}

impl From<u8> for Code {
    fn from(value: u8) -> Self {
        match value {
            0 => Code::Code0,
            1 => Code::Code1,
            2 => Code::Code2,
            3 => Code::Code3,
            _ => unreachable!()
        }
    }
}

/// Statistical and internal information about the parsed packet.
/// See [`fn parse`] for its usage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Info {
    /// Global configuration for all frames in the packet.
    pub frame_config: FrameConfig,
    /// Usage of VBR, [`Some(true)`] if VBR or [`Some(false)`] if CBR or, [`None`] if
    /// it is not a Code 3 packet (i.e. field does not exist).
    pub is_vbr: Option<bool>,
    /// (Non-zero) number of frames that exist in this packet.
    /// 
    /// If it not a Code 3 packet or, if `strict` is enabled and is a Code 3 packet
    /// it will be non-zero otherwise might be zero (i.e no frames are added).
    pub num_frames: usize,
    /// Code or type of packet.
    pub code_no: Code,
}

/// Parser's exported internal information.
pub struct Internal<'a> {
    /// Statistical information about the packet.
    pub info: Info,

    /// Opus padding, [`Some((total_padding, Some(padding_data)))`] if padding is more
    /// than a single byte carrying the additional padding bytes or, [`Some(1, None)`]
    /// if only the padding byte (which is set to zero) exists, making total padding
    /// a byte long or, [`None`] if the packet is unpadded.
    pub padding: Option<(usize, Option<&'a [u8]>)>
}

/// An error that occured during parsing, volating one of the
/// Opus packet handling rules defined in [RFC 6716, Sec 3.4][1].
/// 
/// [1]: https://datatracker.ietf.org/doc/html/rfc6716#section-3.4
#[derive(Debug, PartialEq)]
pub enum Error {
    /// No TOC exists in the packet.
    NoTOC,
    /// Frame is too big (more than 1275 bytes).
    /// 
    /// Note: Only thrown if `strict` is enabled.
    FrameTooBig,
    /// Integer even-length for *Code 1* packets.
    NonOddLength,
    /// Packet is too small to parse correctly.
    /// 
    /// Note: Thrown only in critical conditions, unless `strict` is enabled.
    PacketTooSmall,
    /// Specified packet length overflows the packet size.
    /// 
    /// Note: Only thrown if `strict` feature is enabled.
    LengthOverflow,
    /// Code 3 packet exceeding maximum duration past 120ms.
    /// 
    /// Note: Only thrown if `strict` feature is enabled.
    TooMuchAudio,
    /// Non frame-count integer multiple remainer byte count.
    NonMultipleRemainder,
    /// Code 3 packet having zero audio frames.
    NoAudio,
}

/// Parses a (semi) well-formed non-self-delemiting Opus packets, pushing frames to
/// a vector of parsed frames and returning statistical and select internal data.
pub fn parse<'vec, 'pkt: 'vec>(
    frames: &'vec mut Vec<&'pkt [u8]>, 
    packet: &'pkt [u8]) -> Result<Internal<'pkt>, Error>
{
    if packet.len() < 1 {
        return Err(Error::NoTOC);
    }

    let toc;
    let config;
    let is_stereo;
    let frame_config;
    let code_no;

    let mut is_vbr;
    let mut padding;

    //  0 1 2 3 4 5 6 7
    // +-+-+-+-+-+-+-+-+
    // | config  |s| c |
    // +-+-+-+-+-+-+-+-+
    toc = packet[0].view_bits::<Msb0>();

    config = OPUS_CONFIG_TABLE[toc[..5].load::<usize>()];
    is_stereo = toc[5];
    frame_config = FrameConfig {config, is_stereo};
    code_no = toc[6..].load::<u8>();

    is_vbr = None;
    padding = None;
    
    match code_no {
        // Code 0, 1 frame
        0x0 => {
            let compressed = &packet[1..];

            #[cfg(feature = "strict")]
            if compressed.len() > 1275 {
                return Err(Error::FrameTooBig);
            }

            frames.push(compressed);
        }

        // Code 1, 2 frames
        0x1 => {
            // NOTE: too much strict semantic perhaps.
            if packet.len() % 2 != 0 {
                return Err(Error::NonOddLength);
            }

            let compressed = &packet[1..];

            // data will be split to two equal sized frames (probably CBR).
            let (frame_0, frame_1) = compressed.split_at(compressed.len() / 2);

            #[cfg(feature = "strict")]
            if frame_0.len() > 1275 || frame_1.len() > 1275 {
                return Err(Error::FrameTooBig);
            }

            frames.push(frame_0);
            frames.push(frame_1);
        }

        // Code 2, 2 frames (var. size)
        0x2 => {
            let frame_0_len = parse_frame_length(&packet[1..3]).ok_or(Error::PacketTooSmall)?;
            let compressed = &packet[frame_0_len.1..];

            // offset is needed no more, so redeclare.
            let frame_0_len = frame_0_len.0;

            if compressed.len() < frame_0_len {
                return Err(Error::LengthOverflow);
            }

            frames.push(&compressed[..frame_0_len]);

            // second frame, spanning the remaining is too big.
            #[cfg(feature = "strict")]
            if compressed.len() - frame_0_len > 1275 {
                return Err(Error::FrameTooBig);
            }

            frames.push(&packet[frame_0_len..]);
        },

        // Code 3, multiple frames (var/const. size)
        0x3 => {
            if packet.len() < 2 {
                return Err(Error::PacketTooSmall);
            }

            //  0 1 2 3 4 5 6 7
            // +-+-+-+-+-+-+-+-+
            // |v|p|     M     |
            // +-+-+-+-+-+-+-+-+
            let fcb = packet[1].view_bits::<Msb0>();

            let is_pad;
            let mut n_padb;
            let mut pad_len;
            let num_frames;

            is_vbr = Some(fcb[0]);
            is_pad = fcb[1];
            num_frames = fcb[2..].load();

            n_padb = is_pad as usize;
            pad_len = 0;
            
            #[cfg(feature = "strict")]
            if num_frames < 1 {
                return Err(Error::NoAudio);
            }

            #[cfg(feature = "strict")]
            // At maximum a packet can have
            //
            //  48 -- 2.5ms frames,
            //  24 --   5ms frames,
            //  12 --  10ms frames,
            //   6 --  20ms frames,
            //   3 --  40ms frames and
            //   2 --  60ms frames.
            if config.framesize * num_frames as f32 > 120.0 {
                return Err(Error::TooMuchAudio);
            }

            if is_pad {
                loop {
                    // When Opus padding is used, the number of bytes of padding is encoded
                    // in the bytes following the frame count byte.  Values from 0...254
                    // indicate that 0...254 bytes of padding are included, in addition to
                    // the byte(s) used to indicate the size of the padding.
                    let padb = packet[2 + n_padb] as usize;
                    pad_len += padb;

                    if padb != 255 {
                        break;
                    }

                    // If the value is 255, then the size of the additional padding is 254 bytes,
                    // plus the padding value encoded in the next byte.
                    pad_len -= 1;

                    // Let P (pad_len + n_padb) be the number of header bytes used
                    // to indicate the padding size plus the number of padding bytes
                    // themselves (i.e., P is the total number of bytes added to the
                    // packet).  Then, P MUST be no more than N-2 [R6,R7].
                    if pad_len + n_padb > packet.len() - 2 {
                        return Err(Error::LengthOverflow);
                    }

                    n_padb += 1;
                }
            }

            let pad_pos;

            // let R=N-2-P be the number of bytes remaining in the packet after subtracting
            // the (optional) padding.
            let len_compressed = packet.len().checked_sub(n_padb + pad_len + 2).ok_or(Error::PacketTooSmall)?;

            if let Some(_) = is_vbr {
                let mut frame_pos = n_padb + 2;

                for _ in 0..num_frames-1 {
                    let frame_len = parse_frame_length(&packet[frame_pos..]).ok_or(Error::PacketTooSmall)?;
                    
                    // frame data begins after length and ends at next boundary.
                    let frame_off = frame_pos + frame_len.1;
                    let frame = &packet[frame_off..(frame_off+frame_len.0)];

                    #[cfg(feature = "strict")]
                    if len_compressed < frame.len() {
                        return Err(Error::PacketTooSmall)?;
                    }

                    frames.push(frame);

                    // set beginning of next frame
                    frame_pos = frame_off + frame_len.0;
                }

                if len_compressed > frame_pos { 
                    return Err(Error::PacketTooSmall);
                }

                // remaining bytes belong to the last VBR frame.
                let frame_len = len_compressed - frame_pos;

                #[cfg(feature = "strict")]
                if frame_len > 1275 {
                    return Err(Error::FrameTooBig);
                }

                frames.push(&packet[frame_pos..frame_pos + frame_len]);

                pad_pos = frame_pos + frame_len;
            } else {
                // for CBR each frame is of R/M length. R MUST be a multiple of M.
                if len_compressed % num_frames != 0 {
                    return Err(Error::NonMultipleRemainder);
                }

                let frame_len = len_compressed / num_frames;
                let mut frame_pos = n_padb + 2;

                // all frames have the same length if CBR
                for _ in 0..num_frames {
                    frames.push(&packet[frame_pos..frame_pos + frame_len]);

                    frame_pos += frame_len;
                }

                pad_pos = frame_pos;
            }

            #[cfg(feature = "strict")]
            if packet.len() - pad_pos > pad_len {
                return Err(Error::PacketTooSmall)?;
            }

            if is_pad {
                padding = Some((pad_len + n_padb, if pad_len == 0 {
                    None
                } else { 
                    Some(&packet[pad_pos..]) 
                }));
            }
        },

        _ => unreachable!()
    };

    let num_frames = frames.len();

    Ok(Internal {
        info: Info {
            frame_config, 
            code_no: code_no.into(),
            is_vbr,
            num_frames
        },
        padding
    })
}
