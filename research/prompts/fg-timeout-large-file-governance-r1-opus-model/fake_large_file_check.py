#!/usr/bin/env python3
"""假判法：探针 probe.py 把它拷成假判法树里的 scripts/gate/large-file-check.py，钩子 ㉒ 照起它。
行为由环境变量 PROBE_FAKE_MODE 选（钩子起判法时环境原样带过来）；每起一次往 PROBE_FAKE_CALLS 指的文件追加一行参数。
  empty            打一行没有超限项的 JSON、退 0
  raise            抛 Python 异常（退 1、标准输出是空的）
  json-then-noise  先打 JSON、再打一行别的，退 0
  noise-then-big   先打一行别的、再打一行有超限项的 JSON，退 1
  exit-three       打带 error 的 JSON、退 3
  keys-missing     打一行 JSON，too_big 里那一项缺 path、size、where，退 1
  slow-with-child  起一个 sleep 子进程（与它同组，模仿判法起的 git），自己睡 60 秒再打 JSON
"""
import json
import os
import subprocess
import sys
import time

mode = os.environ.get("PROBE_FAKE_MODE", "empty")
calls = os.environ.get("PROBE_FAKE_CALLS")
if calls:
    with open(calls, "a", encoding="utf-8") as handle:
        handle.write(" ".join(sys.argv[1:]) + "\n")
guard_mode = sys.argv[sys.argv.index("--guard") + 1] if "--guard" in sys.argv else "commit-index"
result = {"mode": guard_mode, "limit": 104857600, "limit_text": "104857600 字节（100 MiB）", "checked": 1,
          "too_big": [], "unreadable": [], "not_run": "", "error": ""}
if mode == "raise":
    raise RuntimeError("假判法按 PROBE_FAKE_MODE=raise 抛的异常")
if mode == "json-then-noise":
    print(json.dumps(result))
    print("判法多打的一行")
    sys.exit(0)
if mode == "noise-then-big":
    print("判法多打的一行")
    result["too_big"] = [{"path": "big.bin", "size": 104857601, "where": "暂存区", "commit": "", "title": ""}]
    print(json.dumps(result))
    sys.exit(1)
if mode == "exit-three":
    result["error"] = "假判法按 PROBE_FAKE_MODE=exit-three 报的 git 没跑成"
    print(json.dumps(result))
    sys.exit(3)
if mode == "keys-missing":
    result["too_big"] = [{"name": "big.bin"}]
    print(json.dumps(result))
    sys.exit(1)
if mode == "slow-with-child":
    subprocess.Popen(["sleep", "200"])
    time.sleep(60)
print(json.dumps(result))
sys.exit(0)
