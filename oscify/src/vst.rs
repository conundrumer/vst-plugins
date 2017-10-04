use time;
use vst2::event::{Event};
use vst2::{plugin, api};

use logger;
use config;
use osc;
use midi::Message;
use sender;
use midi_pitch::MidiPitch;

fn u7_into_f32(x: u8) -> f32 {
    x as f32 / (0x80 as f32) // should be 0x7F but 0x80 centers things and pressure goes that high
}

const EPOCH_DELTA: i64 = 2208988800i64;
const NTP_SCALE: f64 = 4294967295.0_f64;
// offset should be in nanoseconds, less than a second
fn get_time(t: time::Timespec, offset: f64) -> (u32, u32) {
    let sec = t.sec + EPOCH_DELTA;
    let frac = (t.nsec as f64 + offset) * NTP_SCALE / 1e10;
    (sec as u32, frac as u32)
}

#[derive(Debug)]
pub struct OscifyPlugin {
    sample_rate: f32,
    block_size: i64,
    pub osc_sender: osc::OscSender,
    pub entries: Vec<config::Entry>,
    pub entry_index: usize,
    pub phase: f32,
    pub params: [f32; 8],
    pub midi_pitch: MidiPitch
}

const CC_TIMBRE: u8 = 74;
const CC_PAN: u8 = 10;
impl OscifyPlugin {
    fn process_midi_event(&mut self, msg: &Message, time: (u32, u32)) {
        match msg {
            &Message::NoteOff { channel, key, velocity } =>
                self.send_note(sender::NoteMessage {
                    note_on: false,
                    channel,
                    key,
                    velocity: u7_into_f32(velocity)
                }, time),
            &Message::NoteOn { channel, key, velocity } => {
                if let Some(pitch) = self.midi_pitch.get_pending_pitch(channel, key) {
                    self.send_channel(sender::ChannelMessage {
                        channel_type: sender::ChannelType::Pitch,
                        channel,
                        key,
                        value: pitch
                    }, time)
                } else { // no pitch bend, set pitch to as it is
                    self.send_channel(sender::ChannelMessage {
                        channel_type: sender::ChannelType::Pitch,
                        channel,
                        key,
                        value: key as f32
                    }, time)
                }
                self.send_note(sender::NoteMessage {
                    note_on: true,
                    channel,
                    key,
                    velocity: u7_into_f32(velocity)
                }, time)
            },
            &Message::PitchBend { channel, value } => {
                if let Some(pitch) = self.midi_pitch.get_pitch(channel, value) {
                    let key = self.midi_pitch.get_key(channel);
                    self.send_channel(sender::ChannelMessage {
                        channel_type: sender::ChannelType::Pitch,
                        channel,
                        key,
                        value: pitch
                    }, time)
                }
            },
            &Message::ChannelPressure { channel, pressure } => {
                let key = self.midi_pitch.get_key(channel);
                self.send_channel(sender::ChannelMessage {
                    channel_type: sender::ChannelType::Pressure,
                    channel,
                    key,
                    value: u7_into_f32(pressure)
                }, time)
            },
            &Message::ControlChange { channel, controller, value } => {
                let key = self.midi_pitch.get_key(channel);
                let value = u7_into_f32(value);
                let channel_type = match controller {
                    CC_TIMBRE => Some(sender::ChannelType::Timbre),
                    CC_PAN => Some(sender::ChannelType::Pan),
                    _ => None
                };
                if let Some(channel_type) = channel_type {
                    self.send_channel(sender::ChannelMessage {
                        channel_type,
                        channel,
                        key,
                        value
                    }, time)
                }
            },
            _ => ()
        }
        self.midi_pitch.process_midi_event(&msg);
    }
    fn process_param_event(&mut self, index: usize, value: f32) {
        self.send_param(sender::ParamMessage { param_index: index, value }, (0, 0));
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
            sample_rate: 0.,
            block_size: 0,
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

    fn set_sample_rate(&mut self, rate: f32) { self.sample_rate = rate; }

    fn set_block_size(&mut self, size: i64) { self.block_size = size; }

    fn process_events(&mut self, events: &api::Events) {
        let current_time = time::get_time();
        debug!("Received {} events:", events.num_events);
        for &e in events.events_raw() {
            let event: Event = Event::from(unsafe { *e });
            match event {
                Event::Midi(ev) =>
                    if let Ok(msg) = Message::try_from(&ev.data) {
                        debug!("[{}] {} {:?}", self.osc_sender.id, ev.delta_frames, msg);
                        let t = get_time(current_time, (ev.delta_frames as f64) / (self.sample_rate as f64) * 1e9);
                        self.process_midi_event(&msg, t);
                    } else {
                        error!("[{}] invalid midi: {:?}", self.osc_sender.id, ev.data)
                    },
                _ => debug!("[{}] non-midi event", self.osc_sender.id)
            }
        }
        self.flush_midi_events();
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
                self.process_param_event(index, value);
                self.flush_midi_events();
            },
            _ => ()
        }
    }
}
