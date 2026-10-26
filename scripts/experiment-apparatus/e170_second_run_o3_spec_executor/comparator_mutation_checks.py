"""E170 第 1 次重跑第三段变异驱动用的检查（登记第九节）：在执行器目录的一份副本里跑（副本里的执行器或比对器可能被改过一处），
标准输出一行一条具名断言 `CHECK name=<名> holds=true|false`，收尾 `CHECK name=__complete__ holds=true`。从第一次的同名文件拷来改：
判定一律取严格跑法；P8 期望「在」、P10 一族与 Q3-pc 是这一次新增的。断言五类：
  q3:<流>:<行名>:verdict / :landing  坏状态每行（含 P10）的严格跑法在 / 不在、落点对期望（P 行与盲测族期望不在，P8 与 N 行期望在，P10 期望不在）；
  q3_pc:exactly_one                   Q3 比对器阳性对照（期望表翻掉一行 P 行的期望判定，比出恰 1 处）；
  anchor:<流>:<编号>                  第七节锚点 A1–A7、A11–A13（executor_main.anchor_assertions 原样）；
  q2_rejudged_batch1:<流>             副本里的执行器把批 1 逐态重判一遍（严格跑法），与实现判定逐态比，不同为 0；
  q2_pc:exactly_one                   Q2 比对器阳性对照（comparator_q2_implementation_verdicts 在冻结那一份判定上翻一态）。
变异驱动只认基线上为真的那几条：变异让其中任一条转假（或单测红）记抓到。文件名以 comparator 起头，不进冻结的 sha256。
"""

from __future__ import annotations

import argparse
import os
import sys

import executor_main
from comparator_q2_implementation_verdicts import implementation_passes_both, positive_control_differences, read_frozen_lines, \
    read_stream_tables
from comparator_q3_bad_states import judge_and_compare, positive_control_differences as q3_positive_control_differences
from executor_core import VERDICT_IN, judge_state
from replay_prefix import UnitChecksumCache


def check(name: str, holds: bool) -> None:
    print(f"CHECK name={name} holds={str(bool(holds)).lower()}", flush=True)


def anchor_checks(stream_name: str, context) -> None:
    captured: list[str] = []
    executor_main.anchor_assertions(stream_name, context, captured.append)
    for line in captured:
        fields = dict(token.split("=", 1) for token in line.split() if "=" in token)
        identifier = fields.get("id")
        if identifier == "A1":
            check(f"anchor:{stream_name}:A1_landed_part", fields.get("landed_part") == "true")
            check(f"anchor:{stream_name}:A1_two_element_part", fields.get("two_element_part") == "true")
        elif identifier is not None:
            check(f"anchor:{stream_name}:{identifier}", fields.get("holds") == "true")


def rejudged_batch_one_checks(dumps: str, contexts) -> None:
    _, implementation = read_stream_tables(dumps)
    for stream_name, context in contexts.items():
        cache = UnitChecksumCache()
        different = 0
        for state in context.dump.states:
            if state.batch_number != 1:
                continue
            judgement = judge_state(context, state, cache)
            executor_in = judgement.strict_verdict == VERDICT_IN
            if executor_in != implementation_passes_both(implementation[stream_name][state.state_number]) or not judgement.digest_matches:
                different += 1
        check(f"q2_rejudged_batch1:{stream_name}", different == 0)


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第三段变异驱动的检查")
    parser.add_argument("--dumps", required=True)
    parser.add_argument("--frozen-verdicts", required=True)
    arguments = parser.parse_args(argument_list)
    comparisons, contexts, _ = judge_and_compare(arguments.dumps)
    for comparison in comparisons:
        prefix = f"q3:{comparison.stream}:{comparison.expected.name}"
        check(f"{prefix}:verdict", comparison.verdict_matches)
        check(f"{prefix}:landing", comparison.landing_matches)
    differences_q3, _ = q3_positive_control_differences(comparisons)
    check("q3_pc:exactly_one", differences_q3 == 1)
    for stream_name, context in contexts.items():
        anchor_checks(stream_name, context)
    rejudged_batch_one_checks(arguments.dumps, contexts)
    frozen = {(row["stream"], int(row["state"])): row for row in read_frozen_lines(arguments.frozen_verdicts, "state")}
    states, implementation = read_stream_tables(arguments.dumps)
    differences, _ = positive_control_differences(frozen, states, implementation, "strict")
    check("q2_pc:exactly_one", differences == 1)
    check("__complete__", True)
    return 0


if __name__ == "__main__":
    os.environ.setdefault("PYTHONDONTWRITEBYTECODE", "1")
    sys.exit(main(sys.argv[1:]))
