"""e171-conclusion-r3 攻方：在 O3 冻结执行器的副本上造三个「变坏的 O3」（只改副本，原目录不动）。
v1 已确认下界丢了：ack(c) 恒为 0（A(c) = [0, begun]）。
v2 只判恢复得出：L(c) 有就判「在」，不再看 L(c) 在不在 A(c)（只在整池拒、择不出根时判「不在」）。
v3 A(c) 取版本表全部各版：v1 再加 begun(c) 恒为最后一版。
用法：python3 make_o3_variants.py <v1 根> <v2 根> <v3 根>   （每个根下有 research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/）
"""
import os
import sys

EXECUTOR = "research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor"
ACK = "                acknowledged = version.version_index\n"
ACK_V = "                acknowledged = 0  # opus-r3 变异 v1：已确认下界丢了\n"
LANDED = "    landed_is_allowed = recovery.landed is not None and any(version.projection == recovery.landed for version in allowed.allowed)\n"
LANDED_V = "    landed_is_allowed = recovery.landed is not None  # opus-r3 变异 v2：只判恢复得出\n"
BEGUN = "    begun = max(begun_candidates) if begun_candidates else acknowledged\n"
BEGUN_V = "    begun = len(table.versions) - 1  # opus-r3 变异 v3：已开始上界丢了\n"


def replace_once(path, old, new):
    text = open(path, encoding="utf-8").read()
    assert text.count(old) == 1, f"{path}：旧串命中 {text.count(old)} 次"
    with open(path + ".tmp", "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new, 1))
    os.replace(path + ".tmp", path)


v1, v2, v3 = sys.argv[1:4]
replace_once(os.path.join(v1, EXECUTOR, "version_table.py"), ACK, ACK_V)
replace_once(os.path.join(v2, EXECUTOR, "executor_core.py"), LANDED, LANDED_V)
replace_once(os.path.join(v3, EXECUTOR, "version_table.py"), ACK, ACK_V)
replace_once(os.path.join(v3, EXECUTOR, "version_table.py"), BEGUN, BEGUN_V)
print("variants made")
