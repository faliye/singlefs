"""E171 第 1 次重跑第一段·新树：按序跑的脚本每一步跑完之后的判法（只读产物与导出，不写仓）。退 0 是这一步绿，退 1 是红（打出哪一项不符），退 2 是用法错。

  verdict <产物> <键>=<值> …            名为 verdict 的结果行（取最后一条）里各键等于给的值；值写成 `>0` 时要大于 0
  line <产物> <行名> <键>=<值> …        名为 <行名> 的结果行里，有一条同时满足这几项（值可写 `>0`、`prefix:<前缀>`）
  calls_differ <基线产物> <对照产物> <字段> <流名> <期望不同的调用号，逗号隔开，空写 none>
                                       两份产物 name=call 行（那条流）按调用号比 <字段>，不同的调用号要恰是期望的那几个；另打每个不同那次对照里的 o1_comparison
  call_o1 <产物> <流名> <调用号> <期望的 o1_comparison>
  o3 <执行器产物> <流名，逗号隔开>      完成标记在、出错 0、摘要对不上 0、导出问题 0、严格与暂代判「不在」都是 0、给的每条流都有态
  z1 <导出目录一> <导出目录二>          作废 Z1 的快查：三条流 oracle-verdicts.tsv、call-outcomes.tsv、writable-mount-probe.tsv、ops.tsv、states.tsv 逐字节同
                                       （call-return-values.tsv 与 impl-verdicts.tsv 去掉耗时列那一比归比对器）
  rows <驱动产物>                      驱动的 freeze_check 前后都是 differences=0；每个 row 行 status=ok、device_exit=0 且 o3_status=ran；
                                       device_exit=253（超时）的行不算红、打出行号（交给超时重跑那一步）
  rows_without_executor <驱动产物>     第二段整表（--no-executor，rows_orchestrator_segment_two.py）：没有 freeze_check 行（不跑执行器的驱动不打）；
                                       每个 row 行要么 status=ok、device_exit 是 0 或 253、device_exit=0 时 o3_status=skipped_by_request，
                                       要么 status 是登记第一节读法 5 不进分母的那几类（compile_failed、not_found、ambiguous_name、anchor_not_unique）且 device_exit=-；
                                       253（超时）与没编成、没找着的行不算红，逐类打出行号；250–252、254（作废 Z3：包装自己的结局，驱动已重跑一次仍撞）同 rows 一样算红，
                                       另把行号打在 wrapper_outcomes=（多机时多半是排队或被总上限挤掉，交主 agent 定删行目录重跑还是照 Z3 记进超时一类）
"""
import os
import sys

sys.path.insert(0, '/home/fy5090/code/singlefs/scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes')
import frozen_o3  # noqa: E402

STREAMS = ('new_pool_file_creation', 'fixed_script_through_e_then_unmount', 'first_file_unlinked_then_inodes_across_one_leaf')
Z1_FILES = ('oracle-verdicts.tsv', 'call-outcomes.tsv', 'writable-mount-probe.tsv', 'ops.tsv', 'states.tsv')


def fields_of(line):
    return dict(token.split('=', 1) for token in line.split() if '=' in token)


def result_lines(path, name):
    with open(path, encoding='utf-8') as handle:
        return [fields_of(line) for line in handle if line.startswith('E7RESULT ') and f' name={name} ' in line + ' ']


def matches(actual, expected):
    if actual is None:
        return False
    if expected == '>0':
        return actual.isdigit() and int(actual) > 0
    if expected.startswith('prefix:'):
        return actual.startswith(expected[len('prefix:'):])
    return actual == expected


def check_pairs(found, pairs):
    wrong = []
    for pair in pairs:
        key, expected = pair.split('=', 1)
        if not matches(found.get(key), expected):
            wrong.append(f'{key}={found.get(key)}（要 {expected}）')
    return wrong


def verdict(path, pairs):
    found = result_lines(path, 'verdict')
    if not found:
        print(f'✗ {path} 没有 verdict 行')
        return 1
    wrong = check_pairs(found[-1], pairs)
    print(('✗ ' + '；'.join(wrong)) if wrong else f'✓ verdict 符合 {len(pairs)} 项')
    return 1 if wrong else 0


def line(path, name, pairs):
    found = result_lines(path, name)
    if any(not check_pairs(fields, pairs) for fields in found):
        print(f'✓ 有一条 name={name} 符合 {len(pairs)} 项')
        return 0
    print(f'✗ name={name} 共 {len(found)} 条，没有一条同时符合 {pairs}')
    return 1


def calls_of(path, stream):
    return {int(fields['call']): fields for fields in result_lines(path, 'call') if fields.get('stream') == stream}


def calls_differ(baseline, control, field, stream, expected):
    left, right = calls_of(baseline, stream), calls_of(control, stream)
    differing = sorted(index for index in set(left) | set(right) if left.get(index, {}).get(field) != right.get(index, {}).get(field))
    wanted = [] if expected == 'none' else [int(token) for token in expected.split(',')]
    o1 = {index: right.get(index, {}).get('o1_comparison') for index in differing}
    print(f'{field} 不同的调用 {differing}（要 {wanted}）；对照里那几次的 o1_comparison {o1}')
    return 0 if differing == wanted else 1


def call_o1(path, stream, index, expected):
    actual = calls_of(path, stream).get(int(index), {}).get('o1_comparison')
    print(f'{stream} 第 {index} 次调用 o1_comparison={actual}（要 {expected}）')
    return 0 if actual == expected else 1


def o3(path, streams):
    reading = frozen_o3.read_executor_output(path)
    strict_not_in = sum(strict == 'not_in' for strict, _ in reading.verdicts.values())
    provisional_not_in = sum(provisional == 'not_in' for _, provisional in reading.verdicts.values())
    per_stream = {stream: sum(key[0] == stream for key in reading.verdicts) for stream in streams.split(',')}
    print(f'complete={reading.complete} states={reading.state_lines} per_stream={per_stream} strict_not_in={strict_not_in} '
          f'provisional_not_in={provisional_not_in} errors={reading.error_states} digest_mismatches={reading.digest_mismatch_states} '
          f'dump_problems={reading.dump_problems} base_unrecoverable={reading.base_unrecoverable_streams}')
    good = (reading.complete and reading.error_states == 0 and reading.digest_mismatch_states == 0 and reading.dump_problems == 0
            and strict_not_in == 0 and provisional_not_in == 0 and all(count > 0 for count in per_stream.values()))
    return 0 if good else 1


def z1(first, second):
    differing, compared = [], 0
    for stream in STREAMS:
        for name in Z1_FILES:
            left, right = os.path.join(first, stream, name), os.path.join(second, stream, name)
            if not os.path.exists(left) and not os.path.exists(right):
                continue
            compared += 1
            if not (os.path.exists(left) and os.path.exists(right)) or open(left, 'rb').read() != open(right, 'rb').read():
                differing.append(f'{stream}/{name}')
    print(f'z1_quick compared={compared} differing={len(differing)} {differing}')
    return 0 if compared > 0 and not differing else 1


def rows(path):
    with open(path, encoding='utf-8') as handle:
        lines = handle.read().splitlines()
    freeze = [line for line in lines if line.startswith('freeze_check ')]
    row_fields = [fields_of(line) for line in lines if line.startswith('row ')]
    bad = [fields for fields in row_fields if fields.get('status') != 'ok' or fields.get('device_exit') not in ('0', '253')
           or (fields.get('device_exit') == '0' and fields.get('o3_status') != 'ran')]
    timed_out = [fields.get('row_number') for fields in row_fields if fields.get('device_exit') == '253']
    freeze_good = len(freeze) == 2 and all(' differences=0 ' in line + ' ' for line in freeze)
    print(f'rows={len(row_fields)} not_ok={[(f.get("row_number"), f.get("status"), f.get("device_exit")) for f in bad]} '
          f'timed_out={",".join(timed_out) or "none"} freeze_checks_clean={freeze_good}')
    return 0 if row_fields and not bad and freeze_good else 1


NOT_LOCATED_STATUSES = ('compile_failed', 'not_found', 'ambiguous_name', 'anchor_not_unique')
WRAPPER_OUTCOMES = ('250', '251', '252', '254')


def rows_without_executor(path):
    with open(path, encoding='utf-8') as handle:
        lines = handle.read().splitlines()
    freeze = [line for line in lines if line.startswith('freeze_check ')]
    row_fields = [fields_of(line) for line in lines if line.startswith('row ')]

    def acceptable(fields):
        if fields.get('status') == 'ok':
            exit_code = fields.get('device_exit')
            return exit_code in ('0', '253') and (exit_code != '0' or fields.get('o3_status') == 'skipped_by_request')
        return fields.get('status') in NOT_LOCATED_STATUSES and fields.get('device_exit') == '-'

    bad = [fields for fields in row_fields if not acceptable(fields)]

    def numbers(predicate):
        return ','.join(fields.get('row_number', '?') for fields in row_fields if predicate(fields)) or 'none'

    print(f'rows={len(row_fields)} not_ok={[(f.get("row_number"), f.get("status"), f.get("device_exit"), f.get("o3_status")) for f in bad]} '
          f'timed_out={numbers(lambda f: f.get("device_exit") == "253")} '
          f'wrapper_outcomes={numbers(lambda f: f.get("device_exit") in WRAPPER_OUTCOMES)} '
          f'not_located={numbers(lambda f: f.get("status") in NOT_LOCATED_STATUSES)} freeze_lines={len(freeze)}')
    return 0 if row_fields and not bad and not freeze else 1


def main(arguments):
    command, rest = arguments[0], arguments[1:]
    if command == 'verdict':
        return verdict(rest[0], rest[1:])
    if command == 'line':
        return line(rest[0], rest[1], rest[2:])
    if command == 'calls_differ':
        return calls_differ(*rest)
    if command == 'call_o1':
        return call_o1(*rest)
    if command == 'o3':
        return o3(*rest)
    if command == 'z1':
        return z1(*rest)
    if command == 'rows':
        return rows(*rest)
    if command == 'rows_without_executor':
        return rows_without_executor(*rest)
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
