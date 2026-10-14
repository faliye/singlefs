//! c794-r1 攻方原型（Opus 腿）。只在草稿副本里编、跑；冻结副本只读。
//! MODE=grid：六种镜像形（瞬时隔离、持久坏一份、丢写落在复用槽、写者漏释放、写者假隔离、活单元记进 defer）× 四条 checker 臂，
//!   在事件之后几个时刻各判一次；另比瞬时隔离与写者漏释放两份镜像逐字节相同不相同。
//! MODE=enum：瞬时隔离那一段历史（与不注入的对照）里事件那次发布的写流，照崩溃点重放的枚举域枚举每个崩溃状态，每个状态恢复之后跑池级 checker。
//! 全部在内存稀疏盘上（`SparseBlockDevice`）。
#[path = "../../singlefs-harness/tests/common_admission_pool/mod.rs"]
mod pool;

mod flaky;
mod grid;
mod enumerate;

fn main() {
    match std::env::var("MODE").unwrap_or_default().as_str() {
        "grid" => grid::main_grid(),
        "enum" => enumerate::main_enum(),
        other => panic!("MODE={other:?} 认不得：grid | enum"),
    }
}
