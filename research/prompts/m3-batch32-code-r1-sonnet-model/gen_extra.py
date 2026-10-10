#!/usr/bin/env python3
"""生成 extra_mutations.tsv（本腿自己加的变异行，格式同 crates/mutations.tsv：六段，换行写成 \\n）。"""
import sys
NEW_TESTS = [
    "whichever_location_names_a_node_walked_before",
    "sixteen_kibibytes_beside_a_node",
    "over_a_child_whose_two_location_checksums_are_wrong",
    "goes_down_the_type_zero_branch_on_either_device",
    "does_not_go_down_that_copy",
    "below_a_level_one_node_whose",
    "verifying_as_a_node",
]
ARGS = "-p singlefs-harness --test checker_known_bad_images -- " + " ".join(NEW_TESTS)
walk = "crates/singlefs-checker/src/walk.rs"
rows = [
 ("E1 G4 钉哪一读：容器、inode 孩子、数据单元三处读单元不判 I-2.1（只取第一份校验和对得上的）", walk,
  "        crate::image::read_referenced_unit_with_the_location_it_was_read_from(\n            self.reader,\n            &mut self.judgements,\n            &pointer.locations,\n            unit_bytes,\n            what,\n        )\n        .map(|(unit, location_it_was_read_from)| {",
  "        let _ = what;\n        pointer\n            .locations\n            .iter()\n            .find_map(|location| {\n                self.reader\n                    .read(location.device, location.slot * SLOT_BYTES, unit_bytes)\n                    .filter(|content| crc32_castagnoli_table(content) == location.checksum)\n                    .map(|content| (content, *location))\n            })\n        .map(|(unit, location_it_was_read_from)| {",
  ARGS, "an_inode_leaf_with_a_non_zero_reserved_byte_beside_a_copy_verifying_as_a_node_reddens_the_reserved_bytes_invariant_on_either_device"),
 ("E2 F-KEY 只退数据单元那一处", walk,
  "        let placement = (pointer.locations[0].device, pointer.locations[0].slot);\n        if !self.visited_units.insert(placement_it_was_read_from) {",
  "        let placement = (pointer.locations[0].device, pointer.locations[0].slot);\n        if !self.visited_units.insert(placement) {",
  ARGS, "a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before"),
 ("E3 F-KEY 只退 inode 叶容器那一处", walk,
  "    ) -> InodeSubtreeInTheWalk {\n        if !self.visited_units.insert(placement_it_was_read_from) {",
  "    ) -> InodeSubtreeInTheWalk {\n        if !self.visited_units.insert(child_placement) {",
  ARGS, "an_inode_leaf_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before"),
 ("E4 F-KEY 只退 inode 内部孩子那一处", walk,
  "            walked_first_by_this_version: true,\n        };\n        if !self.visited_units.insert(placement_it_was_read_from) {",
  "            walked_first_by_this_version: true,\n        };\n        if !self.visited_units.insert(child_placement) {",
  ARGS, "an_inode_leaf_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before"),
 ("E5 F-KEY 只退容器树内部条目的孩子那一处", walk,
  "        let child_placement = (child.locations[0].device, child.locations[0].slot);\n        if !self.visited_units.insert(placement_it_was_read_from) {",
  "        let child_placement = (child.locations[0].device, child.locations[0].slot);\n        if !self.visited_units.insert(child_placement) {",
  ARGS, "a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before"),
 ("E6 F-ID-b 去掉两份都没自证过时的读法（当读不出）", walk,
  "                        .or_else(|| {\n                            read_unit_without_judging(self.reader, &child, node_bytes())\n                                .map(|unit| unit[UNIT_HEADER_CLASS_TAG_OFFSET])\n                        });",
  "                        ;",
  ARGS, "a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device"),
 ("E7 F-ID-b 取最后一份自证过的副本（不是第一份）", walk,
  "    child.locations.iter().find_map(|location| {\n        let offset = location.slot * SLOT_BYTES;\n        // 头按最短的单元（码 2，16 KiB）那么长读：32 KiB 的单元头也在这前 16 KiB 里。",
  "    child.locations.iter().rev().find_map(|location| {\n        let offset = location.slot * SLOT_BYTES;\n        // 头按最短的单元（码 2，16 KiB）那么长读：32 KiB 的单元头也在这前 16 KiB 里。",
  ARGS, "a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device"),
 ("E8 F-ID-b 自证判据不看长度（头里的类标签对、16 KiB 校验和对得上就算）", walk,
  "        reader\n            .read(location.device, offset, unit_bytes)\n            .filter(|content| crc32_castagnoli_table(content) == location.checksum)\n            .map(|_content| class_tag_in_the_header)",
  "        reader\n            .read(location.device, offset, node_bytes())\n            .filter(|content| crc32_castagnoli_table(content) == location.checksum)\n            .map(|_content| class_tag_in_the_header)",
  ARGS, "a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device"),
 ("F1-a 候选改法 F1（容器那一处）：合并与父单元的键也按读回那一份的落点", walk,
  "        let child_placement = (child.locations[0].device, child.locations[0].slot);\n        if !self.visited_units.insert(placement_it_was_read_from) {",
  "        let child_placement = placement_it_was_read_from;\n        if !self.visited_units.insert(placement_it_was_read_from) {",
  "-p singlefs-harness --test checker_known_bad_images -- --exact probe_of_the_sonnet_leg_for_batch32_i49 --nocapture", "probe_of_the_sonnet_leg_for_batch32_i49"),
 ("F1-b 候选改法 F1（容器那一处）：诞生代号记在读回那一份的落点上（位置条目对调之后交给原函数）", walk,
  "        self.judge_birth_identity_of_a_referenced_unit(&unit, child, &container_name);",
  "        {\n            let mut child_read_from_first = *child;\n            if placement_it_was_read_from != (child.locations[0].device, child.locations[0].slot) {\n                child_read_from_first.locations.swap(0, 1);\n            }\n            self.judge_birth_identity_of_a_referenced_unit(&unit, &child_read_from_first, &container_name);\n        }",
  "-p singlefs-harness --test checker_known_bad_images -- --exact probe_of_the_sonnet_leg_for_batch32_i49 --nocapture", "probe_of_the_sonnet_leg_for_batch32_i49"),
]
with open(sys.argv[1] if len(sys.argv) > 1 else "extra_mutations.tsv", "w", encoding="utf-8") as out:
    for name, file, original, replacement, args, named in rows:
        esc = lambda text: text.replace("\n", "\\n")
        out.write("\t".join([name, file, esc(original), esc(replacement), args, named]) + "\n")
print(len(rows), "rows")
