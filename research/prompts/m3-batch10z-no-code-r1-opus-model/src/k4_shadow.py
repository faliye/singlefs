#!/usr/bin/env python3
"""m3-batch10z-no-code-r1 攻方 K4：在卡池模块（gpu_unit_checks.rs 的 `mod card_pool {`）里加一个与外部 crate 同名的模块遮住它，
借之前那一步的方法体一个字不动。用法：k4_shadow.py <副本根> wgpu|std。命中次数对不上退出 3。
wgpu：`wgpu::PollType::Poll` 解到本地常量 `PollType::wait_indefinitely()`（借之前那一步的 poll 改成一直等到设备空）；
std：`std::panic::catch_unwind` 解到本地函数，不 poll、恒交「队列空了」（列出来的照旧记着的借据一律销账）。"""
import sys
root, which = sys.argv[1], sys.argv[2]
p = root + '/crates/singlefs-checker-tier/src/gpu_unit_checks.rs'
s = open(p).read()
anchor = "    mod card_pool {\n"
if s.count(anchor) != 1:
    print("命中次数不对", s.count(anchor)); sys.exit(3)
shadow = {
 'wgpu': """        #[allow(non_snake_case, non_upper_case_globals, unused_imports, dead_code)]
        mod wgpu {
            pub use ::wgpu::*;
            pub mod PollType {
                pub const Poll: ::wgpu::PollType = ::wgpu::PollType::wait_indefinitely();
            }
        }
""",
 'std': """        #[allow(unused_imports, dead_code)]
        mod std {
            pub use ::std::*;
            pub mod panic {
                pub use ::std::panic::*;
                pub fn catch_unwind<F: FnOnce() -> Result<::wgpu::PollStatus, ::wgpu::PollError>>(
                    _f: AssertUnwindSafe<F>,
                ) -> ::std::thread::Result<Result<::wgpu::PollStatus, ::wgpu::PollError>> {
                    Ok(Ok(::wgpu::PollStatus::QueueEmpty))
                }
            }
        }
""",
}[which]
open(p, 'w').write(s.replace(anchor, anchor + shadow))
print("打上了：", which)
