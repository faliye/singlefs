"""W1：照丙第 2 件「先去掉以 run-with-memory-cap: 起头的行再读」，在被判的判法函数上比去行前后。只 import 两份判法（不走 __main__、不起 Verus、不起 cargo）。
  广谱变异：escalation_evidence_problems（升档证据，文件头「升档」①–③）拿 w1-wrapper-hits.sh 落的真包装日志判，原样一次、去行一次。
  形式化：outcome_problem 与 QueueRefusal.record（252 那一处「包装说」的那一句）拿 w1-queue-world.sh 落的真包装 stderr 判，去行之后再判。
  另数去行之后留下的、查机器那一支打的行（不以 run-with-memory-cap: 起头的）。
用法：w1-strip-judges.py <树> <hits 目录> <世界目录…>"""
import importlib.util
import json
import os
import sys

PREFIX = "run-with-memory-cap:"


def load(root, relative, name):
    spec = importlib.util.spec_from_file_location(name, os.path.join(root, relative))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def strip(text):
    return "".join(line for line in text.splitlines(keepends=True) if not line.startswith(PREFIX))


root, hits, worlds = sys.argv[1], sys.argv[2], sys.argv[3:]
os.environ.pop("VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE", None)
broad = load(root, "scripts/mutation/broad-mutation.py", "broad_mutation_attack")
verus = load(root, "scripts/gate/verus-proof-check.py", "verus_proof_check_attack")
read = lambda path: open(path, encoding="utf-8").read()
cases = {
    "time": (broad.TIME_LIMIT_HIT_EXIT, 0.1, 1, 2, "1", "2", read(os.path.join(hits, "time-first.log")), read(os.path.join(hits, "time-escalated.log"))),
    "memory": (broad.MEMORY_CAP_HIT_EXIT, float(1 << 20), 64 << 20, 128 << 20, "64M", "128M",
               read(os.path.join(hits, "memory-first.log")), read(os.path.join(hits, "memory-escalated.log"))),
}
for kind, (exit_code, baseline, first_amount, escalated_amount, first_text, escalated_text, first_log, escalated_log) in cases.items():
    for arm, transform in (("raw", lambda text: text), ("stripped", strip)):
        problems = broad.escalation_evidence_problems(exit_code, exit_code, baseline, first_amount, escalated_amount, first_text, escalated_text,
                                                      transform(first_log), transform(escalated_log))
        print(f"name=broad_escalation_evidence kind={kind} arm={arm} problems={len(problems)} first={json.dumps(problems[:1], ensure_ascii=False)}")
for world in worlds:
    label = os.path.basename(world.rstrip("/"))
    stderr = strip(read(os.path.join(world, "stderr")))
    status = int(open(os.path.join(world, "status")).read()) if os.path.exists(os.path.join(world, "status")) else None
    kept = [line for line in stderr.splitlines() if not line.startswith(("  ✗", "  →", "      ", "命令自己的 stderr"))
            or "此刻放得下" in line]
    print(f"name=strip_leftovers world={label} lines_after_strip={len(stderr.splitlines())} host_lookup_lines_left={len(kept)}")
    for line in kept[:3]:
        print(f"  left={line[:150]}")
    if status is None:
        continue
    outcome = {"exit": status, "timed_out": False, "wrapper": verus.WRAPPER_OWN_EXITS.get(status), "json": None, "diagnostics": stderr, "stderr": stderr}
    problem = verus.outcome_problem(outcome, {"memory_cap": "200M", "timeout": 60}) or ""
    state = verus.QueueRefusal()
    if status == 252:
        state.record("proof", "proof:attack", outcome)
    where = state.first[0] if state.first else ""
    print(f"name=verus_wrapper_outcome world={label} exit={status} own_lines={len(verus.wrapper_own_lines(outcome))} "
          f"first_own_line={verus.wrapper_own_lines(outcome)[0][:70] if verus.wrapper_own_lines(outcome) else ''}")
    print(f"  queue_refusal_where={where[:160]}")
    print(f"  problem_has_peer_numbers={'可用 ' in problem and '此刻放得下' in problem} problem={problem[:200]}")
