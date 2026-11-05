#!/usr/bin/env python3
"""d137-gov-r2 攻方腿（opus-b）格 3：拿仓里的 scripts/agents/agent-watch.py 的 unit_owner、foreign_unit、verdict_counts 算给定输入的结果。
用法：python3 unit_owner_cases.py --repo <仓根> --work <新的空目录>
登记目录、状态目录都设到 --work 底下（DETACHED_RUN_STATE_DIRECTORY、AGENT_WATCH_STATE_ROOT），不读本机真登记。"""
import importlib.util
import os
import stat
import sys

sys.dont_write_bytecode = True


def main(repo, work):
    runs = os.path.join(work, "runs")
    os.makedirs(runs)
    os.environ["DETACHED_RUN_STATE_DIRECTORY"] = runs
    os.environ["AGENT_WATCH_STATE_ROOT"] = os.path.join(work, "state")
    os.environ.pop("AGENT_WATCH_BREAK", None)
    spec = importlib.util.spec_from_file_location("agent_watch_probe", f"{repo}/scripts/agents/agent-watch.py")
    watch = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(watch)

    def registry(label, owner=None):
        os.makedirs(os.path.join(runs, label))
        if owner is not None:
            with open(os.path.join(runs, label, "owner-session"), "w", encoding="utf-8") as handle:
                handle.write(owner + "\n")

    me, other = "S-me", "S-other"
    registry("own", me)
    registry("theirs", other)
    registry("empty-owner", "")          # 起它时调用方没设 CLAUDE_CODE_SESSION_ID（detached-run.sh 写空的）
    registry("legacy")                   # 改动之前起的：登记里没有 owner-session
    registry("unreadable", me)
    os.chmod(os.path.join(runs, "unreadable"), 0)
    sessions = {me}
    # 告警记录那一半：照 agent-watch.py core_usage_report 里那三行的写法（owner、watched、foreign_unit）逐条算
    cases = (("own", None, me), ("own", None, other), ("theirs", None, me), ("theirs", None, other),
             ("empty-owner", None, me), ("empty-owner", None, other), ("legacy", None, me), ("legacy", None, other),
             ("unreadable", None, me), ("unreadable", None, other), ("missing-registry", None, other),
             ("theirs", me, other), ("own", other, me), ("bad label", None, other))
    for label, recorded, writer in cases:
        owner = watch.unit_owner(label, recorded)
        watched = writer in sessions or (owner is not None and owner in sessions)
        foreign = watch.foreign_unit(owner, sessions)
        outcome = "skipped-silently" if not watched else ("logged-foreign" if foreign else "woken")
        print(f"name=record_half label={label!r} record_unit_session={recorded} record_session={writer} watched_session={me} "
              f"unit_owner={owner} watched={watched} foreign={foreign} outcome={outcome}")
    os.chmod(os.path.join(runs, "unreadable"), stat.S_IRWXU)
    # 判定行那一半：同名单元重起之后，watch 输出里旧一次的判红行按此刻登记里新一次的 owner-session 认
    registry("reused", other)            # 此刻登记：别的会话重起的那一次（旧一次挪成 reused@<秒>）
    os.makedirs(os.path.join(runs, "reused@1000"))
    with open(os.path.join(runs, "reused@1000", "owner-session"), "w", encoding="utf-8") as handle:
        handle.write(me + "\n")
    output = os.path.join(work, "watch-output.txt")
    with open(output, "w", encoding="utf-8") as handle:
        handle.write("  ✗ reused 在用 0.4 核（登记 31）：最近 600 秒平均（旧一次，S-me 起的）\n")
        handle.write("  ✗ reused 在用 0.3 核（登记 31）：最近 600 秒平均（旧一次，S-me 起的）\n")
        handle.write("  ✗ own 在用 0.2 核（登记 31）：最近 600 秒平均\n")
    green, red, red_lines, partial, foreign = watch.verdict_counts(output, sessions)
    print(f"name=verdict_half_reused_label watched_session={me} red_counted={red} foreign_not_counted={foreign} "
          f"registry_now={watch.unit_owner('reused')} old_run_owner={me}")
    green, red, red_lines, partial, foreign = watch.verdict_counts(output, set())
    print(f"name=verdict_half_no_session red_counted={red} foreign_not_counted={foreign}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 5 or sys.argv[1] != "--repo" or sys.argv[3] != "--work":
        print("用法：unit_owner_cases.py --repo <仓根> --work <新的空目录>", file=sys.stderr)
        sys.exit(2)
    if os.path.exists(sys.argv[4]) and os.listdir(sys.argv[4]):
        print(f"--work {sys.argv[4]} 不是空目录", file=sys.stderr)
        sys.exit(2)
    os.makedirs(sys.argv[4], exist_ok=True)
    sys.exit(main(sys.argv[2], sys.argv[4]))
