"""攻方变异 W2（m4-batch2-code-r1，T8 必红世界 ③）：在 rebuild.rs 的非测试代码中段加一行带「#[cfg(test)]」字样的注释，
其后 `use crate::walk::check_pool_image;` 并在 rebuild_index_tables 里指名它。用法：mutate_whitelist.py <rebuild.rs> apply|restore"""
import shutil
import sys

path = sys.argv[1]
action = sys.argv[2]
backup = path + ".orig"
if action == "restore":
    shutil.copyfile(backup, path)
    sys.exit(0)
shutil.copyfile(path, backup)
with open(backup, encoding="utf-8") as source:
    text = source.read()
anchor = "/// 从当前版本与那几片容器的整单元字节现算三张表。\n"
assert text.count(anchor) == 1, "函数文档锚点"
text = text.replace(
    anchor,
    "// 下面这个函数的用例在文件末尾 #[cfg(test)] 模块里（攻方变异 W2 加的一行注释）。\n"
    "use crate::walk::check_pool_image;\n\n" + anchor,
)
call_anchor = "    let payload_capacity_in_bytes = DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET;\n"
assert text.count(call_anchor) == 1, "调用锚点"
text = text.replace(
    call_anchor,
    call_anchor + "    let _walk_entry_named_by_the_rebuild = check_pool_image;\n",
)
with open(path, "w", encoding="utf-8") as target:
    target.write(text)
