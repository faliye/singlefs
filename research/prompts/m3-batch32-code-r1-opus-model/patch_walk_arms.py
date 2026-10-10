#!/usr/bin/env python3
"""给仓副本的 walk.rs 加臂开关（环境变量 OPUS_B32_KEY_ARM）。只改副本。每处替换恰好命中一次，否则退出 3。
用法：patch_walk_arms.py <walk.rs 路径>"""
import sys
path = sys.argv[1]
src = open(path, encoding="utf-8").read()

def once(old, new):
    global src
    n = src.count(old)
    if n != 1:
        sys.exit(f"替换命中 {n} 次（要 1 次）：{old[:80]!r}")
    src = src.replace(old, new)

# 1 臂开关
once("use std::cmp::Ordering;\n",
"""use std::cmp::Ordering;

/// 攻方臂（只在攻方副本里）：一个单元的身份按哪个落点认。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum OpusKeyArm {
    /// 冻结副本今天的样子：走过按读回那一份，诞生代号、合并、父单元、叶序按第 0 条。
    Today,
    /// 第 31 批的样子：全按第 0 条（F-KEY 之前）。
    B31,
    /// 实现员提的候选：诞生代号按读回那一份记，合并照旧按第 0 条。
    BirthRb,
    /// 一个单元的身份 = 读回那一份的落点：走过、诞生代号、合并、父单元、叶序都按它（F-KEY 那几处）。
    AllRb,
    /// AllRb 之外，F-KEY 那几处读回的不是第 0 条那一份时，读回那一份的落点在 I-3.5 的折叠表（unit_of_each_placement）里记成它自己。
    AllRbMap,
    /// 诞生代号按读回那一份记，第 0 条落点还没记过时也记一份（or_insert），合并照旧按第 0 条。
    Both,
    /// 仿 GPU 今天的写法：诞生代号按第 0 条记，但只在第 0 条落点在走过表里时记得下（`w_judge_birth_identity`）。
    GpuEmu,
}

static OPUS_KEY_ARM: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(u8::MAX);

fn opus_key_arm_of(name: &str) -> OpusKeyArm {
    match name {
        "today" => OpusKeyArm::Today,
        "b31" => OpusKeyArm::B31,
        "birthrb" => OpusKeyArm::BirthRb,
        "allrb" => OpusKeyArm::AllRb,
        "allrbmap" => OpusKeyArm::AllRbMap,
        "both" => OpusKeyArm::Both,
        "gpuemu" => OpusKeyArm::GpuEmu,
        other => panic!("臂 {other} 不认得"),
    }
}

/// 探针在同一个进程里换臂（单线程跑）。
pub fn opus_set_key_arm(name: &str) {
    let code = opus_key_arm_of(name) as u8;
    OPUS_KEY_ARM.store(code, std::sync::atomic::Ordering::SeqCst);
}

fn opus_key_arm() -> OpusKeyArm {
    const ARMS: [OpusKeyArm; 7] = [
        OpusKeyArm::Today,
        OpusKeyArm::B31,
        OpusKeyArm::BirthRb,
        OpusKeyArm::AllRb,
        OpusKeyArm::AllRbMap,
        OpusKeyArm::Both,
        OpusKeyArm::GpuEmu,
    ];
    match OPUS_KEY_ARM.load(std::sync::atomic::Ordering::SeqCst) {
        u8::MAX => opus_key_arm_of(std::env::var("OPUS_B32_KEY_ARM").as_deref().unwrap_or("today")),
        code => ARMS[usize::from(code)],
    }
}

/// AllRb 臂下合并、父单元、叶序用读回那一份的落点，别的臂用第 0 条。
fn opus_identity_key(first_location: (u32, u64), read_back: (u32, u64)) -> (u32, u64) {
    if opus_key_arm() == OpusKeyArm::AllRb {
        read_back
    } else {
        first_location
    }
}
""")

# 2 诞生代号记在哪
once("""    fn judge_birth_identity_of_a_referenced_unit(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        what: &str,
    ) {""", """    fn judge_birth_identity_of_a_referenced_unit(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        what: &str,
    ) {
        self.judge_birth_identity_of_a_referenced_unit_read_back_from(unit, pointer, what, None);
    }

    fn judge_birth_identity_of_a_referenced_unit_read_back_from(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        what: &str,
        read_back: Option<(u32, u64)>,
    ) {""")
once("""        self.birth_txg_of_walked_units.insert(
            (pointer.locations[0].device, pointer.locations[0].slot),
            birth_txg,
        );""", """        let first_location = (pointer.locations[0].device, pointer.locations[0].slot);
        let read_back = read_back.unwrap_or(first_location);
        match opus_key_arm() {
            OpusKeyArm::Today | OpusKeyArm::B31 => {
                self.birth_txg_of_walked_units.insert(first_location, birth_txg);
            }
            OpusKeyArm::BirthRb | OpusKeyArm::AllRb => {
                self.birth_txg_of_walked_units.insert(read_back, birth_txg);
            }
            OpusKeyArm::Both => {
                self.birth_txg_of_walked_units.insert(read_back, birth_txg);
                self.birth_txg_of_walked_units.entry(first_location).or_insert(birth_txg);
            }
            OpusKeyArm::GpuEmu => {
                if self.visited_units.contains(&first_location) {
                    self.birth_txg_of_walked_units.insert(first_location, birth_txg);
                }
            }
        }""")

# 3 B31：走过的键退回第 0 条
once("""        .map(|(unit, location_it_was_read_from)| {
            (
                unit,
                (
                    location_it_was_read_from.device,
                    location_it_was_read_from.slot,
                ),
            )
        })""", """        .map(|(unit, location_it_was_read_from)| {
            let key = if opus_key_arm() == OpusKeyArm::B31 {
                (pointer.locations[0].device, pointer.locations[0].slot)
            } else {
                (
                    location_it_was_read_from.device,
                    location_it_was_read_from.slot,
                )
            };
            (unit, key)
        })""")

# 4 容器
once("""        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
            return;
        }
        let container_name = format!("{name}的容器");""", """        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(opus_identity_key(
                child_placement,
                placement_it_was_read_from,
            ));
            return;
        }
        let container_name = format!("{name}的容器");""")
once("""        self.judge_birth_identity_of_a_referenced_unit(&unit, child, &container_name);""",
     """        self.judge_birth_identity_of_a_referenced_unit_read_back_from(
            &unit,
            child,
            &container_name,
            Some(placement_it_was_read_from),
        );""")

# 5 叶容器
once("""        placement_it_was_read_from: (u32, u64),
    ) -> InodeSubtreeInTheWalk {
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
            return InodeSubtreeInTheWalk {
                leaves_in_leaf_order: self
                    .inode_leaf_order_below_a_walked_unit
                    .get(&child_placement)
                    .cloned()
                    .unwrap_or_default(),
                walked_first_by_this_version: false,
            };
        }
        let mut leaves_in_leaf_order: Vec<InodeLeafContainerInLeafOrder> = Vec::new();""",
"""        placement_it_was_read_from: (u32, u64),
    ) -> InodeSubtreeInTheWalk {
        let child_placement = opus_identity_key(child_placement, placement_it_was_read_from);
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
            return InodeSubtreeInTheWalk {
                leaves_in_leaf_order: self
                    .inode_leaf_order_below_a_walked_unit
                    .get(&child_placement)
                    .cloned()
                    .unwrap_or_default(),
                walked_first_by_this_version: false,
            };
        }
        let mut leaves_in_leaf_order: Vec<InodeLeafContainerInLeafOrder> = Vec::new();""")
once("""        self.judge_birth_identity_of_a_referenced_unit(unit, child, "inode 树的叶容器");""",
     """        self.judge_birth_identity_of_a_referenced_unit_read_back_from(
            unit,
            child,
            "inode 树的叶容器",
            Some(placement_it_was_read_from),
        );""")

# 6 内部孩子
once("""        let not_walked_below = InodeSubtreeInTheWalk {
            leaves_in_leaf_order: Vec::new(),
            walked_first_by_this_version: true,
        };
        if !self.visited_units.insert(placement_it_was_read_from) {""",
"""        let not_walked_below = InodeSubtreeInTheWalk {
            leaves_in_leaf_order: Vec::new(),
            walked_first_by_this_version: true,
        };
        let child_placement = opus_identity_key(child_placement, placement_it_was_read_from);
        if !self.visited_units.insert(placement_it_was_read_from) {""")
once("""        self.judge_birth_identity_of_a_referenced_unit(unit, child, what);
        let Some(view) = self.index_node_view_judging_a_zero_entry_width(""",
"""        self.judge_birth_identity_of_a_referenced_unit_read_back_from(
            unit,
            child,
            what,
            Some(placement_it_was_read_from),
        );
        let Some(view) = self.index_node_view_judging_a_zero_entry_width(""")

# 7 数据单元
once("""        let placement = (pointer.locations[0].device, pointer.locations[0].slot);
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(placement);
            return;
        }
        if !self.judge_unit_header(&unit, 1, "数据单元") {
            return;
        }
        self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, "数据单元");""",
"""        let placement = opus_identity_key(
            (pointer.locations[0].device, pointer.locations[0].slot),
            placement_it_was_read_from,
        );
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(placement);
            return;
        }
        if !self.judge_unit_header(&unit, 1, "数据单元") {
            return;
        }
        self.judge_birth_identity_of_a_referenced_unit_read_back_from(
            &unit,
            &pointer,
            "数据单元",
            Some(placement_it_was_read_from),
        );""")

open(path, "w", encoding="utf-8").write(src)
print("patched", path)
