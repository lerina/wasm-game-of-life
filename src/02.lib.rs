mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// ======================================================================
// We have several ways of exposing the universe's cells to JavaScript. 
// To begin, we will implement std::fmt::Display for Universe, 
// which we can use to generate a Rust String of the cells rendered as text 
// characters. This Rust String is then copied from the WebAssembly linear 
// memory into a JavaScript String in the JavaScript's garbage-collected heap, 
// and is then displayed by setting HTML textContent
//

// It is important that we have #[repr(u8)], 
// so that each cell is represented as a single byte. 
// It is also important that the Dead variant is 0 
// and that the Alive variant is 1, 
// so that we can easily count a cell's live neighbors with addition.
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

// Next, let's define the universe. 
// The universe has a width and a height, 
// and a vector of cells of length width * height.
#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}


// To access the cell at a given row and column, 
// we translate the row and column into an index 
// into the cells vector, 
impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    // In order to calculate the next state of a cell, 
    // we need to get a count of how many of its neighbors are alive.
    //
    // The live_neighbor_count method uses deltas and modulo to avoid special 
    // casing the edges of the universe with ifs. When applying a delta of -1, 
    // we add self.height - 1 and let the modulo do its thing, rather than 
    // attempting to subtract 1. row and column can be 0, and if we attempted 
    // to subtract 1 from them, there would be an unsigned integer underflow.
    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

}//^-- impl Universe

// Now we have everything we need to compute the next generation from 
// the current one! 


// Each of the Game's rules follows a straightforward translation into a 
// condition on a match expression. Additionally, because we want JavaScript 
// to control when ticks happen, we will put this method inside 
// a #[wasm_bindgen] block, so that it gets exposed to JavaScript.


// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    // We define a constructor that initializes the universe 
    // with an interesting pattern of live and dead cells. 
    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    // Rendering to Canvas Directly from Memory
    // Generating (and allocating) a String in Rust 
    // and then having wasm-bindgen convert it to a valid JavaScript string 
    // makes unnecessary copies of the universe's cells. 
    // As the JavaScript code already knows the width and height of the universe, 
    // and can read WebAssembly's linear memory that make up the cells directly, 
    // we'll modify the render method to return a pointer to the start of 
    // the cells array.
    //
    // Also, instead of rendering Unicode text, 
    // we'll switch to using the Canvas API
    // 
    // old version 
    // pub fn render(&self) -> String {
    //    self.to_string()
    // }
   // 
   // To get the necessary information from the Rust implementation, 
   // we'll need to add some more getter functions for a universe's 
   // width, height, and pointer to its cells array. 
   // All of these are exposed to JavaScript as well
   pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }


}//^-- impl Universe

// The state of the universe is represented as a vector of cells. 
// To make this human readable, let's implement a basic text renderer. 
// The idea is to write the universe line by line as text, and for each cell 
// that is alive, print the Unicode character ◼ ("black medium square"). 
// For dead cells, we'll print ◻ (a "white medium square").
use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

