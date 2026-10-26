#!/usr/bin/env python3
"""W1 件 2 的探针：「先去掉以 run-with-memory-cap: 起头的行再读」对两处判法读到的行各做了什么。
用法：strip_demo.py <仓根>
读真的 scripts/mutation/broad-mutation.py 的 escalation_evidence_problems（升档证据要日志里有包装打的「撞了内存上限 …」一句）
与真的 scripts/gate/verus-proof-check.py 的 wrapper_own_lines（收包装自己的 ✗ 与 → 行）。行的样子照 run-with-memory-cap.sh 里的 echo 与 say 原文。"""
import importlib.util
import os
import sys

root = os.path.abspath(sys.argv[1])
sys.dont_write_bytecode = True


def load(name, relative):
    spec = importlib.util.spec_from_file_location(name, os.path.join(root, relative))
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


broad = load("broad_mutation_demo", "scripts/mutation/broad-mutation.py")
verus = load("verus_proof_check_demo", "scripts/gate/verus-proof-check.py")


def strip_prefixed(text):
    return "\n".join(line for line in text.split("\n") if not line.startswith("run-with-memory-cap:"))


queue_line = "run-with-memory-cap: 排队（已等 0 秒，至多 3600 秒）：slice 已占 36.4 GiB，这一条要 4.0 GiB（峰值表里没有这条），总上限 40.1 GiB；账上在跑 3 条"
hint_line = "  → 第二台此刻放得下（可用 43.0 GiB、要 8.0 GiB）：改用 bash scripts/hosts/multi-host-run.sh --peer --memory-cap 64M --label moved-from-queue-x -- cargo test"
lookup_failed = "run-with-memory-cap: 没查到别的机器：host-resources.py 退 3：ssh: connect to host 第二台 port 22: Connection refused"
hit_64 = "run-with-memory-cap: 撞了内存上限 64M（scope singlefs-memory-cap-1-2 的 Result=oom-kill，命令退出码 137）"
hit_256 = "run-with-memory-cap: 撞了内存上限 256M（scope singlefs-memory-cap-1-3 的 Result=oom-kill，命令退出码 137）"
first_log = "\n".join(["cargo mutants 的输出一行", queue_line, hint_line, lookup_failed, hit_64, "BROAD_MUTATION_ESCALATION_EVIDENCE key=k sequence=1 step=s attempt=first limit=64M exit=250"])
escalated_log = first_log.replace("64M", "256M").replace(hit_64, hit_256).replace("attempt=first", "attempt=escalated")
baseline, first_amount, escalated_amount = 20.0 * 1024 ** 2, 64 * 1024 ** 2, 256 * 1024 ** 2   # 基线峰值 20M：256M 不低于 10 倍


def problems(first, second):
    return broad.escalation_evidence_problems(250, 250, baseline, first_amount, escalated_amount, "64M", "256M", first, second)


raw = problems(first_log, escalated_log)
stripped = problems(strip_prefixed(first_log), strip_prefixed(escalated_log))
print(f"EVIDENCE 原样读：证据缺 {len(raw)} 样 {raw}")
print(f"EVIDENCE 先去掉 run-with-memory-cap: 起头的行再读：证据缺 {len(stripped)} 样 {stripped}")

stderr_text = "\n".join([queue_line, hint_line, lookup_failed, "  ✗ 内存不够排不上：这一条要 9.0 GiB（峰值表里没有这条），比 slice 的总上限 8.0 GiB 还大，等多久都放不下；命令没跑",
                         "  → 怎么办：上限写小一点再跑（真要这么多就先调小余量 RUN_WITH_MEMORY_CAP_RESERVE，依据写在 scripts/hosts/run-with-memory-cap.sh 文件头）"])
for label, text in (("原样", stderr_text), ("先去掉 run-with-memory-cap: 起头的行", strip_prefixed(stderr_text))):
    own = verus.wrapper_own_lines({"stderr": text})
    print(f"VERUS wrapper_own_lines {label}：{len(own)} 行，头一行 = {own[0][:70] if own else None}")
