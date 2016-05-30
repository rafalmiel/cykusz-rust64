use core::ptr::Unique;

use memory::Frame;
use memory::PAGE_SIZE;
use super::{VirtAddr, PhysAddr};
use super::table;
use super::table::ENTRY_CNT;
use super::page::Page;
use super::entry::*;
use x86;

unsafe fn flush(addr: usize) {
    asm!("invlpg ($0)" :: "r" (addr) : "memory");
}

const P4: *mut table::Table<table::Level4> = 0xffffffff_fffff000 as *mut _;

pub struct Mapper {
    p4: Unique<table::PageDirectory>,
}

impl Mapper {
    pub const fn new() -> Mapper {
        unsafe {
            Mapper {
                p4: Unique::new(P4),
            }
        }
    }

    pub fn p4(&self) -> &table::PageDirectory {
        unsafe { self.p4.get() }
    }

    pub fn p4_mut(&mut self) -> &mut table::PageDirectory {
        unsafe { self.p4.get_mut() }
    }

    pub fn map_to(&mut self, page: Page, frame: Frame, flags: Entry) {
        let mut p3 = self.p4_mut().next_table_create(page.p4_index());
        let mut p2 = p3.next_table_create(page.p3_index());
        let mut p1 = p2.next_table_create(page.p2_index());

        p1[page.p1_index()].set(frame, flags | PRESENT);

        unsafe {
            // Do we need it here?
            x86::tlb::flush(page.address());
        }
    }

    pub fn unmap(&mut self, page: Page) {
        let p1 = self.p4_mut()
                     .next_table_mut(page.p4_index())
                     .and_then(|p3| p3.next_table_mut(page.p3_index()))
                     .and_then(|p2| p2.next_table_mut(page.p2_index()))
                     .expect("Mapping code does not support huge pages");

        let mut pd = p1[page.p1_index()];

        pd.clear();

        unsafe {
            x86::tlb::flush(page.address());
        }
    }

    pub fn translate(&self, virt_addr: VirtAddr) -> Option<PhysAddr> {
        let offset = virt_addr % PAGE_SIZE;

        self.translate_page(Page::new(virt_addr))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];

                if let Some(start_frame) = p3_entry.frame() {
                    if p3_entry.contains(HUGE_PAGE) {
                        return Some(Frame {
                            number: start_frame.number +
                                    page.p2_index() * ENTRY_CNT +
                                    page.p1_index(),
                        });
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];

                    if let Some(start_frame) = p2_entry.frame() {
                        if p2_entry.contains(HUGE_PAGE) {
                            return Some(Frame {
                                number: start_frame.number + page.p1_index(),
                            });
                        }
                    }
                }

                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
          .and_then(|p2| p2.next_table(page.p2_index()))
          .and_then(|p1| p1[page.p1_index()].frame())
          .or_else(huge_page)
    }
}
