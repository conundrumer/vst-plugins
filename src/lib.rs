extern crate time;

#[macro_use] extern crate log;
extern crate simplelog;

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate rosc;

#[macro_use] extern crate vst2;

mod logger;
mod config;
mod osc;
mod midi;
mod vst;
mod sender;
mod midi_pitch;

plugin_main!(vst::OscifyPlugin);
