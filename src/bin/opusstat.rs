use std::env;

use ffmpeg_next::format;
use ffmpeg_next::codec;

mod utils;

fn main() {
    let input_file = env::args()
    .nth(1)
    .expect("Input file unspecified");

    let mut input = format::input(&input_file).unwrap();

    let packets = input
    .packets()
    .filter(|p| p.0.codec().id() == codec::Id::OPUS);

    let mut last_info = None;
    let mut num_same_conf = 0;
    let mut frames = Vec::<_>::new();

    for (stream, packet) in packets {
        let mut frames_scope = utils::VecScope::new(&mut frames);

        let internal = opus_rs::packet::parser::parse(
            &mut frames_scope, 
            packet.data().unwrap()).unwrap();
        let info = internal.info;

        if last_info != Some(info) || last_info == None {
            println!("s={} mode={:?} bwidth={:?} dur={:?}ms nframes={:?} code={:?} vbr?={} stereo?={} \
                      pad={:?}", 

            stream.id(),
            info.frame_config.config.mode,
            info.frame_config.config.bandwith,
            info.frame_config.config.framesize,
            info.num_frames,
            info.code_no as usize,
            match info.is_vbr {
                Some(v) => v.to_string(),
                None => "?".to_string()
            }, 
            info.frame_config.is_stereo,
            match internal.padding {
                Some(p) => p.0,
                None => 0
            }
        );

            num_same_conf = 0;
        } else {
            num_same_conf += 1;
            print!("  \r... {}", num_same_conf);
        }

        last_info = Some(internal.info);
    }
}