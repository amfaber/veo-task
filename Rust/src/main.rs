use veo_task::yarhdlc::{decode, FLAG_SEQUENCE, FrameType};

// As the instructions allow for a small bit of interpretation,
// I will describe my assumptions here.

// I interpreted the coordinate system to be the following
//  --- --- --- --- ---
// |0,0|   |   |   |4,0|
//  --- --- --- --- ---
// |   |   |   |   |   |
//  --- --- --- --- ---
// |   |   |   |   |   |
//  --- --- --- --- ---
// |   |   |   |   |   |
//  --- --- --- --- ---
// |0,4|   |   |   |4,4|
//  --- --- --- --- ---
//
// Position = (x,y)
// ↑ Up => y - 1
// ↓ Down => y + 1
// → Right => x + 1
// ← Left => x - 1

// "Leaving the board is an illegal move" => Any move that would have caused
// the character to leave the board is discarded, and the game proceeds.

// "If the same instruction occurs three times in a row, all three instructions
// should be discarded" => When three of the same type are found, they are immediately
// discarded, allowing a fourth and even fifth instruction of the same type to get through.
// A sixth will ofcourse form a new run of 3 consecutive identical instructions, which will
// again result in their removal.

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
#[allow(dead_code)]
enum Move {
    Up = 1,
    Down = 2,
    Right = 3,
    Left = 4,
}

impl Move{
    fn from_u8(value: u8) -> Option<Self>{
        match value{
            1 => Some(Self::Up),
            2 => Some(Self::Down),
            3 => Some(Self::Right),
            4 => Some(Self::Left),
            _ => None
        }
    }
}

struct MoveIterator<'a>{
    start: usize,
    end: usize,
    data: &'a [u8],
    buffer: Vec<u8>,
}

impl<'a> MoveIterator<'a>{
    fn new(data: &'a [u8]) -> Self{
        Self{
            start: 0,
            end: 1,
            data,
            buffer: Vec::new(),
        }
    }
}


// The move iterator borrows the buffer containing all the received frames.
// Calling next finds the next sequence enclosed by HDLC flag sequences on
// both sides and feeds it to the decoder. 

// Frames without any data (in this case ACK frames) are skipped.

// The output buffer for decoding is owned by the iterator and is thus reused
// between calls to avoid repeated allocation. A small amount of refactoring
// could even use a static array instead, completely avoiding the allocation.
impl<'a> Iterator for MoveIterator<'a>{
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        loop{
            let byte = *self.data.get(self.end)?;
            if byte == FLAG_SEQUENCE {
                let control = decode(&self.data[self.start..=self.end], &mut self.buffer).unwrap();
                self.start = self.end + 1;
                self.end += 2;
                match control.frame_type{
                    FrameType::Data => {
                        let mv = Move::from_u8(self.buffer[0]).unwrap();
                        self.buffer.clear();
                        return Some(mv)
                    },
                    FrameType::Acknowledge => continue,
                    FrameType::NegativeAcknowledge => continue,
                };
            } else {
                self.end += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct PlayerPosition{
    x: i32,
    y: i32,
}



impl PlayerPosition{
    fn update(&mut self, mv: Move){
        match mv{
            Move::Up => self.y = (self.y - 1).clamp(0, 4),
            Move::Down => self.y = (self.y + 1).clamp(0, 4),
            Move::Right => self.x = (self.x + 1).clamp(0, 4),
            Move::Left => self.x = (self.x - 1).clamp(0, 4),
        }
    }

    fn print(&self){
        // We want to print a square like this example of the player position at (0, 4)
        // ██████████
        // ██████████
        // ██████████
        // ██████████
        // xx████████
        // to show the current position of the player.
        // Each row consists of 10 squares (█) and a newline, and a total of 5 lines.
        // This makes the stride for the y coordinate 11, and the stride of the x-coordinate 2.
        let mut out = Vec::with_capacity(11*5);
        for _x in 0..5{
            for _y in 0..5{
                out.push('█');
                out.push('█');
            }
            out.push('\n');
        }

        out[(self.y * 11 + self.x * 2) as usize] = 'x';
        out[(self.y * 11 + self.x * 2 + 1) as usize] = 'x';
        println!("{}", out.iter().collect::<String>());
    }
}


fn main() {
    let mut args = std::env::args();
    // If any argument is passed, we prertty print the position of the player as we
    // execute the moves
    let print = args.nth(1).is_some();
    
    // The input data is included in the binary for simplicity.
    // In a real use case these would probably be lazily received over some connection,
    // potentially with some buffering
    let data = include_bytes!("../../transmission.bin");
    let iter = MoveIterator::new(data);

    // Only the previous 3 moves are kept to allow for discarding triplets
    let mut prev_moves: [Option<Move>; 3] = Default::default();
    let mut prev_idx = 0;
    let mut player = PlayerPosition{ x: 0, y: 4 };
    for mv in iter{
        // The oldest move is pushed out and applied to the player position now that it is safe to do so
        if let Some(old_move) = prev_moves.get(prev_idx).unwrap(){
            player.update(*old_move);
            if print{
                player.print();
            }
        }
        prev_moves[prev_idx] = Some(mv);

        // If the previous 3 moves are identical, they are all discarded.
        let three_in_a_row = prev_moves[1..].iter().all(|&ele| (prev_moves[0] == ele));
        if three_in_a_row{
            prev_moves = Default::default();
        }
        prev_idx = (prev_idx + 1) % prev_moves.len();
    }

    // Apply the rest of the moves in the buffer in the same order that they occured in the instructions
    for _ in 0..prev_moves.len(){
        if let Some(mv) = prev_moves.get(prev_idx).unwrap(){
            player.update(*mv);
        }
        prev_idx = (prev_idx + 1) % prev_moves.len();
        if print{
            player.print();
        }
    }

    // Report the result.
    println!("{:?}", player);
}
