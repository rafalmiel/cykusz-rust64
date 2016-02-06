pub use self::area_frame_allocator::AreaFrameAllocator;

mod area_frame_allocator;

use spin::Mutex;
use multiboot2::{MemoryAreaIter};

pub const PAGE_SIZE: usize = 4096;

static ALLOCATOR: Mutex<Option<AreaFrameAllocator>> = Mutex::new(None);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    pub fn new(address: usize) -> Frame {
        Frame { number: address / PAGE_SIZE }
    }

    pub fn address(&self) -> usize {
        self.number * PAGE_SIZE
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub fn init(kernel_start: usize,
            kernel_end: usize,
            multiboot_start: usize,
            multiboot_end: usize,
            memory_areas: MemoryAreaIter) {
    *ALLOCATOR.lock() = Some(AreaFrameAllocator::new(kernel_start as usize,
                                                     kernel_end as usize,
                                                     multiboot_start as usize,
                                                     multiboot_end as usize,
                                                     memory_areas));
}

pub fn allocate() -> Option<Frame> {
    let mut a = ALLOCATOR.lock();

    if let Some(ref mut al) = *a {
        return al.allocate_frame();
    }

    None
}
