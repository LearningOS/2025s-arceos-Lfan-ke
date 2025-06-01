//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]
extern crate alloc;

use alloc::collections::BTreeMap;
use allocator::{BaseAllocator, ByteAllocator, AllocResult, AllocError};
use core::ptr::NonNull;
use core::alloc::Layout;

pub struct LabByteAllocator {
    start: usize,
    stop: usize,
    inner: BTreeMap<usize, usize>,  // ptr : len
    used: usize,
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self {
            start: 0,
            stop : 0,
            inner: BTreeMap::new(),
            used : 0,
        }
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.stop = start + size;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unimplemented!();
    }
}

// 思路：分形，目前先测试无为而治的算法
// 但是测试时间过长（毕竟虚拟机上跑虚拟机，速度指数级下降……所以就提交一个无为而治的版本吧……）
// 还想测试：蒙特卡洛完全随机、二分法（分形）等等

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();

        if self.used + size > (self.stop - self.start) {
            return Err(AllocError::NoMemory);
        }

        let mut prev = self.start;

        for (&ptr, &len) in self.inner.iter() {
            let gap_start = prev;
            let gap_end = ptr;

            let gap_size = gap_end - gap_start;

            let aligned_start = (gap_start + align - 1) & !(align - 1);
            let end = aligned_start + size;

            if end <= gap_end {
                self.inner.insert(aligned_start, size);
                self.used += size;
                return Ok(unsafe { NonNull::new_unchecked(aligned_start as *mut u8) });
            }

            prev = ptr + len;
        }

        let gap_start = prev;
        let gap_end = self.stop;

        if gap_end > gap_start {
            let aligned_start = (gap_start + align - 1) & !(align - 1);
            let end = aligned_start + size;

            if end <= gap_end {
                self.inner.insert(aligned_start, size);
                self.used += size;
                return Ok(unsafe { NonNull::new_unchecked(aligned_start as *mut u8) });
            }
        }

        Err(AllocError::NotAllocated)
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        let ptr_addr = pos.as_ptr() as usize;
        if let Some(len) = self.inner.remove(&ptr_addr) {
            self.used -= len;
        }
    }

    fn total_bytes(&self) -> usize {
        self.stop - self.start
    }

    fn used_bytes(&self) -> usize {
        self.used
    }

    fn available_bytes(&self) -> usize {
        self.total_bytes() - self.used
    }
}
