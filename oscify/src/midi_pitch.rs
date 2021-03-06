use midi::Message;

const RPN: (u8, u8) = (101, 100);
const DATA_ENTRY: (u8, u8) = (6, 38);

const RPN_MSB: u8 = RPN.0;
const RPN_LSB: u8 = RPN.1;
const DATA_ENTRY_MSB: u8 = DATA_ENTRY.0;
const DATA_ENTRY_LSB: u8 = DATA_ENTRY.1;

const PITCH_BEND: (u8, u8) = (0, 0);

fn to_u14((msb, lsb): (u8, u8)) -> u16 {
    ((msb as u16) << 7) | (lsb as u16)
}

#[derive(Debug, Copy, Clone)]
enum NoteStatus {
    Off,
    On(u8),
    PendingBend(f32)
}

#[derive(Debug)]
pub struct MidiPitch {
    selected_param: (u8, u8),
    pitch_bend: (u8, u8),
    keys: [NoteStatus; 16] // index is channel
}

impl MidiPitch {
    pub fn new() -> Self {
        MidiPitch {
            selected_param: (0x7F, 0x7F),
            pitch_bend: (2, 0), // 2 semitones
            keys: [NoteStatus::Off; 16]
        }
    }

    pub fn get_key(&self, channel: u8) -> u8 {
        match self.keys[channel as usize] {
            NoteStatus::On(key) => key,
            _ => 0
        }
    }

    pub fn get_pitch(&mut self, channel: u8, pitch_bend_amount: u16) -> Option<f32> {
        let channel = channel as usize;
        let pitch_bend = (to_u14(self.pitch_bend) as f32) / ((1 << 7) as f32);
        let pitch_bend_amount = (pitch_bend_amount as f32) / ((1 << 14) as f32);
        let bend = (pitch_bend_amount * 2. - 1.) * pitch_bend;

        match self.keys[channel] {
            NoteStatus::On(key) => Some((key as f32) + bend),
            _ => {
                self.keys[channel] = NoteStatus::PendingBend(bend);
                None
            }
        }
    }

    pub fn get_pending_pitch(&self, channel: u8, key: u8) -> Option<f32> {
        match self.keys[channel as usize] {
            NoteStatus::PendingBend(bend) => Some((key as f32) + bend),
            _ => None
        }
    }

    pub fn process_midi_event(&mut self, msg: &Message) {
        match msg {
            &Message::NoteOff { channel, .. } =>
                self.keys[channel as usize] = NoteStatus::Off,
            &Message::NoteOn { channel, key, .. } =>
                self.keys[channel as usize] = NoteStatus::On(key),
            &Message::ControlChange { controller, value, .. } =>
                match controller {
                    RPN_MSB =>
                        self.selected_param = (value, self.selected_param.1),
                    RPN_LSB =>
                        self.selected_param = (self.selected_param.0, value),
                    DATA_ENTRY_MSB =>
                        match self.selected_param {
                            PITCH_BEND => self.pitch_bend = (value, self.pitch_bend.1),
                            _ => info!("Unknown param: {:?}", self.selected_param)
                        },
                    DATA_ENTRY_LSB =>
                        match self.selected_param {
                            PITCH_BEND => self.pitch_bend = (self.pitch_bend.0, value),
                            _ => info!("Unknown param: {:?}", self.selected_param)
                        },
                    _ => ()
                },
            _ => ()
        }
    }
}
