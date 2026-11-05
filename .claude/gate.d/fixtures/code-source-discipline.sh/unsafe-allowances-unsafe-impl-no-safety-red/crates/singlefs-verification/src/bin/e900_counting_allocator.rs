//! 样本：同 unsafe-allowances-unsafe-impl-green，只是 unsafe impl 上方那段注释不以 // SAFETY: 起头。
#![deny(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

struct CountingAllocator {
    in_use_bytes: AtomicU64,
}

// 每个方法原样转给 System，计数只改原子量，不碰被分配的内存。
#[allow(unsafe_code, reason = "样本：GlobalAlloc 是 unsafe trait")]
unsafe impl GlobalAlloc for CountingAllocator {
    // SAFETY: 调用方按 GlobalAlloc::alloc 的契约给 layout，这里原样转给 System。
    #[allow(
        unsafe_code,
        reason = "样本：GlobalAlloc 的方法签名要求 unsafe fn"
    )]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: layout 由调用方保证合法，转给 System 约束不变。
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            self.in_use_bytes.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        pointer
    }

    // SAFETY: 调用方按 GlobalAlloc::dealloc 的契约给 pointer 与 layout，这里原样转给 System。
    #[allow(unsafe_code, reason = "样本：GlobalAlloc 的方法签名要求 unsafe fn")]
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: pointer 是用同一个 layout 从 System 分出来的，原样还给 System。
        unsafe { System.dealloc(pointer, layout) };
        self.in_use_bytes.fetch_sub(layout.size() as u64, Ordering::Relaxed);
    }
}

#[global_allocator]
static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator { in_use_bytes: AtomicU64::new(0) };

fn main() {
    let bytes = vec![0u8; 16];
    assert_eq!(bytes.len(), 16);
}
