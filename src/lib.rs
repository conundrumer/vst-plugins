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

pub fn test_config() {
    let mut oscify: vst::OscifyPlugin = Default::default();
    debug!("{:?}", oscify);
    let msg = midi::Message::NoteOn {
        channel: 1,
        key: 2,
        velocity: 3
    };
    debug!("{:?}", msg);

    let note_message = sender::NoteMessage {
        note_on: true,
        channel: 3,
        key: 36,
        velocity: 0.75
    };

    let channel_message = sender::ChannelMessage {
        channel_type: sender::ChannelType::Pitch,
        channel: 3,
        key: 36,
        value: 0.5
    };

    let param_message = sender::ParamMessage {
        param_index: 0,
        value: 0.75
    };

    for i in 0..10 {
        oscify.entry_index = i;
        oscify.send_note(note_message);
        oscify.send_channel(channel_message);
        oscify.send_param(param_message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config() {
        test_config();
    }
}
