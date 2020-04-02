extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

static BLOCK_SIZE: u32 = 10;
static ROWS: u32 = 25;
static COLS: u32 = 30;

pub struct Game {
    gl: GlGraphics,
    board: Board,
    score: u32,
    playing: bool,
}

pub struct Board {
    rows: u32,
    cols: u32, 
    food: Food,
    snek: Snek,
}

pub struct Food {
    block: Block,
    block_size: u32,
}

#[derive(Clone)]
pub struct Block(u32, u32);


pub struct Snek {
    body: Vec<Block>,
    block_size: u32,
    just_ate: bool,
    gl: GlGraphics,
    dir: Direction,
}

#[derive(PartialEq, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Snek {
    
    //initialize the snek image
    pub fn init(&mut self, args: &RenderArgs)
    {
        
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        
        let squares: Vec<graphics::types::Rectangle> = self.body
        .iter()
        .map(|block| Block(block.0 * self.block_size, block.1 * self.block_size))
        .map(|block| graphics::rectangle::square(block.0 as f64, block.1 as f64, self.block_size as f64))
        .collect();
        
        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            squares
                .into_iter()
                .for_each(|square| graphics::rectangle(WHITE, square, transform, gl));
        })
    }
    
    //check for self collision
    fn collision(&self, x: u32, y: u32) -> bool
    {
        self.body.iter().any(|block| x == block.0 && y == block.1)
    }
    
    //move mr snek 
    pub fn update(&mut self, just_ate: bool, cols: u32, rows: u32) -> bool 
    {
        let mut new_head: Block = (self.body[0]).clone();//.expect("mr snek has no head :(")).clone();
        
        if (self.dir == Direction::Up && new_head.1 == 0)
        || (self.dir == Direction::Down && new_head.1 == rows - 1)
        || (self.dir == Direction::Left && new_head.0 == 0)
        || (self.dir == Direction::Right && new_head.0 == cols - 1)
        {
            return false;
        }
        
        match self.dir 
        {
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1
        }
        
        if !just_ate 
        {
            self.body.pop();
        }
        
        if self.collision(new_head.0, new_head.1)
        {
            return false;
        }
        
        self.body.insert(0, new_head);
        true
    }
}

impl Food {
    
    fn init(&mut self, gl: &mut GlGraphics, args: &RenderArgs)
    {
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        
        let x = self.block.0 * self.block_size;
        let y = self.block.1 * self.block_size;
        
        let square = graphics::rectangle::square(x as f64, y as f64, self.block_size as f64);
        
        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            graphics::rectangle(RED, square, transform, gl)
        });
    }
    
    //was the food eaten?
    fn update(&mut self, s: &Snek) -> bool
    {
        let front = &s.body[0];
        if front.0 == self.block.0 && front.1 == self.block.1 { true }
        else { false }
    }
    
}

impl Game {
    fn init(&mut self, args: &RenderArgs) 
    {
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        
        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(BLUE, gl);
        });
        
        self.board.snek.init(args);
        self.board.food.init(&mut self.gl, args);
    }
    
    fn update(&mut self, args: &UpdateArgs) -> bool
    {
        if !self.board.snek.update(self.board.snek.just_ate, self.board.cols, self.board.rows)
        {
            return false;
        }
        
        if self.board.snek.just_ate
        {
            self.score += 1;
            self.board.snek.just_ate = false;
        }
        
        self.board.snek.just_ate = self.board.food.update(&self.board.snek);
        if self.board.snek.just_ate
        {
            use rand::Rng;
            use rand::thread_rng;
            let mut r = thread_rng();
            loop 
            {
                let new_x = r.gen_range(0, self.board.cols) as u32;
                let new_y = r.gen_range(0, self.board.rows) as u32;
                if !self.board.snek.collision(new_x, new_y)
                {
                    self.board.food = 
                    Food 
                    {
                        block: Block(new_x, new_y),
                        block_size: BLOCK_SIZE,
                    };
                    break;   
                }
            }
        }
        false
    }
    
    fn pressed(&mut self, btn: &Button) {
        let prev_dir = self.board.snek.dir.clone();
        
        self.board.snek.dir = match btn 
        {
            &Button::Keyboard(Key::Up)
                if prev_dir != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down)
                if prev_dir != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left)
                if prev_dir != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right)
                if prev_dir != Direction::Left => Direction::Right,
            _ => prev_dir
        };
    }
}

fn main() {
    
    let opengl = OpenGL::V3_2;
    
    let mut window: Window = WindowSettings::new(
        "snek game",
        [400, 500]
    )
    .graphics_api(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap();
    
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        board: Board {
            rows: ROWS,
            cols: COLS,
            food: Food {
                block: Block(10, 10),
                block_size: BLOCK_SIZE
            },
            snek: Snek {
                body: vec![ Block(1, 1) ],
                block_size: BLOCK_SIZE,
                just_ate: false,
                gl: GlGraphics::new(opengl),
                dir: Direction::Down
            }
        },
        score: 0,
        playing: true
    };
    
    let mut events = Events::new(EventSettings::new()).ups(16);
    while let Some(e) = events.next(&mut window) {
        
        if let Some(r) = e.render_args() {
            game.init(&r);
        }
        
        if let Some(u) = e.update_args() {
            game.update(&u);
        }
        
        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }



}
