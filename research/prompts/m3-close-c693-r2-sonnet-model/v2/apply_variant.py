#!/usr/bin/env python3
"""正推腿 m3-close-c693-r2 V2：在草稿仓副本里给 F2 退回加「分失败种类」的补丁（只改副本，不改主工作区）。

用法：python3 -I apply_variant.py --repo <仓副本根> --variant <none|blacklist|whitelist|objectlevel>
  none         三份源码复位成开工时的样子（第一次调用时把原件存成 <仓副本根>/../pristine-<名>.rs；之后以它为准）
  blacklist    F2 不为「树表条目被拒收」（UnitMalformed 且 what = 树表条目；只读挂载是 TreeTableEntryMalformed）退回
  whitelist    F2 只为「字节丢了」退：UnitUnreadable、MappingMiss、MappingStillUnreadable（只读挂载的 UnitMalformed 也算，它含载荷校验和不对）
  objectlevel  whitelist 加上：条目只因 flags 有未知位被拒、且那棵树不是 inode / extent / 分配记录 / 记账时，读树表的人跳过它（对象级 EIO 的量过用近似）
补丁落点：recovery.rs 的 recover() 的 Err(failure) 臂、mounted_read.rs 的 mount_read_only 的 Err(failure_of_the_selected_version) 臂、
  mount.rs 的 rebuild_previous_version 失败臂；各在调 F2 判法之前先问失败种类。
退出码：0；2 用法错或原文命中不是恰好一次。
"""
import hashlib, os, shutil, sys

PRISTINE_SHA256 = {
    "recovery": "67382a1385fdd6cd7d3297bc1e40de868bd8071e52bdf2db6c398eae12b5116b",
    "mounted_read": "5c46bc343db4705ad03a3467938e71ee7a08ffd11b21fcf656896ee2661dd000",
    "mount": "655aaad1d3804753fbd145445f8fec35b4b594ad435a77f89c8197f508e99cfb",
}
args = dict(zip(sys.argv[1::2], sys.argv[2::2]))
repo, variant = args.get("--repo"), args.get("--variant")
if not repo or variant not in ("none", "blacklist", "whitelist", "objectlevel"):
    print(__doc__); sys.exit(2)
root = os.path.join(repo, "crates/singlefs-core/src/")
keep = os.path.join(os.path.dirname(os.path.abspath(repo)), "pristine-" + os.path.basename(os.path.abspath(repo)))
os.makedirs(keep, exist_ok=True)
for name in PRISTINE_SHA256:
    saved = os.path.join(keep, name + ".rs")
    if not os.path.exists(saved):
        shutil.copy(root + name + ".rs", saved)
    got = hashlib.sha256(open(saved, "rb").read()).hexdigest()
    if got != PRISTINE_SHA256[name]:
        print(f"name=pristine_sha256_differs file={name}.rs expected={PRISTINE_SHA256[name]} got={got}（crates 在这份量过的读数之后改过，下面的落点读数不保证）")
    shutil.copy(saved, root + name + ".rs")


def sub(path, old, new):
    text = open(root + path).read()
    if text.count(old) != 1:
        print(f"原文命中 {text.count(old)} 次：{path} | {old[:60]}"); sys.exit(2)
    open(root + path, "w").write(text.replace(old, new))


if variant == "none":
    sys.exit(0)
if variant == "blacklist":
    helper = '''

/// 量过用（草稿副本，黑名单）：F2 不退的失败种类——树表条目被这个读者拒收（只认 what=树表条目这一个字串）。
#[must_use]
pub fn failure_may_retreat(failure: &RecoveryFailure) -> bool {
    !matches!(failure, RecoveryFailure::UnitMalformed { what: "树表条目" })
}
'''
    open_cond = "!matches!(f, OpenPoolForReadFailure::TreeTableEntryMalformed) && !matches!(f, OpenPoolForReadFailure::Walk(inner) if !crate::recovery::failure_may_retreat(inner))"
else:
    helper = '''

/// 量过用（草稿副本，白名单）：F2 只为「字节丢了」退：单元两条位置都读不到校验和相符的那份、映射里没有这个 key、位置提示与映射都读不回字节。
#[must_use]
pub fn failure_may_retreat(failure: &RecoveryFailure) -> bool {
    matches!(
        failure,
        RecoveryFailure::UnitUnreadable { .. }
            | RecoveryFailure::MappingMiss { .. }
            | RecoveryFailure::MappingStillUnreadable { .. }
    )
}
'''
    open_cond = "match f { OpenPoolForReadFailure::Walk(inner) => crate::recovery::failure_may_retreat(inner), OpenPoolForReadFailure::UnitMalformed { .. } => true, _ => false }"
sub("recovery.rs", '''        Err(failure) => {
            let version_fallen_back_to_walked = match policy {
                JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {''', '''        Err(failure) => {
            let version_fallen_back_to_walked = match policy {
                _ if !failure_may_retreat(&failure) => None,
                JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {''')
open(root + "recovery.rs", "a").write(helper)
sub("mounted_read.rs", '''            Err(failure_of_the_selected_version) => {
                let version_fallen_back_to_opened =''', '''            Err(failure_of_the_selected_version) => {
                if !({ let f = &failure_of_the_selected_version; %s }) {
                    return Err(MountReadOnlyFailure::Open(failure_of_the_selected_version));
                }
                let version_fallen_back_to_opened =''' % open_cond)
sub("mount.rs", '''            Err(failure_of_the_selected_version) => {
                let Some(version_fallen_back_to) =''', '''            Err(failure_of_the_selected_version) => {
                if !matches!(&failure_of_the_selected_version, MountError::Recovery(inner) if crate::recovery::failure_may_retreat(inner)) {
                    return Err(failure_of_the_selected_version);
                }
                let Some(version_fallen_back_to) =''')
if variant == "objectlevel":
    sub("recovery.rs", '''        let entry =
            TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;''', '''        let entry = match TreeTableEntry::parse(entry_bytes) {
            Some(entry) => entry,
            None if entry_is_refused_only_for_unknown_flags_on_a_tree_no_file_read_needs(entry_bytes) => continue,
            None => return Err(TreeTableEntriesRefusal::EntryMalformed),
        };''')
    open(root + "recovery.rs", "a").write('''

/// 量过用（草稿副本，对象级）：条目只因 flags 有未知位被拒、且那棵树不是读文件要走的四棵（inode、extent、分配记录、记账）时，读树表的人跳过它。
fn entry_is_refused_only_for_unknown_flags_on_a_tree_no_file_read_needs(bytes: &[u8]) -> bool {
    if bytes.len() != 200 || bytes[bytes.len() - 76..].iter().any(|byte| *byte != 0) {
        return false;
    }
    let kind = u16::from_le_bytes([bytes[10], bytes[11]]);
    let flags = u16::from_le_bytes([bytes[12], bytes[13]]);
    flags != 0
        && !(flags == 1 && kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        && ![TREE_KIND_INODE, TREE_KIND_EXTENT, TREE_KIND_ALLOCATION, TREE_KIND_ACCOUNTING].contains(&kind)
}
''')
print("name=variant_applied variant=" + variant)
