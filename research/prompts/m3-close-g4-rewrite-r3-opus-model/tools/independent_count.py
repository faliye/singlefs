#!/usr/bin/env python3
"""第二条路（不共用原型与枚举库的代码）：按 crash.rs 文件头写的规则（段内每次写各取几态、任意组合；长于一个扇区又罩住旧内容的
原地覆写多一态「撕裂」；每段去掉「整段全持久」那一态，流末加一态全持久），只拿各段写的形状与每次系统配置槽写落在哪块盘哪一槽、
写的是新槽还是全 0，手算每条流的状态数、撕裂态数、单盘全废态数，以及丙-I 判别子（全池自证槽世代号最大的 > mkfs 写的 1）判不出的态数。
不读盘、不跑恢复：一槽自证与否只按「最后落在它上面的是整写的新槽 / 整写的全 0 / 撕裂 / 什么都没落（留基镜像的那一份）」算。
用法：python3 -I independent_count.py；每条流一行 name=independent_count …"""
from itertools import product

def other(segment_kind_counts):
    """非系统配置槽的段：每次写两态（单元写是 COW；journal 记录写进空槽、根槽写一个扇区，按规则都不取第三态）。"""
    return [(2 ** n) - 1 for n in segment_kind_counts]

def run(name, base, segments, others, generation_of_mkfs=1):
    """base：{(盘, 槽): ('valid', 世代号) | ('zero',)}；segments：每段一串写 (盘, 槽, 'new'|'zero', 世代号, 可撕)；
    others：非系统配置槽的段各自去掉全持久之后的态数（与系统配置段的先后不影响计数，各段独立）。"""
    total = sum(others) + 1
    torn = 0
    dead = {}
    state = dict(base)
    for writes in segments:
        choices = [('n', 't', 'p') if w[4] else ('n', 'p') for w in writes]
        for combo in product(*choices):
            if all(c == 'p' for c in combo):
                continue
            total += 1
            is_torn = 't' in combo
            torn += is_torn
            slots = dict(state)
            for (device, slot, content, generation, _), c in zip(writes, combo):
                if c == 'p':
                    slots[(device, slot)] = ('valid', generation) if content == 'new' else ('zero',)
                elif c == 't':
                    slots[(device, slot)] = ('torn',)
            devices = sorted({d for d, _ in slots})
            dead_devices = tuple(d for d in devices if all(slots[(d, s)][0] != 'valid' for s in (0, 1)))
            if dead_devices:
                highest = max((v[1] for v in slots.values() if v[0] == 'valid'), default=None)
                missed = len(dead_devices) < len(devices) and (highest is None or highest <= generation_of_mkfs)
                key = ('torn' if is_torn else 'whole', dead_devices, 'generation_discriminator_silent' if missed else 'generation_discriminator_fires')
                dead[key] = dead.get(key, 0) + 1
        for (device, slot, content, generation, _) in writes:
            state[(device, slot)] = ('valid', generation) if content == 'new' else ('zero',)
    print(f"name=independent_count flow={name} states={total} torn_states={torn}")
    for key in sorted(dead):
        print(f"name=independent_count flow={name} domain={key[0]} dead={list(key[1])} {key[2]} count={dead[key]}")

V = lambda g: ('valid', g)
# 第一个文件之后起的 remount 流（r2 的 remount）：段形 2 sc、2 sc、14 单元、2 journal、1 根、2 sc、12 单元、2 journal、1 根、2 sc。
# 正确实现：每块盘两槽都自证，世代号取 g、g+1（具体值只影响判别子，第一个文件之后都 > 1）。
others_remount = other([14, 2, 1, 12, 2, 1])
g = 10
correct = [[(0, 0, 'new', g + 1, True), (1, 0, 'new', g + 1, True)], [(0, 1, 'new', g + 2, True), (1, 1, 'new', g + 2, True)],
           [(0, 0, 'new', g + 3, True), (1, 0, 'new', g + 3, True)], [(0, 1, 'new', g + 4, True), (1, 1, 'new', g + 4, True)]]
run('remount', {(0, 0): V(g - 1), (0, 1): V(g), (1, 0): V(g - 1), (1, 1): V(g)}, correct, others_remount)
# MUT-B1：盘 0 写完新槽，同段把另一槽清成全 0；前缀在同一开关下跑过，所以基镜像上盘 0 已有一槽是全 0（不可撕：罩住的全是 0、流里更早没写过）。
mut_b1 = [[(0, 0, 'new', g + 1, False), (0, 1, 'zero', 0, True), (1, 0, 'new', g + 1, True)],
          [(0, 1, 'new', g + 2, True), (0, 0, 'zero', 0, True), (1, 1, 'new', g + 2, True)],
          [(0, 0, 'new', g + 3, True), (0, 1, 'zero', 0, True), (1, 0, 'new', g + 3, True)],
          [(0, 1, 'new', g + 4, True), (0, 0, 'zero', 0, True), (1, 1, 'new', g + 4, True)]]
run('remount-SINGLEFS_PROTO_MUT_B1', {(0, 0): ('zero',), (0, 1): V(g), (1, 0): V(g - 1), (1, 1): V(g)}, mut_b1, others_remount)
# MUT-B2：取号两轮并成一段（四个写），两次轮换照旧。
mut_b2 = [correct[0] + correct[1], correct[2], correct[3]]
run('remount-SINGLEFS_PROTO_MUT_B2', {(0, 0): V(g - 1), (0, 1): V(g), (1, 0): V(g - 1), (1, 1): V(g)}, mut_b2, others_remount)
# mkfs 之后第一次可写挂载（r3 的 amf_remount）：段形 2 sc、2 sc、2 journal、1 根、2 sc、2 journal、1 根、2 sc；mkfs 两槽都写世代号 1。
others_amf = other([2, 1, 2, 1])
mkfs_base = {(0, 0): V(1), (0, 1): V(1), (1, 0): V(1), (1, 1): V(1)}
amf = [[(0, 0, 'new', 2, True), (1, 0, 'new', 2, True)], [(0, 1, 'new', 3, True), (1, 1, 'new', 3, True)],
       [(0, 0, 'new', 4, True), (1, 0, 'new', 4, True)], [(0, 1, 'new', 5, True), (1, 1, 'new', 5, True)]]
run('amf_remount', mkfs_base, amf, others_amf)
amf_b1 = [[(0, 0, 'new', 2, True), (0, 1, 'zero', 0, True), (1, 0, 'new', 2, True)],
          [(0, 1, 'new', 3, True), (0, 0, 'zero', 0, True), (1, 1, 'new', 3, True)],
          [(0, 0, 'new', 4, True), (0, 1, 'zero', 0, True), (1, 0, 'new', 4, True)],
          [(0, 1, 'new', 5, True), (0, 0, 'zero', 0, True), (1, 1, 'new', 5, True)]]
run('amf_remount-SINGLEFS_PROTO_MUT_B1', mkfs_base, amf_b1, others_amf)
run('amf_remount-SINGLEFS_PROTO_MUT_B2', mkfs_base, [amf[0] + amf[1], amf[2], amf[3]], others_amf)
# mkfs 崩在最后一段、盘 1 两槽都没落（全 0，不可撕），之后那次可写挂载（补写盘 1）：盘 1 的头两写罩住的是 0 且流里没写过，只取两态。
crash_base = {(0, 0): V(1), (0, 1): V(1), (1, 0): ('zero',), (1, 1): ('zero',)}
rewrite = [[(0, 0, 'new', 2, True), (1, 1, 'new', 1, False)], [(0, 1, 'new', 3, True), (1, 0, 'new', 2, False)],
           [(0, 0, 'new', 4, True), (1, 1, 'new', 3, True)], [(0, 1, 'new', 5, True), (1, 0, 'new', 4, True)]]
run('rewrite_after_a_make_filesystem_crash_1', crash_base, rewrite, others_amf)
# 第二轮 W1 第三截那八个基线：六个基线上那次挂载的段形同 remount（20519），两个（落在第一个文件之后那一次轮换段上的）多一次发布
# （12 单元、2 journal、1 根、2 sc），每个 sc 段两写都可撕（9 − 1 = 8 态）。
remount_states = sum(others_remount) + 4 * 8 + 1
longer_states = sum(other([14, 2, 1, 12, 2, 1, 12, 2, 1])) + 5 * 8 + 1
print(f"name=independent_count flow=second_crash_bases states_each=[{remount_states}, {longer_states}] total={6 * remount_states + 2 * longer_states}")
