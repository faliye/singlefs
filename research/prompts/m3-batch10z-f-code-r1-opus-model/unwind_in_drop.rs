//! 第 10z 批第六块代码轮攻方（opus）：`GpuCardLease::drop` 那一形（接住 wgpu 的 panic、记成没等成、再 `resume_unwind`）在用例本身已经 panic、
//! 借据在展开时丢掉的时候会怎样。只验 Rust 的语义，不碰卡：`poll` 换成一个必 panic 的闭包（等的设备丢了时 wgpu 走 handle_error_fatal）。
//! 用法：`unwind_in_drop <arm>`，arm 是 as-shipped（照原样再抛）或 not-while-panicking（展开中不再抛，只记没等成）。
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

struct Lease {
    rethrow_while_panicking: bool,
}

impl Drop for Lease {
    fn drop(&mut self) {
        let polled: std::thread::Result<()> =
            catch_unwind(AssertUnwindSafe(|| panic!("Device::poll：Parent device is lost")));
        eprintln!("DROP caught_inner_panic={} thread_panicking={}", polled.is_err(), std::thread::panicking());
        if let Err(panic) = polled {
            if self.rethrow_while_panicking || !std::thread::panicking() {
                resume_unwind(panic);
            }
        }
    }
}

fn main() {
    let arm = std::env::args().nth(1).expect("arm");
    let rethrow_while_panicking = arm == "as-shipped";
    // 一条用例线程：借据活着时用例 panic（设备丢了，提交或没接住的错误 panic）
    let joined = std::thread::spawn(move || {
        let _lease = Lease { rethrow_while_panicking };
        panic!("用例：wgpu error: device lost");
    })
    .join();
    println!("UNWIND arm={arm} test_thread_joined_with_panic={} process_survived=true", joined.is_err());
    // 另一形：用例没 panic，借据正常丢掉时等的那一下 panic
    let joined = std::thread::spawn(move || {
        let _lease = Lease { rethrow_while_panicking };
    })
    .join();
    println!("UNWIND arm={arm} plain_drop_thread_panicked={}", joined.is_err());
}
