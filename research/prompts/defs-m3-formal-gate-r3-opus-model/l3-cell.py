#!/usr/bin/env python3
"""L3：一格的输出由 verus-proof-check.py 自己的 outcome_problem / problem_howto / report_cell 打（函数级，捏造 outcome，不起 Verus、不起包装），退 1。
用法：python3 l3-cell.py <仓副本根> <all-not-run|mixed|all-red>"""
import importlib.util, os, sys

copy, kind = sys.argv[1], sys.argv[2]
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("vpc", os.path.join(copy, "research", "scripts", "verus-proof-check.py"))
vpc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(vpc)
options = {"memory_cap": "4G", "timeout": 600}
queued = {"exit": 252, "timed_out": False, "wrapper": vpc.WRAPPER_OWN_EXITS[252], "json": None, "diagnostics": "",
          "stderr": "run-with-memory-cap: 排队\n  ✗ 内存不够排不上：等了 600 秒\n  → 怎么办：bash research/scripts/run-with-memory-cap.sh --status 看谁占着\n"}
evicted = dict(queued, exit=254, wrapper=vpc.WRAPPER_OWN_EXITS[254], stderr="  ✗ 被总上限挤掉\n  → 怎么办：看 --status\n")
not_run = [(f"第 {n} 行「排不上 {n}」：{vpc.outcome_problem(o, options)}", vpc.problem_howto(o, "（这一格自己的出路）")) for n, o in ((2, queued), (3, evicted))]
real = [("第 4 行「改坏了也验得过」改坏之后 Verus 照样验过（verified 2、errors 0）：证明没盯住这一处", "补证明或收紧 spec，让这一处改坏就验不过")]
problems = {"all-not-run": not_run, "mixed": not_run + real, "all-red": real}[kind]
vpc.report_cell(problems)
sys.exit(1)
