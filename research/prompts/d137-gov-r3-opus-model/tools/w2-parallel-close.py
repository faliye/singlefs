#!/usr/bin/env python3
"""第二轮 W2 修补（同一份声明文件按写者分段）撑不撑得住：declare-parallel 不写写者，「此刻并行」收的是最近开的那一段、不一定是写它的那个程序的。
用法：w2-parallel-close.py <根>（import <根>/scripts/hosts/core_usage.py；只调它的 parse_declarations、judge_series、verdict_lines，不起子进程、不写文件）。
造的历史（第几秒，从树根起跑算）：
  第 0 秒   程序乙（写者 222）写「此刻串行（预计 7200 秒）」
  第 1000 秒 程序甲（写者 111）写「此刻串行：…」（没写预计秒数，算一个窗口 600 秒）
  第 1100 秒 程序乙写「此刻并行」：declare-parallel 写的行不带写者（core_usage.py 的 declare 那一支），照 parse 后的样子写
  之后程序甲卡住不收；整棵树一直 0.2 核。
对照：同一段历史，「此刻并行」那一行带上写者 222（假如 declare-parallel 写写者）。
期望（文件头「声明」：一个程序拖过它算数的秒数，别的程序的声明不替它续命）：程序甲第 1600 秒起过期，第 2400、4800 秒的窗口解释不了、该叫醒。"""
import os
import sys

sys.path.insert(0, os.path.join(sys.argv[1], "scripts", "hosts"))
import core_usage as cu  # noqa: E402

origin = 1_000_000.0
registered = 31


def series(now):
    moments = list(range(300, int(now), 300)) + [now]
    return [{"root": 1, "time": origin + m, "started": origin, "cpu": 0.2 * m, "peer_drivers": 0, "thread_cpu": {"1/0/0": 0.2 * m}} for m in moments]


def text(parallel_writer):
    tail = f"\t{cu.WRITER_MARK}{parallel_writer}" if parallel_writer else ""
    return (f"{origin}\t{cu.SERIAL_MARK_WORD}（预计 7200 秒）：程序乙的计时窗口必须单线程\t{cu.WRITER_MARK}222\n"
            f"{origin + 1000}\t{cu.SERIAL_MARK}程序甲独占共享状态必须串行\t{cu.WRITER_MARK}111\n"
            f"{origin + 1100}\t{cu.PARALLEL_MARK}{tail}\n")


for label, writer in (("此刻并行不带写者（declare-parallel 今天的写法）", None), ("对照：此刻并行带写者 222", "222")):
    for now in (2400, 4800, 7000, 7500):
        verdict = cu.judge_series(series(now), *cu.parse_declarations(text(writer)), registered, 600)
        line = cu.verdict_lines(verdict, "本机")[0]
        print(f"name=w2_parallel_close case={label} now={now} status={verdict['status']} declared_in_window={verdict['declared_in_window']:.0f} "
              f"excused={verdict['declaration_excused']} reasons={verdict['reasons']}")
        print(f"  line: {line[:220]}")
