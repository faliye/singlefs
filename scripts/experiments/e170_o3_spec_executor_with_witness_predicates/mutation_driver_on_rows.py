"""E170 第 2 次重跑第二段：X21（W 恒假）在第 242、269 行、X27（T 恒真）在第 270 行各跑一趟（`research/prompts/e170-r2-prereg.md` 第六节 Q2-pc、Q3-pc，第九节）。

在草稿目录里拷一份 v2 的冻结文件，照变异表 `scripts/mutations/e170_o3_spec_executor_with_witness_predicates.tsv`（每行：名字、原文、替换文）
把名字以 `--mutation` 起头的那一条施加上去：原文在拷贝的 .py 文件里合起来恰好命中一次才施加，否则退 3、不跑。施加之后在拷贝里跑
`executor_main.py`，产物写到 `--output`。驱动只打命中的文件名与次数，不打原文与替换文（第二段写参照谓词的人不读 v2 的 W、T 判法，5.4）。
v2 目录本身一个字节不动；拷贝跑完删掉。文件名以 mutation_driver 起头，不进 v2 冻结的 sha256。

跑法：python3 mutation_driver_on_rows.py --mutation X21 --table <变异表> --scratch <草稿目录> --dumps <导出目录> --batches 1,2 --workers <N> --output <新文件>
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True

EXECUTOR_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
FILES_NOT_COPIED = ("comparator", "mutation_driver", "witness_reference", "run_")


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="v2 的一条变异在一份导出上跑一趟")
    parser.add_argument("--mutation", required=True)
    parser.add_argument("--table", required=True)
    parser.add_argument("--scratch", required=True)
    parser.add_argument("--dumps", required=True)
    parser.add_argument("--batches", default="1,2")
    parser.add_argument("--workers", type=int, required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    with open(arguments.table, encoding="utf-8") as handle:
        rows = [line.rstrip("\n").split("\t") for line in handle if line.strip()]
    chosen = [row for row in rows if row[0] == arguments.mutation or row[0].startswith(arguments.mutation + "_")]
    if len(chosen) != 1:
        print(f"变异表里名字以 {arguments.mutation} 起头的行有 {len(chosen)} 条", file=sys.stderr)
        return 3
    name, original, replacement = chosen[0][0], chosen[0][1].replace("\\n", "\n"), chosen[0][2].replace("\\n", "\n")
    copy_directory = os.path.join(arguments.scratch, f"mutation-copy-{arguments.mutation}")
    if os.path.exists(copy_directory):
        print(f"拷贝目录 {copy_directory} 已存在", file=sys.stderr)
        return 3
    os.makedirs(copy_directory)
    try:
        for file_name in sorted(os.listdir(EXECUTOR_DIRECTORY)):
            source = os.path.join(EXECUTOR_DIRECTORY, file_name)
            if os.path.isfile(source) and not file_name.startswith(FILES_NOT_COPIED) and not file_name.endswith(".pyc"):
                shutil.copy2(source, os.path.join(copy_directory, file_name))
        hits = {}
        for file_name in sorted(os.listdir(copy_directory)):
            if file_name.endswith(".py"):
                with open(os.path.join(copy_directory, file_name), encoding="utf-8") as handle:
                    count = handle.read().count(original)
                if count:
                    hits[file_name] = count
        print(f"name=mutation_anchor mutation={name} hits={','.join(f'{file}:{count}' for file, count in hits.items()) or 'zero'}")
        if sum(hits.values()) != 1:
            return 3
        target = os.path.join(copy_directory, next(iter(hits)))
        with open(target, encoding="utf-8") as handle:
            text = handle.read()
        with open(target, "w", encoding="utf-8") as handle:
            handle.write(text.replace(original, replacement, 1))
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        completed = subprocess.run([sys.executable, "executor_main.py", "--dumps", arguments.dumps, "--batches", arguments.batches,
                                    "--workers", str(arguments.workers), "--output", os.path.abspath(arguments.output)],
                                   cwd=copy_directory, env=environment)
        print(f"name=mutation_run mutation={name} executor_exit={completed.returncode} output={arguments.output}")
        return completed.returncode
    finally:
        shutil.rmtree(copy_directory)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
