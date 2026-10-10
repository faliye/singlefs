#!/usr/bin/env python3
"""m3-batch29-code-r1 攻方 L2：实例表链片数上限的读码模型（推的，不跑 GPU）。

把冻结副本里三处链循环的控制流照抄成 Python，在一组「链长 × 根的角色 × 中途断开 × 与最新根共用一片」的形上逐形算：
  走读内核 w_walk_instance_table_chain（crash_judge_checker.wgsl）：上限 256 片、走过的单元不再走（上下文 0 由最新根与候选根共用，
      被抛弃的根各起一份、候选集外的根收割那一遍共用一份、F 之下的根各起一份）、读不出 / 身份不对 / 末片就停；走满记站点 61。
  扫描内核 r_instance_pages_after_first（候选根，c_candidate_count >= 2 且有已释放记录时）与 r_instance_rows（每条根，I-3.12）：
      上限 256 片，读不出 / 身份不对就交回读不出，走满记站点 37 / 38（第 29 批加的两句）。
  CPU（walk.rs）：读链不设上限。
每形打印一行 RESULT：GPU 的结局（判了 / 判不了的站点）、CPU 读到的每条根的行是否与 GPU 判了时读到的相同；
再按「删掉第 29 批那两句」（变异 M）各算一次：M 下 GPU 判了、而某条根的链 CPU 读得比 GPU 长 ⇒ 这一形能抓到 M。
用法：python3 l2_chain_model.py
"""
import itertools

CAP = 256


class Page:
    def __init__(self, name, container, readable=True):
        self.name = name
        self.container = container
        self.readable = readable
        self.next = None


def chain(prefix, length, break_at=None):
    pages = [Page(f"{prefix}{i}", i, readable=(i != break_at)) for i in range(length)]
    for a, b in zip(pages, pages[1:]):
        a.next = b
    return pages


def walk_kernel(first, visited):
    """w_walk_instance_table_chain：返回 (走满 256 片没有)。"""
    p = first
    page = 0
    while page < CAP:
        if not p.readable:
            return False
        if p.name in visited:
            return False
        visited.add(p.name)
        if p.container != page:
            return False
        if p.next is None:
            return False
        p = p.next
        page += 1
    return True


def scan_kernel(first, with_batch29_lines):
    """r_instance_rows / r_instance_pages_after_first 的控制流：返回 ('read', 读到的片数) / ('unreadable', k) / ('undecidable', None) / ('truncated', 256)。"""
    p = first
    for step in range(CAP):
        if not p.readable or p.container != step:
            return ("unreadable", step)
        if p.next is None:
            return ("read", step + 1)
        p = p.next
    return ("undecidable", None) if with_batch29_lines else ("truncated", CAP)


def cpu_reader(first):
    p = first
    step = 0
    while True:
        if not p.readable or p.container != step:
            return ("unreadable", step)
        if p.next is None:
            return ("read", step + 1)
        p = p.next
        step += 1


def run(shape, with_batch29_lines):
    newest, other, role = shape["newest"], shape["other"], shape["role"]
    # 走读内核：最新根先走（上下文 0），候选根接着在同一个上下文里走；被抛弃的根、F 之下的根各起一份，收割那一遍对候选集外的根共用一份
    context0 = set()
    if walk_kernel(newest[0], context0):
        return "undecidable-61"
    if role == "candidate":
        if walk_kernel(other[0], context0):
            return "undecidable-61"
    else:
        if walk_kernel(other[0], set()):  # 被抛弃的根（ABANDONED 阶段）或 F 之下的根（ROOTS_BELOW_THE_FLOOR 阶段）各起一份
            return "undecidable-61"
        if walk_kernel(other[0], set()):  # 收割那一遍（HARVEST_ROOTS_OUTSIDE_THE_CANDIDATES）
            return "undecidable-61"
    # 扫描内核
    roots = [("newest", newest), (role, other)]
    if role == "candidate":
        for _, pages in roots:  # r_references_of_root(every = true) → r_instance_pages_after_first
            outcome = scan_kernel(pages[0], with_batch29_lines)
            if outcome[0] == "undecidable":
                return "undecidable-37"
    truncated = []
    for name, pages in roots:  # I-3.12 每条根 r_instance_rows
        outcome = scan_kernel(pages[0], with_batch29_lines)
        if outcome[0] == "undecidable":
            return "undecidable-38"
        if outcome[0] == "truncated" and cpu_reader(pages[0])[0] == "read":
            truncated.append(name)  # GPU 读满 256 片交回读不出、CPU 读到末片：两边读到的行不同（CPU 也在那之后读不出的，两边都不判，不算）
    cpu_differs = [name for name, pages in roots if outcome_of_cpu_differs(pages)]
    if truncated:
        return "decided-but-rows-truncated:" + ",".join(truncated)
    return "decided" + ("" if not cpu_differs else "-cpu-reads-more:" + ",".join(cpu_differs))


def outcome_of_cpu_differs(pages):
    cpu = cpu_reader(pages[0])
    gpu = scan_kernel(pages[0], True)
    return gpu[0] != "undecidable" and cpu != gpu


def shapes():
    for newest_length, other_length, role, break_at, share in itertools.product(
        [1, 200, 255, 256, 257], [255, 256, 257, 300], ["candidate", "abandoned", "below-floor"], [None, 100, 255, 256], [None, (1, 100), (1, 250)]
    ):
        newest = chain("N", newest_length)
        other = chain("R", other_length, break_at)
        if share is not None:
            j, k = share  # 最新根第 j 片的链指针指到这条根的第 k 片（那一片的容器号是 k：在最新根那条链上身份不对）
            if j >= newest_length or k >= other_length:
                continue
            newest[j - 1].next = other[k]
            newest = newest[:j] + [other[k]]
        yield {"newest": newest, "other": other, "role": role, "name": f"newest={newest_length} other={other_length} role={role} break={break_at} share={share}"}


def main():
    counts = {}
    killers = []
    for shape in shapes():
        with_lines = run(shape, True)
        without = run(shape, False)
        counts[(with_lines, without)] = counts.get((with_lines, without), 0) + 1
        if without.startswith("decided-but-rows-truncated"):
            killers.append(shape["name"] + " -> " + with_lines + " / M: " + without)
        print(f"RESULT l2 {shape['name']} batch29={with_lines} mutant_without_the_two_lines={without}")
    for (with_lines, without), count in sorted(counts.items()):
        print(f"RESULT l2-summary batch29={with_lines} mutant={without} shapes={count}")
    print(f"RESULT l2-killers count={len(killers)}")
    for line in killers[:12]:
        print(f"RESULT l2-killer {line}")


if __name__ == "__main__":
    main()
