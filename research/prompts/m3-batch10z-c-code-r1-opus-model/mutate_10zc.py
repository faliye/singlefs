#!/usr/bin/env python3
"""第 10z 批第三块代码轮第一轮攻方：K3「改回去照绿」的探针。在冻结副本的一份拷贝里逐条改坏 gpu_memory_ledger.rs 的一处
（原文必须恰好命中一次），跑 harness 档 `--lib -- gpu_memory_ledger::tests`（这一批新加、改过的用例都在这一组里），看有没有一条红；
`--with-models` 时另跑攻方模型三个目标（拷进那份拷贝的 tests/），抄 ATTACK 行。跑完还原。整条脚本经内存包装跑。
用法：mutate_10zc.py <冻结副本的拷贝> <编译目录> [--with-models] <变异名…>（不给名字列出全部）。"""
import os, pathlib, subprocess, sys

FILE = "crates/singlefs-harness/src/gpu_memory_ledger.rs"
MUTANTS = {
    "lost-state-reverted-to-registered": (
        "        self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;",
        "        self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;"),
    "lost-state-promises-no-driver-share": (
        "        DriverShareOnTheCard::NotRegistered => booked.leases_mebibytes,\n        DriverShareOnTheCard::RegisteredDeviceNotStarted\n        | DriverShareOnTheCard::DeviceStarted\n        | DriverShareOnTheCard::DeviceLostNotStartedAgain => booked",
        "        DriverShareOnTheCard::NotRegistered | DriverShareOnTheCard::DeviceLostNotStartedAgain => booked.leases_mebibytes,\n        DriverShareOnTheCard::RegisteredDeviceNotStarted\n        | DriverShareOnTheCard::DeviceStarted => booked"),
    "lost-state-not-registered": (
        "            DriverShareOnTheCard::NotRegistered => false,\n            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted\n            | DriverShareOnTheCard::DeviceLostNotStartedAgain => true,",
        "            DriverShareOnTheCard::NotRegistered | DriverShareOnTheCard::DeviceLostNotStartedAgain => false,\n            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted => true,"),
    "endings-notify-one": (
        "            self.every_work_ended.notify_all();",
        "            self.every_work_ended.notify_one();"),
    "endings-do-not-wait": (
        "        while *works_not_yet_ended > 0 {",
        "        while false && *works_not_yet_ended > 0 {"),
    "record-the-caught-after-waiting": (
        "        if let Some(caught) = caught_in_the_scope {\n            self.record_an_out_of_memory(caught);\n        }\n        ending.end_and_wait_for_the_others();",
        "        ending.end_and_wait_for_the_others();\n        if let Some(caught) = caught_in_the_scope {\n            self.record_an_out_of_memory(caught);\n        }"),
    "blocks-taken-largest-first": (
        "        self.blocks_not_taken_locked().pop_first()",
        "        self.blocks_not_taken_locked().pop_last()"),
    "store-after-a-storing-error": (
        "            if self.storing_error.is_some() {\n                continue;\n            }",
        "            if false && self.storing_error.is_some() {\n                continue;\n            }"),
    "a-failed-block-counts-as-stored": (
        "                Err(error) => self.storing_error = Some(error),",
        "                Err(error) => {\n                    self.storing_error = Some(error);\n                    self.blocks_stored += 1;\n                }"),
    "stop-ignored-after-an-out-of-memory": (
        "        StopOfTheRun::Requested => AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,",
        "        StopOfTheRun::Requested => AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory,"),
    "card-keeps-taking-after-the-store-is-gone": (
        "                BlockHandedToTheStore::TheStoreIsGone => return CardLeftTheBlocks::TheStoreIsGone,",
        "                BlockHandedToTheStore::TheStoreIsGone => {}"),
    "a-refused-block-is-given-back": (
        "            BlockJudgedOnACard::Refused { reason } => return CardLeftTheBlocks::Refused { reason },",
        "            BlockJudgedOnACard::Refused { reason } => {\n                blocks.give_back(pending_index);\n                return CardLeftTheBlocks::Refused { reason };\n            }"),
    "stop-checked-after-taking-a-block": (
        "    while !must_stop() {\n        let Some(pending_index) = blocks.take_a_block() else {\n            break;\n        };",
        "    loop {\n        let Some(pending_index) = blocks.take_a_block() else {\n            break;\n        };\n        if must_stop() {\n            break;\n        }"),
    "an-out-of-memory-card-says-nothing-left": (
        "                blocks.give_back(pending_index);\n                return CardLeftTheBlocks::OutOfMemory { reason };",
        "                blocks.give_back(pending_index);\n                let _ = reason;\n                return CardLeftTheBlocks::NothingLeftOrStopped;"),
    "blocks-left-counted-from-not-taken": (
        "    !self.blocks_not_taken_locked().is_empty()\n    }",
        "    self.blocks_not_taken_locked().len() > 1\n    }"),
}
MODEL_TARGETS = ["attack_10zc_endings", "attack_10zc_rounds", "attack_10zc_ledger"]

def cargo(copy, target, which):
    return subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline", "-p", "singlefs-harness", *which],
                          cwd=copy, env={**os.environ, "CARGO_TARGET_DIR": target}, capture_output=True, text=True)

def main():
    copy, target, rest = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3:]
    with_models = "--with-models" in rest
    names = [name for name in rest if name != "--with-models"]
    if not names:
        print("\n".join(MUTANTS)); return
    path = copy / FILE
    original = path.read_bytes()
    for name in names:
        old, new = MUTANTS[name]
        text = original.decode()
        if text.count(old) != 1:
            print(f"MUTANT name={name} anchor_hits={text.count(old)} result=anchor-broken", flush=True); continue
        path.write_bytes(text.replace(old, new, 1).encode())
        try:
            run = cargo(copy, target, ["--lib", "--", "gpu_memory_ledger::tests"])
            models = None
            if with_models:
                models = cargo(copy, target, [*sum((["--test", t] for t in MODEL_TARGETS), []), "--", "--nocapture", "--test-threads", "1"])
        finally:
            path.write_bytes(original)
        summary = [l for l in run.stdout.splitlines() if l.startswith("test result:")]
        failed = [l.split()[1] for l in run.stdout.splitlines() if l.startswith("test ") and l.endswith("FAILED")]
        verdict = "caught" if run.returncode != 0 else "survived"
        tail = summary[-1] if summary else (run.stderr.strip().splitlines()[-1:] or ["?"])[0]
        print(f"MUTANT name={name} exit={run.returncode} result={verdict} summary={tail} failed={failed}", flush=True)
        if models is not None:
            for line in models.stdout.splitlines():
                if "ATTACK" in line:
                    print(f"MUTANT name={name} model {line[line.index('ATTACK'):]}", flush=True)
            print(f"MUTANT name={name} model_exit={models.returncode}", flush=True)

main()
