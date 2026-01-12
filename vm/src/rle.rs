pub struct RleArr {
    pub base_ln: usize,
    pub deltas: Vec<u8>,
    pub counts: Vec<u8>,
}

impl RleArr {
    pub fn new() -> Self {
        Self {
            base_ln: 0,
            deltas: Vec::new(),
            counts: Vec::new(),
        }
    }

    /// Increases the value of the last element in the counts array. Use when the OpCode belongs to the same line as the previous one
    pub fn incr_count(&mut self) {
        if let Some(count) = self.counts.last_mut() {
            *count += 1;
        }
    }

    #[allow(dead_code)]
    /// Increase the value of the last element in the deltas array. Mainly used for instructions whose delta is greater than 1
    pub fn incr_delta(&mut self) {
        if let Some(delta) = self.deltas.last_mut() {
            *delta += 1;
        }
    }

    /// Add new delta with a vale of 1 and a count with a value of 0. Use when starting a new line.
    pub fn add_rle(&mut self) {
        self.deltas.push(1);
        self.counts.push(0);
    }

    pub fn get_ln(&self, offset: usize) -> usize {
        let mut delta_acc = 0;
        let mut count_acc = 0;

        for (delta, count) in self.deltas.iter().zip(self.counts.iter()) {
            delta_acc += *delta as usize;
            count_acc += *count as usize;

            if offset < count_acc {
                return self.base_ln + delta_acc;
            }
        }

        self.base_ln
    }
}
