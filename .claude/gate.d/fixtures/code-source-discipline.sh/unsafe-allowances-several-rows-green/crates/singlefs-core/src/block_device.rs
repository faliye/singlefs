//! 样本：core 放开两个函数，打洞那个一处 unsafe，读内存堆那个两处，各提到自己那一行登记的词。

/// 打洞。
#[allow(unsafe_code, reason = "样本：std 没有 fallocate 的安全封装")]
fn punch_hole_keeping_the_file_length(descriptor: i32, length: i64) -> i32 {
    // SAFETY: fallocate 只读它的整数参数；描述符由调用方持有，偏移与长度已核过范围。
    let return_code = unsafe { fallocate(descriptor, length) };
    return_code
}

/// 读这张卡最大的那一块内存堆。
#[allow(unsafe_code, reason = "样本：读内存堆只能经 hal 层")]
fn largest_device_local_memory_heap(adapter: i32) -> i32 {
    // SAFETY: 只借出 hal 层的适配器，这里不销毁也不改它。
    let borrowed_adapter = unsafe { borrow_the_hal_adapter(adapter) };
    // SAFETY: 物理设备句柄取自同一个 hal 适配器，只读。
    let heap_bytes = unsafe { read_the_memory_heap(borrowed_adapter) };
    heap_bytes
}

extern "C" {
    fn fallocate(descriptor: i32, length: i64) -> i32;
    fn borrow_the_hal_adapter(adapter: i32) -> i32;
    fn read_the_memory_heap(adapter: i32) -> i32;
}
