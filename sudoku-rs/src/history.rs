use std::collections::VecDeque;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct History<T> {
    records: VecDeque<T>,
}

impl<T> History<T> {
    pub fn push(&mut self, grid: T, history_limit: usize) {
        if history_limit == 0 {
            return;
        }
        if self.records.len() >= history_limit {
            self.records.pop_front();
        }
        self.records.push_back(grid);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.records.pop_back()
    }
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self {
            records: Default::default(),
        }
    }
}
