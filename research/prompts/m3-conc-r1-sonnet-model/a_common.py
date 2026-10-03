"""三份产物共用的读法：按 `E7RESULT name=...` 行取 key=value。只读 research/results/ 下的产物。"""
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = os.path.join(ROOT, "research", "results")
E169_ONE = "e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out"
E169_TWO = "e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out"
E169_THREE = "e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out"
E166 = "e166-parallel-commit-shared-state-partition-2026-10-01.out"
E167_ONE = "e167-multi-inflight-crash-cut-2026-10-01-r2.out"
E167_TWO = "e167-multi-inflight-crash-cut-second-2026-10-01-r3.out"
E167_THREE = "e167-multi-inflight-crash-cut-third-2026-10-01.out"


def rows(name, prefix):
    """→ [(行号, {字段: 值})]，行号是产物自己的行号（从 1 起）。"""
    out = []
    with open(os.path.join(R, name), encoding="utf-8") as handle:
        for number, line in enumerate(handle, 1):
            if line.startswith("E7RESULT " + prefix):
                out.append((number, dict(re.findall(r"(\S+?)=(\S+)", line))))
    return out


def short(arm):
    """bing0_today → bing0（Q3-3 行的字段名只有短名）。"""
    return arm.split("_")[0]
