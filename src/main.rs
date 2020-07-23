
use crossterm::{ queue, execute, cursor, style, Result};
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{self, enable_raw_mode, disable_raw_mode};
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use std::{thread, time};
use std::io::{stdout, Stdout, Write};
use std::vec::Vec;

static W : u16 = 78;
static H : u16 = 24;
static MX : usize = 24;
static MY : usize = 21;


#[derive(Copy, Clone)]
enum Cell {
    Empty,
    Soil,
    Metal,
    Diamond(Falling),
    Boulder(Falling),
    Player,
    Enemy
}

impl Cell {
    fn crushable( cell: Cell ) -> bool {
        match cell {
            Cell::Empty |
            Cell::Player |
            Cell::Enemy => true,
            _ => false
        }
    }

    fn rock( cell: Cell ) -> bool {
        match cell {
            Cell::Boulder(_) |
            Cell::Diamond(_) => true,
            _ => false
        }
    }

    fn falling( cell: Cell ) -> bool {
        match cell {
            Cell::Boulder(Falling::True) |
            Cell::Diamond(Falling::True) => true,
            _ => false
        }
    }

    fn empty( cell: Cell ) -> bool {
        if let Cell::Empty = cell {
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone)]
enum Falling {
    True,
    False
}

#[derive(Copy, Clone)]
enum PlayerInput {
    None,
    Up,
    Down,
    Left,
    Right
}

#[derive(Copy, Clone)]
enum Scene {
    LevelScene,
    TitleScene,
    GameoverScene,
    LevelupScene
}

struct Game {
    world: Vec<Cell>,
    diamonds: usize,
    initial_world: Vec<Cell>,
    initial_diamonds: usize,
    scene: Scene,
    level: usize,
    counter: u32,
    current_input: PlayerInput
}


/*
Grey	Black
Red	    DarkRed
Green	DarkGreen
Yellow	DarkYellow
Blue	DarkBlue
Magenta	DarkMagenta
Cyan	DarkCyan
White	DarkWhite
*/

impl Game {

    fn draw_cell( &self, ctx: &mut Stdout, cell: Cell ) -> Result<()> {
        match cell {
            Cell::Empty => {
                queue!( ctx,
                    style::Print("   ")
                )?;
            }
            Cell::Soil => {
                queue!( ctx,
                    style::SetForegroundColor( style::Color::DarkYellow ),
                    style::Print(":::")
                )?;
            }
            Cell::Metal => {
                queue!( ctx,
                    style::SetForegroundColor( style::Color::DarkGrey ),
                    style::Print("###")
                )?;
            }
            Cell::Diamond(_) => {
                queue!( ctx,
                    style::SetForegroundColor( style::Color::DarkCyan ),
                    style::Print( "<:>" )
                )?;
            }
            Cell::Boulder(_) => {
                queue!( ctx,
                    style::SetForegroundColor( style::Color::Grey ),
                    style::Print("(O)")
                )?;
            }
            Cell::Player => {
                queue!( ctx,
                    style::SetForegroundColor( style::Color::Green ),
                    style::Print(
                        if self.counter % 10 > 5 { "\\o/" } else { "(o)" }
                    )
                )?;
            }
            Cell::Enemy => {
                queue!( ctx,
                    style::SetForegroundColor(
                        if self.counter % 10 > 5 { style::Color::Red } else { style::Color::DarkRed }
                    ),
                    style::Print(" X ")
                )?;
            }
        }

        Ok(())
    }




    fn draw_level( &self, ctx: &mut Stdout ) -> Result<()> {
        
        queue!( ctx,
            // clear screen
            style::SetBackgroundColor( style::Color::Black ),
            style::SetForegroundColor( style::Color::White ),
            terminal::Clear( terminal::ClearType::All ),
    
            // print level number
            cursor::MoveTo(0,0),
            style::Print( format!("LEVEL {}", self.level) ),

            // print how many diamonds collected
            cursor::MoveTo(W-12,0),
            style::Print( format!("{} / {} ", self.initial_diamonds - self.diamonds, self.initial_diamonds) ),
        )?;

        self.draw_cell(ctx, Cell::Diamond( Falling::True ))?;

        queue!( ctx,
            cursor::MoveTo(0,1)
        )?;

        for _ in 0..MX+2 {
            self.draw_cell(ctx, Cell::Metal)?;
        }

        for y in 0..MY {
            queue!( ctx,
                cursor::MoveTo(0, (y+2) as u16)
            )?;

            self.draw_cell(ctx, Cell::Metal)?;

            for x in 0..MX {
                self.draw_cell(ctx, self.world[y * MX + x])?;
            }

            self.draw_cell(ctx, Cell::Metal)?;
        }

        queue!( ctx,
            cursor::MoveTo(0, H-1)
        )?;

        for _ in 0..MX+2 {
            self.draw_cell(ctx, Cell::Metal)?;
        }

        ctx.flush()?;

        Ok(())
    }




    fn draw_title( &self, ctx: &mut Stdout ) -> Result<()> {
        queue!( ctx,
            // clear screen
            style::SetBackgroundColor( style::Color::Black ),
            style::SetForegroundColor( style::Color::White ),
            terminal::Clear( terminal::ClearType::All ),
    
            // print level
            cursor::MoveTo(35,10),
            style::Print( format!("LEVEL {}", self.level) ),
            cursor::MoveTo(33,12)
        )?;

        self.draw_cell(ctx, Cell::Diamond(Falling::True) )?;
    
        queue!( ctx,
            // how many diamonds?
            style::SetBackgroundColor( style::Color::Black ),
            style::SetForegroundColor( style::Color::White ),
            style::Print( format!("  x  {}", self.initial_diamonds) ),
        )?;

        // flush buffer onto screen
        ctx.flush()?;

        Ok(())
    }




    fn get (&self, x: isize, y: isize) -> Cell {
        if x < 0 || y < 0 {
            return Cell::Metal;
        }
        let x = x as usize;
        let y = y as usize;

        if x >= MX || y >= MY {
            Cell::Metal
        } else {
            self.world[y * MX + x]
        }
    }




    fn new_world( rng: &mut SmallRng ) -> (Vec<Cell>, usize) {

        // dimensions
        let n_cells = MX * MY;
        let mut world = Vec::with_capacity(n_cells);
        let mut diamonds = 0;

        // place cells
        for i in 0..n_cells {
            let number : f64 = rng.gen();

            if number < 0.75 {
                world.push(Cell::Soil);
            } else if number < 0.90 {
                world.push(Cell::Boulder(Falling::False));
            } else if number < 0.95 {
                world.push(Cell::Metal);
            } else if number < 0.98 {
                world.push(Cell::Diamond(Falling::False));
                diamonds += 1;
            } else {
                if i > n_cells/2 {
                    world.push(Cell::Enemy);
                } else {
                    world.push(Cell::Diamond(Falling::False));
                }
                diamonds += 1;
            }
        }

        // place player
        let number : u8 = rng.gen();

        if let Cell::Diamond(_) | Cell::Enemy = world[number as usize] {
            diamonds -= 1;
        }

        world[number as usize] = Cell::Player;
        
        (world, diamonds)
    }




    fn set (&mut self, x: isize, y: isize, val: Cell) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;

        if x >= MX || y >= MY {
            return;
        } else {
            self.world[y * MX + x] = val;
        }
    }





    fn update_inputs( &mut self ) -> Result<()> {

        // event handler
        while poll(time::Duration::from_millis(0))? {

            match read()? {
                Event::Key(event) => {

                    // move player's pad
                    if event.code == KeyCode::Right {
                        self.current_input = PlayerInput::Right;
                    }
                    else if event.code == KeyCode::Left {
                        self.current_input = PlayerInput::Left;
                    }
                    else if event.code == KeyCode::Up {
                        self.current_input = PlayerInput::Up;
                    }
                    else if event.code == KeyCode::Down {
                        self.current_input = PlayerInput::Down;
                    }
                    // if c or ctrl+c is hit, exit
                    else if event.code == KeyCode::Char('c') {
                        cleanup();
                    }
                    else {
                        self.current_input = PlayerInput::None;
                    }
                }

                // we are not interested in mouse, resize events
                _ => {}
            }
        } // end of event handler

        Ok(())
    }




    fn update_player(&mut self) {

        let my = MY as isize;
        let mx = MX as isize;

        let mut pos : Option<(isize, isize)> = None;

        'player_finding:
        for y in 0..my {
            for x in 0..mx {
                if let Cell::Player = self.get(x,y) {
                    pos = Some((x,y));
                    break 'player_finding;
                }
            }
        }

        let x;
        let y;
        match pos {
            Some((xx,yy)) => {
                x = xx;
                y = yy;
            }
            None => {
                self.scene = Scene::GameoverScene;
                self.counter = 0;
                return;
            }
        }

        let (dx, dy) = match self.current_input {
            PlayerInput::Up => (0,-1),
            PlayerInput::Down => (0,1),
            PlayerInput::Left => (-1,0),
            PlayerInput::Right =>  (1,0),
            PlayerInput::None => (0,0)
        };

        let next = self.get(x+dx, y+dy);
        let next_of_next = self.get(x+dx+dx, y+dy+dy);

        let can_move = match (next, self.current_input) {
            (Cell::Player, _) |
            (Cell::Metal, _) => false,
            (Cell::Boulder(_), PlayerInput::Down) => false,
            (Cell::Boulder(_), PlayerInput::Up) |
            (Cell::Diamond(Falling::True), PlayerInput::Up) => false,
            (Cell::Boulder(_), PlayerInput::Right) |
            (Cell::Boulder(_), PlayerInput::Left) if !Cell::empty(next_of_next) => false,
            _ => true,
        };

        if can_move {
            self.set(x, y, Cell::Empty);
            match next {
                Cell::Enemy => {
                    self.scene = Scene::GameoverScene;
                    self.counter = 0;
                }
                Cell::Diamond(_) => {
                    self.diamonds -= 1;
                    if self.diamonds == 0 {
                        self.scene = Scene::LevelupScene;
                        self.counter = 0;
                    }
                }
                Cell::Boulder(_) => {
                    self.set(x+dx+dx, y+dy+dy, Cell::Boulder(Falling::True));
                }
                _ => ()
            }
            self.set(x+dx, y+dy, Cell::Player);
        }

        // reset after single use
        self.current_input = PlayerInput::None;

        
    }




    fn update_rocks(&mut self ) {
        let my = MY as isize;
        let mx = MX as isize;

        // boulder update
        for y in (0..my-1).rev() {
            for x in 0..mx {

                let this = self.get(x,y);

                if !Cell::rock( this ) {
                    continue;
                }

                let dx = if self.counter % 2 == 0 {1} else {-1};
                let bottom = self.get(x, y+1);
                let side1 = self.get(x+dx, y);
                let diag1 = self.get(x+dx, y+1);
                let side2 = self.get(x-dx, y);
                let diag2 = self.get(x-dx, y+1);
                
                let falling = Cell::falling(this);
                if falling && Cell::crushable(bottom) {
                    // fall

                    match bottom {
                        Cell::Empty |
                        Cell::Player => {
                            self.set(x, y, Cell::Empty);
                            if let Cell::Boulder(_) = this {
                                self.set(x, y+1, Cell::Boulder(Falling::True) );
                            } else {
                                self.set(x, y+1, Cell::Diamond(Falling::True) );
                            }
                        }
                        
                        Cell::Enemy => {
                            self.set(x, y+1, Cell::Diamond(Falling::True) );
                        }
                        _ => () // not reachable
                    }
                }
                else if !falling && Cell::empty(bottom) {
                    // start falling
                    self.set(x, y, Cell::Empty);
                    if let Cell::Boulder(_) = this {
                        self.set(x, y+1, Cell::Boulder(Falling::True) );
                    } else {
                        self.set(x, y+1, Cell::Diamond(Falling::True) );
                    }

                }
                else if falling && Cell::empty(side1) && Cell::crushable(diag1) {
                    // roll to side 1
                    self.set(x, y, Cell::Empty);
                    if let Cell::Boulder(_) = this {
                        self.set(x+dx, y, Cell::Boulder(Falling::True) );
                    } else {
                        self.set(x+dx, y, Cell::Diamond(Falling::True) );
                    }
                }
                else if falling && Cell::empty(side2) && Cell::crushable(diag2) {
                    // roll to side 2
                    self.set(x, y, Cell::Empty);
                    if let Cell::Boulder(_) = this {
                        self.set(x-dx, y, Cell::Boulder(Falling::True) );
                    } else {
                        self.set(x-dx, y, Cell::Diamond(Falling::True) );
                    }
                }
                else {
                    // stop
                    if let Cell::Boulder(_) = this {
                        self.set(x, y, Cell::Boulder(Falling::False) );
                    } else {
                        self.set(x, y, Cell::Diamond(Falling::False) );
                    }
                }
                
            }
        }
    }

} // impl Game




fn cleanup() {
	execute!( stdout(),
		cursor::Show,
		terminal::LeaveAlternateScreen,
		style::ResetColor
	).unwrap();
	disable_raw_mode().unwrap();
	std::process::exit(0);
}




fn main() {
	match start_game() {
		Ok(_) => {}
		Err(_) => cleanup()
	}
}




fn start_game() -> Result<()> {

    // raw mode = no echoing, manual ctrl+c, no buffering
    enable_raw_mode()?;

    let mut ctx = stdout();

    // init
	queue!( ctx,
		cursor::Hide,
		terminal::EnterAlternateScreen,
		// terminal::SetSize(W,H)
    )?;
    
    let mut rng = SmallRng::from_entropy();

    // initial game state
    let (world, diamonds) = Game::new_world( &mut rng );
    let mut game = Game {
        initial_world: world.clone(),
        initial_diamonds: diamonds,
        world,
        diamonds,
        scene: Scene::TitleScene,
        level: 1,
        counter: 0,
        current_input: PlayerInput::None
    };

    loop {
        match game.scene {
            Scene::TitleScene => {
                game.draw_title( &mut ctx )?;
                if game.counter == 10 {
                    game.scene = Scene::LevelScene;
                }
            }

            Scene::LevelScene => {
                game.draw_level( &mut ctx )?;
                game.update_rocks();
                game.update_player();
            }

            Scene::GameoverScene => {
                game.draw_level( &mut ctx )?;
                game.update_rocks();
                if game.counter == 5 {
                    game.world = game.initial_world.clone();
                    game.diamonds = game.initial_diamonds;
                    game.counter = 0;
                    game.scene = Scene::TitleScene;
                }
            }

            Scene::LevelupScene => {
                game.draw_level( &mut ctx )?;
                game.update_rocks();
                if game.counter == 5 {
                    let (world, diamonds) = Game::new_world( &mut rng );
                    game.initial_world = world;
                    game.initial_diamonds = diamonds;
                    game.world = game.initial_world.clone();
                    game.diamonds = game.initial_diamonds;
                    game.counter = 0;
                    game.scene = Scene::TitleScene;
                    game.level += 1;
                }
            }
        }

        game.update_inputs()?;
        game.counter = if game.counter == u32::MAX { 0 } else { game.counter + 1 };
        thread::sleep( time::Duration::from_millis(60) );
    }

}
