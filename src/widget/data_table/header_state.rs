use std::rc::Rc;
use std::cmp::Ordering;

use crate::props::SorterFn;

use super::{IndexedHeader, IndexedHeaderSingle};

struct CellState {
    width: Option<f64>,
    hidden: bool,
    sort_order: Option<bool>,
}

/// Store for header state
///
/// - column sort order
/// - column hidden

pub struct HeaderState<T: 'static> {
    //headers: Rc<Vec<IndexedHeader<T>>>,
    // map cell_idx => &IndexedHeader
    cell_map: Vec<IndexedHeader<T>>,
    // map cell_idx => CellState,
    cell_state: Vec<CellState>,
    // map col_idx => DataTableColumn
    columns: Vec<Rc<IndexedHeaderSingle<T>>>,
}

impl<T: 'static> HeaderState<T> {
    pub fn new(headers: Rc<Vec<IndexedHeader<T>>>) -> Self {

        let mut cell_map = Vec::new();
        for header in headers.iter() {
            header.extract_cell_list(&mut cell_map);
        }

        let mut cell_state = Vec::new();
        for cell in cell_map.iter() {
            match cell {
                IndexedHeader::Single(cell) => {
                    cell_state.push(CellState {
                        width: None,
                        hidden: cell.column.hidden,
                        sort_order: cell.column.sort_order,
                    });
                }
                IndexedHeader::Group(group) => {
                   cell_state.push(CellState {
                        width: None,
                        hidden: group.hidden,
                        sort_order: None,
                    });
                }
            }
        }

        let mut columns = Vec::new();

        for header in headers.iter() {
            header.extract_column_list(&mut columns);
        }

        Self {
            // headers,
            columns,
            cell_map,
            cell_state,
        }
    }

    pub fn get_width(&self, col_num: usize) -> Option<f64> {
        let cell_idx = self.columns[col_num].cell_idx;
        self.cell_state[cell_idx].width
    }

    pub fn set_width(&mut self, col_num: usize, width: Option<f64>) {
        let cell_idx = self.columns[col_num].cell_idx;
        self.cell_state[cell_idx].width = width;
    }

    pub fn get_column_sorter(&self, cell_idx: usize) -> Option<bool> {
        self.cell_state[cell_idx].sort_order
    }

    /// Set sorter on single column, or reverse direction if exists
    pub fn set_column_sorter(&mut self, cell_idx: usize, order: Option<bool>) {
        let order = order.unwrap_or_else(|| {
            match self.cell_state[cell_idx].sort_order {
                Some(order) => !order,
                None => true,
            }
        });
        for cell in self.cell_state.iter_mut() { cell.sort_order = None; }
        self.cell_state[cell_idx].sort_order = Some(order);
    }

    /// Add sorter or reverse direction if exists
    pub fn add_column_sorter(&mut self, cell_idx: usize, order: Option<bool>) {
       let order = order.unwrap_or_else(|| {
            match self.cell_state[cell_idx].sort_order {
                Some(order) => !order,
                None => true,
            }
        });
        self.cell_state[cell_idx].sort_order = Some(order);
    }

    fn bubble_up_hidden(&mut self, cell_idx: Option<usize>) {
        let cell_idx = match cell_idx {
            Some(cell_idx) => cell_idx,
            None => return,
        };

        let group = match &self.cell_map[cell_idx] {
            IndexedHeader::Group(group) => group,
            IndexedHeader::Single(_) => { return; /* should not happen at all */ }
        };

        let visible = group.children.iter().find(|cell| !self.get_cell_hidden(cell.cell_idx())).is_some();
        self.cell_state[cell_idx].hidden = !visible;
        self.bubble_up_hidden(group.parent);
    }

    pub fn set_hidden(&mut self, cell_idx: usize, hidden: bool) {
        let header = &self.cell_map[cell_idx];

        for idx in header.cell_range() {
            self.cell_state[idx].hidden = hidden;
        }

        self.bubble_up_hidden(header.parent());
    }

    pub fn get_cell_hidden(&self, cell_idx: usize) -> bool {
        self.cell_state[cell_idx].hidden
    }

    pub fn get_column_hidden(&self, col_num: usize) -> bool {
        let cell_idx = self.columns[col_num].cell_idx;
        self.cell_state[cell_idx].hidden
    }

    /*
    pub fn toggle_cell_hidden(&mut self, cell_idx: usize) {
        let hidden = !self.get_cell_hidden(cell_idx);
        self.set_hidden(cell_idx, hidden);
    }
    */

    // Hidden status for colums (Used by the table renderer)
    pub fn hidden_columns(&self) -> Vec<bool> {
        self.columns.iter()
            .map(|cell| self.get_cell_hidden(cell.cell_idx))
            .collect()
    }

    // Hidden status for all cells (Used by the header menu)
    pub fn hidden_cells(&self) -> Vec<bool> {
        self.cell_state.iter()
            .map(|state| state.hidden)
            .collect()
    }

    pub fn cell_count(&self) -> usize {
        self.cell_map.len()
    }

    pub fn columns(&self) -> &[Rc<IndexedHeaderSingle<T>>] {
        &self.columns
    }

    pub fn visible_column_count(&self) -> usize {
        self.columns
            .iter()
            .filter(|cell| !self.get_cell_hidden(cell.cell_idx))
            .count()
    }

    pub fn copy_observed_widths(&mut self, col_idx: usize, observed_widths: &[Option<f64>]) {
        for i in 0..col_idx.min(self.columns.len()) {
            if self.get_width(i).is_none() {
                if let Some(Some(observed_width)) = observed_widths.get(i) {
                    self.set_width(i, Some(*observed_width));
                }
            }
        }
    }

    pub fn create_combined_sorter_fn(&self) -> SorterFn<T> {

         let sorters: Vec<(SorterFn<T>, bool)> = self.columns
            .iter()
            .filter_map(|cell| {
                let cell_idx = cell.cell_idx;
                let order = match self.get_column_sorter(cell_idx) {
                    Some(order) => order,
                    None => return None,
                };

                cell.column.sorter.as_ref().map(|sorter| (sorter.clone(), order))
            })
            .collect();

        SorterFn::new(move |a: &T, b: &T| {
            for (sort_fn, ascending) in &sorters {
                match if *ascending {
                    sort_fn.cmp(a, b)
                } else {
                    sort_fn.cmp(b, a)
                } {
                    Ordering::Equal => { /* continue */ },
                    other => return other,
                }
            }
            Ordering::Equal
        })
    }
}
