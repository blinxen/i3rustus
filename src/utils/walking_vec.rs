#[derive(Clone, Debug)]
pub struct WalkingVec {
    pub buffer: Vec<u8>,
    pub position: usize,
}

impl WalkingVec {
    pub fn walk(&mut self, bytes_num: usize) -> &[u8] {
        let ret = &self.buffer[self.position..self.position + bytes_num];
        self.position += bytes_num;

        ret
    }

    pub fn reached_end(&self) -> bool {
        self.buffer.len() == self.position
    }
}
