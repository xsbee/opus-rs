// https://datatracker.ietf.org/doc/html/rfc6716#section-3.2.1
pub(crate) fn parse_frame_length(bytes: &[u8]) -> Option<(usize, usize)> {
    if bytes.len() < 1 {
        return None;
    }

    let mut length = bytes[0] as usize;
    
    if length > 251 {
        if bytes.len() < 2 {
            length += bytes[1] as usize * 4;

            Some((length, 2))
        } else {
            None
        }
    } else {
        Some((length, 1))
    }
}