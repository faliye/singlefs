#!/usr/bin/env python3
"""J1：生效窗口随「gate 基准」的几种取法开合的时机。造一个带上游的样本仓，走一遍：基准推出去 → 未提交的搬迁 → 提交没推 → 推出去 → GATE_BASE 重新指回搬迁之前。
每一站打一行 name=…：gate 基准是哪个（short 哈希对应的站名）、此刻生效的行、改动范围里剔了哪些只差搬迁的、留下哪些、HEAD 作基准时同样的三样。
用法：probe_window.py --frozen <冻结树> --work <新的草稿目录>"""
import argparse
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, load_library, row, short  # noqa: E402


def station(lag, repo, label, names, base_override=None):
    out = []
    for kind in ("gate", "head"):
        if kind == "head":
            base = "HEAD"
        elif base_override:
            base = base_override
        else:
            shown = subprocess.run(["bash", "-c", 'source scripts/gate/changed-paths.sh && gate_diff_base gate'], cwd=repo.root, capture_output=True, text=True, env=repo.env)
            base = shown.stdout.strip()
        effective, problem = lag.effective_moves(base, repo.root)
        changed = repo.changed(base)
        moved, _ = lag.moved_only_paths(base, changed, cwd=repo.root)
        text, _ = lag.baseline_text(base, "exempt.tsv", cwd=repo.root)
        out.append(f"{kind}_base={names.get(base, base[:7])} effective={short(f'{m.old}->{m.new}' for m in effective)} dropped={short(sorted(moved))} "
                   f"kept={short(p for p in changed if p not in moved)} exempt_baseline_line={text.splitlines()[0] if text else None!r}")
    print(f"name={label} " + " | ".join(out))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    lag = load_library(arguments.frozen)
    os.makedirs(arguments.work, exist_ok=True)
    repo = Repo(os.path.join(arguments.work, "life"), arguments.frozen, upstream=True)
    repo.write("tools/old/a.sh", "echo a # tools/old/a.sh\n")
    repo.write("tools/old/b.sh", "echo b\n")
    repo.write(".claude/hooks/caller.sh", "bash tools/old/a.sh\n")
    repo.write("exempt.tsv", "tools/old/a.sh\t豁免\n")
    repo.write(".claude/gate.d/path-moves.tsv", "# 空\n")
    s0 = repo.commit("S0 基准", push=True)
    names = {s0: "S0"}
    station(lag, repo, "S0_clean", names)
    # 站 1：搬迁做完、登记写了，没提交
    repo.git("mv", "tools/old", "tools/new")
    repo.write("tools/new/a.sh", "echo a # tools/new/a.sh\n")
    repo.write(".claude/hooks/caller.sh", "bash tools/new/a.sh\n")
    repo.write("exempt.tsv", "tools/new/a.sh\t豁免\n")
    repo.register(row("tools/old", "tools/new"))
    station(lag, repo, "S1_uncommitted_move", names)
    # 站 2：提交，没推。gate 基准是上游的 merge-base（S0），head 基准是搬迁之后的 HEAD
    s2 = repo.commit("S2 搬迁")
    names[s2] = "S2"
    station(lag, repo, "S2_committed_unpushed", names)
    # 站 2b：提交之后又改了两处：调用方多加一行真改动；另一个文件补换了一处旧前缀（登记行此刻在 gate 基准下仍生效）
    repo.write(".claude/hooks/caller.sh", "bash tools/new/a.sh\necho 新加的一行\n")
    repo.write("docs.md", "见 tools/new/b.sh\n")
    station(lag, repo, "S2b_followup_edits", names)
    repo.commit("S2b 补")
    # 站 3：推出去。gate 基准追上 HEAD，登记行对门禁不再起作用
    s3 = repo.git("rev-parse", "HEAD").stdout.strip()
    repo.git("push", "-q", "origin", "master")
    names[s3] = "S3"
    station(lag, repo, "S3_pushed", names)
    # 站 3b：推出去之后又补换一处旧前缀的文件：登记行已不生效，这一处照算改动
    repo.write("late.md", "见 tools/old/b.sh\n")
    names[repo.commit("S3b 带旧前缀的新文件", push=True)] = "S3b"
    repo.write("late.md", "见 tools/new/b.sh\n")
    station(lag, repo, "S3b_late_path_fix", names)
    # 站 4：GATE_BASE 指回 S0（窗口被人为重新打开）
    station(lag, repo, "S4_GATE_BASE_S0", names, base_override=s0)
    # 站 5：另一种分法：先登记没搬（旧前缀还在磁盘上）
    other = Repo(os.path.join(arguments.work, "register_first"), arguments.frozen, upstream=True)
    other.write("tools/old/a.sh", "echo a\n")
    other.write(".claude/gate.d/path-moves.tsv", "# 空\n")
    base = other.commit("S0", push=True)
    other.register(row("tools/old", "tools/new"))
    count, problems, note = lag.path_move_report(other.root, base)
    print(f"name=S5_register_first_no_move problems={len(problems)} first={problems[0][0] if problems else None!r}")


main()
