//! An Opus packet is a container of multiple isoconfig Opus frames.
//! 
//! Packets contain a set of frames either of same (usually CBR) or variable size.
//! For variable size frames (usually VBR), variably-sized length fields are present
//! for all but the last frame, for such it becomes non-delimiting. This parser cannot
//! parse the format described in [RFC 6716, Appendix B][1] wherein a length field
//! exists for the aforementioned last frame. Therefore, for Opus multistream (e.g.
//! with [Ogg encapsulation][2]) (upto 255 channels) cannot be parsed thru this.
//! 
//! [1]: https://datatracker.ietf.org/doc/html/rfc6716#appendix-B
//! [2]: https://datatracker.ietf.org/doc/html/rfc784

pub mod parser;
pub mod coder;
pub mod config;
pub(crate) mod utils;