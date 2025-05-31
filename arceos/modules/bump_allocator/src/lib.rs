#![no_std]

use core::alloc::Layout;
use core::ptr::NonNull;
use allocator::{AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
/// > 字节分配从低到高s→b，页从高到低p←e
pub struct EarlyAllocator <const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize,
}

impl <const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> EarlyAllocator<PAGE_SIZE> {
        Self {
            start: 0,
            end  : 0,
            b_pos: 0,
            p_pos: 0,
            // 分配了多少指针的计数，归零就重置指针
            count: 0,
        }
    }
    
    fn can_alloc_bytes(&self, size: usize, align: usize) -> bool {
        let aligned_pos = (self.b_pos + align - 1) & !(align - 1);
        aligned_pos + size <= self.p_pos
    }

    fn can_alloc_pages(&self, num_pages: usize, align_pow2: usize) -> bool {
        let size = num_pages * PAGE_SIZE;
        let aligned_pos = (self.p_pos - size) & !(align_pow2 - 1);
        aligned_pos >= self.b_pos
    }
}

impl <const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        todo!()
    }
}

impl <const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();
        
        if !self.can_alloc_bytes(size, align) {
            return Err(allocator::AllocError::NoMemory);
        }

        let aligned_pos = (self.b_pos + align - 1) & !(align - 1);
        self.b_pos = aligned_pos + size;
        self.count += 1;

        unsafe { Ok(NonNull::new_unchecked(aligned_pos as *mut u8)) }
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        self.count = self.count.saturating_sub(1);
        if self.count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl <const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        let align = align_pow2.max(PAGE_SIZE);
        let size = num_pages * PAGE_SIZE;

        if !self.can_alloc_pages(num_pages, align) {
            return Err(allocator::AllocError::NoMemory);
        }

        // 计算对齐后的地址
        let aligned_pos = (self.p_pos - size) & !(align - 1);
        self.p_pos = aligned_pos;

        Ok(aligned_pos)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        todo!()
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / PAGE_SIZE
    }
}
