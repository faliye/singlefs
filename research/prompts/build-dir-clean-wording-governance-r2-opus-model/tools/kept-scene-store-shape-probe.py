#!/usr/bin/env python3
"""完工清理闸「任务现场」那一判在两种样本库上各喂一次 Stop：
  selftest-shape：尝试行的 session 列写提交它的会话（completion_cleanup_guard_selftest.py 的 service_task 这样写）；
  store-shape：尝试行的 session 列是空串（task-server/task_server_scheduler_starting.py 起尝试时记录里写 "session": ""，
               task-server/task_server_store.py 的 write_row 照记录的 session 写这一列）。
两份库里任务行一样：本会话、已有结局、environment 设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1，尝试记的现场目录在盘上。
样本库是这个探针自己建的一份 sqlite（表与列照 Store.initialize），经 COMPLETION_CLEANUP_TASK_SERVER_DATABASE 交给钩子，不碰服务自己的库。
另在每一份样本库上数两种查法各取到几行尝试：闸现用的「attempts WHERE session = ?」与按任务号接任务表的「JOIN tasks」。
用法：kept-scene-store-shape-probe.py --repo <仓根> --work <输出目录>
输出 name= 行；退 0 是两臂都跑完（不论拦没拦），退 2 用法错，退 3 钩子没在时限里退出。
"""
import argparse
import json
import os
import signal
import sqlite3
import subprocess
import sys
import time
from datetime import datetime, timezone

SESSION = "s-kept-scene-store-shape"
TASK_NUMBER = 7
KEEP_VARIABLE = "SINGLEFS_KEEP_TEST_TEMPORARY_FILES"
HOOK_TIMEOUT_SECONDS = 600
SAMPLE_DATABASE_NAME = "sample-service-store.db"
# 表与列照 task-server/task_server_store.py 的 Store.initialize（SIGNED_TABLES 那两张）
SIGNED_TABLE_TEXT = ("CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT, task_id INTEGER NOT NULL DEFAULT 0, state TEXT NOT NULL, "
                     "session TEXT NOT NULL, command_key TEXT NOT NULL, record TEXT NOT NULL, signature TEXT NOT NULL)")


def iso(seconds):
    return datetime.fromtimestamp(seconds, timezone.utc).isoformat().replace("+00:00", "Z")


def build_world(root, attempt_session):
    """一臂一个世界：工作根、崩溃放量家目录、会话临时目录根、项目目录与主会话记录、数据根、样本库与现场目录。交 (环境, 钩子输入, 现场目录, 库路径)。"""
    paths = {name: os.path.join(root, name) for name in ("work", "crash-home", "session-temporary-root", "data-root", "prune-control")}
    project = os.path.join(root, "projects", "project-directory")
    for directory in (paths["work"], os.path.join(paths["crash-home"], "runs"), paths["session-temporary-root"], paths["data-root"], project):
        os.makedirs(directory, exist_ok=True)
    transcript = os.path.join(project, SESSION + ".jsonl")
    with open(transcript, "w", encoding="utf-8") as handle:
        handle.write(json.dumps({"type": "user", "timestamp": iso(time.time() - 100), "message": {"role": "user", "content": "开工"}}, ensure_ascii=False) + "\n")
    database = os.path.join(root, SAMPLE_DATABASE_NAME)
    scene = os.path.join(root, "task-server-scenes", f"task-model-{TASK_NUMBER}-1")
    os.makedirs(scene, exist_ok=True)
    with open(os.path.join(scene, "panic.txt"), "w", encoding="utf-8") as handle:
        handle.write("判红那一趟留下的临时文件\n")
    connection = sqlite3.connect(database)
    try:
        for table in ("tasks", "attempts"):
            connection.execute(SIGNED_TABLE_TEXT.format(table=table))
        task = {"id": TASK_NUMBER, "state": "finished", "session": SESSION, "environment": {KEEP_VARIABLE: "1"}, "outcome": "command-failed"}
        connection.execute("INSERT INTO tasks(id, task_id, state, session, command_key, record, signature) VALUES(?, 0, 'finished', ?, '', ?, '')",
                           (TASK_NUMBER, SESSION, json.dumps(task)))
        attempt = {"task_id": TASK_NUMBER, "number": 1, "state": "finished", "session": attempt_session, "scene_directory": scene}
        connection.execute("INSERT INTO attempts(task_id, state, session, command_key, record, signature) VALUES(?, 'finished', ?, '', ?, '')",
                           (TASK_NUMBER, attempt_session, json.dumps(attempt)))
        connection.commit()
    finally:
        connection.close()
    environment = dict(os.environ)
    environment.pop("COMPLETION_CLEANUP_BREAK", None)
    environment.update({"SINGLEFS_WORK_ROOT": paths["work"], "SINGLEFS_CRASH_AMPLIFICATION_HOME": paths["crash-home"],
                        "COMPLETION_CLEANUP_SCRATCHPAD_ROOT": paths["session-temporary-root"], "AGENT_HOOK_DETECTIONS": os.path.join(root, "detections.jsonl"),
                        "COMPLETION_CLEANUP_TASK_SERVER_DATABASE": database, "SINGLEFS_DATA_ROOT": paths["data-root"],
                        "PRUNE_SERVER_CONTROL_SELFTEST_ROOT": paths["prune-control"]})
    hook_input = {"hook_event_name": "Stop", "session_id": SESSION, "transcript_path": transcript, "stop_hook_active": False}
    return environment, hook_input, scene, database


def run_hook(hook, environment, hook_input, log_prefix):
    """钩子放进它自己的进程组起；超时只对这一组发信号。交 (退出码, 标准输出)。"""
    process = subprocess.Popen(["bash", hook], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=environment, start_new_session=True)
    try:
        out, err = process.communicate(json.dumps(hook_input).encode(), timeout=HOOK_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()
        print(f"name=kept_scene_store_shape_error hook_timeout_seconds={HOOK_TIMEOUT_SECONDS}")
        sys.exit(3)
    for suffix, data in (("stdout", out), ("stderr", err)):
        with open(f"{log_prefix}.{suffix}", "wb") as handle:
            handle.write(data)
    return process.returncode, out.decode("utf-8", "replace")


def query_rows(database):
    connection = sqlite3.connect(f"file:{database}?mode=ro", uri=True)
    try:
        by_column = connection.execute("SELECT COUNT(*) FROM attempts WHERE session = ?", (SESSION,)).fetchone()[0]
        joined = connection.execute("SELECT COUNT(*) FROM attempts JOIN tasks ON attempts.task_id = tasks.id WHERE tasks.session = ?", (SESSION,)).fetchone()[0]
    finally:
        connection.close()
    return by_column, joined


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    hook = os.path.join(arguments.repo, ".claude", "hooks", "completion-cleanup-guard.sh")
    if not os.path.isfile(hook):
        print(f"name=kept_scene_store_shape_error hook_missing={hook}")
        return 2
    verdict = {}
    for arm, attempt_session in (("selftest-shape", SESSION), ("store-shape", "")):
        root = os.path.join(arguments.work, f"kept-scene-{arm}")
        os.makedirs(root, exist_ok=False)
        environment, hook_input, scene, database = build_world(root, attempt_session)
        code, out = run_hook(hook, environment, hook_input, os.path.join(arguments.work, f"kept-scene-{arm}-hook"))
        blocked = '"decision": "block"' in out
        listed = scene in out
        named = f"task-server/task clean {TASK_NUMBER}" in out
        by_column, joined = query_rows(database)
        verdict[arm] = blocked and listed and named
        print(f"name=kept_scene_store_shape arm={arm} attempt_session_column={attempt_session or 'empty'} hook_exit={code} "
              f"blocked={str(blocked).lower()} scene_listed={str(listed).lower()} clean_named={str(named).lower()} "
              f"guard_query_rows={by_column} joined_query_rows={joined}")
    print(f"name=kept_scene_store_shape_verdict selftest_shape_listed={str(verdict['selftest-shape']).lower()} "
          f"store_shape_listed={str(verdict['store-shape']).lower()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
