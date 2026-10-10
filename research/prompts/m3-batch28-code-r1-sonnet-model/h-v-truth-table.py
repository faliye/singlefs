#!/usr/bin/env python3
"""H-V 真值表：CPU 新写法、GPU 新写法（照 wgsl 逐行抄）、按条款写的规格、CPU 旧写法，在两份副本的全部状态上比。不碰盘、不碰仓。

一份副本的状态：UNREAD（读不出头），或 (类标签 0..9, 自证过吗)。自证过 = 按那一类的长度读回的整单元校验和对得上，未登记的类（不在 1、2、3）没有长度、恒不自证。
类型段说的类 named：None（类型段不认得）、1、2、3。
"""
import itertools, sys

NONE = 0xFFFFFFFF
REGISTERED = (1, 2, 3)
states = ['UNREAD'] + [(c, v) for c in range(10) for v in (False, True)]
nameds = [None, 1, 2, 3]


def verifies(s):
    return s != 'UNREAD' and s[0] in REGISTERED and s[1]


def cpu_new(copies, named):
    other = None
    for s in copies:
        if s == 'UNREAD':
            continue
        c = s[0]
        if c not in REGISTERED:
            continue
        if not s[1]:
            continue
        if c != named and other is None:
            other = c
    return other


def cpu_old(copies, named):
    for s in copies:
        if s == 'UNREAD':
            continue
        c = s[0]
        if c not in REGISTERED:
            continue
        if not s[1]:
            continue
        if c == named:
            return None
        return c
    return None


def gpu_new(copies, named):
    n = NONE if named is None else named
    other = NONE
    first, second = copies
    if first != 'UNREAD':
        c = first[0]
        if verifies(first):
            if c != n:
                other = c
    if second != 'UNREAD':
        c = second[0]
        if verifies(second):
            if c != n and other == NONE:
                other = c
    return None if other == NONE else other


def spec(copies, named):
    for s in copies:
        if verifies(s) and s[0] != named:
            return s[0]
    return None


def cpu_old_form_mutant(copies, named):
    """第 26 批旧变异（头里的类就是类型段那一类的副本不验整单元校验和）搬到新写法上：if !verifies && c != named {continue}。"""
    other = None
    for s in copies:
        if s == 'UNREAD':
            continue
        c = s[0]
        if c not in REGISTERED:
            continue
        if not s[1] and c != named:
            continue
        if c != named and other is None:
            other = c
    return other


def cpu_row_2930_form(copies, named):
    """本批替换的第 2930–2933 行：if c == named && other.is_none() {break}，在验校验和之前。"""
    other = None
    for s in copies:
        if s == 'UNREAD':
            continue
        c = s[0]
        if c not in REGISTERED:
            continue
        if c == named and other is None:
            break
        if not s[1]:
            continue
        if c != named and other is None:
            other = c
    return other


def gpu_old_form_mutant(copies, named):
    """第 26 批 GPU 旧变异：if (c == named || verifies(first)) 套在第一份上。"""
    n = NONE if named is None else named
    other = NONE
    first, second = copies
    if first != 'UNREAD':
        c = first[0]
        if c == n or verifies(first):
            if c != n:
                other = c
    if second != 'UNREAD':
        c = second[0]
        if verifies(second):
            if c != n and other == NONE:
                other = c
    return None if other == NONE else other


def gpu_row_2936_form(copies, named):
    """本批替换的第 2936 行：读到第一份的头之后 if (c == named) return NONE。"""
    n = NONE if named is None else named
    other = NONE
    first, second = copies
    if first != 'UNREAD':
        c = first[0]
        if c == n:
            return None
        if verifies(first):
            if c != n:
                other = c
    if second != 'UNREAD':
        c = second[0]
        if verifies(second):
            if c != n and other == NONE:
                other = c
    return None if other == NONE else other


bad_cpu_gpu = bad_cpu_spec = 0
differs = {'cpu_old_form': 0, 'cpu_row_2930': 0, 'gpu_old_form': 0, 'gpu_row_2936': 0}
changed = {}
total = 0
for named in nameds:
    for copies in itertools.product(states, repeat=2):
        total += 1
        a, b, g, s = cpu_new(copies, named), cpu_old(copies, named), gpu_new(copies, named), spec(copies, named)
        bad_cpu_gpu += a != g
        differs['cpu_old_form'] += cpu_old_form_mutant(copies, named) != a
        differs['cpu_row_2930'] += cpu_row_2930_form(copies, named) != a
        differs['gpu_old_form'] += gpu_old_form_mutant(copies, named) != g
        differs['gpu_row_2936'] += gpu_row_2936_form(copies, named) != g
        bad_cpu_spec += a != s
        if a != b:
            key = (named, 'first=' + ('UNREAD' if copies[0] == 'UNREAD' else ('named' if copies[0][0] == named else 'other')) + ('+v' if verifies(copies[0]) else '-v'),
                   'second verifies other' if verifies(copies[1]) and copies[1][0] != named else 'second else')
            changed.setdefault((key[1], key[2]), set()).add((b, a))
print('状态总数', total)
print('CPU新 与 GPU新 不同的状态数', bad_cpu_gpu)
print('CPU新 与 按条款写的规格 不同的状态数', bad_cpu_spec)
print('旧写法与新写法交回不同的状态按形分：')
for k in sorted(changed):
    print('  ', k, '旧→新', sorted(changed[k], key=str))
print('变异写法与新写法交回不同的状态数（0 = 在全部状态上等价）：')
for k, v in differs.items():
    print('  ', k, v)
sys.exit(1 if bad_cpu_gpu or bad_cpu_spec else 0)
