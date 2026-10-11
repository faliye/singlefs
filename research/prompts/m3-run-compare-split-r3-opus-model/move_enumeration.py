# 攻方腿模型（m3-run-compare-split-r3）：在一棵树（副本拷出来的 Cargo.toml、Cargo.lock、crates/）上做簇 0 那一种纯搬家——
# 把 crash.rs 的枚举那几段（发布归属、段归属、枚举表、展开与撕裂、状态计划）原样挪进新叶子 crash_enumeration.rs，
# crash.rs 补 `pub use` 与 `use`，叶子补 crash.rs 原来的 use 块，lib.rs 加 `pub mod`；再加编译器要的 pub(crate)
# （第一遍 cargo check 的 E0603/E0616/E0624 逐条定下的 8 处项头与 12 处字段 / 方法，原样写死在下面）。
# 用法：move_enumeration.py <树根> [--facts-import]   第二个参数另把 crash_facts.rs 第 28 行的 `use crate::crash::{…}` 改指叶子。
import re, sys
root = sys.argv[1]
src = root + "/crates/singlefs-checker-tier/src"
lines = open(src + "/crash.rs").read().split("\n")
assert lines[343] == "pub enum Layer0PublishOfState {" and lines[2323] == "struct Layer0StatePlan<'plan> {", "crash.rs 行号与 E5 副本对不上"
ranges = [(339, 412), (532, 548), (1561, 1630), (1886, 2508)]
moved = [l for i, l in enumerate(lines, 1) if any(a <= i <= b for a, b in ranges)]
keep = [l for i, l in enumerate(lines, 1) if not any(a <= i <= b for a, b in ranges)]
names = ["Layer0PublishOfState", "publish_of_each_segment", "Layer0SegmentOfState", "TornImageTableRow", "Layer0EnumerationTables",
         "layer0_enumeration_tables", "layer0_plan_state_count", "Layer0SegmentExpansion", "TearableInPlaceOverwrites",
         "torn_image_of_in_place_overwrite", "layer0_state_count", "layer0_state_count_with_torn_in_place_overwrites",
         "quick_tier_expansion", "full_expansion"]
keep.insert(43, "pub use crate::crash_enumeration::{" + ", ".join(names) + "};\nuse crate::crash_enumeration::*;")
new = ["//! 层 0 的枚举：段怎么展开、每个状态的持久集合（从 `crash.rs` 原样挪来）。", ""] + lines[5:42] + [""] + moved + [""]
heads = 0
for i, l in enumerate(new):
    if re.match(r"^(struct|enum|fn|const) ", l):
        new[i] = "pub(crate) " + l; heads += 1
fields = {("WritesWithTornImages", "writes"), ("Layer0StatePlan", "segments"), ("Layer0StatePlan", "expansion_by_segment"),
          ("Layer0StatePlan", "state_ranges_by_segment"), ("Layer0StatePlan", "state_count")}
methods = {"of", "new", "segment_of_state", "publish_of_state", "persisted_writes_of_state", "segment_of_state_for_findings", "segment_label"}
inner = 0; current = None
for i, l in enumerate(new):
    m = re.match(r"^(?:pub(?:\(crate\))? )?struct (\w+)", l)
    if m: current = m.group(1)
    if l.startswith("}"): current = None
    fm = re.match(r"^    (\w+): ", l)
    if current and fm and (current, fm.group(1)) in fields:
        new[i] = "    pub(crate) " + l[4:]; inner += 1
    mm = re.match(r"^    fn (\w+)", l)
    if mm and mm.group(1) in methods:
        new[i] = "    pub(crate) " + l[4:]; inner += 1
open(src + "/crash.rs", "w").write("\n".join(keep))
open(src + "/crash_enumeration.rs", "w").write("\n".join(new))
lib = open(src + "/lib.rs").read()
assert lib.count("pub mod crash;\n") == 1
open(src + "/lib.rs", "w").write(lib.replace("pub mod crash;\n", "pub mod crash;\npub mod crash_enumeration;\n"))
facts = 0
if "--facts-import" in sys.argv:
    f = open(src + "/crash_facts.rs").read().split("\n")
    assert f[27].startswith("use crate::crash::{layer0_enumeration_tables")
    f[27] = f[27].replace("use crate::crash::{", "use crate::crash_enumeration::{"); facts = 1
    open(src + "/crash_facts.rs", "w").write("\n".join(f))
print(f"MOVE moved_lines={len(moved)} head_visibility_edits={heads} in_item_visibility_edits={inner} facts_import_edits={facts}")
