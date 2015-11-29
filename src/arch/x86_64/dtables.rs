#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
   /// Size of the DT.
   pub limit: u16,
   /// Pointer to the memory region containing the DT.
   pub base: u64
} 
