"""本腿提的改法 B0R（只在我的模型上量过、被攻过零轮），在仓副本里按环境变量 R3_FIX_B0_RECOVER_THE_BASE 开，默认关、与冻结副本同义：
基镜像的上界不只取基镜像里带根 magic 的扇区，再并上「对基镜像本身照 journal 恢复（JournalPolicy::Consult）实际走的那条根」——
基镜像里有记录落了、根没落的那一次发布时，恢复重建出的那一代比基镜像里任何一条根都新，B0 截掉了它。
用法：python3 patch_fix.py <仓副本根>（在 patch_mutations.py 之后）"""
import sys, os
path = f"{sys.argv[1]}/crates/singlefs-checker-tier/src/crash_amplification.rs"
s = open(path, encoding="utf-8").read()
old = '''            newest_root_in_the_base_image: newest_root_in_the_base_image(flow.base),'''
new = '''            newest_root_in_the_base_image: if std::env::var_os("R3_FIX_B0_RECOVER_THE_BASE").is_some() {
                newest_root_in_the_base_image(flow.base)
                    .into_iter()
                    .chain(
                        singlefs_core::recovery::recover(
                            flow.base,
                            singlefs_core::recovery::JournalPolicy::Consult,
                        )
                        .effective_root
                        .map(|(instance, checkpoint_txg)| (checkpoint_txg, instance)),
                    )
                    .max()
            } else {
                newest_root_in_the_base_image(flow.base)
            },'''
assert s.count(old) == 1
s = s.replace(old, new, 1)
open(path + ".tmp", "w", encoding="utf-8").write(s); os.replace(path + ".tmp", path)
print("patched fix behind R3_FIX_B0_RECOVER_THE_BASE", path)
