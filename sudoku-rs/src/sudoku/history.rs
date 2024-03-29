use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(super) struct History<T> {
    limit: usize,
    past_records: VecDeque<T>,
    future_records: VecDeque<T>,
}

impl<T> History<T> {
    pub(super) fn with_limit(limit: usize) -> Self {
        Self {
            limit,
            past_records: VecDeque::with_capacity(limit),
            future_records: VecDeque::with_capacity(limit),
        }
    }

    pub(super) fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
        self.past_records.truncate(limit);
        self.future_records.truncate(limit);
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

    pub(super) fn can_go_back(&self) -> bool {
        !self.past_records.is_empty()
    }

    pub(super) fn can_go_forward(&self) -> bool {
        !self.future_records.is_empty()
    }
}

impl<T: Eq> History<T> {
    pub(super) fn push(&mut self, record: T) {
        self.future_records.clear();

        if let Some(past_record) = self.past_records.front() {
            if &record == past_record {
                return;
            }
        }

        self.push_bounded(true, record);
    }
}

impl<T: Clone> History<T> {
    pub(super) fn go_back(&mut self, current_record: &T) -> Option<T> {
        if let Some(past_record) = self.past_records.pop_front() {
            self.push_bounded(false, current_record.clone());
            Some(past_record)
        } else {
            None
        }
    }

    pub(super) fn go_forward(&mut self, current_record: &T) -> Option<T> {
        if let Some(future_record) = self.future_records.pop_front() {
            self.push_bounded(true, current_record.clone());
            Some(future_record)
        } else {
            None
        }
    }
}

pub(super) const DEFAULT_LIMIT: usize = 255;

impl<T> Default for History<T> {
    fn default() -> Self {
        Self::with_limit(DEFAULT_LIMIT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history() {
        let mut history = History::<i32>::with_limit(3);
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
        assert!(history.go_back(&-1).is_none());
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
        assert!(history.go_forward(&-1).is_none());
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        history.push(0);
        history.push(0);

        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        history.push(1);
        history.push(1);

        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        // [0,1] 2 []
        assert_eq!(history.go_back(&2), Some(1));
        // [0] 1 [2]

        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        // [0] 1 [2]
        assert_eq!(history.go_back(&1), Some(0));
        // [] 0 [1,2]

        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        // [] 0 [1,2]
        assert_eq!(history.go_forward(&0), Some(1));
        // [0] 1 [2]

        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        history.push(3);
        history.push(3);
        // [0, 3] ? []

        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        assert_eq!(history.go_back(&4), Some(3));
        assert_eq!(history.go_back(&3), Some(0));
        assert_eq!(history.go_back(&0), None);
    }
}
