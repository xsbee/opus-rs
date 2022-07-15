//! This module defines structures and enumerations based on the TOC.
//! TOC (Table of Contents) signals which of the various modes and
//! configurations a packet uses for coding one or multiple isoconfig
//! Opus frames upto 120 ms (Code 3 only).
//! 
//! See [RFC 6716, Section 3.1][1].
//!
//! [1]: (https://datatracker.ietf.org/doc/html/rfc6716#section-3.1)

#[derive(Clone, Copy, Debug, PartialEq)]
/// Operating mode used for packet coding.
pub enum Mode {
    /// [SILK][2]-only mode for use in low bitrate with wide-band or
    /// more narrow bandwidth connections.
    /// 
    /// [2]: https://en.wikipedia.org/wiki/SILK
    SILK,
    /// [CELT][3]-only mode for very low delay speech transmission as well
    /// as music transmission narrow-band to full-band.
    /// 
    /// [3]: https://en.wikipedia.org/wiki/CELT
    CELT,
    /// Hybrid (SILK+CELT) mode for super-wide-band or full-band speech at
    /// medium bitrates.
    Hybrid
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Bandwidth of the transmitted signal.
pub enum Bandwidth {
    /// 0-4 kHz (8 kHz samplerate).
    Narrow,
    /// 0-6 kHz (12 kHz samplerate).
    Medium,
    /// 0-8 kHz (16 kHz samplerate).
    Wide,
    /// 0-12 kHz (24 kHz samplerate).
    SuperWide,
    /// 0-20 kHz (48 kHz samplerate).
    /// 
    /// Although the [sampling theorem][4] allows a bandwidth as large as half
    /// the sampling rate, Opus never codes audio above 20 kHz, as that is
    /// the generally accepted upper limit of human hearing.
    /// 
    /// [4]: https://en.wikipedia.org/wiki/Nyquist%E2%80%93Shannon_sampling_theorem
    FullBand
}

/// TOC configuration field.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Config {
    pub mode: Mode,
    pub bandwith: Bandwidth,
    /// Length of an Opus frame, can be 2.5/5/10/20/40/60 ms depending
    /// on the mode used for coding. Any other value pertains to [Opus
    /// custom][5], which is unsupported here.
    /// 
    /// [5]: https://datatracker.ietf.org/doc/html/rfc6716#section-6.2
    pub framesize: f32,
}


#[derive(Clone, Copy, Debug, PartialEq)]
/// Coding configuration of a Opus frame, it is one of the major dictating
/// factors of grouping multiple frames in a packet.
/// 
/// > A single packet may contain multiple audio frames, so long as they share a
/// > common set of parameters, including the operating mode, audio
/// > bandwidth, frame size, and channel count (mono vs. stereo).
pub struct FrameConfig {
    /// TOC configuration field.
    pub config: Config,
    /// Stereophonic or monophonic signal.
    /// 
    /// An Opus decoder may decode as monophonic or stereophonic as per preference,
    /// however it must accept both monophonic and stereophonic frames.
    pub is_stereo: bool 
}

impl Default for Config {
    /// Default according to the reference implementation (libopus).
    /// 
    /// - Full-band CELT-mode.
    /// - 20 ms frames.
    fn default() -> Self {
        Self { 
            mode: Mode::CELT, 
            bandwith: Bandwidth::FullBand, 
            framesize: 20.0
        }
    }
}

impl Default for FrameConfig {
    /// Default according to the reference implementation (libopus).
    /// 
    /// - Full-band CELT-mode.
    /// - 20 ms frames.
    /// - Stereophonic.
    fn default() -> Self {
        Self { config: Config::default(), is_stereo: true }
    }
}

/// Possible configurations according to the `config` field of the TOC byte.
pub static OPUS_CONFIG_TABLE: [Config; 32] = [
    Config {mode: Mode::SILK, bandwith: Bandwidth::Narrow, framesize: 10.0},
    Config {mode: Mode::SILK, bandwith: Bandwidth::Narrow, framesize: 20.0},
    Config {mode: Mode::SILK, bandwith: Bandwidth::Narrow, framesize: 40.0},
    Config {mode: Mode::SILK, bandwith: Bandwidth::Narrow, framesize: 60.0},
    
    Config {mode: Mode::SILK, bandwith: Bandwidth::Medium, framesize: 10.0},
    Config {mode: Mode::SILK, bandwith: Bandwidth::Medium, framesize: 20.0},
    Config {mode: Mode::SILK, bandwith: Bandwidth::Medium, framesize: 40.0},
    Config {mode: Mode::SILK, bandwith: Bandwidth::Medium, framesize: 60.0},

    Config {mode: Mode::SILK, bandwith: Bandwidth::Wide, framesize: 10.0},
    Config {mode: Mode::SILK, bandwith: Bandwidth::Wide, framesize: 20.0},
    Config {mode: Mode::SILK, bandwith: Bandwidth::Wide, framesize: 40.0},
    Config {mode: Mode::SILK, bandwith: Bandwidth::Wide, framesize: 60.0},
    
    Config {mode: Mode::Hybrid, bandwith: Bandwidth::SuperWide, framesize: 10.0},
    Config {mode: Mode::Hybrid, bandwith: Bandwidth::SuperWide, framesize: 20.0},
    
    Config {mode: Mode::Hybrid, bandwith: Bandwidth::FullBand, framesize: 10.0},
    Config {mode: Mode::Hybrid, bandwith: Bandwidth::FullBand, framesize: 20.0},
    
    Config {mode: Mode::CELT, bandwith: Bandwidth::Narrow, framesize: 2.5},
    Config {mode: Mode::CELT, bandwith: Bandwidth::Narrow, framesize: 5.0},
    Config {mode: Mode::CELT, bandwith: Bandwidth::Narrow, framesize: 10.0},
    Config {mode: Mode::CELT, bandwith: Bandwidth::Narrow, framesize: 20.0},

    Config {mode: Mode::CELT, bandwith: Bandwidth::Wide, framesize: 2.5},
    Config {mode: Mode::CELT, bandwith: Bandwidth::Wide, framesize: 5.0},
    Config {mode: Mode::CELT, bandwith: Bandwidth::Wide, framesize: 10.0},
    Config {mode: Mode::CELT, bandwith: Bandwidth::Wide, framesize: 20.0},

    Config {mode: Mode::CELT, bandwith: Bandwidth::SuperWide, framesize: 2.5},
    Config {mode: Mode::CELT, bandwith: Bandwidth::SuperWide, framesize: 5.0},
    Config {mode: Mode::CELT, bandwith: Bandwidth::SuperWide, framesize: 10.0},
    Config {mode: Mode::CELT, bandwith: Bandwidth::SuperWide, framesize: 20.0},
    
    Config {mode: Mode::CELT, bandwith: Bandwidth::FullBand, framesize: 2.5},
    Config {mode: Mode::CELT, bandwith: Bandwidth::FullBand, framesize: 5.0},
    Config {mode: Mode::CELT, bandwith: Bandwidth::FullBand, framesize: 10.0},
    Config {mode: Mode::CELT, bandwith: Bandwidth::FullBand, framesize: 20.0}
];
