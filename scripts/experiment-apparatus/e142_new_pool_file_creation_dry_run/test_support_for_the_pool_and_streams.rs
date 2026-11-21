//! 几份测试共用的辅助：样本文件字节、造好的池与它的各个下标、mkfs 之后的池。

use super::*;

pub(super) fn sample_file() -> Vec<u8> {
    // 与 main() 里的 file_bytes 同一个公式（对齐 crates 的 `index % 251`，见 main() 里的注释）。
    (0..3000u32).map(|index| u8::try_from(index % 251).expect("小于 256")).collect()
}

/// mkfs → 取号 → 暖机两次空发布 → 新池新建文件。返回的三个下标分别是 mkfs 之后（取号从这里起）、
/// 取号之后（暖机从这里起）、暖机之后（事务从这里起）。
pub(super) struct BuiltPool {
    pub(super) recording: RecordingPool,
    pub(super) genesis: MkfsOutput,
    pub(super) output: TransactionOutput,
    pub(super) mkfs_operation_count: usize,
    pub(super) acquisition_operation_count: usize,
    pub(super) warm_up_operation_count: usize,
}

pub(super) fn built_pool() -> BuiltPool {
    let parameters = PoolParameters::settled_two_devices();
    let (mut recording, genesis) = mkfs(&parameters);
    let mkfs_operation_count = recording.operations.len();
    let instance = acquire_instance(&mut recording, &parameters);
    let acquisition_operation_count = recording.operations.len();
    let (_, last_warm_up_record) = warm_up(&mut recording, &parameters, &genesis, instance);
    let warm_up_operation_count = recording.operations.len();
    let output = publish_first_file(&mut recording, &parameters, &genesis, &sample_file(), instance, last_warm_up_record.as_deref(), NEW_POOL_FILE_CREATION_TXG);
    BuiltPool { recording, genesis, output, mkfs_operation_count, acquisition_operation_count, warm_up_operation_count }
}

// ───────── E142 第二十一次跑：字段表（R56）、新根段后三项的施加与读、宽度表 ─────────

pub(super) fn after_mkfs_pool() -> Pool {
    mkfs(&PoolParameters::settled_two_devices()).0.pool.clone()
}
