use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use memory;
use arch::mm::entry::*;

pub const ENTRY_CNT: usize = 512;

pub type PageDirectory = Table<Level4>;

pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_CNT],
    level: PhantomData<L>,
}

impl<L> Table<L> where L: TableLevel {
    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.clear();
        }
    }
}
 
impl<L> Table<L> where L: HierarchicalLevel {
 
    fn next_table_addr(&self, idx: usize) -> Option<usize> {
        let e = self.entries[idx];
        
        if e.contains(PRESENT) && !e.contains(HUGE_PAGE) {
            let addr = self as *const _ as usize;
            
            return Some((addr << 9) | idx << 12);
        }
        
        None
    }
    
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        if let Some(addr) = self.next_table_addr(index) {
            unsafe {
                return Some(&*(addr as *const _)) 
            };
        }
        
        None
    }

    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        if let Some(addr) = self.next_table_addr(index) {
            unsafe {
                return Some(&mut *(addr as *mut _)) 
            };
        }
        
        None
    }
    
    pub fn next_table_create(&mut self, idx: usize) -> &mut Table<L::NextLevel> {        
        if self.next_table_addr(idx).is_none() {
            let frame = memory::allocate().expect("Out of memory");
            
            self.entries[idx].set(frame, PRESENT | WRITABLE);
            
            self.next_table_mut(idx).unwrap().clear();
        }
        
        self.next_table_mut(idx).unwrap()
    }
}


impl<L> Index<usize> for Table<L> where L: TableLevel
{
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel
{
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

pub trait TableLevel {}

pub enum Level4 {}
enum Level3 {}
enum Level2 {}
enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}