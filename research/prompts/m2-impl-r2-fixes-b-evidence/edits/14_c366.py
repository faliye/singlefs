import os

from edit_lib import edit

HERE = os.path.dirname(os.path.abspath(__file__))
WU = "crates/singlefs-harness/tests/second_transaction_supplement_two_warm_up_counter.rs"
edit(WU, open(os.path.join(HERE, "14_c366_old.rs.txt"), encoding="utf-8").read(),
     open(os.path.join(HERE, "14_c366_new.rs.txt"), encoding="utf-8").read())
edit(WU, """//! - 可写挂载这一层唯一分得开两个量的可达盘面：所选根自己那条记录两份都读不出（2026-09-23 harness 那一组查出，
//!   随机历史 3600 步里重挂时两个量处处相等，只有这一格分得开）。D23（journal 的角色与格式） 已定项 14 注 3 定前缀末的计数器
//!   取环里读得出的最大那一个（用户 2026-09-23 定），于是写行那次发布 txg 5、jsn 4，之后每次发布的 jsn 与 tail 都比 txg 小 1。""", """//! - 可写挂载这一层没有可达盘面：C554 乙判不出按真（用户 2026-09-27 定，C579）。原来那一格（所选根自己那条记录两份都读不出，
//!   2026-09-23 harness 那一组查出，随机历史 3600 步里重挂时两个量处处相等，只有这一格分得开）在可写挂载的读阶段被乙拒，
//!   这里改钉「可写挂载拒、只读挂载照常择 (1, 4)」。""")
edit(WU, """use singlefs_core::mount::{mount_writable, Mounted};""", """use singlefs_core::mount::{
    mount_writable, MountError, NewerPublishWitness, RollbackTarget, SelectedVersionAgainstTheWitness,
    StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::mounted_read::mount_read_only;""")
edit(WU, """use singlefs_harness::memory_pool::SparseBlockDevice;""", """use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};""")
print("14_c366 done")
edit(WU, open(os.path.join(HERE, "14_c366_helpers_removed.rs.txt"), encoding="utf-8").read(), "")
edit(WU, """use common::{
    build_pool, geometry, parameters, publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};""", """use common::{
    build_pool, disk_snapshot, parameters, publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};""")
print("14_c366 helpers removed")
edit(WU, """use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};""", """use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};""")
edit(WU, """use singlefs_core::system_configuration::SystemConfiguration;
""", "")
edit(WU, """use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::RetainedOperation;""", """use singlefs_harness::memory_pool::SparseBlockDevice;""")
print("14_c366 imports trimmed")
