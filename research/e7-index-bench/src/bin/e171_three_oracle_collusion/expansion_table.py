"""E171 F4 扩表（登记 5.2「扩表」、第十节 F4、第五节 5.8 第三段）：步长 4 的挑法比步长 8 多出来的行，按挑法脚本的输出顺序接在 117 行后面编号。

挑法本身只在 `select_bad_implementations.py` 一处（登记 5.2，sha256 4032a0e0…）：这里起它两次（步长 8、步长 4），不另写一份挑法。
表是 `crates/mutations.tsv` 去掉 E171 装置自己的变异行（`bad_implementation_table.without_the_rows_of_the_device_itself`，与 A6、A7 同一个去法）。
扩表的行与步长 8 重叠的不重跑（登记 5.2「与步长 8 重叠 56 行不重跑」），序号从 118 起，刀号 3。行文 sha256 取这张表里那一行的：
登记 5.9 只钉了 117 行的行文，扩表的行没有登记过的行文可比，驱动照名字取行时与这里写下的比。

跑法（在本目录）：python3 expansion_table.py --mutations-table <crates/mutations.tsv> --output <新文件>
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile

from bad_implementation_table import SelectedRow, line_digest_prefix, read_mutation_rows, without_the_rows_of_the_device_itself

HERE = os.path.dirname(os.path.abspath(__file__))
SELECTION_SCRIPT = os.path.join(HERE, "select_bad_implementations.py")
STEP_OF_THE_REGISTERED_TABLE = 8
STEP_OF_THE_EXPANSION = 4
EXPANSION_SLICE_NUMBER = 3
FIRST_EXPANSION_ROW_NUMBER = 118  # 登记 5.9 的表编到 117
EXPECTED_EXPANSION_ROWS = 164  # 登记第四节第 7 条与 A6：步长 4 独有 164 行
DIGEST_OF_AN_AMBIGUOUS_NAME = "ambiguous"


def script_selection(table_path: str, step: int) -> list[tuple[str, int, str]]:
    """起挑法脚本，交回它的输出（文件、组内下标、变异名），保持它的顺序。"""
    output = subprocess.run([sys.executable, SELECTION_SCRIPT, table_path, str(step)], check=True, capture_output=True, text=True).stdout
    selection = []
    for line in output.splitlines():
        file, group_index, _, name = line.split("\t")
        selection.append((file, int(group_index), name))
    return selection


def expansion_rows(table_text: str, first_number: int = FIRST_EXPANSION_ROW_NUMBER) -> list[SelectedRow]:
    """步长 4 的输出里、名字不在步长 8 输出里的行，按步长 4 的输出顺序编号（从 `first_number` 起）。名字在表里不止一行时行文摘要写 `ambiguous`，驱动照名字取行时会判它找不到唯一一行、走补位。"""
    text_without_the_device_rows, _ = without_the_rows_of_the_device_itself(table_text)
    with tempfile.TemporaryDirectory(prefix="e171-expansion-") as scratch:
        table_path = os.path.join(scratch, "mutations.tsv")
        with open(table_path, "w", encoding="utf-8") as handle:
            handle.write(text_without_the_device_rows)
        names_of_the_registered_step = {name for _, _, name in script_selection(table_path, STEP_OF_THE_REGISTERED_TABLE)}
        expansion_step = script_selection(table_path, STEP_OF_THE_EXPANSION)
        lines_by_name: dict[str, list[str]] = {}
        for row in read_mutation_rows(table_path):
            lines_by_name.setdefault(row.name, []).append(row.raw_line)
    rows = []
    for file, group_index, name in expansion_step:
        if name in names_of_the_registered_step:
            continue
        raw_lines = lines_by_name.get(name, [])
        digest = line_digest_prefix(raw_lines[0]) if len(raw_lines) == 1 else DIGEST_OF_AN_AMBIGUOUS_NAME
        rows.append(SelectedRow(first_number + len(rows), EXPANSION_SLICE_NUMBER, file, group_index, name, digest))
    return rows


def selected_row_line(row: SelectedRow) -> str:
    """与 `bad-implementations-117.tsv` 同一个六列格式（`bad_implementation_table.read_selected_rows` 读得进）。"""
    return f"{row.number}\t{row.slice_number}\t{row.file}\t{row.group_index}\t{row.name}\t{row.line_digest_prefix}\n"


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 F4 扩表")
    parser.add_argument("--mutations-table", required=True)
    parser.add_argument("--output", required=True, help="新文件；已存在就拒绝")
    arguments = parser.parse_args(argument_list)
    if os.path.exists(arguments.output):
        print(f"已存在，不覆盖：{arguments.output}", file=sys.stderr)
        return 2
    with open(arguments.mutations_table, encoding="utf-8") as handle:
        rows = expansion_rows(handle.read())
    with open(arguments.output, "x", encoding="utf-8") as handle:
        handle.writelines(selected_row_line(row) for row in rows)
    print(f"expansion rows={len(rows)} expected={EXPECTED_EXPANSION_ROWS} first={rows[0].number if rows else 'none'} "
          f"last={rows[-1].number if rows else 'none'} ambiguous_names={sum(row.line_digest_prefix == DIGEST_OF_AN_AMBIGUOUS_NAME for row in rows)}")
    return 0 if len(rows) == EXPECTED_EXPANSION_ROWS else 3


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
