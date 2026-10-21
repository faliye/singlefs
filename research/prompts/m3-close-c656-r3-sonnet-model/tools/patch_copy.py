"""仓副本里唯一的改动：崩溃点重放逐状态展开那个函数的可见性 pub(crate) → pub（原型照它的计划逐态取镜像）。
mkfs、恢复、挂载、池级 checker 一行不改；乙与甲七的写序由原型把入库 mkfs 的录制流重排得到。
用法：python3 patch_copy.py <副本根>"""
import pathlib
import sys

tree = pathlib.Path(sys.argv[1])
crash = tree / "crates/singlefs-checker-tier/src/crash.rs"
text = crash.read_text()
old = "pub(crate) fn visit_crash_state_range_of_the_read_back_flow_with_slices("
assert text.count(old) == 1, "crash.rs 锚点变了"
crash.write_text(text.replace(old, "pub fn visit_crash_state_range_of_the_read_back_flow_with_slices("))
print("patched", crash)
