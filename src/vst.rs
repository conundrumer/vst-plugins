// use vst2::plugin::{Category, Plugin, Info, CanDo};
// use vst2::api::{Supported};
use vst2::event::{Event};
use vst2::{plugin, api};

use logger;
use config;
use osc;
use midi::Message;
use sender;
use midi_pitch::MidiPitch;

#[derive(Debug)]
pub struct OscifyPlugin {
    pub osc_sender: osc::OscSender,
    pub entries: Vec<config::Entry>,
    pub entry_index: usize,
    pub phase: f32,
    pub params: [f32; 8],
    pub midi_pitch: MidiPitch
}

fn u7_into_f32(x: u8) -> f32 {
    x as f32 / (0x80 as f32) // should be 0x7F but 0x80 centers things and pressure goes that high
}

const CC_TIMBRE: u8 = 74;
const CC_PAN: u8 = 10;
impl OscifyPlugin {
    fn process_midi_event(&self, msg: &Message) {
        match msg {
            &Message::NoteOff { channel, key, velocity } =>
                self.send_note(sender::NoteMessage {
                    note_on: false,
                    channel,
                    key,
                    velocity: u7_into_f32(velocity)
                }),
            &Message::NoteOn { channel, key, velocity } =>
                self.send_note(sender::NoteMessage {
                    note_on: true,
                    channel,
                    key,
                    velocity: u7_into_f32(velocity)
                }),
            &Message::PitchBend { channel, value } =>
                self.send_channel(sender::ChannelMessage {
                    channel_type: sender::ChannelType::Pitch,
                    channel,
                    key: self.midi_pitch.get_key(channel),
                    value: self.midi_pitch.get_pitch(channel, value)
                }),
            &Message::ChannelPressure { channel, pressure } =>
                self.send_channel(sender::ChannelMessage {
                    channel_type: sender::ChannelType::Pressure,
                    channel,
                    key: self.midi_pitch.get_key(channel),
                    value: u7_into_f32(pressure)
                }),
            &Message::ControlChange { channel, controller, value } =>
                match controller {
                    CC_TIMBRE =>
                        self.send_channel(sender::ChannelMessage {
                            channel_type: sender::ChannelType::Timbre,
                            channel,
                            key: self.midi_pitch.get_key(channel),
                            value: u7_into_f32(value)
                        }),
                    CC_PAN =>
                        self.send_channel(sender::ChannelMessage {
                            channel_type: sender::ChannelType::Pan,
                            channel,
                            key: self.midi_pitch.get_key(channel),
                            value: u7_into_f32(value)
                        }),
                    _ => ()
                },
            _ => ()
        }
    }
    fn process_param_event(&self, index: usize, value: f32) {
        self.send_param(sender::ParamMessage { param_index: index, value });
    }
}

impl Default for OscifyPlugin {
    fn default() -> Self {
        logger::init().unwrap_or_else(|_err| { /* can't log, do nothing */ });

        let entries = config::load().unwrap_or_else(|err| {
            error!("Couldn't load config: {}", err);
            vec![]
        });

        let osc_sender = osc::OscSender::new();
        if osc_sender.is_err() {
            error!("Couldn't connect")
        }

        OscifyPlugin {
            osc_sender: osc_sender.unwrap(),
            entries,
            entry_index: 0,
            phase: 0.0,
            params: [0.0; 8],
            midi_pitch: MidiPitch::new()
        }
    }
}

const PARAM_ENTRY: i32 = 0;
const PARAM_PHASE: i32 = 1;

impl plugin::Plugin for OscifyPlugin {
    fn get_info(&self) -> plugin::Info {
        plugin::Info {
            name: "Oscify".to_string(),
            vendor: "delu".to_string(),
            unique_id: 9002,
            category: plugin::Category::Analysis,
            inputs: 0,
            outputs: 0,
            parameters: 10,
            ..plugin::Info::default()
        }
    }

    fn process_events(&mut self, events: &api::Events) {
        for &e in events.events_raw() {
            let event: Event = Event::from(unsafe { *e });
            match event {
                Event::Midi(ev) =>
                    if let Ok(msg) = Message::try_from(&ev.data) {
                        debug!("[{}] Received: {:?}", self.osc_sender.id, msg);
                        self.midi_pitch.process_midi_event(&msg);
                        self.process_midi_event(&msg);
                    } else {
                        error!("[{}] Received invalid midi: {:?}", self.osc_sender.id, ev.data)
                    },
                _ => debug!("[{}] Received non-midi event", self.osc_sender.id)
            }
        }
    }

    fn can_do(&self, can_do: plugin::CanDo) -> api::Supported {
        debug!("[{}] can_do: {:?}", self.osc_sender.id, can_do);
        api::Supported::Yes
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            PARAM_ENTRY => "Entry".to_string(),
            PARAM_PHASE => "Phase".to_string(),
            2...9 => {
                let index = index as usize - 2;
                self.entries.get(self.entry_index)
                    .and_then(|entry| entry.params.get(index as usize) )
                    .map(|s| s.clone())
                    .unwrap_or_else(|| format!("Param {}", index))
            },
            _ => "".to_string()
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            PARAM_ENTRY => {
                if let Some(entry) = self.entries.get(self.entry_index) {
                    format!("{}: {}", self.entry_index, entry.name)
                } else {
                    format!("{}: undefined", self.entry_index)
                }
            },
            PARAM_PHASE => format!("{:.0}˚", 360.0 * self.phase),
            2...9 => self.params[index as usize - 2].to_string(),
            _ => "".to_string()
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            PARAM_ENTRY => self.entry_index as f32 / 100.0,
            PARAM_PHASE => self.phase,
            2...9 => self.params[index as usize - 2],
            _ => 0.0
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            PARAM_ENTRY => self.entry_index = (value * 100.0) as usize,
            PARAM_PHASE => self.phase = value,
            2...9 => {
                let index = index as usize - 2;
                self.params[index] = value;
                self.process_param_event(index, value)
            },
            _ => ()
        }
    }
}
