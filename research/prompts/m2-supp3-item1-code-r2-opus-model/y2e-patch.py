#!/usr/bin/env python3
"""攻方腿（Opus）m2-supp3-item1-code-r2 · Y2 冷启动那一半：在已打过 y2-patch.py 的副本上再装 OPUS_Y2=e：
B6（补齐字节从最后一个载荷字节算起）只在可写挂载与回退自己的恢复里生效，冷启动 `recover` 不受影响。用法：python3 y2e-patch.py SLOT"""
import sys, os
slot = sys.argv[1]
def once(path, old, new):
    full = os.path.join(slot, path); text = open(full, encoding="utf-8").read()
    assert text.count(old) == 1, (path, old[:60], text.count(old))
    open(full, "w", encoding="utf-8").write(text.replace(old, new))
once("crates/singlefs-core/src/allocator.rs", "pub struct OpusMountGuard;", """thread_local! {
    pub static OPUS_IN_MOUNT_RECOVERY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
pub struct OpusRecoveryGuard;
impl OpusRecoveryGuard {
    #[must_use]
    pub fn enter() -> Self {
        OPUS_IN_MOUNT_RECOVERY.with(|flag| flag.set(true));
        Self
    }
}
impl Drop for OpusRecoveryGuard {
    fn drop(&mut self) {
        OPUS_IN_MOUNT_RECOVERY.with(|flag| flag.set(false));
    }
}
pub struct OpusMountGuard;""")
once("crates/singlefs-core/src/unit.rs", "    if bytes[payload_end..].iter().any(|byte| *byte != 0) {",
"""    let opus_padding_start = if crate::allocator::opus_mode() == "e"
        && crate::allocator::OPUS_IN_MOUNT_RECOVERY.with(std::cell::Cell::get)
    {
        payload_end - 1
    } else {
        payload_end
    };
    if bytes[opus_padding_start..].iter().any(|byte| *byte != 0) {""")
once("crates/singlefs-core/src/mount.rs", """    shadow_ledger: ShadowLedger,
) -> Result<Mounted, MountError> {
    let superblock = choose_superblock(&*devices)?;""", """    shadow_ledger: ShadowLedger,
) -> Result<Mounted, MountError> {
    let _opus_recovery_guard = crate::allocator::OpusRecoveryGuard::enter();
    let superblock = choose_superblock(&*devices)?;""")
text = open(os.path.join(slot, "crates/singlefs-core/src/mount.rs"), encoding="utf-8").read()
anchor = """) -> Result<Mounted, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let chosen_root = choose_root(&*devices, &superblock).ok_or(RecoveryFailure::NoValidRoot)?;"""
once("crates/singlefs-core/src/mount.rs", anchor, anchor.replace("{\n    let superblock", "{\n    let _opus_recovery_guard = crate::allocator::OpusRecoveryGuard::enter();\n    let superblock", 1))
print("patched e", slot)
