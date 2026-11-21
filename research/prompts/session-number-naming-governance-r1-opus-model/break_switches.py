#!/usr/bin/env python3
"""攻方腿模型：钩子文件头点名的弄坏开关，打开后自证红的格是不是逐格等于文件头点名的格（正文第 6 问后半）；
另判自证的不变量缺口：变异钩子（去掉「只在续跑里」的条件）的自证红不红。

用法：python3 break_switches.py <被判树> <运行目录>
  读 <被判树>/.claude/hooks/session-number-stop-check.sh 文件头的开关表，读 rerun.sh 逐个跑出的
  <运行目录>/switch-<开关>.out/.rc、<运行目录>/selftest-plain.out/.rc、<运行目录>/selftest-mutant-loop-guard.out/.rc。
每行：name=switch-<开关> exit=<自证退出码> named=<文件头点名的格数> red=<实际红的格数> extra=<红了而没点名的> missing=<点名了而没红的> verdict=<ok|hit>
"""
import os
import re
import sys

HEADER_SWITCH = re.compile(r"^#\s+SESSION_NUMBER_STOP_CHECK_BREAK=(\S+)")
HEADER_LABELS = re.compile(r"「([^」]+)」")
RED_LINE = re.compile(r"✗ 自检：(.+?) 应当是")


def header_table(hook):
    table, current = {}, None
    with open(hook, encoding="utf-8") as handle:
        for line in handle:
            if not line.startswith("#"):
                break
            found = HEADER_SWITCH.match(line)
            if found:
                current = found.group(1)
                table[current] = []
                continue
            if current and re.match(r"^#\s{6,}「", line):
                table[current] += HEADER_LABELS.findall(line)
            elif current and not re.match(r"^#\s{6,}", line):
                current = None
    return table


def red_labels(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return [found.group(1) for found in RED_LINE.finditer(handle.read())]


def read_code(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read().strip()
    except OSError:
        return "-"


def main(tree, work):
    hook = os.path.join(tree, ".claude", "hooks", "session-number-stop-check.sh")
    table = header_table(hook)
    broken = 0
    plain_code = read_code(os.path.join(work, "selftest-plain.rc"))
    print(f"name=selftest-plain exit={plain_code} verdict={'control-ok' if plain_code == '0' else 'control-broken'}")
    broken += plain_code != "0"
    print(f"name=header-switches count={len(table)} switches={','.join(sorted(table))}")
    for switch, named in sorted(table.items()):
        out = os.path.join(work, f"switch-{switch}.out")
        if not os.path.isfile(out):
            print(f"name=switch-{switch} verdict=control-broken note=没有跑出 {out}")
            broken += 1
            continue
        red = red_labels(out)
        extra = [label for label in red if not any(label.startswith(prefix) or prefix.startswith(label) for prefix in named)]
        missing = [prefix for prefix in named if not any(label.startswith(prefix) or prefix.startswith(label) for label in red)]
        verdict = "ok" if not extra and not missing and red else "hit"
        print(f"name=switch-{switch} exit={read_code(os.path.join(work, f'switch-{switch}.rc'))} named={len(named)} red={len(red)} "
              f"extra={'|'.join(extra) or '-'} missing={'|'.join(missing) or '-'} verdict={verdict}")
    mutant_code = read_code(os.path.join(work, "selftest-mutant-loop-guard.rc"))
    mutant_out = os.path.join(work, "selftest-mutant-loop-guard.out")
    mutant_red = red_labels(mutant_out) if os.path.isfile(mutant_out) else []
    if mutant_code == "0":
        mutant_verdict = "hit"
    elif mutant_code == "1" and mutant_red:
        mutant_verdict = "ok"
    else:
        mutant_verdict = "control-broken"
        broken += 1
    print(f"name=selftest-mutant-loop-guard exit={mutant_code} red={len(mutant_red)} "
          f"verdict={mutant_verdict} note=变异去掉「只在续跑里」的条件；自证退 0 即变异活下来，退别的码而没有红行是装置没起来")
    return 1 if broken else 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    sys.exit(main(sys.argv[1], sys.argv[2]))
