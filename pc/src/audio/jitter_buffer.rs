use std::collections::BTreeMap;

pub struct JitterBuffer {
    buffer: BTreeMap<u16, Vec<i16>>,
    target_frames: usize,
    max_frames: usize,
    next_seq: u16,
}

impl JitterBuffer {
    pub fn new(target_frames: usize, max_frames: usize) -> Self {
        Self {
            buffer: BTreeMap::new(),
            target_frames,
            max_frames,
            next_seq: 0,
        }
    }

    pub fn push(&mut self, seq: u16, frame: Vec<i16>) {
        self.buffer.insert(seq, frame);
        if self.buffer.len() > self.max_frames {
            if let Some(&oldest) = self.buffer.keys().next() {
                self.buffer.remove(&oldest);
            }
        }
    }

    pub fn pop(&mut self) -> Option<Vec<i16>> {
        if self.buffer.len() < self.target_frames {
            return None;
        }
        if let Some(&seq) = self.buffer.keys().next() {
            if seq == self.next_seq || self.buffer.len() >= self.max_frames {
                self.next_seq = seq.wrapping_add(1);
                return self.buffer.remove(&seq);
            }
        }
        if let Some(&seq) = self.buffer.keys().next() {
            self.next_seq = seq.wrapping_add(1);
            return self.buffer.remove(&seq);
        }
        None
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
