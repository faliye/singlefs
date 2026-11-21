#!/usr/bin/env python3
"""第 11 问：派发闸 ④ 读写范围表新加的八类行之后，对这几类派发提示里「写进 / 写到 / 更新 <路径>」那一种点名判成什么。

用法：python3 dispatch_scope_model.py --frozen <被判的那棵树>
  从被判那棵树的 .claude/hooks/runner-dispatch-guard.sh 原样取出 SCOPE_PATH_START 到 MUTATION_TABLE_MENTION 之前的几条正则，
  与 expanded_mention、scope_patterns、out_of_scope_mentions 三个函数，在本进程里执行；写范围表、lib_write_scope.py、lib_work_root.py
  用那棵树里的。报告行那一支（report_line_mentions，与 ⑰ 共用的解析）换成恒空：这里只量「写进 / 写到 / 更新」那一种。
  每格给一段派发提示里会出现的句子与这句话的本意（是不是叫子 agent 写那条路径），看 ④ 判出的越界路径。
输出：每格一行 `SCOPE name=<格> agent=<类型> outside=<④ 判越界的路径> asks_agent_to_write=<本意> verdict=<…>`，末行 `SCOPE name=done emitted=<行数>`。
  verdict：false-refusal（本意不叫它写，④ 照拒）、missed（本意叫它写写范围外的路径，④ 放过）、correct。
退出码：0 跑完；1 取不出那几段；2 用法错。
"""
import argparse
import importlib.util
import json
import os
import re
import sys

sys.dont_write_bytecode = True   # 从被判那棵树导入的两份库不往那棵树里写 __pycache__
EMITTED = 0
ROUND = "research/prompts/probe-round-r2"
CASES = (
    ("sweep-input-lists-what-main-agent-updated", "sweep", "这一阶段做成的事：\n- 更新 .claude/kb/tooling.md 的任务服务一节\n- 改了检出钩子", False),
    ("sweep-candidate-table-in-work-root", "sweep", "候选表写进 $SINGLEFS_WORK_ROOT/probe-sync/candidates.md", True),
    ("prior-art-says-who-writes-kb", "prior-art", "结论要进 kb 的，由主 agent 交写回员写进 .claude/kb/prior-art.md。", False),
    ("prior-art-negated-kb", "prior-art", "不要写进 .claude/kb/prior-art.md。", False),
    ("attack-leg-told-where-verdict-goes", "three-way-attack", f"主 agent 收齐之后把判决写进 {ROUND}-main-verification.md。", False),
    ("attack-leg-own-report", "three-way-attack", f"报告写进 {ROUND}-opus-output.md", True),
    ("attack-leg-given-defense-leg-report", "three-way-attack", f"报告写进 {ROUND}-sonnet-output.md", None),
    ("defense-leg-told-earlier-model-may-be-updated", "three-way-defense", "第一轮的模型可以拷过去改，改完更新 research/prompts/probe-round-r1-sonnet-model/probe.py 里的格名。", None),
    ("materials-told-to-write-judged-list", "three-way-materials", f"开工快照写进 {ROUND}-snapshot/judged-sha256.txt", True),
    ("designer-told-what-runner-writes-later", "experiment-designer", "执行员之后把实验页写进 .claude/kb/experiments/e999.md，你只写登记。", False),
    ("designer-own-preregistration", "experiment-designer", "登记写进 research/prompts/e999-preregistration.md", True),
    ("triage-told-who-adds-assertions", "mutation-triage", "补断言由主 agent 另派实现员写进 crates/singlefs-core/tests/probe.rs。", False),
)


def emit(name, **fields):
    global EMITTED
    EMITTED += 1
    print("SCOPE " + " ".join([f"name={name}"] + [f"{key}={json.dumps(value, ensure_ascii=False)}" for key, value in fields.items()]), flush=True)


def load_module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def block(source, start_marker, end_marker):
    return source[source.index(start_marker):source.index(end_marker)]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    arguments = parser.parse_args()
    hooks = os.path.join(arguments.frozen, ".claude", "hooks")
    sys.path.insert(0, hooks)
    try:
        work_roots = load_module(os.path.join(hooks, "lib_work_root.py"), "lib_work_root")
        write_scope = load_module(os.path.join(hooks, "lib_write_scope.py"), "lib_write_scope")
        with open(os.path.join(hooks, "runner-dispatch-guard.sh"), encoding="utf-8") as handle:
            guard = handle.read()
        space = {"os": os, "re": re, "work_roots": work_roots, "write_scope": write_scope, "SCRATCH_MENTION_PREFIX": work_roots.mention_prefix_pattern(),
                 "break_switch_is": lambda item: False, "note_unjudged": lambda *items: None, "SCRIPT_HOOK_DIR": hooks,
                 "report_line_mentions": lambda prompt: []}
        exec(compile(block(guard, "SCOPE_PATH_START = ", "MUTATION_TABLE_MENTION = "), "guard_constants", "exec"), space)
        exec(compile(block(guard, "def expanded_mention(text):", "def parse_moment(text):"), "guard_scope_functions", "exec"), space)
    except (OSError, ValueError, SyntaxError, AttributeError) as error:
        emit("setup-error", error=repr(error)[:300])
        print(f"SCOPE name=done emitted={EMITTED}", flush=True)
        return 1
    for name, agent, sentence, asks in CASES:
        prompt = sentence.replace("$SINGLEFS_WORK_ROOT", work_roots.work_root())
        outside = space["out_of_scope_mentions"](arguments.frozen, agent, prompt)
        if asks is None:
            verdict = "judged-by-reader"
        elif asks:
            verdict = "correct" if not outside or sentence.startswith("开工快照") else "false-refusal"
            if sentence.startswith("开工快照"):
                verdict = "correct" if outside else "missed"
        else:
            verdict = "false-refusal" if outside else "correct"
        emit(name, agent=agent, outside=outside, asks_agent_to_write=("unclear" if asks is None else "yes" if asks else "no"), verdict=verdict)
    print(f"SCOPE name=done emitted={EMITTED}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
