pub use self::area_frame_allocator::AreaFrameAllocator;
use arch::memory::paging::PhysicalAddress;

mod area_frame_allocator;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    pub fn containing_address(address: usize) -> Frame {
        Frame { number: address / PAGE_SIZE }
    }
    
    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}
