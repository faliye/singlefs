#!/usr/bin/env python3
"""把攻方腿的一条变异照原文恰好命中一次换掉（在 rerun.sh 建的副本里改，不碰仓）。用法：apply_mutation.py <副本根> <M2|M3|M4|M5>"""
import os
import sys

MUTATIONS = {
    "M2": ("crates/singlefs-core/src/admission.rs", [
        ("""                RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA,
                retirement_tree_internal_levels,
                RETIREMENT_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA,
            ]""", """                0 * RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA,
                0 * retirement_tree_internal_levels,
                0 * RETIREMENT_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA,
            ]"""),
        ("""                    RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA
                        .checked_mul(inode_records)""", """                    (0 * RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA)
                        .checked_mul(inode_records)"""),
        ("""                        nodes.checked_add(central_mapping_entries_of_one_retirement_container_cow(
                            retirement_tree_internal_levels,
                        ))""", """                        nodes.checked_add(0 * central_mapping_entries_of_one_retirement_container_cow(
                            retirement_tree_internal_levels,
                        ))"""),
        ("""                checkpoint_cost_of_one_retirement_container_cow(retirement_tree_internal_levels).0;""",
         """                0 * checkpoint_cost_of_one_retirement_container_cow(retirement_tree_internal_levels).0;"""),
        ("""                + DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST,
            u64::checked_add,
        )
        .and_then(|slots| slots.checked_add(retirement_tree_internal_levels))""", """                + 0 * DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST,
            u64::checked_add,
        )
        .and_then(|slots| slots.checked_add(0 * retirement_tree_internal_levels))"""),
    ]),
    "M3": ("crates/singlefs-core/src/retirement_tree.rs", [
        ("""            RetirementContainerCapacity::FromTheContainerFormat => RETIREMENT_CONTAINER_RECORDS,""",
         """            RetirementContainerCapacity::FromTheContainerFormat => 1,"""),
    ]),
    "M4": ("crates/singlefs-core/src/inode_tree.rs", [
        ("""            if right_pair_fits {
                self.absorb_the_right_neighbour(current_position);
            } else if left_pair_fits {
                self.absorb_the_right_neighbour(current_position - 1);
                current_position -= 1;
            } else {""", """            if left_pair_fits {
                self.absorb_the_right_neighbour(current_position - 1);
                current_position -= 1;
            } else if right_pair_fits {
                self.absorb_the_right_neighbour(current_position);
            } else {"""),
    ]),
    "M5": ("crates/singlefs-core/src/mount.rs", [
        ("""        &ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table),""",
         """        &ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table)[..0],"""),
    ]),
}


def main():
    root, name = sys.argv[1], sys.argv[2]
    relative, replacements = MUTATIONS[name]
    path = os.path.join(root, relative)
    with open(path, encoding="utf-8") as source:
        text = source.read()
    for old, new in replacements:
        count = text.count(old)
        if count != 1:
            sys.exit(f"{name}: 原文在 {relative} 里命中 {count} 次，不是 1 次")
        text = text.replace(old, new)
    temporary = path + ".mutating"
    with open(temporary, "w", encoding="utf-8") as target:
        target.write(text)
    os.replace(temporary, path)
    print(f"{name}: {relative} 改了 {len(replacements)} 处")


if __name__ == "__main__":
    main()
