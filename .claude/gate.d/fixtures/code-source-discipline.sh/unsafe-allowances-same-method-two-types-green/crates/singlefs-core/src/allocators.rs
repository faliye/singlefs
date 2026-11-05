//! 样本：两个类型各实现一次 GlobalAlloc、各有 alloc 与 dealloc；方法按 <类型>::<方法> 各登一行，同名方法不撞键。SAFETY 注释写在放开属性与项头之间。
use std::alloc::{GlobalAlloc, Layout, System};

pub struct CountingAllocator;

pub struct TracingAllocator;

#[allow(unsafe_code, reason = "样本：GlobalAlloc 是 unsafe trait")]
// SAFETY: 原样转给 System，不碰被分配的内存。
unsafe impl GlobalAlloc for CountingAllocator {
    #[allow(unsafe_code, reason = "样本：GlobalAlloc 的方法签名要求 unsafe fn")]
    // SAFETY: 调用方按契约给 layout，原样转给 System。
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: layout 由调用方保证合法，原样转给 System。
        unsafe { System.alloc(layout) }
    }

    #[allow(unsafe_code, reason = "样本：GlobalAlloc 的方法签名要求 unsafe fn")]
    // SAFETY: 调用方按契约给 pointer 与 layout，原样转给 System。
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: pointer 是用同一个 layout 从 System 分出来的，原样还给 System。
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[allow(unsafe_code, reason = "样本：GlobalAlloc 是 unsafe trait")]
// SAFETY: 原样转给 System，不碰被分配的内存。
unsafe impl GlobalAlloc for TracingAllocator {
    #[allow(unsafe_code, reason = "样本：GlobalAlloc 的方法签名要求 unsafe fn")]
    // SAFETY: 调用方按契约给 layout，原样转给 System。
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: layout 由调用方保证合法，原样转给 System。
        unsafe { System.alloc(layout) }
    }

    #[allow(unsafe_code, reason = "样本：GlobalAlloc 的方法签名要求 unsafe fn")]
    // SAFETY: 调用方按契约给 pointer 与 layout，原样转给 System。
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: pointer 是用同一个 layout 从 System 分出来的，原样还给 System。
        unsafe { System.dealloc(pointer, layout) }
    }
}
