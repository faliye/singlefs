//! 样本：读内存堆那个函数两处 unsafe，读堆那一处的 SAFETY 注释只提到适配器、没提到 hal。

/// 读这张卡最大的那一块内存堆。
#[allow(unsafe_code, reason = "样本：读内存堆只能经 hal 层")]
fn largest_device_local_memory_heap(adapter: i32) -> i32 {
    // SAFETY: 只借出 hal 层的适配器，这里不销毁也不改它。
    let borrowed_adapter = unsafe { borrow_the_hal_adapter(adapter) };
    // SAFETY: 物理设备句柄取自同一个适配器，只读。
    let heap_bytes = unsafe { read_the_memory_heap(borrowed_adapter) };
    heap_bytes
}

extern "C" {
    fn borrow_the_hal_adapter(adapter: i32) -> i32;
    fn read_the_memory_heap(adapter: i32) -> i32;
}
