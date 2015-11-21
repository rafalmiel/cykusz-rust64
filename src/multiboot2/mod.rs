mod memory_map;
pub use self::memory_map::{MemoryMapTag, MemoryArea, MemoryAreaIter};

#[allow(unused)]
pub unsafe fn load(address: usize) -> &'static BootInformation {
    let multiboot = &*(address as *const BootInformation);
    assert!(multiboot.has_valid_end_tag());    
    multiboot
}

#[repr(C)]
#[allow(unused)]
pub struct BootInformation {
    pub total_size: u32,
    _reserved: u32,
    first_tag: Tag,
}


#[repr(C)]
#[allow(unused)]
struct Tag {
    typ: u32,
    size: u32,
}

#[allow(unused)]
impl BootInformation {
    pub fn memory_map_tag(&self) -> Option<&'static MemoryMapTag> {
        self.get_tag(6).map(|tag| unsafe {
            &*(tag as *const Tag as *const MemoryMapTag)
        })
    }

    fn has_valid_end_tag(&self) -> bool {
        const END_TAG: Tag = Tag { typ: 0, size: 8 };

        let self_ptr = self as *const _;
        let end_tag_addr = self_ptr as usize + (self.total_size - END_TAG.size) as usize;
        let end_tag = unsafe { &*(end_tag_addr as *const Tag)  };

        end_tag.typ == END_TAG.typ && end_tag.size == END_TAG.size
    }

    fn get_tag(&self, typ: u32) -> Option<&'static Tag> {
        self.tags().find(|tag| tag.typ == typ)
    }

    fn tags(&self) -> TagIter {
        TagIter { current: &self.first_tag as *const _ }
    }
}

#[allow(unused)]
struct TagIter {
    current: *const Tag,    
}

#[allow(unused)]
impl Iterator for TagIter {
    type Item = &'static Tag;

    fn next(&mut self) -> Option<&'static Tag> {
        match unsafe { &*self.current } {
            &Tag{ typ: 0, size: 8 } => None,
            tag => {
                let mut tag_addr = self.current as usize;
                tag_addr += tag.size as usize;
                tag_addr = ((tag_addr - 1) & !0x7) + 0x8; //align at 8 byte
                self.current = tag_addr as *const _;

                Some(tag)
            },
        }
    }
}
