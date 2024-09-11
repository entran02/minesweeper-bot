#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Posn {
    pub row: i32,
    pub col: i32,
}

impl Posn {
    pub fn new(row: i32, col: i32) -> Self {
        Posn { row, col }
    }

    // string representation of the position
    pub fn to_string(&self) -> String {
        format!("Posn(row = {}, col = {})", self.row, self.col)
    }

    // string representation of coordinates
    pub fn coords(&self) -> String {
        format!("({}, {})", self.col + 1, self.row + 1)
    }

    // returns if the position is in the range of rows and cols
    pub fn in_range(&self, rows: i32, cols: i32) -> bool {
        self.row >= 0 && self.row < rows && self.col >= 0 && self.col < cols
    }

    // return 8 blocks surrounding the position
    pub fn surrounding(&self) -> Vec<Posn> {
        let mut positions = Vec::new();
        for row in (self.row - 1)..=(self.row + 1) {
            for col in (self.col - 1)..=(self.col + 1) {
                if row != self.row || col != self.col {
                    positions.push(Posn::new(row, col));
                }
            }
        }
        positions
    }
    
    // returns 8 blocks surrounding the position that are in range
    pub fn surrounding_in_range(&self, rows: i32, cols: i32) -> Vec<Posn> {
        let test = self.surrounding()
            .into_iter()
            .filter(|posn| posn.in_range(rows, cols))
            .collect();
        test
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posn_creation() {
        let pos = Posn::new(2, 3);
        assert_eq!(pos.row, 2);
        assert_eq!(pos.col, 3);
    }

    #[test]
    fn test_to_string() {
        let pos = Posn::new(1, 4);
        assert_eq!(pos.to_string(), "Posn(row = 1, col = 4)");
    }

    #[test]
    fn test_coordinates() {
        let pos = Posn::new(1, 4);
        assert_eq!(pos.coords(), "(5, 2)"); // col + 1, row + 1
    }

    #[test]
    fn test_in_range() {
        let pos = Posn::new(2, 3);
        assert!(pos.in_range(5, 5));
        assert!(!pos.in_range(2, 3));
    }

    #[test]
    fn test_surrounding() {
        let pos = Posn::new(1, 1);
        let surrounding = pos.surrounding();
        assert_eq!(surrounding.len(), 8); // 8 surrounding positions
        assert!(surrounding.contains(&Posn::new(0, 0)));
        assert!(surrounding.contains(&Posn::new(2, 2)));
    }

    #[test]
    fn test_surrounding_in_range() {
        let pos = Posn::new(0, 0);
        let surrounding = pos.surrounding_in_range(3, 3);
        assert_eq!(surrounding.len(), 3); // Only (0,1), (1,0), (1,1) are valid
        assert!(surrounding.contains(&Posn::new(1, 1)));
        assert!(!surrounding.contains(&Posn::new(-1, -1))); // Out of bounds
    }
}