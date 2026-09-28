#!/usr/bin/env python3
"""B1 校验：用 r1 攻方 kinds 世界的原始输出，核「种类串 + 重叠关系签名」
是否把 kinds.out 里合并的两个条件签名分开。

只做算术／字符串解析，不编译、不跑 crates/ 代码：
kinds.out 的 detail 字段已经把两个条件签名的差异点写全了
（reuse / root_slot_rewrite / journal_slot_rewrite 三个布尔），
这里只是把这三个布尔映射到本设计提出的「4 位重叠签名」的其中一位
（RootRecordFua 那一位），核对它确实能把两条签名分开。
"""
import pathlib
import re

OUT = pathlib.Path("/home/fy5090/code/singlefs/research/prompts/m3-prune-gpu-r1-opus-model/outputs/kinds.out")

def main():
    text = OUT.read_text()
    line = [l for l in text.splitlines() if l.startswith("E7RESULT name=attack_kind_string")][0]
    detail_match = re.search(r"detail=\[(.*)\]", line)
    assert detail_match, "kinds.out 里没有 detail= 字段，核不了"
    detail = detail_match.group(1)
    # detail 形如: "SS|UUU...|JJ|R|SS=>{"reuse=false,root_slot_rewrite=false,...", "reuse=false,root_slot_rewrite=true,..."}"
    kind_string, rest = detail.split("=>", 1)
    signatures = re.findall(r'"([^"]*)"', rest)
    assert len(signatures) == 2, f"期待 2 条条件签名，读到 {len(signatures)} 条：{signatures}"

    def overlap_bit_root_slot(signature: str) -> bool:
        # 本设计 B1 的「重叠关系签名」里 RootRecordFua 那一位：
        # 这次根槽写是不是重写了路径上更早的根槽写（root_slot_rewrite=true 就是「根环回卷」）。
        return "root_slot_rewrite=true" in signature

    bits = [overlap_bit_root_slot(s) for s in signatures]
    node_identities = [(kind_string, bit) for bit in bits]
    distinct_node_identities = len(set(node_identities))

    print(f"kind_string={kind_string!r}")
    for sig, bit in zip(signatures, bits):
        print(f"  签名={sig!r} -> root_slot_fua_overlap_bit={bit}")
    print(f"distinct_kind_strings_here=1 distinct_condition_signatures={len(signatures)} "
          f"distinct_node_identities_with_overlap_bit={distinct_node_identities}")
    assert distinct_node_identities == len(signatures), (
        "B1 设计失败：加了重叠签名之后，这两条条件签名仍被并成同一个节点身份"
    )
    print("PASS: 加一位 RootRecordFua 重叠位之后，kinds 世界这一条种类串下的两个条件签名分成了两个不同的节点身份")

if __name__ == "__main__":
    main()
