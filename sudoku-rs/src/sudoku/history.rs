use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct History<T: Clone + Debug> {
    limit: usize,
    past_records: VecDeque<T>,
    future_records: VecDeque<T>,
}

impl<T: Clone + Debug> History<T> {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit,
            past_records: VecDeque::with_capacity(limit),
            future_records: VecDeque::with_capacity(limit),
        }
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
        self.past_records.truncate(limit);
        self.past_records.truncate(limit);
    }

    fn push_bounded(&mut self, past: bool, value: T) {
        if self.limit == 0 {
            return;
        }

        let queue = if past {
            &mut self.past_records
        } else {
            &mut self.future_records
        };

        if queue.len() >= self.limit {
            queue.pop_back();
        }
        queue.push_front(value);
    }

    pub fn push(&mut self, record: T) {
        self.future_records.clear();

        self.push_bounded(true, record);
    }

    pub fn go_back(&mut self, current_record: &T) -> Option<T> {
        if let Some(past_record) = self.past_records.pop_front() {
            self.push_bounded(false, current_record.clone());
            Some(past_record)
        } else {
            None
        }
    }

    pub fn can_go_back(&self) -> bool {
        !self.past_records.is_empty()
    }

    pub fn go_forward(&mut self, current_record: &T) -> Option<T> {
        if let Some(future_record) = self.future_records.pop_front() {
            self.push_bounded(true, current_record.clone());
            Some(future_record)
        } else {
            None
        }
    }

    pub fn can_go_forward(&self) -> bool {
        !self.future_records.is_empty()
    }
}

pub const DEFAULT_LIMIT: usize = 255;

impl<T: Clone + Debug> Default for History<T> {
    fn default() -> Self {
        Self::with_limit(DEFAULT_LIMIT)
    }
}
