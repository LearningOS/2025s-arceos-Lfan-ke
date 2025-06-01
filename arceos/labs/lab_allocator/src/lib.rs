//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{BaseAllocator, ByteAllocator, AllocResult};
use core::ptr::NonNull;
use core::alloc::Layout;

pub struct LabByteAllocator;

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        unimplemented!();
    }
    
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unimplemented!();
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        unimplemented!();
    }
    
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        unimplemented!();
    }
    
    fn total_bytes(&self) -> usize {
        unimplemented!();
    }
    
    fn used_bytes(&self) -> usize {
        unimplemented!();
    }
    
    fn available_bytes(&self) -> usize {
        unimplemented!();
    }
}
