use memory::{Frame, FrameAllocator};
use multiboot2::{MemoryAreaIter, MemoryArea};

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: usize,
               kernel_end: usize,
               multiboot_start: usize,
               multiboot_end: usize,
               memory_areas: MemoryAreaIter)
               -> AreaFrameAllocator {

        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::new(0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::new(kernel_start),
            kernel_end: Frame::new(kernel_end),
            multiboot_start: Frame::new(multiboot_start),
            multiboot_end: Frame::new(multiboot_end),
        };

        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.areas
                                .clone()
                                .filter(|area| {
                                    let address = area.base_addr + area.length - 1;
                                    Frame::new(address as usize) >=
                                    self.next_free_frame
                                })
                                .min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::new(area.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            let frame = Frame { number: self.next_free_frame.number };

            let current_area_last_frame = {
                let address = area.base_addr + area.length - 1;
                Frame::new(address as usize)
            };

            if frame > current_area_last_frame {
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                self.next_free_frame = Frame { number: self.kernel_end.number + 1 };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                self.next_free_frame = Frame { number: self.multiboot_end.number + 1 };
            } else {
                self.next_free_frame.number += 1;
                return Some(frame);
            }

            self.allocate_frame()
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        unimplemented!()
    }
}
