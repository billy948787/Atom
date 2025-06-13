#[derive(Debug, Clone, Default)]
pub struct Matrix<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T: Clone + Default> Matrix<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        let data = vec![T::default(); rows * cols];
        Matrix { data, rows, cols }
    }

    pub fn default(rows: usize, cols: usize) -> Self {
        Matrix::new(rows, cols)
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        if row < self.rows && col < self.cols {
            Some(&self.data[row * self.cols + col])
        } else {
            None
        }
    }

    pub fn set(&mut self, row: usize, col: usize, value: T) -> Result<(), String> {
        if row < self.rows && col < self.cols {
            self.data[row * self.cols + col] = value;
            Ok(())
        } else {
            Err("Index out of bounds".to_string())
        }
    }
}
