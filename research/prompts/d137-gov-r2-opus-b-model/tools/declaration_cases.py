#!/usr/bin/env python3
"""d137-gov-r2 攻方腿（opus-b）格 2、格 5：拿仓里的 scripts/hosts/core_usage.py 算给定输入的结果，不起子进程、不读 /proc。
用法：python3 declaration_cases.py --repo <仓根>
每一格打一行 name=<格> …，结果只取决于输入（随机取样固定种子 137）。"""
import importlib.util
import random
import sys

sys.dont_write_bytecode = True
REGISTERED = 31          # .claude/heavy-thread-default 的登记值（阈值 = 31 / 4 = 7.75）
WINDOW = 600


def load(repo):
    spec = importlib.util.spec_from_file_location("core_usage_probe", f"{repo}/scripts/hosts/core_usage.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def samples_until(now, cores, step=300):
    """树根在 0 秒起跑、一条线程以 cores 的速率一直算；每 step 秒取一次样，最后一次在 now。"""
    times = list(range(step, int(now), step)) + [now]
    return [{"time": float(t), "cpu": cores * t, "thread_cpu": {"1/1/0": cores * t}, "root": 1, "started": 0.0} for t in times]


def judge(core, text, now, cores):
    entries, problems = core.parse_declarations(text)
    verdict = core.judge_series(samples_until(now, cores), entries, problems, REGISTERED, WINDOW)
    line = core.verdict_lines(verdict, "probe ")[0]
    return verdict, line


def show(name, verdict, line, extra=""):
    marks = []
    for word in ("声明解释不了", "声明解释得了", "这一窗口里没有声明串行段", "拖过了它算数的", "写坏的"):
        if word in line:
            marks.append(word)
    print(f"name={name} {extra}status={verdict['status']} cores={verdict['cores']:.3f} span={verdict['span']:.0f} "
          f"declared={verdict['declared_in_window']:.1f} excused={verdict['declaration_excused']} problems={len(verdict['problems'])} "
          f"line_has={'|'.join(marks) or '-'}")


def main(repo):
    core = load(repo)
    reason = "计时窗口必须单线程跑完"
    # 格 2-甲：给定 cores、span、declared 直接调 declaration_excuses_low，与文件头公式对照
    for cores, declared in ((1.0, 0), (1.0, 1), (1.0, 500), (1.0, 522), (1.0, 523), (1.0, 600), (0.0, 599.9), (0.0, 600), (7.8, 0)):
        reading = {"cores": cores, "span": 600.0, "threshold": REGISTERED / 4, "declared_in_window": float(declared)}
        undeclared = 600.0 - declared
        formula = declared > 0 and (undeclared <= 0 or cores * 600.0 / undeclared >= REGISTERED / 4)
        print(f"name=excuse_formula cores={cores} span=600 declared={declared} excused={core.declaration_excuses_low(reading)} header_formula={formula}")
    # 格 2-乙：预计秒数的边界（声明写在第 1000 秒，从不闭合；1 核）
    for expected_text, label in (("", "none"), ("（预计 0 秒）", "zero"), ("（预计 07 秒）", "leading-zero"), ("（预计 7200 秒）", "7200"),
                                 ("（预计 7201 秒）", "7201"), ("（预计 99999 秒）", "99999"), ("（预计 900秒）", "no-space")):
        text = f"1000\t此刻串行{expected_text}：{reason}\n"
        for now in (1500, 1600, 1677, 1700, 8200, 8277, 8300, 8800, 9000):
            verdict, line = judge(core, text, now, 1.0)
            show("expected_seconds", verdict, line, f"expected={label} now={now} ")
    # 格 2-丙：声明跨窗口（第 100 秒写、预计 900 秒、从不闭合；0 核卡死）
    for now in (700, 900, 1000, 1100, 1300):
        verdict, line = judge(core, f"100\t此刻串行（预计 900 秒）：{reason}\n", now, 0.0)
        show("across_windows", verdict, line, f"now={now} ")
    # 格 2-丁：几个写者共用一份声明文件（同一棵树里几个程序都往 SINGLEFS_SERIAL_DECLARATIONS 写）
    single = f"0\t此刻串行（预计 7200 秒）：{reason}\n"
    shared_close = single + "3000\t此刻串行：另一个程序的串行段写满八个字\n3100\t此刻并行\n"
    shared_extend = single + "7000\t此刻串行（预计 7200 秒）：另一个程序的串行段写满八个字\n"
    for name, text, now in (("writer_alone", single, 3600), ("writer_closed_by_other", shared_close, 3600),
                            ("writer_alone", single, 8000), ("writer_extended_by_other", shared_extend, 8000),
                            ("writer_extended_by_other", shared_extend, 14000), ("writer_extended_by_other", shared_extend, 14300)):
        verdict, line = judge(core, text, now, 0.2)
        show("shared_file", verdict, line, f"case={name} now={now} ")
    # 格 2-戊：写坏的声明文件
    for name, text in (("not-seconds", f"abc\t此刻串行：{reason}\n"), ("milliseconds", f"1500000\t此刻串行：{reason}\n"),
                       ("parallel-typo-keeps-open", f"1000\t此刻串行（预计 7200 秒）：{reason}\n1100\t此刻 并行\n"),
                       ("short-reason", "1000\t此刻串行：太短\n"), ("binary-garbage", "\x00\xff\t\x01\n")):
        verdict, line = judge(core, text, 1500, 0.0)
        show("broken_file", verdict, line, f"case={name} ")
    # 格 2-己：告警记录那一半拿 alert_record 取整之后的读数判，与判的那一行（verdict 原值）比
    rng = random.Random(137)
    disagree = {"line_wakes_record_silent": 0, "line_silent_record_wakes": 0}
    example = {}
    trials = 200000
    for _ in range(trials):
        span = 600.0 + rng.random() * 300.0
        declared = rng.random() * span
        cores = rng.random() * 8.0
        verdict = {"status": "low", "cores": cores, "parallel": cores, "registered": REGISTERED, "threshold": REGISTERED / 4,
                   "window": WINDOW, "span": span, "declared_in_window": declared, "reasons": [reason]}
        direct = core.declaration_excuses_low(verdict)
        record = core.alert_record(verdict, "probe ", "detached-run.sh watch", "probe", 1, "停下", "✗ probe")
        rounded = core.declaration_excuses_low(record)
        if direct != rounded:
            key = "line_silent_record_wakes" if direct else "line_wakes_record_silent"
            disagree[key] += 1
            example.setdefault(key, (cores, span, declared, record["cores"], record["span"], record["declared_in_window"]))
    print(f"name=record_rounding trials={trials} line_wakes_record_silent={disagree['line_wakes_record_silent']} "
          f"line_silent_record_wakes={disagree['line_silent_record_wakes']}")
    for key, (cores, span, declared, rc, rs, rd) in sorted(example.items()):
        print(f"name=record_rounding_example kind={key} cores={cores!r} span={span!r} declared={declared!r} "
              f"record_cores={rc} record_span={rs} record_declared={rd}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3 or sys.argv[1] != "--repo":
        print("用法：declaration_cases.py --repo <仓根>", file=sys.stderr)
        sys.exit(2)
    sys.exit(main(sys.argv[2]))
