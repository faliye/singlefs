#!/usr/bin/env python3
"""攻方腿探针：把造的变体行与现存记录、kb、判决里的真行喂判据库 task-and-session-numbers 那一形态，逐条打出库判拒还是放，
与腿用 scripts/kb/append-record.py --dry-run 量到的退出码（0 放、7 拒）对一遍。
用法：python3 probe_cases.py --library <lib-forbidden-notations.py> --repo <仓根>
退出码：0 每条的库判定与登记的量到值一致；1 有不一致（逐条打出）；2 用法错或真行取不到。
每条一行：name=probe_case id=<编号> class=<类> rule=<按规则该拒/该放> library=<refuse|pass> measured=<refuse|pass> agree=<yes|no> hits=<命中>"""
import argparse
import importlib.util
import sys

FORM = 'task-and-session-numbers'
JUDGED_AS = 'records/probe.md'   # 当成 records 下一份 .md 判：后缀在 judged_suffixes 里、不在排除前缀下

# (编号, 类, 按规则该拒还是该放, 腿量到的库判定, 文字)；量到值来自 append-record.py --dry-run 的退出码（7 拒、0 放）
CONSTRUCTED = (
    ('C1', 'control', 'refuse', 'refuse', '- 对照：主工作区跑同一条（任务 21036 (5b)）同样退 78'),
    ('C2', 'control', 'refuse', 'refuse', '- 是观察者交回的盘那一族，它的批 105 在修'),
    ('C3', 'control', 'refuse', 'refuse', '- 计划那一步连撞三次上限（同 20461、20814，已认领）'),
    ('C4', 'control', 'refuse', 'refuse', '- 批 118 量读数：退 0'),
    ('V01', 'missed-task', 'refuse', 'pass', '- 对照：主工作区跑同一条（任务：21036）同样退 78'),
    ('V02', 'missed-task', 'refuse', 'pass', '- 对照：主工作区跑同一条，任务（21036）同样退 78'),
    ('V03', 'missed-task', 'refuse', 'pass', '- 对照：主工作区跑同一条（任务 #21036）同样退 78'),
    ('V04', 'missed-task', 'refuse', 'pass', '- 对照：主工作区跑同一条（任务 21036/21037）同样退 78'),
    ('V05', 'missed-task', 'refuse', 'pass', '- 对照：主工作区跑同一条（任务 21036-21040）同样退 78'),
    ('V06', 'missed-task', 'refuse', 'pass', '- 对照：主工作区跑同一条（任务 21036: 退 78）'),
    ('V07', 'missed-task', 'refuse', 'pass', '- 计划那一步连撞三次上限（同 20461 与 20814，已认领）'),
    ('V08', 'missed-task', 'refuse', 'pass', '- 计划那一步连撞三次上限（同 20461/20814）'),
    ('V09', 'missed-task', 'refuse', 'pass', '- 计划那一步连撞三次上限（同 20461）'),
    ('V10', 'missed-task', 'refuse', 'pass', '- 计划那一步连撞三次上限（与 20461、20814 相同）'),
    ('V22', 'missed-task', 'refuse', 'pass', '- Four tasks (12579, 12580) ran out of memory'),
    ('V23', 'missed-task', 'refuse', 'pass', '- 交了 task#21036，退 78'),
    ('V24', 'missed-task', 'refuse', 'pass', '- 对照（task=21036）同样退 78'),
    ('V11', 'missed-short-code', 'refuse', 'pass', '- 是观察者交回的盘那一族（5b 在修），不是补丁带进来的'),
    ('V12', 'missed-short-code', 'refuse', 'pass', '- 是观察者交回的盘那一族（5b、10 在修）'),
    ('V13', 'missed-short-code', 'refuse', 'pass', '- 是观察者交回的盘那一族（e152）在修'),
    ('V14', 'missed-short-code', 'refuse', 'pass', '- 是观察者交回的盘那一族（5B）在修'),
    ('V15', 'missed-batch', 'refuse', 'pass', '- 是观察者交回的盘那一族，同批 #105 在修'),
    ('V16', 'missed-batch', 'refuse', 'pass', '- 第 105–106 批在修'),
    ('V17', 'missed-batch', 'refuse', 'pass', '- 第 105、106 批在修'),
    ('V18', 'missed-batch', 'refuse', 'pass', '- 批次号 118 量读数：退 0'),
    ('V19', 'missed-batch', 'refuse', 'pass', '- 变异行压到门禁跑完再进表，批 6 只在副本里改、交补丁'),
    ('V20', 'missed-batch', 'refuse', 'pass', '- 批 11 只改日语，会让三语不齐'),
    ('V21', 'missed-batch', 'refuse', 'pass', '- 批次：records/2026-11-19-进仓文字不写任务号与会话短号.md#第 3 批'),
    ('R3', 'remedy-compliant', 'pass', 'pass', '- 对照：没打补丁的主工作区跑同一条 `bash scripts/crash/crash-amplification.sh --item x` 同样退 78，日志里那一行「reason=没经服务给卡…退 78」：是改 GPU 服务交回路径那一路的会话在修的那一族'),
    ('R4', 'remedy-compliant', 'pass', 'pass', '- 改算键库那一批量读数：退 0'),
    ('R5', 'remedy-compliant', 'pass', 'pass', '- 每次 fsync 的记录数等于数据单元数（`seq 批 1` 是 8 条）'),
    ('R6', 'remedy-compliant', 'pass', 'pass', '- 几何敏感性：6 槽小环（`a6`）、64 槽环（A）'),
    ('R7', 'remedy-compliant', 'pass', 'pass', '- 甲：零轮，A/B 读数：任务 12400（未改绿、变异红）'),
    ('R8', 'remedy-compliant', 'pass', 'pass', '- 从 4G 起的有 1180 个任务，它们合计要撞 1013 次'),
    ('R9', 'remedy-compliant', 'pass', 'pass', '- mkfs 写的头一批字节就是它'),
    ('B1', 'exemption-borrowed', 'refuse', 'pass', '- 九项崩溃点重放连撞三次上限，A/B 读数：见上；同 20461、20814，任务 21036 已认领'),
    ('B2', 'exemption-borrowed', 'refuse', 'pass', '- 长跑拆分：无；任务 21036 同 20461、20814 在跑'),
    ('B3', 'exemption-borrowed', 'refuse', 'pass', '| 备注 | 它的批 105 在修，第 6 批交回 |'),
    ('B4', 'exemption-borrowed', 'refuse', 'pass', '- 对照（`任务 21036`）同样退 78，`(5b)` `批 105` 在修'),
    ('B5', 'exemption-borrowed', 'refuse', 'pass', '```\n- 任务 21036 (5b) 批 105 在修'),
    ('B6', 'exemption-borrowed', 'refuse', 'pass', '```\n\n摘录之后的正文：任务 21036 (5b) 批 105 在修。'),
    ('B7', 'exemption-borrowed', 'refuse', 'pass', '~~~ 任务 21036 (5b) 批 105 在修'),
    ('R1', 'remedy-followed', 'unclear', 'pass', '- 对照：主工作区跑同一条（`task-server/task log 21036`）同样退 78'),
    ('R2', 'remedy-followed', 'unclear', 'pass', '- 对照：主工作区跑同一条同样退 78（TASK_SERVER_RESULT task=21036 type=command-failed exit=78 attempts=1）'),
    ('F10', 'reserved-form-variant', 'pass', 'refuse', '- 甲：只补测试与变异行；零轮，**A/B 读数**：任务 12400'),
    ('F11', 'reserved-form-variant', 'pass', 'refuse', '- 甲：只补测试与变异行；零轮，A/B读数：任务 12400（未改绿、变异红）'),
    ('F12', 'indented-code', 'unclear', 'refuse', '    任务 21036 退 0'),
    ('F13', 'false-task-count', 'pass', 'refuse', '- 从 4G 起的任务 1180 个，它们合计要撞 1013 次'),
)

# (编号, 类, 按规则该拒还是该放, 腿量到的库判定, 仓里的文件, 行号, 那一行里必须有的片段)：真行在跑的那一刻从仓里取
REAL_LINES = (
    ('a', 'missed-task', 'refuse', 'pass', 'records/2026-11-01-判定库服务化.md', 473, '任务（5073–5075、5080）'),
    ('b', 'missed-task', 'refuse', 'pass', 'records/2026-11-12-subagent账本清欠.md', 63, '任务：14098'),
    ('c', 'missed-task', 'refuse', 'pass', 'records/2026-11-02-文档代码逐项审核.md', 549, '任务 5922/5924'),
    ('d', 'missed-task', 'refuse', 'pass', 'records/2026-11-16-三方豁免取消与存量补轮.md', 595, '任务 19336/19337'),
    ('e', 'missed-task', 'refuse', 'pass', 'records/2026-11-01-判定库服务化.md', 327, '同 3642/3644'),
    ('f', 'missed-task', 'refuse', 'pass', 'records/2026-11-11-欠账表按代码重梳调度.md', 88, '同 13495'),
    ('g', 'missed-task', 'refuse', 'pass', 'records/2026-10-31-task-server巡检.md', 13, '任务 #1748'),
    ('j', 'missed-short-code', 'refuse', 'pass', 'records/2026-10-18-里程碑三收尾与近期欠账.md', 166, '（b6 推的）'),
    ('k', 'missed-batch', 'refuse', 'pass', 'records/2026-10-19-里程碑四GPU垃圾回收开工.md', 22, '批 #2'),
    ('l', 'missed-batch', 'refuse', 'pass', 'records/2026-11-10-全量提交门禁收尾调度.md', 117, '批 #1'),
    ('m', 'missed-batch', 'refuse', 'pass', 'records/2026-10-13-Verus证明审计.md', 26, '第 7–13 批'),
    ('o', 'exemption-borrowed', 'refuse', 'pass', 'records/2026-10-23-lkmm绑定复用与依赖脚本修正.md', 18, '批 3 交回报告'),
    ('r', 'false-batch-size', 'pass', 'refuse', '.claude/kb/decisions/16-发布语义.md', 177, 'seq 批 1 是 8 条'),
    ('s', 'false-batch-size', 'pass', 'refuse', '.claude/kb/experiments/08-大文件与小文件是否该走不同写路径.md', 43, '攒批 64'),
    ('t', 'false-generic-ordinal', 'pass', 'refuse', '.claude/kb/experiments/100-系统配置的三段几何.md', 6, '第一批字节'),
    ('u', 'false-step-label', 'pass', 'refuse', '.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md', 416, '（a6）'),
    ('v', 'false-step-label', 'pass', 'refuse', '.claude/kb/layout/01-first-txn.md', 441, '（a1）'),
    ('w', 'reserved-form-variant', 'pass', 'refuse', 'research/prompts/device-library-write-many-code-r3-main-verification.md', 42, 'A/B 读数任务 12400'),
    ('x4', 'false-task-count', 'pass', 'refuse', 'records/2026-11-10-全量提交门禁收尾调度.md', 276, '任务 1180 个'),
)


def load_library(path):
    spec = importlib.util.spec_from_file_location('lib_forbidden_notations', path)
    library = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(library)
    return library


def verdict_of(library, text):
    hits = library.all_hits(FORM, text, JUDGED_AS)
    return ('refuse' if hits else 'pass'), ';'.join(f'{name}:{matched}' for _, _, name, matched in hits)


def real_line(repo, relative, number, must_contain):
    with open(f'{repo}/{relative}', encoding='utf-8') as handle:
        lines = handle.read().split('\n')
    if number > len(lines) or must_contain not in lines[number - 1]:
        return None
    return lines[number - 1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--library', required=True)
    parser.add_argument('--repo', required=True)
    arguments = parser.parse_args()
    library = load_library(arguments.library)
    disagreements, checked = 0, 0
    rows = [(case_id, kind, rule, measured, text, 'constructed') for case_id, kind, rule, measured, text in CONSTRUCTED]
    for case_id, kind, rule, measured, relative, number, must_contain in REAL_LINES:
        text = real_line(arguments.repo, relative, number, must_contain)
        if text is None:
            print(f'name=probe_case_missing id={case_id} source={relative}:{number} want_fragment={must_contain}')
            return 2
        rows.append((case_id, kind, rule, measured, text, f'{relative}:{number}'))
    for case_id, kind, rule, measured, text, source in rows:
        library_verdict, hits = verdict_of(library, text)
        agree = 'yes' if library_verdict == measured else 'no'
        checked += 1
        disagreements += agree == 'no'
        print(f'name=probe_case id={case_id} class={kind} rule={rule} library={library_verdict} measured={measured} agree={agree} '
              f'source={source} hits={hits or "-"}')
    wrong_way = sum(1 for _, _, rule, measured, _, _ in rows if rule in ('refuse', 'pass') and rule != measured)
    print(f'name=probe_summary cases={checked} disagreements={disagreements} rule_vs_library_differs={wrong_way}')
    return 1 if disagreements else 0


if __name__ == '__main__':
    sys.exit(main())
