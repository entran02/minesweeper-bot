use crate::cell::Cell;
use crate::info;
use crate::posn::Posn;
use fantoccini::{Client, ClientBuilder, Locator};
use fantoccini::error::CmdError;
use std::collections::HashSet;
use rand::seq::IteratorRandom;
use tokio::time::{Instant, Duration};

pub struct Board {
    pub log: bool,
    pub mark_flags: bool,
    pub rows: usize,
    pub cols: usize,
    mines: usize,
    blank: HashSet<Cell>,
    bombs: HashSet<Cell>,
    numbers: HashSet<Cell>,
    workset: HashSet<Cell>,
    matrix: Vec<Vec<Cell>>,
    client: Client,
    pub face_locator: String,
}

impl Board {
    pub async fn new(log: bool, mark_flags: bool, client: Client) -> Result<Self, fantoccini::error::CmdError> {
        let start_time = Instant::now();

        let rows = 16;
        let cols = 30;
        let mines = 99;
        let link = "https://minesweeperonline.com/";
        //let client = ClientBuilder::native().connect("http://localhost:9515").await.expect("failed to connect to webdriver.");

        client.goto(link).await?;
        client.wait().for_element(Locator::Css(".square.blank")).await?;

        let face_locator = "#face".to_string(); // Adjust based on HTML structure
        client.find(Locator::Css(&face_locator)).await?;

        let mut matrix: Vec<Vec<Cell>> = Vec::new();
        let blank: HashSet<Cell> = HashSet::new();
        let bombs: HashSet<Cell> = HashSet::new();
        let numbers: HashSet<Cell> = HashSet::new();
        let workset: HashSet<Cell> = HashSet::new();

        let mut board = Board {
            log,
            mark_flags,
            rows,
            cols,
            mines,
            blank: HashSet::new(),
            bombs: HashSet::new(),
            numbers: HashSet::new(),
            workset: HashSet::new(),
            matrix: vec![],
            client: client.clone(),
            face_locator,
        };
        
        board.init_fields_and_cells().await?;

        if log {
            let elapsed = start_time.elapsed();
            println!(
                "\t{:?} seconds to initialize a board with {} cells",
                elapsed.as_secs_f32(),
                board.rows * board.cols
            );
        }

        Ok(board)
    }

    pub async fn play(&mut self) -> Result<(), CmdError> {
        while !self.blank.is_empty() {
            let to_flag = self.get_cells_to_flag();
            println!("Cells to flag: {:?}", to_flag);
            if !to_flag.is_empty() {
                self.log_action("flag".to_string());
                self.flag_all(to_flag).await;
            } else {
                let to_reveal = self.get_cells_to_reveal();
                if !to_reveal.is_empty() {
                    self.log_action("reveal".to_string());
                    self.reveal_all(to_reveal).await;
                } else {
                    self.log_action("random".to_string());
                    self.reveal_random().await;
                }
            }
        }
        Ok(())
    }

    pub async fn reset_game(&mut self) {
        self.log_action("reset".to_string());
        self.client.find(Locator::Css("#face")).await.unwrap().click().await.unwrap();
        self.blank.clear();
        self.bombs.clear();
        self.numbers.clear();
        self.workset.clear();
        for row in &mut self.matrix {
            for cell in row {
                cell.reset();
                self.blank.insert((*cell).clone());
            }
        }
    }

    pub async fn reveal_random(&mut self) -> Result<(), fantoccini::error::CmdError> {
        // Filter blank cells that have no non-zero number neighbors
        let no_numbers: HashSet<_> = self.blank.iter()
            .filter(|&&ref cell| cell.non_zero_number_neighbors().is_empty())
            .cloned()
            .collect();

        // Initialize the lowest probability and random cell
        let mut lowest_prob: f64;
        let mut random_cell = if no_numbers.is_empty() {
            lowest_prob = 1.0;
            self.blank.iter().choose(&mut rand::thread_rng()).unwrap().clone()
        } else {
            lowest_prob = (self.mines - self.bombs.len()) as f64 / no_numbers.len() as f64;
            no_numbers.iter().choose(&mut rand::thread_rng()).unwrap().clone()
        };

        // Check all cells in the workset for a better probability
        for cell in &self.workset {
            let blank_neighbors = cell.blank_neighbors();
            if !blank_neighbors.is_empty() {
                let prob = cell.bombs_remaining() as f64 / blank_neighbors.len() as f64;
                if prob < lowest_prob {
                    lowest_prob = prob;
                    random_cell = blank_neighbors.iter().choose(&mut rand::thread_rng()).unwrap().clone().clone();
                }
            }
        }

        // Click on the chosen random cell and update from it
        random_cell.click().await?;
        self.update_from(vec![random_cell].into_iter().collect()).await?;

        Ok(())
    }

    pub async fn init_fields_and_cells(&mut self) -> Result<(), fantoccini::error::CmdError> {
        for row in 0..self.rows {
            let mut row_vec = vec![];
            for col in 0..self.cols {
                let cell = Cell::new(row as i32, col as i32, self.client.clone());
                row_vec.push(cell.clone());
                self.blank.insert(cell);
            }
            self.matrix.push(row_vec);
        }

        let matrix_snapshot = self.matrix.clone();
        for row in &mut self.matrix {
            for cell in row {
                let neighbors: HashSet<Cell> = cell.neighbors_posns(self.rows as i32, self.cols as i32)
                    .into_iter()
                    .map(|pos| matrix_snapshot[pos.row as usize][pos.col as usize].clone())
                    .collect();
                cell.assign_neighbors(neighbors);
            }
        }

        Ok(())
    }

    fn log_action(&self, action: String) {
        if self.log {
            println!("{}", action);
        }
    }

    async fn flag_all(&mut self, to_flag: HashSet<Cell>) {
        for mut cell in to_flag {
            println!("Preparing to flag the cell.");
            cell.flag(self.mark_flags).await.unwrap();
            println!("Cell flagged successfully.");
            self.blank.remove(&cell);
            self.bombs.insert(cell);
        }
    }

    pub async fn update_from(&mut self, mut workset: HashSet<Cell>) -> Result<(), CmdError> {
        let start_time = Instant::now();
        let mut counter = 0;
        let mut visited: HashSet<Cell> = HashSet::new();

        while !workset.is_empty() {
            // Pop an element from the workset
            let mut popped = workset.iter().next().cloned().unwrap();
            workset.remove(&popped);

            // If the popped cell is not already visited
            if !visited.contains(&popped) {
                visited.insert(popped.clone());

                // Call update on the cell
                let (updated, boom) = popped.update().await?;

                // If there's a boom, reset the game
                if boom {
                    self.reset_game().await;
                    return Ok(());
                } else if updated {
                    // Add blank neighbors of the popped cell to the workset
                    workset.extend(popped.blank_neighbors().into_iter().cloned());

                    // Remove from blank and add to numbers set
                    self.blank.remove(&popped);
                    self.numbers.insert(popped.clone());

                    // If the cell should be added to the workset, do so
                    if popped.should_add_to_workset() {
                        self.workset.insert(popped);
                    }

                    counter += 1;
                }
            }
        }

        // Log the time taken to update the cells
        let elapsed = start_time.elapsed();
        self.log_action(format!("\t{:.3} seconds to update {} cells", elapsed.as_secs_f32(), counter));

        Ok(())
    }

    async fn reveal_all(&mut self, to_reveal: HashSet<Cell>) {
        if to_reveal.len() == self.blank.len() {
            for cell in &to_reveal {
                cell.click().await.unwrap();
                self.blank.remove(cell);
                self.numbers.insert(cell.clone());
            }
            self.log_action("complete".to_string());
        } else {
            let mut to_update = HashSet::new();
            for cell in &to_reveal {
                cell.click().await.unwrap();
                to_update.insert(cell.clone());
            }
            self.update_from(to_update).await;
        }
    }

    fn get_cells_to_flag(&self) -> HashSet<Cell> { 
        let mut to_flag = HashSet::new();
        for cell in &self.workset {
            let neighbors_to_flag = cell.get_neighbors_to_flag();
            to_flag.extend(neighbors_to_flag.into_iter().cloned());
            println!("Neighbors to flag: {:?}", to_flag);
        }
        to_flag
    }

    fn get_cells_to_reveal(&mut self) -> HashSet<Cell> {
        let mut to_reveal = HashSet::new();
        let mut to_discard = HashSet::new();
        for cell in &self.workset {
            let (exhausted, neighbors) = cell.get_neighbors_to_reveal();
            if exhausted {
                to_discard.insert(cell.clone());
            }
            to_reveal.extend(neighbors.into_iter().cloned());
        }
        self.workset = self.workset.difference(&to_discard).cloned().collect();
        to_reveal
    }

    pub fn to_string(&self) -> String {
        let mut out = format!(
            "Board(rows = {},\n\tcols = {}\n\tmines = {}\n\tblank.size = {}\n\tbombs.size = {}\n\tnumbers.size = {}\n\tworkset.size = {}\n",
            self.rows, self.cols, self.mines, self.blank.len(), self.bombs.len(), self.numbers.len(), self.workset.len()
        );
        out.push_str("\tMatrix as String:\n");
        for row in &self.matrix {
            out.push_str("\t\t");
            for cell in row {
                out.push_str(&format!("{} ", cell.field_string()));
            }
            out.push('\n');
        }
        out
    }
}