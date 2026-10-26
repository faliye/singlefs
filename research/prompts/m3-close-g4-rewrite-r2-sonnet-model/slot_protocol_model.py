#!/usr/bin/env python3
"""单盘系统配置槽写协议的抽象模型（辩方腿 m3-close-g4-rewrite-r2）。

只模拟一块盘的两个系统配置槽。读法抄自 crates/singlefs-core/src/transaction.rs 的 write_system_configuration_slot：
写时读盘，取这块盘自证过的最大世代号，世代号 = 它 + 1（一份都没有时从 1 起），落的槽 = 世代号 mod 2。
一次可写挂载的写序，段与段之间是池屏障：取号两轮各一段，之后每次发布末尾的轮换各一段（暖机、写行、用户发布、抬 F 的槽写都是「一段里一个槽写」）。
崩溃状态：第 k 段之前整段持久，第 k 段里每个写取三态（P 落了 / N 没落 / T 撕裂，槽读不出自证），第 k 段之后一个没落。
崩溃之后再起一次可写挂载，再崩，共 depth 次。判据：每个崩溃状态上这块盘至少一个槽自证过。
这不是核心层的代码，也不调它：它只证「按这一读法写出来的协议」不留无槽盘，并用对照臂证明模型分辨得出坏协议。
"""
import itertools

# 槽状态：世代号（自证过），或 None（空白、清零或撕裂，读得出但自证不过）。


def valid(slots):
    return [g for g in slots if g is not None]


def run_without_crash(start, programme, hide_newest=False):
    """无崩溃执行 programme，交回每段的写 [(种类, 槽号, 世代号或 None)]。
    种类：'w' 一次轮换式槽写；'z' 把上一个 'w' 的另一槽清零（对照臂 MUT-B1）。
    hide_newest：读故障臂，这次运行里自证过、世代号最大的那一槽读回全 0，直到运行自己写到它为止。"""
    slots = list(start)
    hidden = None
    if hide_newest and valid(slots):
        hidden = slots.index(max(valid(slots)))
    segments, last_pos = [], None
    for segment in programme:
        writes = []
        for kind in segment:
            if kind == "w":
                seen = [g for i, g in enumerate(slots) if g is not None and i != hidden]
                gen = (max(seen) if seen else 0) + 1
                pos = gen % 2
                slots[pos] = gen
                if pos == hidden:
                    hidden = None
                last_pos = pos
                writes.append(("w", pos, gen))
            else:
                pos = 1 - last_pos
                slots[pos] = None
                writes.append(("z", pos, None))
        segments.append(writes)
    return segments


def apply_write(state, write, outcome):
    kind, pos, gen = write
    if outcome == "P":
        state[pos] = gen
    elif outcome == "T":
        state[pos] = None
    return state


def crash_states(start, programme, hide_newest=False, outcomes="PNT"):
    """programme 的全部崩溃状态（含跑完后的终态），交回 (槽, 描述)。
    outcomes：每个写取哪几态。"PNT" 是崩溃点重放的枚举域（撕裂并进可摆）；"PN" 是崩溃注入第二、三截的抽样域（整写子集，撕裂并进没持久，
    crates/singlefs-checker-tier/src/crash_injection.rs 文件头第 5 行）。"""
    segments = run_without_crash(start, programme, hide_newest)
    base = list(start)
    for k, writes in enumerate(segments):
        for choice in itertools.product(outcomes, repeat=len(writes)):
            last = k == len(segments) - 1
            if all(c == "P" for c in choice) and not last:
                continue  # 全落等于下一段的空子集，由下一段摆
            state = list(base)
            for write, c in zip(writes, choice):
                apply_write(state, write, c)
            yield tuple(state), f"seg{k}:{''.join(choice)}"
        for write in writes:
            apply_write(base, write, "P")


def mount_programme(publishes, merged_rounds=False, mut_b1=False):
    """取号两轮（段间屏障）加 publishes 次发布末尾轮换。merged_rounds：对照臂，两轮并成一段（少了轮间屏障）。"""
    one = ["w", "z"] if mut_b1 else ["w"]
    rounds = [one + one] if merged_rounds else [one, one]
    return rounds + [list(one) for _ in range(publishes)]
