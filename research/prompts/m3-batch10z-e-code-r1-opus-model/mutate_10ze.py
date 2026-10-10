#!/usr/bin/env python3
"""第 10z 批第五块代码轮第一轮攻方：harness 档改回去的探针。
用法（经内存包装）：python3 mutate_10ze.py <冻结副本的拷贝> <编译目录> <变异名…>
每条：在拷贝的 gpu_memory_ledger.rs 上定点改（原文必须恰好命中一次）、跑 harness 档 lib 的 gpu_memory_ledger 用例与
本模型 attack_10ze_rounds 的叫停只出现一臂（ATTACK_10ZE_SCRIPTS=5000，断言 K1-5 四条），记两边红没红，再还原。"""
import os, subprocess, sys
LEDGER = "crates/singlefs-harness/src/gpu_memory_ledger.rs"
MUTANTS = {
    "H1-largest-is-the-last-judged": (
        "                .map_or(pending_index, |largest| largest.max(pending_index)),",
        "                .map_or(pending_index, |_largest| pending_index),"),
    "H2-next-card-ignores-the-stop": (
        "            .is_some_and(|first| queue.may_take(*first, stop))",
        "            .is_some_and(|_first| stop != StopOfTheCards::TheRunFailed)"),
    "H3-stop-before-any-judged-takes-the-head": (
        "                self.largest_judged.is_some_and(|largest| first < largest)",
        "                self.largest_judged.map_or(true, |largest| first < largest)"),
    "H4-less-or-equal": (
        "                self.largest_judged.is_some_and(|largest| first < largest)",
        "                self.largest_judged.is_some_and(|largest| first <= largest)"),
    "H5-short-cards-counted-as-left": (
        "            .filter(|index| !self.cards_short_of_memory.contains(index))",
        "            .filter(|_index| true)"),
    "H6-record-on-taking-not-on-judging": (
        "        match judge_a_block(pending_index) {",
        "        blocks.record_a_judged_block(pending_index);\n        match judge_a_block(pending_index) {"),
    "H11-device-never-lost-at-return": (
        "        if self.is_lost() {\n            DeviceWhenTheWorkReturned::Lost",
        "        if false && self.is_lost() {\n            DeviceWhenTheWorkReturned::Lost"),
    "H12-device-read-after-waiting": (
        "        ending.end_and_wait_for_the_others();\n        self.out_of_memory_explaining_a_failure_of_the_work(\n            work.identifier(),\n            reports_before,\n            device_when_the_work_returned,\n        )",
        "        ending.end_and_wait_for_the_others();\n        let _ = device_when_the_work_returned;\n        self.out_of_memory_explaining_a_failure_of_the_work(\n            work.identifier(),\n            reports_before,\n            self.device_when_the_work_returns(),\n        )"),
    "H19-stop-with-one-waiting-block-stops": (
        "        StopOfTheRun::Requested if blocks_waiting == 0 => AfterARoundOfTheCards::StoppedOnRequest,",
        "        StopOfTheRun::Requested if blocks_waiting <= 1 => AfterARoundOfTheCards::StoppedOnRequest,"),
}

def run(copy, target, args, extra_env=None):
    env = dict(os.environ, CARGO_TARGET_DIR=target)
    env.update(extra_env or {})
    p = subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline", "-p", "singlefs-harness"] + args,
                       cwd=copy, env=env, capture_output=True, text=True)
    out = p.stdout + p.stderr
    summary = [l for l in out.splitlines() if l.startswith("test result:") or " ... FAILED" in l or l.startswith("error")]
    return p.returncode, summary

def main():
    copy, target, names = sys.argv[1], sys.argv[2], sys.argv[3:]
    path = os.path.join(copy, LEDGER)
    for name in names:
        old, new = MUTANTS[name]
        original = open(path, encoding="utf-8").read()
        hits = original.count(old)
        if hits != 1:
            print(f"MUTANT {name} anchor_hits={hits} skipped")
            continue
        open(path, "w", encoding="utf-8").write(original.replace(old, new, 1))
        try:
            rc_lib, lib = run(copy, target, ["--lib", "--", "gpu_memory_ledger"])
            rc_model, model = run(copy, target, ["--test", "attack_10ze_rounds", "--", "k1_5_monotonic", "--test-threads", "1"],
                                  {"ATTACK_10ZE_SCRIPTS": "5000"})
        finally:
            open(path, "w", encoding="utf-8").write(original)
        print(f"MUTANT {name} lib_rc={rc_lib} model_rc={rc_model}")
        for line in lib:
            print(f"  lib: {line}")
        for line in model:
            print(f"  model: {line}")

if __name__ == "__main__":
    main()
