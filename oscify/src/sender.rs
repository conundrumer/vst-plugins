use std::fmt;

use rosc::OscType;

use config;
use config::EntryType;
use vst::OscifyPlugin;

#[derive(Debug, Copy, Clone)]
pub struct NoteMessage {
    pub note_on: bool,
    pub channel: u8,
    pub key: u8,
    pub velocity: f32
}

#[derive(Debug, Copy, Clone)]
pub struct ChannelMessage {
    pub channel_type: ChannelType,
    pub channel: u8,
    pub key: u8,
    pub value: f32
}

#[derive(Debug, Copy, Clone)]
pub struct ParamMessage {
    pub param_index: usize,
    pub value: f32
}


#[derive(Debug, Copy, Clone)]
pub enum ChannelType { Pitch, Pressure, Timbre, Pan }

impl fmt::Display for ChannelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ChannelType::Pitch => write!(f, "pitch"),
            ChannelType::Pressure => write!(f, "press"),
            ChannelType::Timbre => write!(f, "timbre"),
            ChannelType::Pan => write!(f, "pan")
        }
    }
}

#[derive(Debug)]
enum AddressNode<'a> {
    String(String),
    Str(&'a str),
    U8(u8),
    Usize(usize),
    Ch(ChannelType),
    None,
    DoNotSend
}
impl<'a> From<String> for AddressNode<'a> {
    fn from(x: String) -> Self { AddressNode::String(x) }
}
impl<'a> From<&'a str> for AddressNode<'a> {
    fn from(x: &'a str) -> Self { AddressNode::Str(x) }
}
impl<'a> From<usize> for AddressNode<'a> {
    fn from(x: usize) -> Self { AddressNode::Usize(x) }
}
impl<'a> From<u8> for AddressNode<'a> {
    fn from(x: u8) -> Self { AddressNode::U8(x) }
}
impl<'a> From<ChannelType> for AddressNode<'a> {
    fn from(x: ChannelType) -> Self { AddressNode::Ch(x) }
}

impl<'a> fmt::Display for AddressNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AddressNode::String(ref x) => write!(f, "/{}", x),
            AddressNode::Str(x) => write!(f, "/{}", x),
            AddressNode::Usize(x) => write!(f, "/{}", x),
            AddressNode::U8(x) => write!(f, "/{}", x),
            AddressNode::Ch(x) => write!(f, "/{}", x),
            _ => Ok(())
        }
    }
}

impl<'a> AddressNode<'a> {
    fn should_send(&self) -> bool {
        match self {
            &AddressNode::DoNotSend => false,
            _ => true
        }
    }

    fn base(entry: Option<&config::Entry>, entry_index: usize) -> AddressNode {
        entry
            .map(|&config::Entry { ref address, .. }| address[..].into())
            .unwrap_or_else(|| entry_index.into())
    }

    fn id(entry: Option<&config::Entry>, (channel, key): (u8, u8)) -> AddressNode {
        let entry_type = entry.map(|e| e.entry_type).unwrap_or(EntryType::Mono);
        match entry_type {
            EntryType::Mono => AddressNode::None,
            EntryType::Poly => channel.into(),
            EntryType::Drum => match entry {
                Some(&config::Entry { keys: Some(ref keys), .. }) =>
                    keys.get(&key)
                        .map(|key_name| key_name[..].into())
                        .unwrap_or_else(|| key.into()),
                _ => key.into()
            },
            _ => AddressNode::DoNotSend
        }
    }

    fn param(entry: Option<&config::Entry>, param_index: usize) -> AddressNode {
        entry
            .and_then(|e| e.params.get(param_index))
            .map(|param_name| param_name[..].into())
            .unwrap_or_else(|| param_index.into())
    }
}

const NS_NODE: &str = "/oscify";
const NOTE_NODE: &str = "/note";
const PARAM_NODE: &str = "/param";
impl OscifyPlugin {
    pub fn send_note(&mut self, NoteMessage { note_on, channel, key, velocity }: NoteMessage, t: (u32, u32)) {
        let entry = self.entries.get(self.entry_index);

        let id_address_node = AddressNode::id(entry, (channel, key));
        if !id_address_node.should_send() { return }

        let base_address_node = AddressNode::base(entry, self.entry_index);

        self.osc_sender.push(
            format!("{}{}{}{}/on", NS_NODE, base_address_node, id_address_node, NOTE_NODE),
            OscType::Bool(note_on),
            t
        );

        if note_on {
            self.osc_sender.push(
                format!("{}{}{}{}/key", NS_NODE, base_address_node, id_address_node, NOTE_NODE),
                OscType::Int(key.into()),
                t
            );
            self.osc_sender.push(
                format!("{}{}{}{}/vel", NS_NODE, base_address_node, id_address_node, NOTE_NODE),
                OscType::Float(velocity),
                t
            );
            self.osc_sender.push(
                format!("{}{}{}{}/phase", NS_NODE, base_address_node, id_address_node, NOTE_NODE),
                OscType::Float(self.phase),
                t
            );
        }
    }

    pub fn send_channel(&mut self, ChannelMessage { channel_type, channel, key, value }: ChannelMessage, t: (u32, u32)) {
        let entry = self.entries.get(self.entry_index);

        let base_address_node = AddressNode::base(entry, self.entry_index);

        let id_address_node = AddressNode::id(entry, (channel, key));
        if !id_address_node.should_send() { return }

        self.osc_sender.push(
            format!("{}{}{}{}", NS_NODE, base_address_node, id_address_node, AddressNode::from(channel_type)),
            OscType::Float(value),
            t
        );
    }

    pub fn send_param(&mut self, ParamMessage { param_index, value } : ParamMessage, t: (u32, u32)) {
        let entry = self.entries.get(self.entry_index);

        let base_address_node = AddressNode::base(entry, self.entry_index);

        let param_address_node = AddressNode::param(entry, param_index);

        self.osc_sender.push(
            format!("{}{}{}{}", NS_NODE, base_address_node, PARAM_NODE, param_address_node),
            OscType::Float(value),
            t
        );
    }

    pub fn flush_midi_events(&mut self) {
        let result = self.osc_sender.flush();
        if result.is_err() {
            error!("Could not flush");
        }
    }
}
