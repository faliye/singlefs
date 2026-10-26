#!/usr/bin/env python3
"""J1：换算按「旧前缀前面不接字母、数字、_、-、.」认边界，前面接 / 也换（./、$ROOT/、绝对路径里的同一段都换）。
旧前缀只有一段（顶层目录）时，别的目录下同名的那一段也被换：vendor/tools/run.sh 的引用被换成 vendor/tools2/run.sh。
样本：顶层 tools 真搬到 tools2（实据齐）；基准里另有 vendor/tools/run.sh 与 vendor/tools2/run.sh（两份不同的实现）；调用方把 vendor/tools 改成 vendor/tools2。
用法：probe_anchor.py --frozen <冻结树> --work <新的草稿目录>"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, load_library, row, short  # noqa: E402


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    lag = load_library(arguments.frozen)
    os.makedirs(arguments.work, exist_ok=True)
    for label, prefix_old, prefix_new, vendor_old, vendor_new in (("one_segment_prefix", "tools", "tools2", "vendor/tools", "vendor/tools2"),
                                                                  ("two_segment_prefix_control", "ops/tools", "ops/tools2", "vendor/ops/tools", "vendor/ops/tools2")):
        repo = Repo(os.path.join(arguments.work, label), arguments.frozen)
        repo.write(f"{prefix_old}/a.sh", "echo a\n")
        repo.write(f"{vendor_old}/run.sh", "echo 第三方的老实现\n")
        repo.write(f"{vendor_new}/run.sh", "echo 第三方的另一份实现，行为不同\n")
        repo.write(".claude/hooks/caller.sh", f"bash {vendor_old}/run.sh\nbash {prefix_old}/a.sh\n")
        repo.write(".claude/gate.d/path-moves.tsv", "# 空\n")
        repo.commit("基准")
        repo.git("mv", prefix_old, prefix_new)
        repo.write(".claude/hooks/caller.sh", f"bash {vendor_new}/run.sh\nbash {prefix_new}/a.sh\n")
        repo.register(row(prefix_old, prefix_new))
        count, problems, note = lag.path_move_report(repo.root, "HEAD")
        changed = repo.changed("HEAD")
        moved, _ = lag.moved_only_paths("HEAD", changed, cwd=repo.root)
        kept, _ = lag.drop_moved_lines("HEAD", [(".claude/hooks/caller.sh", f"bash {vendor_new}/run.sh", 1), (".claude/hooks/caller.sh", f"bash {prefix_new}/a.sh", 2)], cwd=repo.root)
        print(f"name={label} row={prefix_old}->{prefix_new} table_problems={len(problems)} caller_file_dropped={'.claude/hooks/caller.sh' in moved} "
              f"lines_kept_as_new={short(repr(x[1]) for x in kept)}")


main()
