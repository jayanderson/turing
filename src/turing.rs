extern crate rand; 
//use std::vec::Vec;
use rand::Rng;

#[deriving(Eq,Ord,Show)]
enum Direction {
  STAY,
  NORTH,
  EAST,
  SOUTH,
  WEST,
  // TODO: consider diagonal moves
}

/*
impl ToStr for Direction {
  fn to_str(&self) -> ~str {
    match *self {
      STAY => ~"STAY",
      NORTH => ~"NORTH",
      EAST => ~"EAST",
      SOUTH => ~"SOUTH",
      WEST => ~"WEST",
    }
  }
}
*/

impl Direction {
  fn from_u8(val: u8) -> Direction {
    match val {
      0 => STAY,
      1 => NORTH,
      2 => EAST,
      3 => SOUTH,
      4 => WEST,
      _ => fail!("out of range"),
    }
  }
}


/// A finite 2D turing machine definition.
/// - The 'tape' has a size of 'width'*'height'.
/// - There is a current position within the tape.
/// - There are 'states' possible states for the machine.
/// - There are 'symbols' possible symbols at each position.
/// - The table defines transitions. It is a 2D table. Given the current state
///   and the current symbol it gives the next state, the symbol to write, and
///   the direction to move.
#[deriving(Show)]
struct TuringMachine {
  width: u16,
  height: u16,
  states: u8,
  symbols: u8,
  position: u32,
  state: u8,
  // transition [curr_state, read_symbol] -> [next_state, write_symbol, move_direction]
  table: Vec<(u8, u8, Direction)>,
  tape: Vec<u8>,
}

impl TuringMachine {
  pub fn new(width: u16, height: u16, states: u8, symbols: u8) -> ~TuringMachine {
    ~TuringMachine {
      width: width,
      height: height,
      states: states,
      symbols: symbols,
      position: 0,
      state: 0,
      table: TuringMachine::random_table(states, symbols),
      tape: Vec::from_elem((width as uint) * (height as uint), 0u8),
    }
  }

  fn random_table(states: u8, symbols: u8) -> Vec<(u8, u8, Direction)> {
    let mut rng = rand::task_rng();
    Vec::from_fn((states*symbols) as uint, |_| {
      (rng.gen_range(0u8, states), rng.gen_range(0u8, symbols), Direction::from_u8(rng.gen_range(0u8, 4u8)+1))
    })
  }

  // Return true if this step changed a pixel.
  fn step(&mut self) -> bool {
    let curr_symbol = *self.tape.get(self.position as uint);
    let (next_state, write_symbol, move_direction) =
      *self.table.get((self.states*curr_symbol + self.state) as uint);
    *self.tape.get_mut(self.position as uint) = write_symbol;

    // Return whether this changes the picture or not.
    let ret = write_symbol != curr_symbol;

    self.state = next_state;
    let mut x: i32 = (self.position as i32) % (self.width as i32);
    let mut y: i32 = (self.position as i32) / (self.width as i32);
    match move_direction {
      STAY => { },
      NORTH => {
        y -= 1;
        if y < 0 { y = (self.height as i32)-1; }
      },
      EAST => {
        x += 1;
        if x >= (self.width as i32) { x = 0; }
      },
      SOUTH => {
        y += 1;
        if y >= (self.height as i32) { y = 0; }
      },
      WEST => {
        x -= 1;
        if x < 0 { x = (self.width as i32)-1; }
      },
    }
    self.position = (y*(self.width as i32) + x) as u32;
    return ret;
  }

  fn write_image<W: Writer>(&self, out: &mut W) {
    let palette = [
      [0u8,0u8,0u8], // black
      [255u8,255u8,255u8], // white
      [170u8,170u8,170u8], // light gray
      [85u8,85u8,85u8], // dark gray
      [255u8,0u8,0u8], // red
      [0u8,0u8,255u8], // blue
      [0u8,255u8,0u8], // green
    ];
    let len = (self.width as uint) * (self.height as uint) * 3;
    let mut image = Vec::from_elem(len, 0u8);
    let mut i = 0;
    for &val in self.tape.iter() {
      let color = palette[val as uint];
      *image.get_mut(i) = color[0];
      *image.get_mut((i+1) as uint) = color[1];
      *image.get_mut((i+2) as uint) = color[2];
      i += 3;
    }
    out.write(image.slice(0,len));
    out.flush();
  }
}

fn print_err(msg: &str) { //-> Result<(),std::io::IoError> {
  let mut err = ~std::io::stderr();
  err.write(msg.as_bytes());
}


// ffmpeg random input: ffmpeg -y -f rawvideo -s 420x360 -pix_fmt rgb24 -r 24 -i /dev/urandom -an -vcodec mpeg4 random.mp4
// ffmpeg stdin: ffmpeg -y -f rawvideo -s 512x512 -pix_fmt rgb24 -r 24 -i - -an -vcodec mpeg4 out.mp4
// time ./turing | ffmpeg -y -f rawvideo -s 512x512 -pix_fmt rgb24 -i - -an -f mpeg2video - | vlc -
fn main() {
  let states = 4u8;
  let symbols = 3u8;
  let width = 1024u16;
  let height = 768u16;
  let mut machine = TuringMachine::new(width, height, states, symbols);
  let len = (machine.width as uint) * (machine.height as uint);
  let mut out = ~std::io::stdout();

  // Reset the pattern after this step count
  let count = 2500000;
  // print the picture after this step count
  let stops = 10000;

  let mut i = 0;
  let mut change = false;
  loop {
    change = machine.step() || change;
    i += 1;
    if i % stops == 0 {
      machine.write_image(out);
      if !change {
        print_err("new machine: bad\n");
        // new machine
        machine.table = TuringMachine::random_table(machine.states, machine.symbols);
        machine.tape = Vec::from_elem(len, 0u8);
        i = 0;
      } else {
        //print_err("good\n");
        change = true;
      }
    }
    if i >= count {
      print_err("new machine: bad\n");
      // new machine
      machine.table = TuringMachine::random_table(machine.states, machine.symbols);
      machine.tape = Vec::from_elem(len, 0u8);
      i = 0;
    }
  }
}
