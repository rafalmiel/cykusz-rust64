mod entry;
mod table;
mod mapper;
mod page;

use spin::Mutex;
use core::ptr::Unique;

use memory;
use memory::Frame;
use memory::PAGE_SIZE;

use self::table::PageDirectory;
use self::entry::*;
use self::mapper::Mapper;
use self::page::Page;

pub type VirtAddr = usize;
pub type PhysAddr = usize;

static MAPPER: Mutex<Mapper> = Mutex::new(Mapper::new());

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

pub fn unmap(virt: VirtAddr) {
    let mut mapper = MAPPER.lock();

    mapper.unmap(Page::new(virt));
}

pub fn init() {
}
