#!/usr/bin/env python3
"""攻方量法副本：cost-patch.py <量法副本根> <原型目录>。在副本的 core 里加 ATK_SKIP_REHEARSAL / ATK_LAST_REHEARSAL 与 atk_rehearse_for
（量 K1-3），把 block_device.rs 的 check_aligned_and_in_range 放成 pub(crate)，再把 atk_overlay_tests.rs 接到 mounted_session.rs 末尾（K1-2 / K5）。
每处原文恰好命中一次，否则退 3。不是产品代码，只在副本上。"""
import sys
from pathlib import Path

root, model = Path(sys.argv[1]), Path(sys.argv[2])
ms = root / "crates/singlefs-core/src/mounted_session.rs"
bd = root / "crates/singlefs-core/src/block_device.rs"
REPLACEMENTS = [
    (ms, """    remainder: RemainderOfTheUserChange<'_>,
) -> RootRingPushRehearsal {
    let mut devices_keeping_writes_in_memory""",
     """    remainder: RemainderOfTheUserChange<'_>,
) -> RootRingPushRehearsal {
    // 攻方副本的量法（不是产品代码）：ATK_SKIP_REHEARSAL 置着就不预演；量每次预演的挂钟与只进内存的盘记下的块数、字节数。
    if ATK_SKIP_REHEARSAL.load(std::sync::atomic::Ordering::SeqCst) {
        return RootRingPushRehearsal::PushRefused(Box::new(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot));
    }
    let atk_started = std::time::Instant::now();
    let mut devices_keeping_writes_in_memory"""),
    (ms, """    match remainder_after_the_push {
        Ok(()) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {""",
     """    {
        let blocks: u64 = devices_keeping_writes_in_memory.iter().map(|(_, device)| device.written_blocks.len() as u64).sum();
        let bytes: u64 = devices_keeping_writes_in_memory.iter().map(|(_, device)| device.written_blocks.values().map(|block| block.len() as u64).sum::<u64>()).sum();
        let mut last = ATK_LAST_REHEARSAL.lock().expect("锁");
        *last = (atk_started.elapsed().as_nanos(), blocks, bytes, last.3 + 1);
    }
    match remainder_after_the_push {
        Ok(()) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {"""),
    (ms, """/// 每块盘上这一刻隔离的槽数（D28""",
     """/// 攻方副本的量法入口（不是产品代码）：在会话这一刻照「被宽护卡住」那一报的做法预演推环与推完之后的 `change`。
pub fn atk_rehearse_for<Device: BlockDevice>(
    session: &MountedSession,
    devices: &[(DeviceIdentity, Device)],
    change: UserChange<'_>,
) -> RootRingPushRehearsal {
    let current = match &session.current {
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
        PoolVersion::WithoutFile(_) => panic!("量法只在有树的一版上"),
    };
    rehearse_the_push_then_the_remainder_of_the_change(
        session.parameters_and_device_table.parameters_on_disk(),
        devices,
        &session.allocator,
        current,
        session.instance,
        session.shadow_ledger,
        RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
    )
}

/// 攻方副本的量法开关与读数（不是产品代码）。
pub static ATK_SKIP_REHEARSAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// 最近一次预演：(挂钟纳秒, 只进内存的盘记下的块数, 字节数, 累计预演次数)。
pub static ATK_LAST_REHEARSAL: std::sync::Mutex<(u128, u64, u64, u64)> = std::sync::Mutex::new((0, 0, 0, 0));

/// 每块盘上这一刻隔离的槽数（D28"""),
    (bd, """/// 请求要按物理块对齐、不越过设备末尾；两个后端共用这一份判据。
fn check_aligned_and_in_range(""",
     """/// 请求要按物理块对齐、不越过设备末尾；两个后端共用这一份判据。
pub(crate) fn check_aligned_and_in_range("""),
]
for path, original, replacement in REPLACEMENTS:
    text = path.read_text()
    if text.count(original) != 1:
        print(f"{path}: 原文命中 {text.count(original)} 次", file=sys.stderr)
        sys.exit(3)
    path.write_text(text.replace(original, replacement))
ms.write_text(ms.read_text() + (model / "atk_overlay_tests.rs").read_text())
print("量法副本改好")
