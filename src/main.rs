use std::collections::HashMap;

use ggez::{event::EventHandler, graphics, timer, Context, ContextBuilder, GameResult};
use rand::Rng;

const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 1000.0;

const NUM_CELLS: usize = 100; // Number of cells

// Calculate the size of each cell
const CELL_SIZE: f32 = WINDOW_WIDTH / NUM_CELLS as f32;

// Adjust the grid dimensions based on the cell size
const GRID_WIDTH: usize = (WINDOW_WIDTH / CELL_SIZE) as usize;
const GRID_HEIGHT: usize = (WINDOW_HEIGHT / CELL_SIZE) as usize;
const BIT_VEC_SIZE: usize = (GRID_WIDTH * GRID_HEIGHT + 63) / 64;

struct Game {
    cells: Vec<u64>,
    next_cells: Vec<u64>,
    cell_ages: HashMap<(usize, usize), usize>,
}

impl Game {
    fn new() -> Game {
        let mut rng = rand::thread_rng();
        let mut cells = vec![0; BIT_VEC_SIZE];
        let mut cell_ages = HashMap::new();

        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let is_alive = rng.gen_bool(0.5); // 50% chance of being alive
                if is_alive {
                    let bit_index = y * GRID_WIDTH + x;
                    let vec_index = bit_index / 64;
                    let bit_offset = bit_index % 64;
                    cells[vec_index] |= 1 << bit_offset;

                    cell_ages.insert((x, y), 1); // Initialize age as 1
                }
            }
        }

        Game {
            cells,
            next_cells: vec![0; BIT_VEC_SIZE],
            cell_ages,
        }
    }

    fn get_cell_state(&self, x: usize, y: usize) -> bool {
        let bit_index = y * GRID_WIDTH + x;
        let vec_index = bit_index / 64;
        let bit_offset = bit_index % 64;

        (self.cells[vec_index] & (1 << bit_offset)) != 0
    }

    fn set_cell_state(&mut self, x: usize, y: usize, state: bool) {
        let bit_index = y * GRID_WIDTH + x;
        let vec_index = bit_index / 64;
        let bit_offset = bit_index % 64;

        if state {
            self.next_cells[vec_index] |= 1 << bit_offset;
        } else {
            self.next_cells[vec_index] &= !(1 << bit_offset);
        }
    }

    fn update_game_state(&mut self) {
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let alive = self.get_cell_state(x, y);
                let neighbors = self.count_alive_neighbors(x, y);
                let next_state = match (alive, neighbors) {
                    (true, 2) | (_, 3) => true,
                    _ => false,
                };

                if next_state {
                    let age = self.cell_ages.entry((x, y)).or_insert(0);
                    if !alive {
                        *age = 0; // Reset age for newly born cells
                    }
                    *age += 1;
                } else {
                    self.cell_ages.remove(&(x, y)); // Remove age tracking for dead cells
                }

                self.set_cell_state(x, y, next_state);
            }
        }

        std::mem::swap(&mut self.cells, &mut self.next_cells);
    }

    fn count_alive_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;

        for dy in [-1, 0, 1].iter().cloned() {
            for dx in [-1, 0, 1].iter().cloned() {
                if dx == 0 && dy == 0 {
                    continue; // Skip the current cell
                }

                let nx = ((x as isize + dx + GRID_WIDTH as isize) % GRID_WIDTH as isize) as usize;
                let ny = ((y as isize + dy + GRID_HEIGHT as isize) % GRID_HEIGHT as isize) as usize;

                if self.get_cell_state(nx, ny) {
                    count += 1;
                }
            }
        }

        count
    }
}

impl EventHandler<ggez::GameError> for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.update_game_state();
        timer::sleep(std::time::Duration::from_millis(150)); // Control the update rate
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::BLACK);

        // Create a single mesh for a cell
        let cell_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, CELL_SIZE, CELL_SIZE),
            graphics::Color::WHITE,
        )?;

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if let Some(age) = self.cell_ages.get(&(x, y)) {
                    let neighbors = self.count_alive_neighbors(x, y);
                    let color = calculate_color(*age, neighbors);

                    // Draw the cell with the calculated color
                    let draw_params = graphics::DrawParam::default()
                        .dest(ggez::mint::Point2 {
                            x: x as f32 * CELL_SIZE,
                            y: y as f32 * CELL_SIZE,
                        })
                        .color(color);
                    graphics::draw(ctx, &cell_mesh, draw_params)?;
                }
            }
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

fn calculate_color(age: usize, neighbors: usize) -> graphics::Color {
    // Define base colors
    let active_color = graphics::Color::from_rgb(236, 68, 155);
    let stable_color = graphics::Color::from_rgb(153, 244, 67);
    let max_active_age = 10; // Age threshold for a cell to be considered stable

    let color = if age <= max_active_age {
        active_color
    } else {
        stable_color
    };

    // Slightly adjust brightness based on neighbors
    let brightness_factor = 1.0 - (neighbors as f32 * 0.05).min(0.2);
    graphics::Color::new(
        color.r * brightness_factor,
        color.g * brightness_factor,
        color.b * brightness_factor,
        1.0,
    )
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("Game of Life", "Ishwor")
        .window_setup(ggez::conf::WindowSetup::default().title("Conway's Game of Life"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()?;

    let game = Game::new();

    ggez::event::run(ctx, event_loop, game)
}
