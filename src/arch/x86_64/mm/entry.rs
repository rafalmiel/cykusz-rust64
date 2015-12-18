use memory::Frame;

bitflags! {
    flags Entry: u64 {
        const PRESENT       = 1 << 0,
        const WRITABLE      = 1 << 1,
        const USER          = 1 << 2,
        const WRT_THROUGH   = 1 << 3,
        const NO_CACHE      = 1 << 4,
        const ACCESSED      = 1 << 5,
        const DIRTY         = 1 << 6,
        const HUGE_PAGE     = 1 << 7,
        const GLOBAL        = 1 << 8,
        const NO_EXECUTE    = 1 << 63,
    }
}

impl Entry {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
    
    pub fn raw(&self) -> u64 {
        self.bits
    }
    
    pub fn is_unused(&self) -> bool {
        self.bits == 0
    }
    
    pub fn set(&mut self, frame: Frame, flags: Entry) {
        self.bits = frame.address() as u64;
        self.insert(flags);
    }
}