mod utils;

extern crate js_sys;
extern crate fixedbitset;
extern crate web_sys;

use wasm_bindgen::prelude::*;
use fixedbitset::FixedBitSet;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

impl Universe {

    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_col == 0 && delta_row == 0 {
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
}

#[wasm_bindgen]
impl Universe {

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.init_cells();
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.init_cells();
        
    }

    fn init_cells(&mut self) {
        let size = (self.width * self.height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, false);
        }

        self.cells = cells;
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        log!(
            "toggling state of ({:?}, {:?})",
            row,
            column
        );
        self.cells.toggle(idx);
    }

    fn clear_cells(&mut self,  start:(u32,u32), end:(u32,u32)) {
        for row in start.0..end.0 {
            for column in start.1..end.1 {
                self.cells.set(self.get_index(row, column), false);
            }
        }
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 64;
        let height = 64;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, i % 2 == 0 || i % 7 == 0);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn clear(&mut self) {
        self.init_cells();
    }

    pub fn random(&mut self) {
        let size = (self.width * self.height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, js_sys::Math::random() < 0.5);
        }
        self.cells = cells;
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                /*
                log!(
                    "cell [{}, {}] is initially {:?} and has {} live neighbors",
                    row,
                    col,
                    cell,
                    live_neighbors
                );
                */

                next.set(idx, match (cell, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (curr_state, _) => curr_state,

                });
            }
        }

        self.cells = next;
    }

    pub fn insert_glider(&mut self, row: u32, column: u32) {
        self.clear_cells((row-2, column-2), (row+2, column+2));
        self.cells.set(self.get_index(row, column-1), true);
        self.cells.set(self.get_index(row-1, column+1), true);
        self.cells.set(self.get_index(row, column+1), true);
        self.cells.set(self.get_index(row+1, column), true);
        self.cells.set(self.get_index(row+1, column+1), true);
    }

    pub fn insert_pulsar(&mut self, row: u32, column: u32) {
        self.clear_cells((row-7, column-7), (row+7, column+7));

        self.cells.set(self.get_index(row-6, column-2), true);
        self.cells.set(self.get_index(row-6, column-3), true);
        self.cells.set(self.get_index(row-6, column-4), true);

        self.cells.set(self.get_index(row-4, column-6), true);
        self.cells.set(self.get_index(row-3, column-6), true);
        self.cells.set(self.get_index(row-2, column-6), true);

        self.cells.set(self.get_index(row-4, column-1), true);
        self.cells.set(self.get_index(row-3, column-1), true);
        self.cells.set(self.get_index(row-2, column-1), true);

        self.cells.set(self.get_index(row-1, column-2), true);
        self.cells.set(self.get_index(row-1, column-3), true);
        self.cells.set(self.get_index(row-1, column-4), true);

        //

        self.cells.set(self.get_index(row-6, column+2), true);
        self.cells.set(self.get_index(row-6, column+3), true);
        self.cells.set(self.get_index(row-6, column+4), true);

        self.cells.set(self.get_index(row-4, column+6), true);
        self.cells.set(self.get_index(row-3, column+6), true);
        self.cells.set(self.get_index(row-2, column+6), true);

        self.cells.set(self.get_index(row-4, column+1), true);
        self.cells.set(self.get_index(row-3, column+1), true);
        self.cells.set(self.get_index(row-2, column+1), true);

        self.cells.set(self.get_index(row-1, column+2), true);
        self.cells.set(self.get_index(row-1, column+3), true);
        self.cells.set(self.get_index(row-1, column+4), true);

        //

        self.cells.set(self.get_index(row+6, column-2), true);
        self.cells.set(self.get_index(row+6, column-3), true);
        self.cells.set(self.get_index(row+6, column-4), true);

        self.cells.set(self.get_index(row+4, column-6), true);
        self.cells.set(self.get_index(row+3, column-6), true);
        self.cells.set(self.get_index(row+2, column-6), true);

        self.cells.set(self.get_index(row+4, column-1), true);
        self.cells.set(self.get_index(row+3, column-1), true);
        self.cells.set(self.get_index(row+2, column-1), true);

        self.cells.set(self.get_index(row+1, column-2), true);
        self.cells.set(self.get_index(row+1, column-3), true);
        self.cells.set(self.get_index(row+1, column-4), true);

        //

        self.cells.set(self.get_index(row+6, column+2), true);
        self.cells.set(self.get_index(row+6, column+3), true);
        self.cells.set(self.get_index(row+6, column+4), true);

        self.cells.set(self.get_index(row+4, column+6), true);
        self.cells.set(self.get_index(row+3, column+6), true);
        self.cells.set(self.get_index(row+2, column+6), true);

        self.cells.set(self.get_index(row+4, column+1), true);
        self.cells.set(self.get_index(row+3, column+1), true);
        self.cells.set(self.get_index(row+2, column+1), true);

        self.cells.set(self.get_index(row+1, column+2), true);
        self.cells.set(self.get_index(row+1, column+3), true);
        self.cells.set(self.get_index(row+1, column+4), true);
    }
}


