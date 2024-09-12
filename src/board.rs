use fantoccini::{Client, Locator};
use fantoccini::error::CmdError;
use std::collections::HashSet;
use rand::seq::IteratorRandom;
use tokio::time::Instant;
use crate::cell_wrapper::CellWrapper;

pub struct Board {
    pub log: bool,
    pub mark_flags: bool,
    pub rows: usize,
    pub cols: usize,
    mines: usize,
    blank: HashSet<CellWrapper>,
    bombs: HashSet<CellWrapper>,
    numbers: HashSet<CellWrapper>,
    workset: HashSet<CellWrapper>,
    matrix: Vec<Vec<CellWrapper>>,
    client: Client,
}

impl Board {
    pub async fn new(log: bool, mark_flags: bool, client: Client) -> Result<Self, fantoccini::error::CmdError> {
        let start_time = Instant::now();

        let rows = 16;
        let cols = 30;
        let mines = 99;
        let link = "https://minesweeperonline.com/";

        client.goto(link).await?;
        client.wait().for_element(Locator::Css(".square.blank")).await?;

        let face_locator = "#face".to_string();
        client.find(Locator::Css(&face_locator)).await?;

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
            if let Ok(_) = self.client.find(Locator::Css("div#face.facewin")).await {
                println!("Game won!");
                return Ok(());
            }

            let to_flag = self.get_cells_to_flag();
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
                cell.0.borrow_mut().reset();
                self.blank.insert(cell.clone());
            }
        }
    }

    pub async fn reveal_random(&mut self) -> Result<(), fantoccini::error::CmdError> {
        // filter blank cells with no non zero neighbors
        let no_numbers: HashSet<_> = self.blank.iter()
            .filter(|&&ref cell| cell.0.borrow().non_zero_number_neighbors().is_empty())
            .cloned()
            .collect();

        // lowest probability of being a bomb
        let mut lowest_prob: f64;
        let mut random_cell = if no_numbers.is_empty() {
            lowest_prob = 1.0;
            self.blank.iter().choose(&mut rand::thread_rng()).unwrap().clone()
        } else {
            lowest_prob = (self.mines - self.bombs.len()) as f64 / no_numbers.len() as f64;
            no_numbers.iter().choose(&mut rand::thread_rng()).unwrap().clone()
        };

        // check workset for probabilities
        for cell in &self.workset {
            let blank_neighbors = cell.0.borrow().blank_neighbors();
            if !blank_neighbors.is_empty() {
                let prob = cell.0.borrow().bombs_remaining() as f64 / blank_neighbors.len() as f64;
                if prob < lowest_prob {
                    lowest_prob = prob;
                    random_cell = blank_neighbors.iter().choose(&mut rand::thread_rng()).unwrap().clone().clone();
                }
            }
        }

        random_cell.0.borrow().click().await?;
        self.update_from(vec![random_cell].into_iter().collect()).await?;

        Ok(())
    }

    pub async fn init_fields_and_cells(&mut self) -> Result<(), fantoccini::error::CmdError> {
        for row in 0..self.rows {
            let mut row_vec = vec![];
            for col in 0..self.cols {
                let cell = CellWrapper::with_params(row as i32, col as i32, self.client.clone());
                row_vec.push(cell.clone());
                self.blank.insert(cell);
            }
            self.matrix.push(row_vec);
        }

        let matrix_snapshot = self.matrix.clone();
        for row in &mut self.matrix {
            for cell in row {
                let neighbors: HashSet<CellWrapper> = cell.borrow().neighbors_posns(self.rows as i32, self.cols as i32)
                    .into_iter()
                    .map(|pos| matrix_snapshot[pos.row as usize][pos.col as usize].clone())
                    .collect();
                cell.borrow_mut().assign_neighbors(neighbors);
            }
        }

        Ok(())
    }

    fn log_action(&self, action: String) {
        if self.log {
            println!("{}", action);
        }
    }

    async fn flag_all(&mut self, to_flag: HashSet<CellWrapper>) {
        for cell in to_flag {
            cell.flag(self.mark_flags).await.unwrap();
            self.blank.remove(&cell);
            self.bombs.insert(cell);
        }
    }

    pub async fn update_from(&mut self, mut workset: HashSet<CellWrapper>) -> Result<(), CmdError> {
        let start_time = Instant::now();
        let mut counter = 0;
        let mut visited: HashSet<CellWrapper> = HashSet::new();

        // dfs approach
        while !workset.is_empty() {
            // pop first element
            let popped = workset.iter().next().cloned().unwrap();
            workset.remove(&popped);

            // check if the cell has been visited
            if !visited.contains(&popped) {
                visited.insert(popped.clone());

                let (updated, boom) = popped.0.borrow_mut().update().await?;

                if boom {
                    self.reset_game().await;
                    return Ok(());
                } else if updated {
                    workset.extend(popped.0.borrow().blank_neighbors().iter().cloned());

                    self.blank.remove(&popped);
                    self.numbers.insert(popped.clone());

                    if popped.0.borrow().should_add_to_workset() {
                        self.workset.insert(popped);
                    }

                    counter += 1;
                }
            }
        }

        let elapsed = start_time.elapsed();
        self.log_action(format!("\t{:.3} seconds to update {} cells", elapsed.as_secs_f32(), counter));

        Ok(())
    }

    async fn reveal_all(&mut self, to_reveal: HashSet<CellWrapper>) {
        if to_reveal.len() == self.blank.len() {
            for cell in &to_reveal {
                cell.0.borrow().click().await.unwrap();
                self.blank.remove(cell);
                self.numbers.insert(cell.clone());
            }
            self.log_action("complete".to_string());
        } else {
            let mut to_update = HashSet::new();
            for cell in &to_reveal {
                cell.0.borrow().click().await.unwrap();
                to_update.insert(cell.clone());
            }
            self.update_from(to_update).await;
        }
    }

    fn get_cells_to_flag(&self) -> HashSet<CellWrapper> { 
        let mut to_flag = HashSet::new();
        for cell in &self.workset {
            let neighbors_to_flag = cell.0.borrow().get_neighbors_to_flag();
            to_flag.extend(neighbors_to_flag.iter().cloned());
        }
        to_flag
    }
    
    fn get_cells_to_reveal(&mut self) -> HashSet<CellWrapper> {
        let mut to_reveal = HashSet::new();
        let mut to_discard = HashSet::new();
        for cell in &self.workset {
            let (exhausted, neighbors) = cell.0.borrow().get_neighbors_to_reveal();
            if exhausted {
                to_discard.insert(cell.clone());
            }
            to_reveal.extend(neighbors.iter().cloned());
        }
        self.workset = self.workset.difference(&to_discard).cloned().collect();
        to_reveal
    }
}