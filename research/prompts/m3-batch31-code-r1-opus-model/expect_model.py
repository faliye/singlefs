#!/usr/bin/env python3
"""攻方的期望模型：按 kb（I-2.1、I-9.2、I-9.17、I-9.23 那几行）与读码写下的分支规则，算树根第一条条目改坏之后
红不红 I-2.1、I-4.8、I-7.2、I-7.4 与身份那一条；与 probe 的 PROBE 行逐行比（只比这五条）。
不 import 仓里任何东西；GPU 走读内核这几支读码同形（w_walk_container_tree_root、w_judge_every_copy_of_a_child_the_walk_does_not_go_into），
所以这份期望也就是 GPU 那一侧该给的（GPU 没跑）。
用法：expect_model.py <PROBE 行所在的日志>；末行 `MODEL agree=… disagree=…`。"""
import re
import sys

IDENTITY = {"tomb": "I-9.23", "lineage": "I-9.17", "inode": "I-9.2"}
# 每种副本状态：自证成哪一类（按它自己那一类的长度）、按头里那一类的长度对不对得上、16 KiB 的 CRC 对不对得上、32 KiB 对不对得上
STATE = {
    "bad": dict(self_class=None, header_len=False, crc16=False, crc32=False),
    "gone": dict(self_class=None, header_len=False, crc16=False, crc32=False),
    "c16": dict(self_class=None, header_len=False, crc16=True, crc32=False),
    "node": dict(self_class=2, header_len=True, crc16=True, crc32=False),
    "ok": dict(self_class=3, header_len=True, crc16=False, crc32=True),
}
HEADER_CLASS = {"bad": 3, "gone": None, "c16": 3, "node": 2, "ok": 3}


def expect(tree, entry_type, copies):
    s = [STATE[c] for c in copies]
    i21 = False
    failure = False
    identity_red = False
    # 中央映射树的条目照旧指着这片容器（原来的 32 KiB 校验和）：哪块盘上写进了码 2 的字节，那一份就对不上；两块都写了就读不出
    if "node" in copies:
        i21 = True
    if copies.count("node") == 2:
        failure = True
    type_class = {"t0": 2, "rec": 3, "t7": None}[entry_type]

    def judge_every_copy():
        nonlocal i21, failure
        matches = [x["header_len"] for x in s]
        if not all(matches):
            i21 = True
        if not any(matches):
            failure = True

    other = None
    if type_class is not None or tree == "inode":
        for x in s:
            if x["self_class"] is not None and x["self_class"] != type_class and other is None:
                other = x["self_class"]
    fw = other is not None and type_class is not None and any(x["self_class"] == type_class for x in s)
    if other is not None and not fw:
        judge_every_copy()
        identity_red = True
        return finish(i21, failure, identity_red)
    if fw:
        identity_red = True
    if type_class is None:
        judge_every_copy()
        identity_red = True
    elif type_class == 2:
        if tree == "inode":
            return None  # inode 类型段 0 往下走（walk_inode_internal_child），不在这份模型里
        first16 = next((c for c, x in zip(copies, s) if x["crc16"]), None)
        child_class = HEADER_CLASS[first16] if first16 else None
        if child_class not in (None, 2):
            identity_red = True
        judge_every_copy()
        failure = True  # F-L0：层级 ≤ 1 的节点下面不往下走、记走读失败
    else:
        matches = [x["crc32"] for x in s]
        if not all(matches):
            i21 = True
        if not any(matches):
            failure = True
    return finish(i21, failure, identity_red)


def finish(i21, failure, identity_red):
    red = set()
    if i21:
        red.add("I-2.1")
    if i21 or failure:
        red |= {"I-4.8", "I-7.4"}
    if failure:
        red.add("I-7.2")
    if identity_red:
        red.add("ID")
    return red


def main():
    agree = disagree = skipped = 0
    pattern = re.compile(r"PROBE (matrix|l0root) tree=(\w+) level=(\d+) type=(\w+) c0=(\w+) c1=(\w+) red=\[(.*?)\]")
    for line in open(sys.argv[1], encoding="utf-8"):
        m = pattern.match(line)
        if not m:
            continue
        kind, tree, level, entry_type, c0, c1, red = m.groups()
        expected = expect(tree, entry_type, [c0, c1])
        if expected is None:
            skipped += 1
            continue
        got = set(r for r in red.split(",") if r)
        got_cut = {("ID" if r == IDENTITY[tree] else r) for r in got if r in {"I-2.1", "I-4.8", "I-7.2", "I-7.4", IDENTITY[tree]}}
        if got_cut == expected:
            agree += 1
        else:
            disagree += 1
            print(f"DIFF {kind} {tree} {entry_type} c0={c0} c1={c1} model={sorted(expected)} cpu={sorted(got_cut)}")
    print(f"MODEL agree={agree} disagree={disagree} skipped={skipped}")


if __name__ == "__main__":
    main()
