use core::mem::size_of;
use arch::acpi;

#[repr(packed, C)]
pub struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    pub rsdt_address: u32
}

impl Rsdp {
    pub unsafe fn is_valid(&self) -> bool {    
        if &self.signature as &[u8] != b"RSD PTR " {
            false
        } else {
            acpi::util::checksum(self as *const _ as *const u8, size_of::<Rsdp>() as isize)
        }
    }
    
    pub unsafe fn find() -> Option<&'static Rsdp> {
        let iter = (0xE_000..0x100_000).step_by(0x10);
        
        //TODO: Check ebda address
        
        for addr in iter {
            let ptr = &*(addr as *const Rsdp);
                        
            if ptr.is_valid() {
                return Some(ptr)
            }
        }
        
        None
    }
} 
