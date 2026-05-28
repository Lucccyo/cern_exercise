use std::{cmp::Ordering};

pub trait Orderable {
    fn index(&self) -> u32;
}

/// A data structure where the tuple ([`time`], [`index`]) determines the ordering.
///
/// [`element`] is ignored in comparisions.
///
#[derive(Debug, Clone)]
pub struct Element {
    pub time: u64,
    pub index: u32,
    pub element: Vec<u8>,
}

impl Orderable for Element {
    fn index(&self) -> u32 {
        self.index
    }
}

impl Ord for Element {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.time, self.index).cmp(&(other.time, other.index))
    }
}

impl PartialOrd for Element {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Element {}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.time == other.time
    }
}

/// A data structure which buffers and sorts until the elements are sequential and without gaps.
#[derive(Debug)]
pub struct Orderer<T> {
    buffer: Vec<T>,
    min_buffer_index: Option<u32>,
    is_ordered: bool,
    next_expected_index: Option<u32>,
}

impl<T: PartialOrd + Ord + Orderable> Orderer<T> {
    /// Creates a new orderer.
    pub fn new() -> Self {
        Orderer {
            buffer: vec![].into(),
            min_buffer_index: None,
            is_ordered: false,
            next_expected_index: None,
        }
    }

    /// Inserts `value` into the buffer or returns it if it is well ordered
    /// through the closure named pipe.
    ///
    /// If `value` is the logically next element in the sequence, it is returned immediately. More
    /// than one element might be returned if there are buffered elements that become sequential with
    /// the arrival of `value`.
    ///
    /// If `value` is not the logically next element in the sequence, it is buffered until it is
    /// sequential or `flush` is called.
    pub fn push(&mut self, value: T, mut pipe: impl FnMut(T)) {
        if self
            .next_expected_index
            .map(|index| index == value.index())
            .unwrap_or(true)
        {
            // Fast path: `value` is the logically next element.
            let next = value.index().overflowing_add(1).0;
            self.next_expected_index = Some(next);

            // The value is the expected one, and is returned by the closure directly
            pipe(value);
            if self.min_buffer_index == Some(next) {
                self.flush_sequential(&mut pipe);
            }
        } else if self.next_expected_index.map(|index| index > value.index()).unwrap_or(true)
        {
            return
        }
        else {
            // Slow path: `value` must be inserted into the buffer.

            let idx = value.index();
            self.buffer.push(value);
            // We can't ensure that the buffer is ordered after this push.
            self.is_ordered = false;
            // Modifying `min_buffer_index` if it is lower than the current one.
            if self.min_buffer_index.is_none() || self.min_buffer_index.is_some_and(|v| {v > idx}) {
                self.min_buffer_index = Some(idx);
            }

            return
        }
    }

    // Called only by flushes, the sort and remove duplicates is performed only when needed.
    fn sort_and_remove_duplicates(&mut self) {
        self.buffer.sort_unstable();
        self.buffer.dedup();
        self.is_ordered = true;
        return
    }

    // Flushes all buffered elements and resets the data structure.
    //
    // The returned elements are sorted.
    pub fn flush(&mut self) -> Vec<T> {
        self.next_expected_index = None;
        self.min_buffer_index = None;
        self.sort_and_remove_duplicates();
        self.buffer.drain(..).collect()
    }

    // Returns the number of buffered elements.
    pub fn buffered_count(&self) -> u32 {
        self.buffer.len() as u32
    }

    // Flushes the elements in the buffer which are sequential and without gaps, starting at
    // `self.next_expected_index`.
    fn flush_sequential(&mut self, pipe: impl FnMut(T)) {
        if !self.is_ordered {
            // sort the buffer only when there is at least one sequential value.
            self.sort_and_remove_duplicates();
        }
        // Count how many elements can be drained from the buffer.
        let count = self
            .buffer
            .iter()
            .take_while(|el| {
                let is_sequential = match self.next_expected_index {
                    Some(index) => el.index() == index,
                    None => true,
                };

                if is_sequential {
                    self.next_expected_index = Some(el.index().overflowing_add(1).0);
                    true
                } else {
                    false
                }
            })
            .count();

        self.buffer.drain(..count).for_each(pipe);
        self.min_buffer_index = self.buffer.first().map(|el| el.index())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // For the tests, the index of an u32 is the value itself.
    impl Orderable for u32 {
        fn index(&self) -> u32 {
            *self as _
        }
    }

    #[test]
    fn test_push() {
        let mut orderer = Orderer::new();

        // Assert that well ordered elements are returned immediately.
        let mut out = vec![]; orderer.push(0, |e| out.push(e));
        assert_eq!(out, [0]);
        // Assert that unordered elements are buffered and released once sequential.
        let mut out = vec![]; orderer.push(3, |e| out.push(e));
        assert_eq!(out, []);
        let mut out = vec![]; orderer.push(2, |e| out.push(e));
        assert_eq!(out, []);
        let mut out = vec![]; orderer.push(1, |e| out.push(e));
        assert_eq!(out, [1, 2, 3]);
    }

    #[test]
    fn test_flush() {
        let mut orderer = Orderer::new();

        // The first element is not buffered. The next elements are because they don't match the
        // next expected index.
        orderer.push(0, |_| ());
        orderer.push(5, |_| ());
        orderer.push(4, |_| ());
        orderer.push(3, |_| ());
        orderer.push(2, |_| ());

        // Assert that flush returns the buffered elements in sorted order.
        assert_eq!(orderer.flush(), [2, 3, 4, 5]);

        // Assert that the reset.
        assert_eq!(orderer.buffered_count(), 0);
        assert!(orderer.next_expected_index.is_none());
    }

    #[test]
    fn test_overflow() {
        let mut orderer = Orderer::new();

        // Create a sequence from (u32::MAX - 20) to 20.
        let insertion_sequence = {
            let mut sequence = vec![];
            let mut value = u32::MAX - 20;
            for _ in 0..40 {
                sequence.push(value);
                value = value.overflowing_add(1).0;
            }
            sequence
        };

        let mut retrieved_sequence = vec![];
        for &v in &insertion_sequence {
            orderer.push(v, |e| {retrieved_sequence.push(e);});
        }

        // Assert that the order did not change.
        assert_eq!(insertion_sequence, retrieved_sequence);
    }

    #[test]
    fn test_unordered_sequence_becomes_ordered() {
        let mut orderer = Orderer::new();

        let mut retrieved_sequence = vec![];
        orderer.push(0, |e| {retrieved_sequence.push(e);});

        // Create a sequence where the elements are reversed in chunks of 16.
        let insertion_sequence: Vec<u32> = (1..200)
            .collect::<Vec<_>>()
            .chunks_mut(16)
            .flat_map(|chunk| {
                chunk.reverse();
                chunk.to_vec()
            })
            .collect();

        // Push `insertion_seq` into the data structure and retrieve it into `retrieved_seq`.
        for el in insertion_sequence {
            orderer.push(el, |e| {retrieved_sequence.push(e);});
        }
        retrieved_sequence.append(&mut orderer.flush());

        // Assert that all elements were retrieved in sequential order.
        let is_sequential = retrieved_sequence
            .iter()
            .enumerate()
            .all(|(index, &el)| index as u32 == el);
        assert!(is_sequential);
    }

    #[test]
    fn test_complex_type() {
        let mut orderer = Orderer::new();

        let element0 = Element {
            time: 0,
            index: 0,
            element: vec![0, 1, 2],
        };
        let element1 = Element {
            time: 10,
            index: 1,
            element: vec![3, 4],
        };
        let element2 = Element {
            time: 20,
            index: 2,
            element: vec![5],
        };
        let element4 = Element {
            time: 40,
            index: 4,
            element: vec![6, 7, 8],
        };
        let element5 = Element {
            time: 50,
            index: 5,
            element: vec![9, 10, 11, 12, 13, 14, 15, 16],
        };


        let mut out = vec![]; orderer.push(element0.clone(), |e| out.push(e));
        assert_eq!(out, vec![element0]); // Returns inmediately

        let mut out = vec![]; orderer.push(element2.clone(), |e| out.push(e));
        assert_eq!(out, vec![]);
        let mut out = vec![]; orderer.push(element1.clone(), |e| out.push(e));
        assert_eq!(out, vec![element1, element2]);

        // Buffered because element 3 is missing
        let mut out = vec![]; orderer.push(element4.clone(), |e| out.push(e));
        assert_eq!(out, vec![]);
        let mut out = vec![]; orderer.push(element5.clone(), |e| out.push(e));
        assert_eq!(out, vec![]);

        assert_eq!(orderer.flush(), vec![element4, element5]);

        // Assert that the internal buffer is now empty
        assert_eq!(orderer.buffered_count(), 0);
        assert!(orderer.next_expected_index.is_none());
    }

    #[test]
    fn test_duplicated_indices() {
        let mut orderer = Orderer::new();

        let element0 = Element {
            time: 0,
            index: 0,
            element: vec![0, 1, 2],
        };
        let element1 = Element {
            time: 10,
            index: 0,
            element: vec![3, 4],
        };

        let mut out = vec![]; orderer.push(element0.clone(), |e| out.push(e));
        assert_eq!(out, vec![element0]); // Returns inmediately

        let mut out = vec![];
        orderer.push(element1.clone(), |e| out.push(e));
        assert_eq!(out, vec![]);
    }

    #[test]
    fn test_duplicate_in_buffer() {
        let mut orderer = Orderer::new();

        // The first element is not buffered. The next elements are because they don't match the
        // next expected index.
        orderer.push(0, |_| ());
        orderer.push(5, |_| ());
        orderer.push(5, |_| ());
        orderer.push(4, |_| ());
        orderer.push(3, |_| ());
        orderer.push(2, |_| ());

        // Assert that flush returns the buffered elements in sorted order.
        assert_eq!(orderer.flush(), [2, 3, 4, 5]);
    }
}
