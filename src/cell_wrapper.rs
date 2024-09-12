use crate::cell::Cell;
use std::rc::Rc;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::collections::HashSet;
use fantoccini::Client;
use crate::posn::Posn;
use fantoccini::error::CmdError;

#[derive(Clone, Debug)]
pub struct CellWrapper(pub Rc<RefCell<Cell>>);

impl PartialEq for CellWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.borrow().eq(&other.0.borrow())
    }
}

impl Eq for CellWrapper {}

impl Hash for CellWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

impl CellWrapper {
    pub fn new(cell: Cell) -> Self {
        CellWrapper(Rc::new(RefCell::new(cell)))
    }

    pub fn with_params(row: i32, col: i32, client: Client) -> Self {
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
        
        CellWrapper(Rc::new(RefCell::new(cell)))
    }

    pub fn borrow(&self) -> std::cell::Ref<Cell> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<Cell> {
        self.0.borrow_mut()
    }

    pub fn clone_rc(&self) -> Rc<RefCell<Cell>> {
        self.0.clone()
    }

    pub fn blank_neighbors(&self) -> HashSet<CellWrapper> {
        let cell = self.borrow();
        cell.neighbors
            .iter()
            .filter(|neighbor| neighbor.borrow().blank)
            .cloned()
            .collect()
    }

    pub fn bombs_remaining(&self) -> i32 {
        let cell = self.borrow();
        let bomb_neighbors_count = cell.neighbors
            .iter()
            .filter(|neighbor| neighbor.borrow().bomb)
            .count();
        cell.cell_integer - bomb_neighbors_count as i32
    }

    pub async fn flag(&self, mark_flags: bool) -> Result<(), CmdError> {
        self.0.borrow_mut().flag(mark_flags).await
    }
}
