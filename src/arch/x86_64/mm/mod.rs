mod entry;
mod table;

use spin::Mutex;
use core::ptr::Unique;

use memory;
use memory::Frame;
use memory::PAGE_SIZE;

use self::table::PageDirectory;
use self::entry::*;

pub type VirtAddr = usize;
pub type PhysAddr = usize;

const P4: *mut table::Table<table::Level4> = 0xffffffff_fffff000 as *mut _;

pub struct Page {
    number: usize,
}

struct Mapper {
    p4: Unique<PageDirectory>,
}

static MAPPER: Mutex<Mapper> = Mutex::new(Mapper::new());

unsafe fn flush(addr: usize) {
    asm!("invlpg ($0)" :: "r" (addr) : "memory");
}

impl Mapper {
    const fn new() -> Mapper {
        unsafe {
            Mapper {
                p4: Unique::new(P4),
            }
        }
    }
    
    fn p4(&self) -> &PageDirectory {
        unsafe { self.p4.get() }
    }

    fn p4_mut(&mut self) -> &mut PageDirectory {
        unsafe { self.p4.get_mut() }
    }
    
    fn map_to(&mut self, page: Page, frame: Frame, flags: Entry) {
        let mut p3 = self.p4_mut().next_table_create(page.p4_index());
        let mut p2 = p3.next_table_create(page.p3_index());
        let mut p1 = p2.next_table_create(page.p2_index());

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);
        
        unsafe {
            flush(page.address());
        }
    }
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
    
    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }
    
    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }
    
    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }
    
    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}

pub fn map_to(virt: VirtAddr, phys: PhysAddr) {
    let mut mapper = MAPPER.lock();
        
    mapper.map_to(Page::new(virt), Frame::new(phys), PRESENT | WRITABLE);
}

pub fn identity_map(virt: VirtAddr) {
    let mut mapper = MAPPER.lock();
        
    mapper.map_to(Page::new(virt), Frame::new(virt), PRESENT | WRITABLE);
}

pub fn map(virt: VirtAddr) {
    let frame = memory::allocate().expect("Out of memory");
    
    let mut mapper = MAPPER.lock();
    
    mapper.map_to(Page::new(virt), frame, PRESENT | WRITABLE);
}

pub fn init() {
}