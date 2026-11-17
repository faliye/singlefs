"""E201（三oracle合谋今树整表） 研究一侧借用 E171（三oracle合谋测试） 研究一侧模块的那一处：只把那个目录放进 import 路径、交回几条路径，不改、不拷那里的任何文件
（跑前登记 research/prompts/e201-preregistration.md 第五节 5.5「按路径 import 或原样起，不改、不拷」）。

本目录的模块名一律不与那个目录里的重名（比对器按名字 `import row_driver`、`import segment_one_comparator`，同名会串），所以本目录的逐行驱动叫
`current_tree_row_driver.py`。导入这一份之后才 import 那边的模块。
"""

from __future__ import annotations

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
# 变异驱动把本目录拷进草稿目录跑单测：那份拷贝在仓外，仓根由它经这个变量交过来（只有 mutation_driver.py 设）
COPY_REPOSITORY_ROOT_VARIABLE = "E201_REPOSITORY_ROOT_FOR_A_COPY"
REPOSITORY_ROOT = os.environ.get(COPY_REPOSITORY_ROOT_VARIABLE) or os.path.abspath(os.path.join(HERE, "..", "..", ".."))
E171_DIRECTORY = os.path.join(REPOSITORY_ROOT, "scripts", "experiment-apparatus", "e171_three_oracle_collusion_after_the_oracle_fixes")
E171_RELATIVE = os.path.relpath(E171_DIRECTORY, REPOSITORY_ROOT)
# 第三节点名、读法 1 要计划那一步记 sha256 的研究一侧各文件（相对仓根）；本目录自己的源文件另由 full_table_run.own_source_files 收
E171_RESEARCH_SIDE_FILES = tuple(os.path.join(E171_RELATIVE, name) for name in (
    "segment_two_comparator.py", "segment_one_comparator.py", "collusion_matrix.py", "frozen_o3.py", "list_check.py",
    "bad_implementation_table.py", "run_directory.py", "result_lines.py", "device_timing.py", "o3_bad_image_judge.py", "row_driver.py",
    "bad-implementations-step-8.tsv", "bad-implementations-step-4-only.tsv"))
NAME_LISTS = (os.path.join(E171_RELATIVE, "bad-implementations-step-8.tsv"), os.path.join(E171_RELATIVE, "bad-implementations-step-4-only.tsv"))
DEVICE_SOURCE = "crates/singlefs-verification/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"

if E171_DIRECTORY not in sys.path:
    sys.path.insert(1, E171_DIRECTORY)

# 登记第十二节修订 8（E201 自己的第 8 条）：O3 一侧用 journal 记录头 496 字节的执行器目录与 E171 第 3 次重跑的冻结参照（不用 E171_DIRECTORY 里 frozen_o3.py 写的 487 目录与 r2 冻结参照）。
# 只在 E201 自己这一头改：E171_DIRECTORY 里的 frozen_o3.py 一个字节不动，这里把它在本进程里的三个常量换成下面三个（别的模块按 `frozen_o3.<常量>` 在调用时取，不拷）。
O3_EXECUTOR_DIRECTORY_RELATIVE = "scripts/experiment-apparatus/e170_second_run_o3_spec_executor_journal_header_496"
O3_FREEZE_REFERENCE_RELATIVE = "research/results/e170-o3-spec-executor-journal-header-496-2026-11-09-freeze-executor-e171-r3.out"
O3_FILES_OUTSIDE_THE_FREEZE = ("comparator", "mutation_driver")


def point_frozen_o3_at_the_496_directory(module) -> None:
    module.FROZEN_DIRECTORY_RELATIVE = O3_EXECUTOR_DIRECTORY_RELATIVE
    module.FREEZE_REFERENCE_RELATIVE = O3_FREEZE_REFERENCE_RELATIVE
    module.FILES_OUTSIDE_THE_FREEZE = O3_FILES_OUTSIDE_THE_FREEZE


import frozen_o3  # noqa: E402  （E171_DIRECTORY 进了 import 路径之后才取得到）

point_frozen_o3_at_the_496_directory(frozen_o3)
