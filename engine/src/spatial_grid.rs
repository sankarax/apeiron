use crate::food::Food;
use crate::organism::Organism;
pub struct SpatialGrid {
    width: f32,
    height: f32,

    cell_width: f32,
    cell_height: f32,

    columns: usize,
    rows: usize,

    cells: Vec<Vec<usize>>,
}

impl SpatialGrid {
    pub fn new(width: f32, height: f32, target_cell_size: f32) -> Self {
        assert!(width.is_finite() && width > 0.0);
        assert!(height.is_finite() && height > 0.0);

        assert!(target_cell_size.is_finite() && target_cell_size > 0.0);

        let columns = ((width / target_cell_size).ceil() as usize).max(1);

        let rows = ((height / target_cell_size).ceil() as usize).max(1);

        let cell_width = width / columns as f32;

        let cell_height = height / rows as f32;

        let cells = vec![Vec::new(); columns * rows];

        Self {
            width,
            height,

            cell_width,
            cell_height,

            columns,
            rows,

            cells,
        }
    }

    // convert world coordinates to grid coordinates

    fn cell_coordinates(&self, x: f32, y: f32) -> (usize, usize) {
        let x = x.rem_euclid(self.width);

        let y = y.rem_euclid(self.height);

        let column = (x / self.cell_width).floor() as usize;

        let row = (y / self.cell_height).floor() as usize;

        (column.min(self.columns - 1), row.min(self.rows - 1))
    }

    fn cell_index(&self, column: usize, row: usize) -> usize {
        row * self.columns + column
    }

    // grid construction

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    pub fn insert(&mut self, index: usize, x: f32, y: f32) {
        let (column, row) = self.cell_coordinates(x, y);

        let cell_index = self.cell_index(column, row);

        self.cells[cell_index].push(index);
    }

    pub fn rebuild_food(&mut self, food: &[Food]) {
        self.clear();

        for (index, resource) in food.iter().enumerate() {
            self.insert(index, resource.x, resource.y);
        }
    }

    // spatial query

    pub fn for_each_candidate(&self, x: f32, y: f32, radius: f32, mut visit: impl FnMut(usize)) {
        assert!(radius.is_finite() && radius >= 0.0);

        let x = x.rem_euclid(self.width);

        let y = y.rem_euclid(self.height);

        let min_column = ((x - radius) / self.cell_width).floor() as isize;

        let max_column = ((x + radius) / self.cell_width).floor() as isize;

        let min_row = ((y - radius) / self.cell_height).floor() as isize;

        let max_row = ((y + radius) / self.cell_height).floor() as isize;

        // avoid duplicate wrapped cells

        let column_count = ((max_column - min_column + 1) as usize).min(self.columns);

        let row_count = ((max_row - min_row + 1) as usize).min(self.rows);

        for row_offset in 0..row_count {
            let row = (min_row + row_offset as isize).rem_euclid(self.rows as isize) as usize;

            for column_offset in 0..column_count {
                let column = (min_column + column_offset as isize).rem_euclid(self.columns as isize)
                    as usize;

                let index = self.cell_index(column, row);

                for &item_index in &self.cells[index] {
                    visit(item_index);
                }
            }
        }
    }

    // diagnostics

    pub fn dimensions(&self) -> (usize, usize) {
        (self.columns, self.rows)
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn occupied_cells(&self) -> usize {
        self.cells.iter().filter(|cell| !cell.is_empty()).count()
    }

    pub fn item_count(&self) -> usize {
        self.cells.iter().map(Vec::len).sum()
    }
    pub fn rebuild_organisms(&mut self, organisms: &[Organism]) {
        self.clear();

        for (index, organism) in organisms.iter().enumerate() {
            self.insert(index, organism.x, organism.y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_dimensions_are_correct() {
        let grid = SpatialGrid::new(1600.0, 900.0, 50.0);

        assert_eq!(grid.dimensions(), (32, 18),);

        assert_eq!(grid.cell_count(), 576);
    }

    #[test]
    fn inserts_items_into_grid() {
        let mut grid = SpatialGrid::new(100.0, 100.0, 20.0);

        grid.insert(0, 10.0, 10.0);
        grid.insert(1, 50.0, 50.0);
        grid.insert(2, 90.0, 90.0);

        assert_eq!(grid.item_count(), 3);

        assert_eq!(grid.occupied_cells(), 3);
    }

    #[test]
    fn query_finds_nearby_items() {
        let mut grid = SpatialGrid::new(100.0, 100.0, 20.0);

        grid.insert(0, 45.0, 45.0);
        grid.insert(1, 55.0, 55.0);
        grid.insert(2, 90.0, 90.0);

        let mut candidates = Vec::new();

        grid.for_each_candidate(50.0, 50.0, 15.0, |index| candidates.push(index));

        assert!(candidates.contains(&0));

        assert!(candidates.contains(&1));

        assert!(!candidates.contains(&2));
    }

    #[test]
    fn query_wraps_horizontal_boundary() {
        let mut grid = SpatialGrid::new(100.0, 100.0, 20.0);

        grid.insert(0, 98.0, 50.0);

        let mut candidates = Vec::new();

        grid.for_each_candidate(2.0, 50.0, 10.0, |index| candidates.push(index));

        assert!(candidates.contains(&0));
    }

    #[test]
    fn query_wraps_vertical_boundary() {
        let mut grid = SpatialGrid::new(100.0, 100.0, 20.0);

        grid.insert(0, 50.0, 98.0);

        let mut candidates = Vec::new();

        grid.for_each_candidate(50.0, 2.0, 10.0, |index| candidates.push(index));

        assert!(candidates.contains(&0));
    }

    #[test]
    fn query_wraps_corner() {
        let mut grid = SpatialGrid::new(100.0, 100.0, 20.0);

        grid.insert(0, 98.0, 98.0);

        let mut candidates = Vec::new();

        grid.for_each_candidate(2.0, 2.0, 10.0, |index| candidates.push(index));

        assert!(candidates.contains(&0));
    }

    #[test]
    fn large_query_does_not_duplicate_items() {
        let mut grid = SpatialGrid::new(100.0, 100.0, 20.0);

        grid.insert(0, 10.0, 10.0);
        grid.insert(1, 50.0, 50.0);
        grid.insert(2, 90.0, 90.0);

        let mut candidates = Vec::new();

        grid.for_each_candidate(50.0, 50.0, 500.0, |index| candidates.push(index));

        candidates.sort_unstable();

        assert_eq!(candidates, vec![0, 1, 2],);
    }

    #[test]
    fn clearing_grid_removes_items() {
        let mut grid = SpatialGrid::new(100.0, 100.0, 20.0);

        grid.insert(0, 10.0, 10.0);

        grid.clear();

        assert_eq!(grid.item_count(), 0);

        assert_eq!(grid.occupied_cells(), 0);
    }

    #[test]
    fn irregular_world_dimensions() {
        let mut grid = SpatialGrid::new(103.0, 77.0, 20.0);

        grid.insert(0, 102.0, 76.0);

        let mut candidates = Vec::new();

        grid.for_each_candidate(1.0, 1.0, 5.0, |index| candidates.push(index));

        assert!(candidates.contains(&0));
    }
}
