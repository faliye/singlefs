#!/usr/bin/env python3
"""m3-conc-r1 攻方（接手段）：c58_alarm_model.py 第三部分把负载的条目数写死成 base = 60 / 0.6·T。
这里把由测试作者决定的那几步放开扫：自证负载的基线条目数 base、测试盘在屏障里等的时长 latency、比较符号，
看每条自证在正确实现上绿不绿、抓不抓得到五个坏实现，结论随不随 base 翻。只 import c58_alarm_model 的定义，不改它。
用法：c58_selfproof_sweep.py
"""
import os
import sys

sys.dont_write_bytecode = True  # 不在模型目录里留 __pycache__
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import c58_alarm_model as m  # noqa: E402

BASES = list(range(1, 40001))                      # 自证负载的基线条目数（翻倍那一次 = 2·base）
LATENCIES = [0.0, 0.1, 50.0, 100.0, 359.1, 589.6]  # 0 = 内存盘 no-op 屏障；0.1 = tmpfs；后两个 = E44 与今天的 NVMe


def literal(alarm, base, latency):
    # C58 字面：只造「翻倍」那一面，检查必须红
    return alarm.red([2 * base], latency)


def two_sided(alarm, base, latency):
    # 收严一：基线不红 + 翻倍红，交点走生产那条路（不经测试开关）
    return (not alarm.red([base], latency)) and alarm.red([2 * base], latency)


def batching(alarm, base, latency):
    # 收严三：同一次 checkpoint 装两个各 base 条的改动（要 D16 已定项 10 攒批落地才造得出）
    return (not alarm.red([base], latency)) and alarm.red([base, base], latency)


TESTS = {"literal": literal, "two_sided": two_sided, "batching": batching}


def main():
    for strict in (True, False):
        muts = m.mutants(strict)
        comparator = "gt" if strict else "ge"
        for latency in LATENCIES:
            for name, test in TESTS.items():
                usable = 0
                caught_sets = {}
                for base in BASES:
                    if not test(muts[0], base, latency):
                        continue
                    usable += 1
                    caught = tuple(x.name for x in muts[1:] if not test(x, base, latency))
                    caught_sets.setdefault(caught, []).append(base)
                if not caught_sets:
                    print(f"E name=sweep comparator={comparator} latency_us={latency} test={name} "
                          f"bases={len(BASES)} usable_bases=0 caught=- base_range=-")
                    continue
                for caught, bases in sorted(caught_sets.items(), key=lambda kv: kv[1][0]):
                    missed = [x.name for x in muts[1:] if x.name not in caught]
                    print(f"E name=sweep comparator={comparator} latency_us={latency} test={name} "
                          f"bases={len(BASES)} usable_bases={usable} base_min={bases[0]} base_max={bases[-1]} "
                          f"n={len(bases)} caught={','.join(caught) or '-'} missed={','.join(missed) or '-'}")


if __name__ == "__main__":
    main()
