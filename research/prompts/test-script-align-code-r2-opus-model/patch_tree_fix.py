"""攻方腿提的改法（只在我的模型上量过、被攻过零轮），在仓副本里按环境变量 R2_FIX_BOUND_BY_THE_BASE_ROOT 开关，默认关、与冻结副本同：
一个点之前一条根槽写都没有、又没有没做完的发布时，够得着的版本按基镜像里的根（扇区里带根 magic 的，取 (txg, 实例) 最大的）截，
不再整张版本表都进。用法：python3 patch_tree_fix.py <仓副本根>（在 patch_tree.py 之后）"""
import sys, os
path = f"{sys.argv[1]}/crates/singlefs-checker-tier/src/crash_amplification.rs"
s = open(path, encoding="utf-8").read()
def once(old, new):
    global s
    assert s.count(old) == 1, old
    s = s.replace(old, new, 1)
once('''    writes_after_the_point: WritesAfterThePoint,
}

impl JudgeInputsOfTheFlow {''', '''    writes_after_the_point: WritesAfterThePoint,
    /// 原型改法：基镜像里的根（带根 magic 的扇区）里 (txg, 实例) 最大的那一条。
    highest_base_root: Option<(CheckpointTxg, InstanceGeneration)>,
}

impl JudgeInputsOfTheFlow {''')
once('''        Self {
            base_devices_text: format!(''', '''        let highest_base_root = flow
            .base
            .devices
            .values()
            .flat_map(|sparse| sparse.written_sectors().values().cloned().collect::<Vec<_>>())
            .filter(|bytes| bytes.len() >= 36 && &bytes[..4] == b"SFSR")
            .map(|bytes| {
                let (instance, checkpoint_txg) =
                    singlefs_harness::memory_pool::root_identity_written_by(&bytes);
                (checkpoint_txg, instance)
            })
            .max();
        Self {
            highest_base_root,
            base_devices_text: format!(''')
once('''        match highest_reachable_root {
            Some((checkpoint_txg, instance)) => text.push_str(&format!(''', '''        let highest_reachable_root = if highest_reachable_root.is_none()
            && !a_publish_is_unfinished
            && std::env::var_os("R2_FIX_BOUND_BY_THE_BASE_ROOT").is_some()
        {
            text.push_str("versions_bounded_by_the_base_root\\n");
            self.highest_base_root
        } else {
            highest_reachable_root
        };
        match highest_reachable_root {
            Some((checkpoint_txg, instance)) => text.push_str(&format!(''')
open(path + ".tmp", "w", encoding="utf-8").write(s); os.replace(path + ".tmp", path)
print("patched (fix behind R2_FIX_BOUND_BY_THE_BASE_ROOT)", path)
