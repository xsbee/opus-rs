# opus-rs
Opus codec implementation in Rust. For now, it does not decode or encode but just encodes or decodes packets (units of Opus data transmitted over a transport layer).
It would not be finished anytime soon, and I am no audio coding wizard so I don't understand LPC or CELT's underlying workings or a data-compression wizard either so
I don't understand Range Coding either. If I am successful in any of those, it would be suboptimal at best, honestly just use libopus, much of the manpower is inclined
towards there. No one is going to help me. I am all on my own eccentric unsupported very ambitious goals.
