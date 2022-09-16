
pub(crate) struct ColumnSorterState {
    sorters: Vec<(/* col_num */ usize, /* ascending */ bool)>, 
}

impl ColumnSorterState {

    pub fn new(sorters: &[(usize, bool)]) -> Self {
        Self{ sorters: sorters.into() }
    }

    /*
    pub fn get_column_sort_order(&self, col_idx: usize) -> Option<bool> {
        for (idx, ascending) in self.sorters.iter() {
            if *idx == col_idx {
                return Some(*ascending);
            }
        }
        None
    }
    */
    
    /// Set sorter on single column, or reverse direction if exists
    pub fn set_column_sorter(&mut self, col_idx: usize) {
        if self.sorters.len() == 1 {
            let (cur_idx, ascending) = self.sorters[0];
            if cur_idx == col_idx {
                self.sorters = vec![(col_idx, !ascending)];
            } else {
                self.sorters = vec![(col_idx, true)];
            }
        } else {
            self.sorters = vec![(col_idx, true)];
        }
    }
    
    /// Add sorter or reverse direction if exists
    pub fn add_column_sorter(&mut self, col_idx: usize) {
        let mut found = false;
        for (idx, ascending) in self.sorters.iter_mut() {
            if *idx == col_idx {
                *ascending = !*ascending;
                found = true;
            }
        }
        if !found {
            self.sorters.push((col_idx, true));
        }
    }

    pub fn sorters(&self) -> &[(usize, bool)] {
        &self.sorters
    }
}
