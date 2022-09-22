use std::rc::Rc;
use std::cmp::Ordering;

use crate::props::SorterFn;

use super::{Header, DataTableColumn};

/// Store for header state
///
/// - column sort order
/// - column hidden

pub(crate) struct HeaderState<T: 'static> {
    headers: Rc<Vec<Header<T>>>,
    // map col_idx => DataTableColumn
    columns: Vec<DataTableColumn<T>>,
    // map col_idx => ascending
    sorters: Vec<Option<bool>>,
    // map cell_idx => hidden
    hidden: Vec<bool>,
    // map col_idx => width
    widths: Vec<Option<usize>>,
}

impl<T: 'static> HeaderState<T> {
    pub fn new(headers: Rc<Vec<Header<T>>>) -> Self {

        let hidden = Vec::new(); // fixme

        let mut columns = Vec::new();

        for header in headers.iter() {
            header.extract_column_list(&mut columns);
        }

        let sorters = columns.iter()
            .map(|column| column.sort_order)
            .collect();

        Self {
            headers,
            columns,
            sorters,
            hidden,
            widths: Vec::new(),
        }
    }

    pub fn get_width(&self, col_num: usize) -> Option<usize> {
        match self.widths.get(col_num) {
            Some(Some(width)) => Some(*width),
            _ => None,
        }
    }

    pub fn set_width(&mut self, col_num: usize, width: Option<usize>) {
        self.widths.resize((col_num + 1).max(self.widths.len()), None);
        self.widths[col_num] = width;
    }

    pub fn get_column_sorter(&self, col_num: usize) -> Option<bool> {
        match self.sorters.get(col_num) {
            Some(Some(asc)) => Some(*asc),
            _ => None,
        }
    }

    /// Set sorter on single column, or reverse direction if exists
    pub fn set_column_sorter(&mut self, col_num: usize, order: Option<bool>) {
        self.sorters.resize((col_num + 1).max(self.sorters.len()), None);

        let order = order.unwrap_or_else(|| match self.sorters[col_num] {
            Some(asc) => !asc,
            None => true,
        });
        self.sorters.fill(None);
        self.sorters[col_num] = Some(order);
    }

    /// Add sorter or reverse direction if exists
    pub fn add_column_sorter(&mut self, col_num: usize, order: Option<bool>) {
        self.sorters.resize((col_num + 1).max(self.sorters.len()), None);

        let order = order.unwrap_or_else(|| match self.sorters[col_num] {
            Some(asc) => !asc,
            None => true,
        });
        self.sorters[col_num] = Some(order);
    }

    pub fn set_hidden(&mut self, cell_idx: usize, hidden: bool) {
        self.hidden.resize((cell_idx + 1).max(self.hidden.len()), false);
        let header = match find_cell(&self.headers, cell_idx, &mut 0) {
            Some(header) => header,
            None => return,
        };
        
        let cell_end = get_cell_end(header, cell_idx);
        for idx in cell_idx..cell_end {
            self.hidden.resize((idx + 1).max(self.hidden.len()), false);
            self.hidden[idx] = hidden;
        }
        log::info!("HIDDEN {:?}", self.hidden);
    }

    pub fn get_hidden(&mut self, cell_idx: usize) -> bool {
        self.hidden.get(cell_idx).map(|h| *h).unwrap_or(false)
    }

    pub fn toggle_hidden(&mut self, cell_idx: usize) {
        let hidden = !self.get_hidden(cell_idx);
        self.set_hidden(cell_idx, hidden);
    }

    pub fn hidden_cells(&self) -> &[bool] {
        &self.hidden
    }
    
    pub fn columns(&self) -> &[DataTableColumn<T>] {
        &self.columns
    }

    pub fn copy_observed_widths(&mut self, col_idx: usize, observed_widths: &[Option<usize>]) {
        for i in 0..col_idx.min(self.columns.len()) {
            if self.get_width(i).is_none() {
                if self.columns[i].width.contains("fr") { // flex columns
                    if let Some(Some(observed_width)) = observed_widths.get(i) {
                        self.set_width(i, Some(*observed_width + 1));
                    }
                }
            }
        }
    }

    pub fn create_combined_sorter_fn(&self) -> SorterFn<T> {

         let sorters: Vec<(SorterFn<T>, bool)> = self.sorters
            .iter()
            .enumerate()
            .filter_map(|(col_idx, opt_order)| {
                let order = match opt_order {
                    Some(order) => *order,
                    None => return None,
                };

                let column = match self.columns.get(col_idx) {
                    None => return None,
                    Some(column) => column,
                };

                match &column.sorter {
                    None => None,
                    Some(sorter) => Some((sorter.clone(), order)),
                }
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

fn get_cell_end<T: 'static>(cell: &Header<T>, cell_idx: usize) -> usize {
    match cell {
        Header::Single(_) => {
            return cell_idx + 1;
        }
        Header::Group(group) => {
            let mut cur_idx = cell_idx + 1;
            for child in &group.children {
                cur_idx = get_cell_end(child, cur_idx);
            }
            return cur_idx;
        }
    }
}


fn find_cell<'a, T: 'static>(headers: &'a [Header<T>], cell_idx: usize, cur_idx: &mut usize) -> Option<&'a Header<T>> {
    for header in headers {
        if *cur_idx == cell_idx {
            return Some(header);
        }
        match header {
            Header::Single(_) => {
                *cur_idx += 1;
            }
            Header::Group(group) => {
                *cur_idx += 1;
                if let Some(cell) = find_cell(&group.children, cell_idx, cur_idx) {
                    return Some(cell);
                }
            }
        }
    }
    None
}
