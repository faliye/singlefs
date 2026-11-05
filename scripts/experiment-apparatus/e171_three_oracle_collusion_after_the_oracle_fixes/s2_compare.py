#!/usr/bin/env python3
"""E171（三oracle合谋测试） 第 1 次重跑第一段第 2 步的逐文件比（停机 S2）：E170 导出装置 `dump` 写出的流一、流二，与本装置基线
`baseline --streams 1,2` 的导出逐文件比。

判法出处（照原文，不另加）：
  - 原登记第十一节 S2（`git show 31128273b^:research/prompts/e171-preregistration.md` 第 561 行）：「同一棵树上 E170 导出装置 `dump` 写出的
    `base-sectors.bin`、`ops.tsv`、`ops-payload.bin`、`calls.tsv`、`torn.tsv`、`torn-payload.bin`、`states.tsv`、`in-place-subsets.tsv` 与本装置基线的任一个
    sha256 不等，或 `impl-verdicts.tsv` 去掉耗时列之后不逐字节相同」；
  - 第 1 次重跑登记 research/prompts/e171-r1-prereg.md 第十一节 S2（「原登记 S2 原样」）与修订 8 第 8.4 节步表第 2 步（「照旧」）；
  - 单元脚本接着核的 `s2_differing_files=0 s2_compared_files=18` 与产物行形，照第 1 次重跑那一趟的产物
    （`git show 04ceaa904^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-r6-r3.out`）。
当年那一份写在仓外的草稿里、丢了（git 里也没有）；这一份是照上面几处重写的，sha256 与当年记的 8ca65474… 不同（第 2 次重跑登记
research/prompts/e171-r2-prereg.md 修订 4）。

比什么：两条流各九件，共 18 件。八个文件比整份的 sha256；`impl-verdicts.tsv` 每行去掉最后一列（耗时）之后比 sha256，行里记
`file=impl-verdicts.tsv_without_timing`，去法与作废 Z1 那一比同一份（segment_one_comparator.py 的 `lines_without_the_last_column`）。
一边没有那个文件的记 `absent`、算不同：读不到不当成相同。

产物（行序固定）：
  E7RESULT name=config segment=s1 product=s2 e170_dump=<给的路径> device_baseline=<给的路径>
  E7RESULT name=s2 stream=<流> file=<文件> e170_sha256=<…|absent> device_sha256=<…|absent> equal=<true|false>   （18 行）
  E7RESULT name=verdict s2_differing_files=<不同的件数> s2_compared_files=<比了的件数>
  E7RESULT name=done emitted=21
S2 触没触发不在这里判：单元脚本接着用 check_product.py verdict 核 `s2_differing_files=0 s2_compared_files=18`。

用法：s2_compare.py <E170 导出目录> <本装置基线导出目录> <产物（新文件）>
退出码：0 写完（不论有没有不同）；2 用法错或产物已存在；78 两个导出目录有一个不在。
"""

from __future__ import annotations

import hashlib
import os
import sys

from result_lines import ResultLines, field
from run_directory import STREAM_NAMES, file_sha256
from segment_one_comparator import lines_without_the_last_column

# E170 导出装置只导流一、流二（原登记 S2「本装置照抄的两条流」）
S2_STREAMS = STREAM_NAMES[:2]
# 原登记 S2 点名的八个文件，比整份
S2_WHOLE_FILES = ("base-sectors.bin", "ops.tsv", "ops-payload.bin", "calls.tsv", "torn.tsv", "torn-payload.bin", "states.tsv", "in-place-subsets.tsv")
IMPLEMENTATION_VERDICTS_FILE = "impl-verdicts.tsv"
IMPLEMENTATION_VERDICTS_LABEL = "impl-verdicts.tsv_without_timing"
ABSENT = "absent"
EXIT_USAGE = 2
EXIT_RUN_CONDITION = 78


def compared_digest(path: str, without_timing: bool) -> str:
    """一件的 sha256：不在写 absent；impl-verdicts.tsv 先去掉每行最后一列。"""
    if not os.path.isfile(path):
        return ABSENT
    if without_timing:
        return hashlib.sha256(lines_without_the_last_column(path)).hexdigest()
    return file_sha256(path)


def s2_items() -> list[tuple[str, str, str, bool]]:
    """要比的 18 件：(流, 文件名, 产物里记的名字, 去不去耗时列)，次序同产物。"""
    items = []
    for stream in S2_STREAMS:
        items += [(stream, name, name, False) for name in S2_WHOLE_FILES]
        items.append((stream, IMPLEMENTATION_VERDICTS_FILE, IMPLEMENTATION_VERDICTS_LABEL, True))
    return items


def compare(e170_dump: str, device_baseline: str) -> list[tuple[str, str, str, str, bool]]:
    """逐件比：交回 (流, 产物里记的名字, E170 那边的 sha256, 本装置那边的 sha256, 相同否)。两边都是 absent 也算不同。"""
    rows = []
    for stream, name, label, without_timing in s2_items():
        left = compared_digest(os.path.join(e170_dump, stream, name), without_timing)
        right = compared_digest(os.path.join(device_baseline, stream, name), without_timing)
        rows.append((stream, label, left, right, left == right and left != ABSENT))
    return rows


def write_product(output: str, e170_dump: str, device_baseline: str) -> int:
    """写产物；交回不同的件数。"""
    lines = ResultLines(output)
    lines.result(f"name=config segment=s1 product=s2 e170_dump={field(e170_dump)} device_baseline={field(device_baseline)}")
    rows = compare(e170_dump, device_baseline)
    for stream, label, left, right, same in rows:
        lines.result(f"name=s2 stream={stream} file={label} e170_sha256={left} device_sha256={right} equal={str(same).lower()}")
    differing = sum(1 for row in rows if not row[4])
    lines.result(f"name=verdict s2_differing_files={differing} s2_compared_files={len(rows)}")
    lines.finish()
    return differing


def main(argument_list: list[str]) -> int:
    if len(argument_list) != 3:
        print("用法：s2_compare.py <E170 导出目录> <本装置基线导出目录> <产物（新文件）>", file=sys.stderr)
        return EXIT_USAGE
    e170_dump, device_baseline, output = argument_list
    missing = [path for path in (e170_dump, device_baseline) if not os.path.isdir(path)]
    if missing:
        print(f"✗ 导出目录不在：{'、'.join(missing)}（退 78）", file=sys.stderr)
        print("     → 怎么办：先跑出第 1 步与第 2 步的导出再比", file=sys.stderr)
        return EXIT_RUN_CONDITION
    if os.path.exists(output):
        print(f"✗ 产物 {output} 已存在，不覆盖（退 2）", file=sys.stderr)
        print("  → 怎么办：换一个产物名（日期标签加 -2 这类），或者核过旧的那一份没用了再把它挪走；已有的产物一律不覆盖", file=sys.stderr)
        return EXIT_USAGE
    differing = write_product(output, e170_dump, device_baseline)
    print(f"S2 逐文件比：{len(s2_items())} 件，不同 {differing} 件；产物 {output}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
