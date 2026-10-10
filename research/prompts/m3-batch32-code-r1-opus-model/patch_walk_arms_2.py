#!/usr/bin/env python3
"""在 patch_walk_arms.py 之后再加一臂 allrbmap：AllRb 之外，F-KEY 那几处读回的不是第 0 条那一份时，
读回那一份的落点在 I-3.5 的折叠表（unit_of_each_placement）里改记成它自己（只在它此刻记的是这条指针第 0 条落点时改）。"""
import sys
path = sys.argv[1]
src = open(path, encoding="utf-8").read()
def once(old, new):
    global src
    n = src.count(old)
    if n != 1:
        sys.exit(f"替换命中 {n} 次：{old[:80]!r}")
    src = src.replace(old, new)
once("""    if opus_key_arm() == OpusKeyArm::AllRb {
        read_back""", """    if matches!(opus_key_arm(), OpusKeyArm::AllRb | OpusKeyArm::AllRbMap) {
        read_back""")
once("""            OpusKeyArm::BirthRb | OpusKeyArm::AllRb => {""", """            OpusKeyArm::BirthRb | OpusKeyArm::AllRb | OpusKeyArm::AllRbMap => {""")
once("""            let key = if opus_key_arm() == OpusKeyArm::B31 {""", """            let first_location = (pointer.locations[0].device, pointer.locations[0].slot);
            let read_back = (location_it_was_read_from.device, location_it_was_read_from.slot);
            if opus_key_arm() == OpusKeyArm::AllRbMap
                && read_back != first_location
                && self.unit_of_each_placement.get(&read_back) == Some(&first_location)
            {
                self.unit_of_each_placement.insert(read_back, read_back);
            }
            let key = if opus_key_arm() == OpusKeyArm::B31 {""")
open(path, "w", encoding="utf-8").write(src)
print("patched", path)
