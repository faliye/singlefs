//! m3-c554-c572-r1 攻方原型（Opus 腿）。只在草稿副本里编、跑；冻结副本只读。
//! MODE=b1|b2|b4|b5，各格的参数见各自的 main_*。全部在内存稀疏盘上（`SparseBlockDevice`），起点池每段历史各建一次、之后拷。
//! 崩溃状态照崩溃点重放的枚举域取：`enumerate_layer0_with_expansion` + `full_expansion`（每一段都展开，原地覆写三态），
//! 每个状态恢复之后的池级 checker 由枚举器自己判，另挂一个观察者看这几格要看的东西。

#[path = "../../singlefs-harness/tests/common_admission_pool/mod.rs"]
mod pool;

mod b1;
mod b2;
mod b5;
mod util;

fn main() {
    let mode = std::env::var("MODE").unwrap_or_default();
    match mode.as_str() {
        "b1" => b1::main_b1(),
        "b1scan" => b1::main_b1scan(),
        "b4" => b1::main_b4(),
        "b2" => b2::main_b2(),
        "b5" => b5::main_b5(),
        other => panic!("MODE={other:?} 认不得：b1 | b2 | b5"),
    }
}
