#!/usr/bin/env python3
"""探针：三张排除表「只缩不涨」照拆分族比时（lib_lag_split_family.py 的 keyed_family_growth），同族新文件里照抄基准入口已有的锚串、
登几行都不算涨；同一份文件里的 family_growth（按计数比之和）对同一批行会判涨。判的是用户定案「只对这一批放开一次，之后照旧只缩」
在拆分之后还守不守得住。
用法：probe-family-growth.py <冻结副本>（里面要有 scripts/gate/lib_lag_split_family.py）
输出：每格一行 name=family_growth_case case=<格名> base_rows=<基准行数> current_rows=<此刻行数> keyed_grown=<keyed_family_growth 交出的行数> counted_grown=<family_growth 交出的路径数>
退出码：0 跑完（只打读数）；2 用法错或载不进那份库。"""
import hashlib
import importlib.util
import os
import sys


def load(frozen):
    path = os.path.join(frozen, "scripts", "gate", "lib_lag_split_family.py")
    specification = importlib.util.spec_from_file_location("lib_lag_split_family", path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    with open(path, "rb") as handle:
        digest = hashlib.sha256(handle.read()).hexdigest()
    return module, digest


def counted(rows):
    counts = {}
    for path, _anchor in rows:
        counts[path] = counts.get(path, 0) + 1
    return counts


def main(arguments):
    if len(arguments) != 1:
        print("用法：probe-family-growth.py <冻结副本>", file=sys.stderr)
        return 2
    try:
        family, digest = load(arguments[0])
    except (OSError, ImportError, SyntaxError) as error:
        print(f"  ✗ 载不进 lib_lag_split_family.py：{error}", file=sys.stderr)
        return 2
    print(f"name=family_growth_sources lib_lag_split_family_sha256={digest}")
    core_count = "os.cpu_count(), len(os.sched_getaffinity(0))"
    nproc_word = '"nproc"'
    base = [("scripts/gate/admission.py", core_count), ("scripts/gate/admission.py", nproc_word)]
    cases = {
        # 拆分照实挪：两行各归一份拆出的文件，行数不变
        "split-moves-rows": [("scripts/gate/admission_base.py", core_count), ("scripts/gate/admission_base.py", nproc_word)],
        # 此刻表里的样子：同一锚串在族里两份文件各登一行（.claude/heavy-thread-exempt 的 admission_base.py 与 admission_selftest_cells.py 两行 "nproc"）
        "same-anchor-in-two-members": [("scripts/gate/admission_base.py", core_count), ("scripts/gate/admission_base.py", nproc_word),
                                       ("scripts/gate/admission_selftest_cells.py", nproc_word)],
        # 拆分之后新建一份同族文件，照抄基准里的两个锚串各登一行：被放过的是新文件里真取核数的写法
        "new-member-copies-anchors": [("scripts/gate/admission_base.py", core_count), ("scripts/gate/admission_base.py", nproc_word),
                                      ("scripts/gate/admission_thread_probe.py", core_count), ("scripts/gate/admission_thread_probe.py", nproc_word)],
        # 对照：族外的新文件照抄同一锚串
        "outside-family-copies-anchor": [("scripts/gate/admission_base.py", core_count), ("scripts/gate/admission_base.py", nproc_word),
                                         ("scripts/gate/thread_probe.py", core_count)],
        # 对照：族里新文件写一个基准里没有的锚串
        "new-member-new-anchor": [("scripts/gate/admission_base.py", core_count), ("scripts/gate/admission_base.py", nproc_word),
                                  ("scripts/gate/admission_thread_probe.py", "multiprocessing.cpu_count()")],
    }
    base_keys = set(base)
    for name, current in cases.items():
        keyed = family.keyed_family_growth(base_keys, current)
        growth = family.family_growth(counted(base), counted(current))
        print(f"name=family_growth_case case={name} base_rows={len(base)} current_rows={len(current)} "
              f"keyed_grown={len(keyed)} counted_grown={len(growth)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
