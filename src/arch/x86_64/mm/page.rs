use memory::PAGE_SIZE;
use super::VirtAddr;

pub struct Page {
    number: usize,
}

impl Page {
    pub fn new(virt: VirtAddr) -> Page {
        Page {
            number: virt / PAGE_SIZE,
        }
    }

    pub fn address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    pub fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    pub fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    pub fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    pub fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}
