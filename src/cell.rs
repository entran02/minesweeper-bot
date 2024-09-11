use fantoccini::error::CmdError;
use fantoccini::{Client, Locator};
use fantoccini::actions::{MouseActions, PointerAction, InputSource, MOUSE_BUTTON_RIGHT};
use crate::posn::Posn;
use crate::info;
use std::collections::HashSet;
use std::iter;
use tokio::time::Duration;
use std::hash::{Hash, Hasher};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Cell{
    bomb: bool,
    blank: bool,
    number: bool,
    attribute: String,
    cell_integer: i32,
    pub posn: Posn,
    neighbors: HashSet<Cell>,
    client: Client,
    locator: String,
    //pub ATTRIBUTES: HashMap<&'static str, char>
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
            // We are intentionally excluding neighbors from equality checks
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
    pub fn new(row: i32, col: i32, client: Client) -> Self {
        let posn = Posn::new(row, col);
        let locator = format!(r#"#\3{}_{}"#, row + 1, col + 1);
        //let attributes = info::get_reps();
        Cell {
            bomb: false,
            blank: true,
            number: false,
            attribute: "square blank".to_string(),
            cell_integer: -1,
            posn,
            neighbors: HashSet::new(),
            client,
            locator,
            //ATTRIBUTES: attributes
        }
    }

    pub fn assign_neighbors(&mut self, neighbors: HashSet<Cell>) {
        if !self.neighbors.is_empty() {
            panic!("Neighbors already assigned");
        }
        self.neighbors = neighbors;
        //println!("Assigned neighbors for cell {:?}: {:?}", self.posn, self.neighbors);

    }

    pub async fn flag(&mut self, mark_flags: bool) -> Result<(), fantoccini::error::CmdError> {
        if !self.blank {
            panic!("Cannot flag a non-blank cell");
        }
        println!("Flagging: {:?}", self.locator);
        self.bomb = true;
        self.blank = false;

        if mark_flags {
            let element = self.client.find(Locator::Css(&self.locator)).await?;
            let move_action = PointerAction::MoveToElement {
                element: element.clone(),
                duration: Some(Duration::from_millis(500)),  // Optional: Move over 500ms
                x: 0,  // Optional: X offset from the center of the element
                y: 0,  // Optional: Y offset from the center of the element
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
            } else {
                println!("Non-number attribute: {}", self.attribute);
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
        if !self.blank {
            panic!("Cannot click a non-blank cell");
        }

        println!("Selector: {:?}", Locator::Css(&self.locator));
        let element = self.client.find(Locator::Css(&self.locator)).await?;
        element.click().await?;
        Ok(())
    }

    pub fn bomb_neighbors(&self) -> HashSet<&Cell> {
        self.neighbors.iter().filter(|neighbor| neighbor.bomb).collect()
    }

    pub fn blank_neighbors(&self) -> HashSet<&Cell> {
        self.neighbors.iter().filter(|neighbor| neighbor.blank).collect()
    }

    pub fn get_number(&self) -> i32 {
        if !self.number {
            panic!("Cell is not a number");
        }
        println!("Get number Cell integer: {}", self.cell_integer);
        self.cell_integer
    }

    pub fn non_zero_number_neighbors(&self) -> HashSet<&Cell> {
        let neighbors = self.neighbors.iter().filter(|neighbor| neighbor.number && neighbor.get_number() > 0).collect();
        //println!("Non zero number neighbors: {:?}", neighbors);
        //println!("neighbors: {:?}", neighbors);
        //println!("self.neighbors: {:?}", self.neighbors);
        neighbors
    }

    pub fn bombs_remaining(&self) -> i32 {
        self.cell_integer - self.bomb_neighbors().len() as i32
    }

    pub fn get_more_to_flag(&self) -> HashSet<&Cell> {
        let mut pattern_flag: HashSet<&Cell> = HashSet::new();

        for neighbor in self.non_zero_number_neighbors() {
            let diff: HashSet<&Cell> = neighbor.blank_neighbors().difference(&self.blank_neighbors()).cloned().collect();
            if diff.len() as i32 == neighbor.bombs_remaining() - self.bombs_remaining() {
                pattern_flag.extend(diff);
            }
        }
        println!("Pattern flag: {:?}", pattern_flag);
        pattern_flag
    }

    pub fn get_more_to_reveal(&self) -> HashSet<&Cell> {
        let mut pattern_reveal: HashSet<&Cell> = HashSet::new();

        let this_blank: HashSet<_> = self.blank_neighbors();
        for neighbor in self.non_zero_number_neighbors() {
            let other_blank: HashSet<_> = neighbor.blank_neighbors();
            if this_blank.is_subset(&other_blank) && self.bombs_remaining() == neighbor.bombs_remaining() {
                let diff: HashSet<_> = other_blank.difference(&this_blank).cloned().collect();
                if !diff.is_empty() {
                    pattern_reveal.extend(diff);
                }
            }
        }
        pattern_reveal
    }

    pub fn should_add_to_workset(&self) -> bool {
        let result = self.number && self.cell_integer > 0 && !self.blank_neighbors().is_empty();
        println!("Should add to workset for cell {:?}: {}", self, result);
        result
    }

    pub fn get_neighbors_to_flag(&self) -> HashSet<&Cell> {
        let blank_neighbors = self.blank_neighbors();
        if blank_neighbors.len() == self.bombs_remaining() as usize{
            blank_neighbors
        } else {
            self.get_more_to_flag()
        }
    }

    pub fn field_string(&self) -> String {
        if self.blank {
            "▢".to_string()
        } else if self.bomb {
            "⚐".to_string()
        } else {
            self.cell_integer.to_string()
        }
    }

    pub async fn update_attribute(&mut self) -> Result<String, CmdError> {
        let element = self.client.find(Locator::Css(&self.locator)).await?;
        
        // If the attribute exists, update and return it
        if let Some(attribute) = element.attr("class").await? {
            self.attribute = attribute.clone();
            if attribute.contains("bombdeath") {
                self.bomb = true; // This will set bomb to true when a bomb is clicked
            }
            Ok(attribute)
        } else {
            Err(CmdError::NotW3C(Value::String("Attribute not found".to_string())))
        }
    }

    pub async fn update(&mut self) -> Result<(bool, bool), CmdError> {
        if !self.blank {
            panic!("Cannot update a non-blank cell");
        }
    
        // Retrieve the current attribute of the cell (class or other identifier)
        let current_attribute = self.attribute.clone();
        let new_attribute = self.update_attribute().await?;
        let attributes = info::get_reps();
    
        // If the attribute has changed, check whether it's a bomb or a number
        if current_attribute != new_attribute {
            self.attribute = new_attribute.clone();
    
            // Check if the new attribute indicates the cell is a bomb
            if self.bomb {
                // A bomb was clicked, trigger boom
                println!("BOOM!");
                return Ok((false, true));
            } else if attributes.contains_key(&new_attribute.as_str()) {
                // It's not a bomb but a number, process the number
                self.to_number();
                return Ok((true, false)); // Updated with no boom
            }
        }
    
        Ok((false, false)) // No update or boom
    }
    
    pub fn get_neighbors_to_reveal(&self) -> (bool, HashSet<&Cell>) {
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