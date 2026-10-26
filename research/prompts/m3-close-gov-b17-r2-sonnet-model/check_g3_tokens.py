#!/usr/bin/env python3
"""H1：commit-stages 第 2b 步里点名的标识、变量、行名，在驱动 broad-mutation.py、清场包装 gpu-quiesce-run.sh、
crash-amplification-lib.sh 里各出现几次（str.count）。0 次的就是规则点了名而代码里没有。
用法：python3 -I check_g3_tokens.py <仓根>"""
import os, sys

repo = os.path.abspath(sys.argv[1])
FILES = ["scripts/mutation/broad-mutation.py", "scripts/crash/gpu-quiesce-run.sh", "scripts/crash/crash-amplification-lib.sh"]
texts = {path: open(os.path.join(repo, path), encoding="utf-8").read() for path in FILES}
stages = open(os.path.join(repo, ".claude/skills/crash-test/commit-stages.md"), encoding="utf-8").read().split("\n")
section = "\n".join(stages[22:28])      # 第 23–28 行：第 2b 步的引言与 ①–④（行号去文件里现查过，见报告）
TOKENS = ["BROAD_MUTATION_GPU_NOT_RUN", "BROAD_MUTATION_GPU_PLAN", "BROAD_MUTATION_GPU_ITEM", "BROAD_MUTATION_GPU_TOTAL", "IMPORT_GPU_REJECTED",
          "NOT_RUN_NO_GPU_ADAPTER", "--skip-gpu", "--shard", "--import-results", "gpu-verdicts.tsv", "gpu_needed", "gpu_reused", "gpu_not_run",
          "gpu_without_conclusion", "gpu-not-run", "gpu-timeout", "gpu-memory-cap", "gpu-driver-error", "广谱变异 GPU 趟", "gpu:",
          "GPU_QUIESCE_STATE_DIRECTORY", "GPU_QUIESCE_LOG_DIRECTORY", "QUIESCE_STOP_COMMAND", "QUIESCE_STOPPED_CHECK_COMMAND",
          "QUIESCE_START_COMMAND", "QUIESCE_STARTED_CHECK_COMMAND", "CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS", "FLIPPED",
          "escalated-verdicts.tsv", "--unit-environment", "--refresh-list", "--timeout-floor", "--memory-cap", "gpu-quiesce-run.sh"]
print("token\tin_commit_stages_23_28\t" + "\t".join(os.path.basename(path) for path in FILES))
for token in TOKENS:
    print(f"{token}\t{section.count(token)}\t" + "\t".join(str(texts[path].count(token)) for path in FILES))
DRIVER = ["日志里没有带源码位置的编译错误", "一个测试都没跑到", "认不出的退出码"]
print("driver-error-origin\tcount_in_broad-mutation.py")
for phrase in DRIVER:
    print(f"{phrase}\t{texts[FILES[0]].count(phrase)}")
print("skip-stage-two-in-commit-stages-23_28\t" + str(section.count("skip-stage-two")))
