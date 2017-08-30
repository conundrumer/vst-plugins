use self::Message::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Message {

    NoteOff {
        channel: u8,
        key: u8,
        velocity: u8
    },

    NoteOn {
        channel: u8,
        key: u8,
        velocity: u8
    },

    KeyPressure {
        channel: u8,
        key: u8,
        pressure: u8
    },

    ControlChange {
        channel: u8,
        controller: u8,
        value: u8
    },

    ChannelMode {
        channel: u8,
        controller: u8,
        value: u8
    },

    ProgramChange {
        channel: u8,
        program: u8
    },

    ChannelPressure {
        channel: u8,
        pressure: u8
    },

    PitchBend {
        channel: u8,
        value: u16
    },

    SysEx {
        id: u8,
        data: Vec<u8>
    },

    SysCommon {
        status: u8,
        data: [u8; 2]
    },

    SysRealTime {
        status: u8
    }

}

impl Message {
    pub fn try_from(data: &[u8]) -> Result<Message, String> {
        let status = data[0] & 0xF0;
        let channel = data[0] & 0x0F;

        let message = match status {
            0x80 => NoteOff {
                channel,
                key: data[1],
                velocity: data[2]
            },
            0x90 => NoteOn {
                channel,
                key: data[1],
                velocity: data[2]
            },
            0xA0 => KeyPressure {
                channel,
                key: data[1],
                pressure: data[2]
            },
            0xB0 => match data[1] {
                0...119 => ControlChange {
                    channel,
                    controller: data[1],
                    value: data[2]
                },
                120...127 => ChannelMode {
                    channel,
                    controller: data[1],
                    value: data[2]
                },
                _ => return Err("Not a data byte".into())
            },
            0xC0 => ProgramChange {
                channel,
                program: data[1]
            },
            0xD0 => ChannelPressure {
                channel,
                pressure: data[1]
            },
            0xE0 => PitchBend {
                channel,
                value: (data[2] as u16) << 7 | (data[1] as u16)
            },
            0xF0 => match data[0] & 0xF8 {
                0xF0 => SysEx {
                    id: data[1],
                    data: data[2..].to_vec()
                },
                0xF1 | 0xF3 => SysCommon {
                    status: data[0],
                    data: [data[1], 0]
                },
                0xF2 => SysCommon {
                    status: data[0],
                    data: [data[1], data[2]]
                },
                0xF4...0xF7 => SysCommon {
                    status: data[0],
                    data: [0, 0]
                },
                0xF8...0xFF => SysRealTime {
                    status: data[0]
                },
                _ => unreachable!()
            },
            _ => return Err("Not a status byte".into())
        };
        Ok(message)
    }
}
