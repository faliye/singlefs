#!/usr/bin/env python3
"""攻方腿（Opus）m2-supp3-item1-code-r2 · Y2：在仓副本上装「只在挂载里（写行、暖机）生效」的分配代变异，运行时由环境变量 OPUS_Y2 选：
  off = 不变异、只数；a = 挂载里复用改写已回收记录时不改分配代（N2 只落在挂载里）；b = 挂载里每次分配的分配代记成 txg − 1。
只在副本上跑：python3 y2-patch.py SLOT"""
import sys, os
slot = sys.argv[1]
def once(path, old, new):
    full = os.path.join(slot, path); text = open(full, encoding="utf-8").read()
    assert text.count(old) == 1, (path, old[:60], text.count(old))
    open(full, "w", encoding="utf-8").write(text.replace(old, new))
once("crates/singlefs-core/src/allocator.rs",
"fn make_room_for_record_on_device(",
"""thread_local! {
    pub static OPUS_IN_MOUNT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
pub static OPUS_MOUNT_REWRITES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static OPUS_OTHER_REWRITES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static OPUS_MOUNT_RECORDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub fn opus_mode() -> &'static str {
    static MODE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    MODE.get_or_init(|| std::env::var("OPUS_Y2").unwrap_or_else(|_| "off".to_string()))
}
pub struct OpusMountGuard;
impl OpusMountGuard {
    #[must_use]
    pub fn enter() -> Self {
        OPUS_IN_MOUNT.with(|flag| flag.set(true));
        Self
    }
}
impl Drop for OpusMountGuard {
    fn drop(&mut self) {
        OPUS_IN_MOUNT.with(|flag| flag.set(false));
    }
}

fn make_room_for_record_on_device(""")
once("crates/singlefs-core/src/allocator.rs",
"""            existing.generation = generation;
            existing.is_released = false;""",
"""            let in_mount = OPUS_IN_MOUNT.with(std::cell::Cell::get);
            if in_mount {
                OPUS_MOUNT_REWRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            } else {
                OPUS_OTHER_REWRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            if !(in_mount && opus_mode() == "a") {
                existing.generation = generation;
            }
            existing.is_released = false;""")
once("crates/singlefs-core/src/allocator.rs",
"""    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {
""",
"""    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {
        let in_mount = OPUS_IN_MOUNT.with(std::cell::Cell::get);
        if in_mount {
            OPUS_MOUNT_RECORDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        let generation = if in_mount && opus_mode() == "b" {
            CheckpointTxg(generation.0 - 1)
        } else {
            generation
        };
""")
once("crates/singlefs-core/src/mount.rs",
"""    assert_eq!(
        instance, instance_to_acquire,""",
"""    let _opus_mount_guard = crate::allocator::OpusMountGuard::enter();
    assert_eq!(
        instance, instance_to_acquire,""")
print("patched", slot)
