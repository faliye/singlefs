#!/usr/bin/env python3
"""辩方腿探针：把「判决改法换上之后还放过什么」的几十条造的行喂判据库 task-and-session-numbers 那一形态，逐条打出库判拒还是放，
与腿读正则推出的预期（expect）对一遍；不一致的就是腿推错了。
用法：python3 probe_cases.py --library <lib-forbidden-notations.py>
退出码：0 每条与预期一致；1 有不一致（逐条打出）；2 用法错。
每条一行：name=probe_case id=<编号> group=<组> path=<当成哪份文件判> expect=<refuse|pass> library=<refuse|pass> agree=<yes|no> hits=<命中>
这些行是造的，号都是编的，不是现存记录原样；现存真行的出处在报告里给文件:行号。"""
import argparse
import importlib.util
import sys

FORM = 'task-and-session-numbers'
RECORDS = 'records/probe.md'
KB = '.claude/kb/probe.md'
LEG_REPORT = 'research/prompts/probe-sonnet-output.md'
VERDICT = 'research/prompts/probe-main-verification.md'

# (编号, 组, 当成哪份文件判, 文字, 腿读正则推的预期, 插入点前面的文字)
CASES = (
    # 组 single-and-colon：判决第 1 条列的漏拒，改法换上之后还剩的
    ('A1', 'single-tong', RECORDS, '- 合并步同 13495，因为主工作区改动让行键变了', 'pass', ''),
    ('A2', 'colon-after', RECORDS, '- 对照：主工作区跑同一条（任务 21036: 退 78）', 'pass', ''),
    ('A3', 'control-tong-joined', RECORDS, '- 计划那一步连撞三次上限（同 20461、20814，已认领）', 'refuse', ''),
    ('A4', 'control-fullwidth-paren', RECORDS, '- 用户点名的四条人手任务（5073–5075、5080）都是那一路的', 'refuse', ''),
    ('A5', 'measure-after-task', RECORDS, '- 任务 21036 个别格退 1，其余绿', 'pass', ''),
    ('A6', 'control-task-equals', RECORDS, '- 对照（task=21036）同样退 78', 'refuse', ''),
    # 组 batch：放过的单字与前缀
    ('B1', 'append-before-real-batch', RECORDS, '- 依赖表追加第 4 批（等它发来换上那一句才开）：任务服务一侧那几节', 'pass', ''),
    ('B2', 'measure-char-prefix', RECORDS, '- 批 6 处理完再交', 'pass', ''),
    ('B3', 'control-batch', RECORDS, '- 它的批 105 在修', 'refuse', ''),
    ('B4', 'anchor-in-kb', KB, '- 批次：records/2026-11-19-进仓文字不写任务号与会话短号.md#第 3 批', 'pass', ''),
    ('B5', 'backtick-batch', RECORDS, '- 见 `批 105 在修`', 'pass', ''),
    ('B6', 'fence-batch', RECORDS, '```\n批 105 在修\n```', 'pass', ''),
    ('B7', 'control-fence-task', RECORDS, '```\n任务 21036 已认领\n```', 'refuse', ''),
    # 组 short-code：不挨会话词的数字字母括号短号
    ('C1', 'real-short-codes-in-leg-report', LEG_REPORT, '和在改 E202 的会话、(3b)、(10)、(eb) 约窗口', 'pass', ''),
    ('C2', 'real-short-code-in-verdict', VERDICT, '修补批：以 (5b) 借卡那一批落主工作区之后的样子为基线', 'pass', ''),
    ('C3', 'same-in-records', RECORDS, '修补批：以 (5b) 借卡那一批落主工作区之后的样子为基线', 'refuse', ''),
    ('C4', 'step-label-in-kb', KB, '几何敏感性：6 槽小环（a6）、64 槽环（A）、带反例的环', 'pass', ''),
    ('C5', 'step-label-in-records', RECORDS, '几何敏感性：6 槽小环（a6）、64 槽环（A）、带反例的环', 'refuse', ''),
    ('C6', 'backtick-short-code', RECORDS, '- 是观察者交回的盘那一族 `（5b 在修）`', 'pass', ''),
    ('C7', 'control-short-code-with-word-in-kb', KB, '- 是观察者交回的盘那一族（5b 在修）', 'refuse', ''),
    # 组 table：records/ 下以 | 起头的整行放过批号
    ('D1', 'table-dependency-column', RECORDS, '| 治理文件轮三方 | 两条腿 | 第 2 批绿 | 等 | 第 3 批 |', 'pass', ''),
    ('D2', 'table-note-column', RECORDS, '| 备注 | 它的批 105 在修 |', 'pass', ''),
    ('D3', 'table-note-column-in-kb', KB, '| 备注 | 它的批 105 在修 |', 'refuse', ''),
    ('D4', 'control-task-in-table', RECORDS, '| 1 | 读峰值表一遍 CPU 0.086 秒（任务 #1748，第二台上量） | x |', 'refuse', ''),
    # 组 reading-line：保留的 A/B 读数写法
    ('E1', 'reading-without-colon', VERDICT, '- 甲、丁、戊：只补测试与变异行；零轮，A/B 读数任务 12400（未改绿、变异红）', 'refuse', ''),
    ('E2', 'reading-line-start', VERDICT, 'A/B 读数：任务 12400', 'pass', ''),
    ('E3', 'reading-mid-line-with-colon', VERDICT, '- 甲：零轮，A/B 读数：任务 12400（未改绿、变异红）', 'refuse', ''),
    ('E4', 'control-reading-in-kb', KB, 'A/B 读数：任务 12400', 'refuse', ''),
    # 组 result-line：结局行与命令
    ('F1', 'result-line-verbatim', RECORDS, '- 末行 TASK_SERVER_RESULT task=21036 type=succeeded exit=0', 'refuse', ''),
    ('F2', 'backtick-subcommand', RECORDS, '- 日志 `task-server/task log 4897 --all`', 'refuse', ''),
    ('F3', 'result-line-without-id', RECORDS, '- 末行 TASK_SERVER_RESULT type=succeeded exit=0 attempts=1', 'pass', ''),
    ('F5', 'log-file-name-carries-id', RECORDS, '- 日志 task-b0f6b921-20071.attempt-3.log', 'pass', ''),
    ('F4', 'placeholders-in-report', LEG_REPORT, '- 任务 <号>、任务 N、同 <号>、task=<号>、批 N、（<两位>）', 'pass', ''),
)


def load_library(path):
    spec = importlib.util.spec_from_file_location('lib_forbidden_notations', path)
    library = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(library)
    return library


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--library', required=True)
    arguments = parser.parse_args()
    library = load_library(arguments.library)
    disagreements = 0
    for case_id, group, path, text, expect, preceding in CASES:
        hits = library.all_hits(FORM, text, path, preceding)
        measured = 'refuse' if hits else 'pass'
        agree = 'yes' if measured == expect else 'no'
        disagreements += agree == 'no'
        shown = ';'.join(f'{name}:{matched}' for _, _, name, matched in hits)
        print(f'name=probe_case id={case_id} group={group} path={path} expect={expect} library={measured} agree={agree} hits={shown or "-"}')
    print(f'name=probe_summary cases={len(CASES)} disagreements={disagreements}')
    return 1 if disagreements else 0


if __name__ == '__main__':
    sys.exit(main())
