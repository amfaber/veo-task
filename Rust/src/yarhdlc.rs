// A simple read-only Rust adaptation of yahdlc (https://github.com/bang-olufsen/yahdlc)

pub const FLAG_SEQUENCE: u8 = 0x7E;
const CONTROL_ESCAPE: u8 = 0x7D;
#[allow(unused)]
const ALL_STATION_ADDR: u8 = 0xFF;

#[derive(Debug, Clone, PartialEq, Copy)]
struct FrameCheckSequence(u16);

impl FrameCheckSequence{
    const INIT_VALUE: Self = Self(0xFFFF);
    const GOOD_VALUE: Self = Self(0xF0B8);
    #[allow(unused)]
    const INVERT_MASK: Self = Self(0xFFFF);

    const LOOKUP: [u16; 256] = [ 0x0000, 0x1189, 0x2312, 0x329b,
    0x4624, 0x57ad, 0x6536, 0x74bf, 0x8c48, 0x9dc1, 0xaf5a, 0xbed3, 0xca6c,
    0xdbe5, 0xe97e, 0xf8f7, 0x1081, 0x0108, 0x3393, 0x221a, 0x56a5, 0x472c,
    0x75b7, 0x643e, 0x9cc9, 0x8d40, 0xbfdb, 0xae52, 0xdaed, 0xcb64, 0xf9ff,
    0xe876, 0x2102, 0x308b, 0x0210, 0x1399, 0x6726, 0x76af, 0x4434, 0x55bd,
    0xad4a, 0xbcc3, 0x8e58, 0x9fd1, 0xeb6e, 0xfae7, 0xc87c, 0xd9f5, 0x3183,
    0x200a, 0x1291, 0x0318, 0x77a7, 0x662e, 0x54b5, 0x453c, 0xbdcb, 0xac42,
    0x9ed9, 0x8f50, 0xfbef, 0xea66, 0xd8fd, 0xc974, 0x4204, 0x538d, 0x6116,
    0x709f, 0x0420, 0x15a9, 0x2732, 0x36bb, 0xce4c, 0xdfc5, 0xed5e, 0xfcd7,
    0x8868, 0x99e1, 0xab7a, 0xbaf3, 0x5285, 0x430c, 0x7197, 0x601e, 0x14a1,
    0x0528, 0x37b3, 0x263a, 0xdecd, 0xcf44, 0xfddf, 0xec56, 0x98e9, 0x8960,
    0xbbfb, 0xaa72, 0x6306, 0x728f, 0x4014, 0x519d, 0x2522, 0x34ab, 0x0630,
    0x17b9, 0xef4e, 0xfec7, 0xcc5c, 0xddd5, 0xa96a, 0xb8e3, 0x8a78, 0x9bf1,
    0x7387, 0x620e, 0x5095, 0x411c, 0x35a3, 0x242a, 0x16b1, 0x0738, 0xffcf,
    0xee46, 0xdcdd, 0xcd54, 0xb9eb, 0xa862, 0x9af9, 0x8b70, 0x8408, 0x9581,
    0xa71a, 0xb693, 0xc22c, 0xd3a5, 0xe13e, 0xf0b7, 0x0840, 0x19c9, 0x2b52,
    0x3adb, 0x4e64, 0x5fed, 0x6d76, 0x7cff, 0x9489, 0x8500, 0xb79b, 0xa612,
    0xd2ad, 0xc324, 0xf1bf, 0xe036, 0x18c1, 0x0948, 0x3bd3, 0x2a5a, 0x5ee5,
    0x4f6c, 0x7df7, 0x6c7e, 0xa50a, 0xb483, 0x8618, 0x9791, 0xe32e, 0xf2a7,
    0xc03c, 0xd1b5, 0x2942, 0x38cb, 0x0a50, 0x1bd9, 0x6f66, 0x7eef, 0x4c74,
    0x5dfd, 0xb58b, 0xa402, 0x9699, 0x8710, 0xf3af, 0xe226, 0xd0bd, 0xc134,
    0x39c3, 0x284a, 0x1ad1, 0x0b58, 0x7fe7, 0x6e6e, 0x5cf5, 0x4d7c, 0xc60c,
    0xd785, 0xe51e, 0xf497, 0x8028, 0x91a1, 0xa33a, 0xb2b3, 0x4a44, 0x5bcd,
    0x6956, 0x78df, 0x0c60, 0x1de9, 0x2f72, 0x3efb, 0xd68d, 0xc704, 0xf59f,
    0xe416, 0x90a9, 0x8120, 0xb3bb, 0xa232, 0x5ac5, 0x4b4c, 0x79d7, 0x685e,
    0x1ce1, 0x0d68, 0x3ff3, 0x2e7a, 0xe70e, 0xf687, 0xc41c, 0xd595, 0xa12a,
    0xb0a3, 0x8238, 0x93b1, 0x6b46, 0x7acf, 0x4854, 0x59dd, 0x2d62, 0x3ceb,
    0x0e70, 0x1ff9, 0xf78f, 0xe606, 0xd49d, 0xc514, 0xb1ab, 0xa022, 0x92b9,
    0x8330, 0x7bc7, 0x6a4e, 0x58d5, 0x495c, 0x3de3, 0x2c6a, 0x1ef1, 0x0f78 ];
    
    fn update(&mut self, value: u8){
        self.0 = (self.0 >> 8) ^ (Self::LOOKUP[((self.0 ^ value as u16) & 0xFF) as usize]);
    }
}

#[derive(Debug, Clone)]
struct State {
    control_escape: bool,
    fcs: FrameCheckSequence,
    start_index: Option<usize>,
    end_index: Option<usize>,
    src_index: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            control_escape: false,
            fcs: FrameCheckSequence::INIT_VALUE,
            start_index: None,
            end_index: None,
            src_index: 0,
        }
    }
}


#[derive(Debug, Clone)]
pub enum FrameType {
    Data,
    Acknowledge,
    NegativeAcknowledge,
}

#[derive(Debug, Clone)]
pub struct Control {
    pub frame_type: FrameType,
    pub sequence_no: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ControlByte(u8);

#[allow(unused)]
impl ControlByte {
    const S_OR_U: u8 = 0;
    const SEND_SEQ_NO: u8 = 1;
    const S_FRAME_TYPE: u8 = 2;
    const POLL: u8 = 4;
    const RECV_SEQ_NO: u8 = 5;

    const RECEIVE_READY: u8 = 0;
    const RECEIVE_NOT_READY: u8 = 1;
    const REJECT: u8 = 2;
    const SELECTIVE_REJECT: u8 = 3;
}

impl From<ControlByte> for Control {
    fn from(value: ControlByte) -> Self {
        let value = value.0;
        let sequence_no;
        let frame_type;
        if value & (1 << ControlByte::S_OR_U) != 0 {
            if ((value >> ControlByte::S_FRAME_TYPE) & 0x3) == ControlByte::RECEIVE_READY {
                frame_type = FrameType::Acknowledge
            } else {
                frame_type = FrameType::NegativeAcknowledge
            }
            sequence_no = value >> ControlByte::SEND_SEQ_NO;
        } else {
            sequence_no = value >> ControlByte::SEND_SEQ_NO;
            frame_type = FrameType::Data
        };
        Self {
            frame_type,
            sequence_no,
        }
    }
}

impl From<Control> for ControlByte {
    fn from(value: Control) -> Self {
        Self(match value.frame_type {
            FrameType::Data => {
                (value.sequence_no << ControlByte::SEND_SEQ_NO) | (1 << ControlByte::POLL)
            }
            FrameType::Acknowledge => {
                (value.sequence_no << ControlByte::RECV_SEQ_NO) | (1 << ControlByte::S_OR_U)
            }
            FrameType::NegativeAcknowledge => {
                (value.sequence_no << ControlByte::RECV_SEQ_NO)
                    | ControlByte::REJECT << ControlByte::S_FRAME_TYPE
                    | (1 << ControlByte::S_OR_U)
            }
        })
    }
}


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The frame check sequence did not match that in the packet")]
    FrameCheckSequenceInvalid,

    #[error("The data didn't contain a matching pair of flag sequences")]
    NoMessage,

    #[error("Any message should be greater than 4 bytes")]
    TooShort,
}

pub fn decode(data: &[u8], output: &mut Vec<u8>) -> Result<Control, Error>{
    let mut state = State::default();
    let mut data_iter = data.iter().peekable();
    let mut value;
    let mut control: Option<Control> = None;
    while let Some(&byte) = data_iter.next(){
        if let Some(start_index) = state.start_index{
            if byte == FLAG_SEQUENCE{
                if let Some(&&next_byte) = data_iter.peek(){
                    if (next_byte == FLAG_SEQUENCE) ||
                        ((start_index + 1) == state.src_index){
                        continue
                    }
                }

                state.end_index = Some(state.src_index);
                break;
            } else if byte == CONTROL_ESCAPE{
                state.control_escape = true;
            } else {
                if state.control_escape{
                    state.control_escape = false;
                    value = byte ^ 0x20;
                } else {
                    value = byte;
                }

                state.fcs.update(value);

                if state.src_index == start_index + 2{
                    control = Some(ControlByte(byte).into())
                } else if state.src_index > start_index + 2{
                    output.push(value)
                }
            }
        } else {
            if byte == FLAG_SEQUENCE{
                if let Some(&&next_byte) = data_iter.peek(){
                    if next_byte == FLAG_SEQUENCE { continue };
                }
                state.start_index = Some(state.src_index)
            }
        }
        state.src_index += 1;
    }

    // Remove the FCS from the output
    for _ in 0..core::mem::size_of::<u16>(){
        output.pop();
    }

    if let (Some(start), Some(end)) = (state.start_index, state.end_index){
        if end < start + 4{
            return Err(Error::TooShort)
        }
    } else {
        return Err(Error::NoMessage)
    }

    if state.fcs != FrameCheckSequence::GOOD_VALUE{
        return Err(Error::FrameCheckSequenceInvalid)
    }
    

    Ok(control.unwrap())
}