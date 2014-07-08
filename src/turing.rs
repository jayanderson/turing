extern crate rand; 
extern crate toml;

use std::rand::Rng;

#[deriving(PartialEq,Eq,PartialOrd,Ord,Show,Rand)]
enum Direction {
  NORTH,
  EAST,
  SOUTH,
  WEST,

  // TODO: consider other directions:
  //STAY,
  //NORTHEAST,
  //NORTHWEST,
  //SOUTHEAST,
  //SOUTHWEST,
}


// Colors defined as arrays of [R,G,B].
type Color = [u8, .. 3];
static BLACK: Color = [0,0,0];
static WHITE: Color = [255,255,255];
static LIGHT_GRAY: Color = [170,170,170];
static GRAY: Color = [85,85,85];
static RED: Color = [255,0,0];
static GREEN: Color = [0,255,0];
static BLUE: Color = [0,0,255];
static CYAN: Color = [0,255,255];
static MAGENTA: Color = [255,0,255];
static YELLOW: Color = [255,255,0];


/// A finite 2D turing machine definition.
/// - The 'tape' has a size of 'width'*'height'.
/// - There is a current 'position' within the tape.
/// - There are 'states' possible states for the machine.
/// - There are 'symbols' possible symbols at each position.
/// - The table defines transitions. It is a 2D table. Given the current state
///   and the current symbol it gives the next state, the symbol to write, and
///   the direction to move.
#[deriving(Show)]
struct TuringMachine {
  width: uint,
  height: uint,
  states: u8,
  symbols: u8,
  position: uint,
  state: u8,
  // transition [curr_state, read_symbol] -> [next_state, write_symbol, move_direction]
  table: Vec<(u8, u8, Direction)>,
  tape: Vec<u8>,

  // Memory for writing raw image into. Optimization.
  image: Vec<u8>,
}

impl TuringMachine {
  pub fn new(width: uint, height: uint, states: u8, symbols: u8) -> Box<TuringMachine> {
    box TuringMachine {
      width: width,
      height: height,
      states: states,
      symbols: symbols,
      position: 0,
      state: 0,
      table: TuringMachine::random_table(states, symbols),
      tape: Vec::from_elem(width * height, 0u8),
      image: Vec::from_elem(width * height * 3, 0u8),
    }
  }

  fn random_table(states: u8, symbols: u8) -> Vec<(u8, u8, Direction)> {
    let mut rng = std::rand::task_rng();
    Vec::from_fn((states*symbols) as uint, |_| {
      (rng.gen_range(0u8, states), rng.gen_range(0u8, symbols), rng.gen::<Direction>())
    })
  }

  // Return true if this step changed a pixel.
  fn step(&mut self) -> bool {
    let curr_symbol = *self.tape.get(self.position);
    let (next_state, write_symbol, move_direction) =
      *self.table.get((self.states*curr_symbol + self.state) as uint);
    *self.tape.get_mut(self.position) = write_symbol;

    // Return whether this changes the picture or not.
    let ret = write_symbol != curr_symbol;

    self.state = next_state;
    let mut x: uint = self.position % self.width;
    let mut y: uint = self.position / self.width;
    match move_direction {
      NORTH => {
        y = if y == 0 { self.height-1 } else { y-1 };
      },
      EAST => {
        x += 1;
        if x >= self.width { x = 0; }
      },
      SOUTH => {
        y += 1;
        if y >= self.height { y = 0; }
      },
      WEST => {
        x = if x == 0 { self.width-1 } else { x-1 };
      },
    }
    self.position = y*self.width + x;

    return ret;
  }

  /// Writes the current state as an image (bgr24 since that's what vlc seems to expect).
  fn write_image<W: Writer>(&mut self, palette: &Vec<Color>, out: &mut Box<W>) -> std::io::IoResult<()> {
    // Direct to stdout. Slow.
    /*
    for &val in self.tape.iter() {
      let color = palette.get(val as uint);
      try!(out.write_u8(color[0]));
      try!(out.write_u8(color[1]));
      try!(out.write_u8(color[2]));
    }
    */

    // Alternative seems quicker, but still not fast enough:
    /*
    let len = (self.width as uint) * (self.height as uint) * 3;
    let mut image = Vec::with_capacity(len);
    for &val in self.tape.iter() {
      let color = palette.get(val as uint);
      image.push(color[0]);
      image.push(color[1]);
      image.push(color[2]);
    }
    try!(out.write(image.as_slice()));
    */

    // Upfront allocation. Faster, but still not fast enough at higher resolutions.
    // Requires adding 'image: Vec<u8>' on the struct.
    let mut i = 0;
    for &val in self.tape.iter() {
      let color = palette.get(val as uint);
      *self.image.get_mut(i+2u) = color[0];
      *self.image.get_mut(i+1u) = color[1];
      *self.image.get_mut(i+0u) = color[2];
      i += 3;
    }
    try!(out.write(self.image.as_slice()))

    try!(out.flush());
    Ok(())
  }
}


fn load_config() -> toml::Value {
  let path = Path::new("turing.toml");
  let mut file = std::io::File::open(&path);
  let data = match file.read_to_str() {
    Err(why) => fail!("Unable to read config file: {}", why.desc),
    Ok(str) => str,
  };
  from_str(data.as_slice()).unwrap()
}


// These colors correspond to the symbols. Having more symbols than colors will
// result in an error.
fn load_palette(config: &toml::Value) -> Vec<Color> {
  // TODO: Consider randomized colors, palette from config, or some way of
  // ensuring enough colors.
  let palette = vec!( 
    BLACK,
    RED,
    GREEN,
    BLUE,
    WHITE,
    CYAN,
    MAGENTA,
    YELLOW,
    LIGHT_GRAY,
    GRAY,
  );

  if palette.len() < get(config, "turing.symbols") as uint {
    fail!("Too many symbols for possible colors. Fix the code.");
  }

  palette
}


fn get(config: &toml::Value, name: &str) -> i64 {
  config.lookup(name).unwrap().as_integer().unwrap()
}


fn main() {
  let config = load_config();
  let states: u8 = get(&config, "turing.states") as u8;
  let symbols: u8 = get(&config, "turing.symbols") as u8;
  let width: uint = get(&config, "turing.width") as uint;
  let height: uint = get(&config, "turing.height") as uint;
  let mut machine = TuringMachine::new(width, height, states, symbols);
  let len = machine.width * machine.height;
  let mut out = box std::io::stdout();

  // Reset the pattern after this step count
  let count: u32 = get(&config, "turing.reset_steps") as u32;
  // print the picture after this step count
  let stops: u32 = get(&config, "turing.picture_steps") as u32;

  let palette: Vec<Color> = load_palette(&config);

  let mut i = 0;
  let mut change = false;
  loop {
    change = machine.step() || change;
    i += 1;
    if i % stops == 0 {
      if machine.write_image(&palette, &mut out).is_err() {
        fail!("Error writing to stdout");
      }
      if !change {
        // new machine
        machine.table = TuringMachine::random_table(machine.states, machine.symbols);
        machine.tape = Vec::from_elem(len, 0u8);
        i = 0;
      } else {
        change = true;
      }
    }
    if i >= count {
      // new machine
      machine.table = TuringMachine::random_table(machine.states, machine.symbols);
      machine.tape = Vec::from_elem(len, 0u8);
      i = 0;
    }
  }
}
