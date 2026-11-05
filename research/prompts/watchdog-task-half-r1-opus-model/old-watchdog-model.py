#!/usr/bin/env python3
"""watchdog-task-half-r1 攻方腿的模型：改前的看门狗（批 49 补丁之前的 agent-watch.py）对几种任务服务状态报什么，改后的那一份还有没有读服务的地方。

用法：python3 old-watchdog-model.py --before <改前的 agent-watch.py> --after <改后的 agent-watch.py> --anchor <被判仓里 scripts/agents/agent-watch.py 的路径> --work <输出目录>
  两份源码都当成 --anchor 那个路径载入（module.__file__ 取它）：agent-watch.py 按自己所在的路径找 .claude/scripts、.claude/hooks 与 scripts/agents 下的库，
  改前改后两份共用被判仓里的那几份库（批 49 没改它们）。只调函数，不起看门狗、不连真服务：改前那一份读服务的地方经它自己的
  AGENT_WATCH_TASK_SERVER_DIRECTORY 换成 --work 下假的 task-server 目录，假目录用改前那一份自己的 write_fake_task_server 写。
输出：每格一行 name=<格> before=<改前报的告警名，逗号分隔，没有写 无> want=<应当报的> verdict=<pass|fail>；
  末行 name=after-readers 写改后那一份里读服务的函数与写法还剩几处（应当是 0）。全过退 0，有一格不合退 1，用法错退 2。
"""
import argparse
import json
import os
import re
import sys
import time
import types

SESSION = "model"
NOW = time.time()


def load(source_path, module_name, anchor):
    """把 source_path 的源码当成 anchor 那个文件载入（__name__ 不是 __main__，不起 preflight、不跑 main）。"""
    module = types.ModuleType(module_name)
    module.__file__ = anchor
    with open(source_path, encoding="utf-8") as handle:
        source = handle.read()
    sys.modules[module_name] = module
    exec(compile(source, source_path, "exec"), module.__dict__)
    return module, source


def listed(task_id, state, outcome="", exit_code=None):
    """task status --json 会话回话里的一项（形态照改前自证 selftest_task_health 的 listed）。"""
    return {"id": task_id, "state": state, "outcome": outcome, "exit_code": exit_code, "unclaimed": state == "finished", "valid": True,
            "command": "bash overrun-probe.sh", "estimated_seconds": None, "phase": "", "waiting_for": ""}


def queued(task_id, minutes, waiting):
    return {"task": {"id": task_id, "state": "queued", "session": SESSION, "submitted_at": round(NOW - minutes * 60, 3), "attempt_count": 0,
                     "outcome_counts": {}, "command": ["bash", "overrun-probe.sh"]}, "attempts": [], "waiting_for": waiting, "estimated_seconds": None}


def running(task_id, elapsed, estimated, quiet, attempt_count=1, outcome_counts=None):
    return {"task": {"id": task_id, "state": "running", "session": SESSION, "submitted_at": round(NOW - elapsed - 60, 3), "attempt_count": attempt_count,
                     "outcome_counts": outcome_counts or {}, "command": ["bash", "overrun-probe.sh"]},
            "attempts": [{"number": attempt_count, "state": "running", "started_at": round(NOW - elapsed, 3),
                          "progress": {"elapsed_seconds": elapsed, "estimated_seconds": estimated, "log_quiet_seconds": quiet}}]}


def alert_entry(kind, task_id, line, next_step):
    return {"time": round(NOW - 30, 3), "kind": kind, "task": task_id, "line": line, "next_step": next_step}


# (格名, 全服务在跑几条, 会话回话里的任务, 逐条回话, 服务告警记录, control check 退出码, control check 的标准错误, 应当报的告警名)
CELLS = [
    ("overrun-log-growing", 1, [listed(9001, "running")], {9001: running(9001, 1500, 5, 3)}, [], 0, "", {"跑得远超预计"}),
    ("queue-memory-nobody-running", 0, [listed(9002, "queued")],
     {9002: queued(9002, 30, "第二台：此刻用不了（第二台在重启）；本机：内存：要记 30.0 GiB，此刻空闲 40.0 GiB、slice 减去余量放得下 20.0 GiB")}, [], 0, "", {"排队过久"}),
    ("queue-peer-down-other-sessions-running", 3, [listed(9003, "queued")],
     {9003: queued(9003, 30, "第二台此刻用不了：第二台在重启")}, [], 0, "", {"排队过久"}),
    ("queue-memory-other-sessions-running", 2, [listed(9004, "queued")],
     {9004: queued(9004, 30, "本机：内存：要记 30.0 GiB，此刻空闲 4.0 GiB、slice 减去余量放得下 2.0 GiB")}, [], 0, "", set()),
    ("outcome-unclaimed-stalled", 0, [listed(9005, "finished", "stalled", 233)], {}, [], 0, "", {"异常结局"}),
    ("retry-squeezed-twice", 1, [listed(9006, "running")], {9006: running(9006, 60, None, 5, 3, {"squeezed-out": 2})}, [], 0, "", {"反复重试"}),
    ("card-fault-retried-on-another-card", 1, [listed(9007, "running")], {9007: running(9007, 60, None, 5, 2, {"card-fault": 1})},
     [alert_entry("card-fault", 9007, "清场判出卡 127.0.0.1:0 坏（任务 9007 第 1 次尝试）；坏的那几张停用 60 分钟，这一条换别的卡重排",
                  "主 agent 当场弹窗请用户重置那张卡")], 0, "", {"服务告警"}),
    ("cleanup-pending-running", 1, [listed(9008, "running")], {9008: running(9008, 200, None, 100)},
     [alert_entry("cleanup-pending", 9008, "任务 9008 清理尚未确认，保留资源占用：单元里还有进程", "查那几个进程")], 0, "", {"服务告警"}),
    ("shard-advance-failed", 1, [listed(9009, "running")], {9009: running(9009, 200, None, 100)},
     [alert_entry("shard-advance-failed", 9009, "分片的任务 9009 推下一步时出错：OSError: 模型", "看 journalctl")], 0, "", {"服务告警"}),
    ("peer-unavailable-without-task", 0, [], {}, [alert_entry("peer-unavailable", None, "第二台此刻用不了：模型", "修第二台")], 0, "", set()),
    ("service-reachable-but-unhealthy", 0, [], {}, [], 78,
     "  ✗ task-server 回话了但不健康（HTTP 200：任务 9010 排在最前、此刻放得下，却 700 秒没起）", {"任务服务不通"}),
]


def run_cell(before, work, cell):
    """一格：写假的 task-server 目录与状态目录，改前那一份的 task_server_report 只看一眼（record=False，不写状态目录），交它报的告警名。"""
    name, running_count, tasks, details, alerts, check_exit, check_error, _ = cell
    root = os.path.join(work, "cells", name)
    state_directory = os.path.join(root, "state")
    os.makedirs(state_directory, exist_ok=True)
    if alerts:
        with open(os.path.join(state_directory, "alerts.jsonl"), "w", encoding="utf-8") as handle:
            for entry in alerts:
                handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    check_output = "" if check_exit else f"  ✓ task-server 通（模型）：库 {state_directory}/fake-database；在排 1 个、在跑 {running_count} 个"
    directory = os.path.join(root, "task-server")
    before.write_fake_task_server(directory, check_exit=check_exit, check_output=check_output, check_error=check_error,
                                  status_reply={"session": SESSION, "tasks": tasks}, task_details=details)
    os.environ[before.TASK_SERVER_DIRECTORY_VARIABLE] = directory
    os.environ[before.STATE_ROOT_VARIABLE] = os.path.join(root, "watch-state")
    thresholds = argparse.Namespace(task_queue_minutes=10, task_overrun_factor=3, task_retry_count=3, ack=[])
    lines, found = before.task_server_report(thresholds, record=False)
    with open(os.path.join(root, "report.txt"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
        for subject, alert_name, explanation, next_step in found:
            handle.write(f"⚠️ {alert_name}（{subject}）：{explanation}\n   → {next_step}\n")
    return sorted({alert_name for _, alert_name, _, _ in found})


AFTER_READER_NAMES = ("task_server_report", "task_health_report", "task_service_alerts", "build_directory_lines", "run_task_server_script", "write_fake_task_server")
AFTER_READER_PATTERN = re.compile(r'run_task_server_script|TASK_SERVER_ALERTS_NAME|alerts\.jsonl|"--json"|task_server_directory|"control", \["check"\]')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--before", required=True)
    parser.add_argument("--after", required=True)
    parser.add_argument("--anchor", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    if os.environ.get("AGENT_WATCH_BREAK"):
        print("  ✗ 环境里设着 AGENT_WATCH_BREAK：模型要的是没弄坏的两份", file=sys.stderr)
        return 2
    os.makedirs(arguments.work, exist_ok=True)
    before, _ = load(arguments.before, "agent_watch_before", arguments.anchor)
    failed = 0
    for cell in CELLS:
        got = run_cell(before, arguments.work, cell)
        want = sorted(cell[-1])
        verdict = "pass" if got == want else "fail"
        failed += verdict == "fail"
        print(f"name={cell[0]} before={','.join(got) or '无'} want={','.join(want) or '无'} verdict={verdict}", flush=True)
    after, after_source = load(arguments.after, "agent_watch_after", arguments.anchor)
    kept_names = [name for name in AFTER_READER_NAMES if hasattr(after, name)]
    kept_lines = [number for number, line in enumerate(after_source.splitlines(), 1) if AFTER_READER_PATTERN.search(line)]
    verdict = "pass" if not kept_names and not kept_lines else "fail"
    failed += verdict == "fail"
    print(f"name=after-readers functions={','.join(kept_names) or '无'} lines={','.join(map(str, kept_lines)) or '无'} want=无 verdict={verdict}", flush=True)
    print(f"name=done cells={len(CELLS) + 1} failed={failed}", flush=True)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
