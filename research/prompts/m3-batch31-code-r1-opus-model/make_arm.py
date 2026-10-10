#!/usr/bin/env python3
"""在一份 crates 副本上造一条臂（只改副本，不碰仓）。每处替换旧串必须恰好命中一次，否则退 3。
用法：make_arm.py <副本根> <臂>
  base      不改源码，只把 probe_kbi.rs 追加到 checker_known_bad_images.rs 末尾（每条臂都做这一步）
  mut-fl0   base + 删掉 F-L0 那一处调用（crates/mutations.tsv「第31批 F-L0」那一行的替换）：校验路径会红的自证
  fix-key   base + 攻方改法 F-KEY（零轮）：容器树容器、inode 树孩子、read_index_node 三处「走过」按读回的那一份的落点认
"""
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent


def replace_once(path, old, new):
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        print(f"MAKE-ARM 旧串在 {path} 里命中 {count} 次：{old[:80]!r}")
        sys.exit(3)
    path.write_text(text.replace(old, new), encoding="utf-8")


def main():
    root = pathlib.Path(sys.argv[1])
    arm = sys.argv[2]
    walk = root / "crates/singlefs-checker/src/walk.rs"
    kbi = root / "crates/singlefs-harness/tests/checker_known_bad_images.rs"
    with kbi.open("a", encoding="utf-8") as out:
        out.write((HERE / "probe_kbi.rs").read_text(encoding="utf-8"))
    if arm == "base":
        pass
    elif arm == "mut-fl0":
        replace_once(
            walk,
            '        if node.level <= 1 {\n            self.judge_every_copy_of_a_child_the_walk_does_not_go_into(\n'
            '                child,\n                &format!("{name}内部条目的子指针"),\n            );\n',
            "        if node.level <= 1 {\n",
        )
    elif arm == "fix-key":
        replace_once(
            walk,
            "        let Some(unit) = read_referenced_unit(\n            self.reader,\n            &mut self.judgements,\n"
            "            &pointer.locations,\n            node_bytes(),\n            what,\n        ) else {\n"
            "            self.walk_failures\n                .push(format!(\"{what} 两份都读不到对得上的\"));\n"
            "            return None;\n        };\n"
            "        let placement = (pointer.locations[0].device, pointer.locations[0].slot);\n"
            "        if !self.visited_units.insert(placement) {\n",
            "        let Some((unit, location_it_was_read_from)) = crate::image::read_referenced_unit_with_the_location_it_was_read_from(\n"
            "            self.reader,\n            &mut self.judgements,\n"
            "            &pointer.locations,\n            node_bytes(),\n            what,\n        ) else {\n"
            "            self.walk_failures\n                .push(format!(\"{what} 两份都读不到对得上的\"));\n"
            "            return None;\n        };\n"
            "        let placement = (pointer.locations[0].device, pointer.locations[0].slot);\n"
            "        if !self.visited_units.insert((location_it_was_read_from.device, location_it_was_read_from.slot)) {\n",
        )
        replace_once(
            walk,
            "        let Some(unit) = read_referenced_unit(\n            self.reader,\n            &mut self.judgements,\n"
            "            &child.locations,\n            data_unit_bytes(),\n            &what,\n        ) else {\n"
            "            // 容器两份都读不出",
            "        let Some((unit, location_it_was_read_from)) = crate::image::read_referenced_unit_with_the_location_it_was_read_from(\n"
            "            self.reader,\n            &mut self.judgements,\n"
            "            &child.locations,\n            data_unit_bytes(),\n            &what,\n        ) else {\n"
            "            // 容器两份都读不出",
        )
        replace_once(
            walk,
            "        let child_placement = (child.locations[0].device, child.locations[0].slot);\n"
            "        if !self.visited_units.insert(child_placement) {\n",
            "        let child_placement = (child.locations[0].device, child.locations[0].slot);\n"
            "        if !self.visited_units.insert((location_it_was_read_from.device, location_it_was_read_from.slot)) {\n",
        )
        replace_once(
            walk,
            "            let Some(unit) = read_referenced_unit(\n                self.reader,\n                &mut self.judgements,\n"
            "                &child.locations,\n                length,\n                \"inode 树的孩子\",\n            ) else {\n",
            "            let Some((unit, location_it_was_read_from)) = crate::image::read_referenced_unit_with_the_location_it_was_read_from(\n"
            "                self.reader,\n                &mut self.judgements,\n"
            "                &child.locations,\n                length,\n                \"inode 树的孩子\",\n            ) else {\n",
        )
        replace_once(
            walk,
            "            let child_placement = (child.locations[0].device, child.locations[0].slot);\n"
            "            let subtree = match registered_entry_type {\n",
            "            let child_placement = (location_it_was_read_from.device, location_it_was_read_from.slot);\n"
            "            let subtree = match registered_entry_type {\n",
        )
    else:
        print(f"MAKE-ARM 不认得的臂 {arm}")
        sys.exit(2)
    print(f"MAKE-ARM {arm} ok")


if __name__ == "__main__":
    main()
