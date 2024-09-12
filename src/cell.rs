use fantoccini::error::CmdError;
use fantoccini::{Client, Locator};
use fantoccini::actions::{MouseActions, PointerAction, InputSource, MOUSE_BUTTON_RIGHT};
use crate::posn::Posn;
use crate::info;
use std::borrow::BorrowMut;
use std::collections::HashSet;
use tokio::time::Duration;
use std::hash::{Hash, Hasher};
use serde_json::Value;
use crate::cell_wrapper::CellWrapper;

#[derive(Clone, Debug)]
pub struct Cell{
    pub bomb: bool,
    pub blank: bool,
    pub number: bool,
    pub attribute: String,
    pub cell_integer: i32,
    pub posn: Posn,
    pub neighbors:  HashSet<CellWrapper>,
    pub client: Client,
    pub locator: String,
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.bomb == other.bomb
            && self.blank == other.blank
            && self.number == other.number
            && self.cell_integer == other.cell_integer
            && self.posn == other.posn
            && self.attribute == other.attribute
            && self.locator == other.locator
    }
}

impl Eq for Cell {}

impl Hash for Cell {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bomb.hash(state);
        self.blank.hash(state);
        self.number.hash(state);
        self.cell_integer.hash(state);
        self.posn.hash(state);
        self.attribute.hash(state);
        self.locator.hash(state);
    }
}

impl Cell {
    pub fn new(row: i32, col: i32, client: Client) -> CellWrapper {
        let posn = Posn::new(row, col);
        let locator = if row + 1 < 10 {
            format!(r#"#\3{}_{}"#, row + 1, col + 1)
        } else {
            format!(r#"#\31 {}_{}"#, row + 1 - 10, col + 1)
        };
        
        let cell = Cell {
            bomb: false,
            blank: true,
            number: false,
            attribute: "square blank".to_string(),
            cell_integer: -1,
            posn,
            neighbors: HashSet::new(),
            client,
            locator,
        };

        CellWrapper::new(cell)
    }

    pub fn assign_neighbors(&mut self, neighbors: HashSet<CellWrapper>) {
        if !self.neighbors.is_empty() {
            panic!("Neighbors already assigned");
        }
        self.neighbors = neighbors;

    }

    pub async fn flag(&mut self, mark_flags: bool) -> Result<(), fantoccini::error::CmdError> {
        if !self.blank {
            panic!("Cannot flag a non-blank cell");
        }
        self.bomb = true;
        self.blank = false;

        if mark_flags {
            let element = self.client.find(Locator::Css(&self.locator)).await?;
            let move_action = PointerAction::MoveToElement {
                element: element.clone(),
                duration: Some(Duration::from_millis(100)),
                x: 0,
                y: 0,
            };
            let mouse_actions = MouseActions::new("right".to_string())
                .then(move_action)
                .then(PointerAction::Down {
                    button: MOUSE_BUTTON_RIGHT,
                })
                .then(PointerAction::Up {
                    button: MOUSE_BUTTON_RIGHT,
                });
            self.client.perform_actions(mouse_actions).await?;
        }
        Ok(())
    }

    pub fn to_number(&mut self) {
        let reps = info::get_reps();
        if let Some(&num_char) = reps.get(&self.attribute.as_str()) {
            if let Some(digit) = num_char.to_digit(10) {
                self.number = true;
                self.blank = false;
                self.cell_integer = digit as i32;
            }
        } else {
            panic!("Invalid attribute: {}", self.attribute);
        }
    }

    pub fn reset(&mut self) {
        self.bomb = false;
        self.blank = true;
        self.number = false;
        self.attribute = "square blank".to_string();
    }

    pub async fn click(&self) -> Result<(), fantoccini::error::CmdError> {
        /*if !self.blank {
            panic!("Cannot click a non-blank cell");
        }*/

        let element = self.client.find(Locator::Css(&self.locator)).await?;
        element.click().await?;
        Ok(())
    }

    pub fn bomb_neighbors(&self) -> HashSet<CellWrapper> {
        self.neighbors.iter().filter(|neighbor| neighbor.borrow().bomb).cloned().collect()
    }

    pub fn blank_neighbors(&self) -> HashSet<CellWrapper> {
        self.neighbors.iter().filter(|neighbor| neighbor.borrow().blank).cloned().collect()
    }

    pub fn get_number(&self) -> i32 {
        if !self.number {
            panic!("Cell is not a number");
        }
        self.cell_integer
    }

    pub fn non_zero_number_neighbors(&self) -> HashSet<CellWrapper> {
        let neighbors = self.neighbors
            .iter()
            .filter(|neighbor| {
                let cell = neighbor.borrow();
                cell.number && cell.cell_integer > 0
            })
            .cloned()
            .collect();
        neighbors
    }

    pub fn bombs_remaining(&self) -> i32 {
        self.cell_integer - self.bomb_neighbors().len() as i32
    }

    pub fn get_more_to_flag(&self) -> HashSet<CellWrapper> {
        let mut pattern_flag: HashSet<CellWrapper> = HashSet::new();

        for neighbor in self.non_zero_number_neighbors() {
            let diff: HashSet<CellWrapper> = neighbor.blank_neighbors().difference(&self.blank_neighbors()).cloned().collect();
            if diff.len() as i32 == neighbor.bombs_remaining() - self.bombs_remaining() {
                pattern_flag.extend(diff);
            }
        }
        pattern_flag
    }

    pub fn get_more_to_reveal(&self) -> HashSet<CellWrapper> {
        let mut pattern_reveal: HashSet<CellWrapper> = HashSet::new();

        let this_blank: HashSet<CellWrapper> = self.blank_neighbors();

        for neighbor in self.non_zero_number_neighbors() {
            let other_blank: HashSet<CellWrapper> = neighbor.blank_neighbors();

            if this_blank.is_subset(&other_blank) && self.bombs_remaining() == neighbor.bombs_remaining() {
                let diff: HashSet<CellWrapper> = other_blank.difference(&this_blank).cloned().collect();
                if !diff.is_empty() {
                    pattern_reveal.extend(diff);
                }
            }
        }
        pattern_reveal
    }

    pub fn should_add_to_workset(&self) -> bool {
        let result = self.number && self.cell_integer > 0 && !self.blank_neighbors().is_empty();
        result
    }

    pub fn get_neighbors_to_flag(&self) -> HashSet<CellWrapper> {
        let blank_neighbors = self.blank_neighbors();
        if blank_neighbors.len() == self.bombs_remaining() as usize{
            blank_neighbors
        } else {
            self.get_more_to_flag()
        }
    }

    pub async fn update_attribute(&mut self) -> Result<String, CmdError> {
        let element = self.client.find(Locator::Css(&self.locator)).await?;
        
        if let Some(attribute) = element.attr("class").await? {
            // can be mut cell?? or not?? idk lmao
            let cell = self.borrow_mut();
            cell.attribute = attribute.clone();
            if attribute.contains("bombdeath") {
                cell.bomb = true; // hard-reset
            }
            Ok(attribute)
        } else {
            Err(CmdError::NotW3C(Value::String("Attribute not found".to_string())))
        }
    }

    pub async fn update(&mut self) -> Result<(bool, bool), CmdError> {
        if !self.blank {
            self.bomb = true;
            // currently resets and tries again if met with 50/50 scenario
            // TODO: implement random blank square click in this case
            //self.click().await?;
        }
    
        let current_attribute = self.attribute.clone();
        let new_attribute = self.update_attribute().await?;
        let attributes = info::get_reps();
    
        // if square has changed, check if it's a bomb or a number
        if current_attribute != new_attribute {
            self.attribute = new_attribute.clone();
    
            if self.bomb {
                println!("BOOM!");
                return Ok((false, true));
            } else if attributes.contains_key(&new_attribute.as_str()) {
                // if a number, process the number
                self.to_number();
                return Ok((true, false));
            }
        }
    
        Ok((false, false))
    }
    
    pub fn get_neighbors_to_reveal(&self) -> (bool, HashSet<CellWrapper>) {
        if self.get_number() == self.bomb_neighbors().len() as i32 {
            (true, self.blank_neighbors())
        } else {
            (false, self.get_more_to_reveal())
        }
    }

    pub fn neighbors_posns(&self, rows: i32, cols: i32) -> Vec<Posn> {
        self.posn.surrounding_in_range(rows, cols)
    }
}