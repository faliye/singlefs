//! 样本：tests/ 下的文件是自己的 crate 根，harness 的 forbid 管不到它。

#[allow(unsafe_code)]
#[test]
fn poke_the_raw_descriptor() {
    // SAFETY: 描述符 0 在测试进程里一直开着。
    let return_code = unsafe { close_it(0) };
    assert_eq!(return_code, 0);
}

extern "C" {
    fn close_it(descriptor: i32) -> i32;
}
