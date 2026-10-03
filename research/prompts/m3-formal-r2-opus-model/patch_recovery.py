"""把攻方腿的运行时变异开关打进仓副本的 recovery.rs（只在草稿目录里用）。
SINGLEFS_ATTACK_MUTANT 没设或为 0 时，每一处都走原文那一支。"""
import sys
path = sys.argv[1]
src = open(path, encoding="utf-8").read()

def once(old, new):
    global src
    n = src.count(old)
    if n != 1:
        sys.exit(f"锚点命中 {n} 次：{old[:80]!r}")
    src = src.replace(old, new)

once("pub fn replay_journal(\n",
"""/// 攻方腿的运行时变异开关（仓副本专用）：环境变量 SINGLEFS_ATTACK_MUTANT，没设取 0（原文）。
fn attack_mutant() -> u32 {
    static CHOICE: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *CHOICE.get_or_init(|| {
        std::env::var("SINGLEFS_ATTACK_MUTANT")
            .ok()
            .and_then(|text| text.parse().ok())
            .unwrap_or(0)
    })
}

pub fn replay_journal(
""")
# 10：水位之上有两条以上记录就一条都不施加（安全但不单调）
once("    report.above_water = above.len();\n",
"""    report.above_water = above.len();
    if attack_mutant() == 10 && above.len() >= 2 {
        return Ok((report, *root));
    }
""")
# 4 = Z1（一条都不施加），9 = Z2（不看在飞上限）
once("    for record in above.into_iter().take(in_flight_limit) {\n",
"""    let attack_limit = match attack_mutant() {
        4 => 0,
        9 => usize::MAX,
        _ => in_flight_limit,
    };
    for record in above.into_iter().take(attack_limit) {
""")
# 1：锚点读得出那一支 jsn 对不上不断（跳过缺号，从下一条序号为 1 的记录接着施加）
once("            if (record.instance, record.counter) != expected_key {\n                break;\n            }\n",
"""            if (record.instance, record.counter) != expected_key && attack_mutant() != 1 {
                break;
            }
""")
# 2 = 表第 36 行（锚点读不出时不看 txg）
once("        } else if record.checkpoint_txg != chain_start_txg_without_anchor\n",
"        } else if (attack_mutant() != 2 && record.checkpoint_txg != chain_start_txg_without_anchor)\n")
# 3 = 表第 406 行（每条记录都当末条）
once("        if !record_ends_its_publish(record) {\n            continue;\n        }\n",
"""        if !record_ends_its_publish(record) && attack_mutant() != 3 {
            continue;
        }
""")
# 6：实例表指针取记录里的映射根指针；5：分配记录树根清成全零
once("            instance_table: rebuilt.instance_table,\n",
"""            instance_table: if attack_mutant() == 6 {
                record.new_mapping_root
            } else {
                rebuilt.instance_table
            },
""")
once("            allocation_record_tree_root: rebuilt.allocation_record_tree_root,\n",
"""            allocation_record_tree_root: if attack_mutant() == 5 {
                NodePointer::empty_root()
            } else {
                rebuilt.allocation_record_tree_root
            },
""")
open(path, "w", encoding="utf-8").write(src)
print("patched", path)
