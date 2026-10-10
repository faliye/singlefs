#!/usr/bin/env python3
"""第 10z 批代码轮第一轮攻方：K3 改回去照绿的探针。在一份冻结副本的拷贝里逐条改坏 gpu_memory_ledger.rs 的一处（原文必须恰好命中一次），
跑 harness 档 `--lib -- gpu_memory_ledger::tests`，看有没有一条红；跑完还原。整条脚本经内存包装跑。
用法：mutate_k3.py <冻结副本的拷贝> <编译目录> <变异名…>（不给名字列出全部）。"""
import pathlib, subprocess, sys

FILE = "crates/singlefs-harness/src/gpu_memory_ledger.rs"
MUTANTS = {
    "seen-not-reduced-when-a-seen-lease-is-returned": (
        "                    self.seen_on_the_card_at_least_mebibytes = self\n                        .seen_on_the_card_at_least_mebibytes\n                        .saturating_sub(mebibytes);",
        "                    self.seen_on_the_card_at_least_mebibytes = self\n                        .seen_on_the_card_at_least_mebibytes\n                        .saturating_sub(0);"),
    "restart-keeps-the-old-seen": (
        "        self.seen_on_the_card_at_least_mebibytes = 0;\n        self.leases_seen_on_the_card.clear();",
        "        self.leases_seen_on_the_card.clear();"),
    "restart-keeps-the-old-seen-leases": (
        "        self.seen_on_the_card_at_least_mebibytes = 0;\n        self.leases_seen_on_the_card.clear();",
        "        self.seen_on_the_card_at_least_mebibytes = 0;"),
    "seen-replaced-on-equal": (
        "        if seen_now_mebibytes > self.seen_on_the_card_at_least_mebibytes {",
        "        if seen_now_mebibytes >= self.seen_on_the_card_at_least_mebibytes {"),
    "seen-leases-not-replaced": (
        "            self.leases_seen_on_the_card = self.leases.keys().copied().collect();",
        "            let _ = &self.leases_seen_on_the_card;"),
    "restart-refused-when-exactly-fits": (
        "                if capacity_mebibytes < booked.leases_mebibytes {",
        "                if capacity_mebibytes <= booked.leases_mebibytes {"),
    "compiling-mark-dropped-by-the-first-of-two": (
        "        if still_compiling <= 1 {",
        "        if still_compiling <= usize::MAX {"),
    "work-of-the-thread-not-restored": (
        "        WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.set(self.work_of_this_thread_before));",
        "        WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.set(None));"),
    "explaining-takes-the-first-not-the-last": (
        "            .rev()\n            .find(|report| report.works_it_explains.contains(&work))",
        "            .find(|report| report.works_it_explains.contains(&work))"),
    "restart-check-uses-the-ceiling": (
        "                    memory_this_process_may_hold_mebibytes(\n                        readings[card_position],",
        "                    readings[card_position].total_less_reserved_mebibytes.max(memory_this_process_may_hold_mebibytes(\n                        readings[card_position],"),
}

def main():
    copy, target, names = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3:]
    if not names:
        print("\n".join(MUTANTS)); return
    path = copy / FILE
    original = path.read_bytes()
    for name in names:
        old, new = MUTANTS[name]
        text = original.decode()
        if text.count(old) != 1:
            print(f"MUTANT name={name} anchor_hits={text.count(old)} result=anchor-broken", flush=True); continue
        if name == "restart-check-uses-the-ceiling":
            new = new  # 括号在下一处补上
            text = text.replace(old, new, 1).replace(
                "                        self.mebibytes_each_process_keeps_on_a_card,\n                    ),\n                );\n                if capacity_mebibytes",
                "                        self.mebibytes_each_process_keeps_on_a_card,\n                    )),\n                );\n                if capacity_mebibytes", 1)
        else:
            text = text.replace(old, new, 1)
        path.write_bytes(text.encode())
        try:
            # ATTACK_MODEL=1：不跑库单测，改跑攻方模型的随机历史（模型先由 run.sh 拷进这份拷贝的 tests/）
            which = (["--test", "ledger_attack_10z", "--", "random_histories", "--nocapture"] if __import__("os").environ.get("ATTACK_MODEL")
                     else ["--lib", "--", "gpu_memory_ledger::tests"])
            run = subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline", "-p", "singlefs-harness", *which], cwd=copy, env={**__import__("os").environ, "CARGO_TARGET_DIR": target},
                                 capture_output=True, text=True)
        finally:
            path.write_bytes(original)
        summary = [l for l in run.stdout.splitlines() if l.startswith("test result:")]
        failed = [l.split()[1] for l in run.stdout.splitlines() if l.startswith("test ") and l.endswith("FAILED")]
        verdict = "caught" if run.returncode != 0 else "survived"
        for attack in (l for l in run.stdout.splitlines() if "scenario=random" in l):
            print(f"MUTANT name={name} {attack[attack.index('ATTACK'):]}", flush=True)
        print(f"MUTANT name={name} exit={run.returncode} result={verdict} summary={summary[-1] if summary else run.stderr.strip().splitlines()[-1:]} failed={failed}", flush=True)

main()
