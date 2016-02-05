use core::ptr::Unique;

use memory::Frame;
use super::table;
use super::page::Page;
use super::entry::*;

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

    #[allow(dead_code)]
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

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);

        unsafe {
            // Do we need it here?
            flush(page.address());
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
            flush(page.address());
        }
    }
}
