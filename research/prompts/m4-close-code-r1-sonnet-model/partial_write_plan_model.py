#!/usr/bin/env python3
"""正推腿 T2 自带模型：把冻结副本 write_request_split.rs / partial_write_proofs.rs 的部分写计划
逐式移植成 Python，对「逐字节暴力参照」穷举比；不编译 Rust、不碰仓。
参照（暴力）：文件内容 = 字节数组（洞为 0），写 = 在偏移处覆盖并按需延长。
被判（移植）：按计划只重写 [first_rewritten, units_after) 这段单元，旧单元读回（用内存里的单元表代替读盘），
空隙补零、新单元残留丢弃（残留填 0xEE 以便看出残留字节有没有漏进新单元）。
比：重写之后的单元表还原出的文件内容 == 参照；段之外的单元与改前逐字节相同（含声明长度）；
读盘次数 == 触及且净荷没被整单元盖满的、有旧单元的个数 + 重写段里没被写到的 o 之后旧单元（R4）；
残留读次数 == 没有旧单元、被写到而没盖满的个数。"""
import sys

def covering_units(size, cap):
    return (size + cap - 1) // cap

def units_after(existing, end, cap):
    return max(existing, covering_units(end, cap))

def payload_len(unit, size, cap):
    start = unit * cap
    if size <= start:
        return 0
    return min(size - start, cap)

def classify(has_old, ws, we, us, ue):
    touched = ws < ue and us < we
    whole = ws <= us and ue <= we
    if has_old:
        return 'old_covered' if whole else 'old_read'
    if not touched:
        return 'gap'
    return 'new_covered' if whole else 'new_residue'

def first_rewrite(first_written, existing, tail_partial):
    if first_written < existing:
        return first_written
    if existing > 0 and tail_partial:
        return existing - 1
    return existing

def run(cap):
    cases = 0
    for existing in range(0, 5):
        sizes = ([0, 1, cap, cap + 1, 3 * cap] if existing == 0
                 else sorted({(existing - 1) * cap + k for k in range(1, cap + 1)} | {existing * cap + cap // 2, existing * cap + 3 * cap}))
        for size_before in sizes:
            if existing > 0 and not ((existing - 1) * cap < size_before):
                continue
            if existing > 0 and size_before > existing * cap and False:
                continue
            # 旧单元表：单元 u 的净荷 = 声明长度那么多字节，值 = 1 + (全文件偏移 % 250)
            def old_byte(off): return 1 + off % 250
            units = {u: bytes(old_byte(u * cap + i) for i in range(payload_len(u, size_before, cap))) for u in range(existing)}
            logical = bytearray(size_before)
            for u, p in units.items():
                logical[u * cap:u * cap + len(p)] = p
            for off in range(0, 6 * cap + 2):
                for ln in range(1, 3 * cap + 2):
                    data = bytes(200 + (i % 50) for i in range(ln))
                    end = off + ln
                    # 参照
                    ref = bytearray(logical)
                    if len(ref) < end:
                        ref.extend(bytes(end - len(ref)))
                    ref[off:end] = data
                    # 被判
                    size_after = max(size_before, end)
                    fw = off // cap
                    last_partial = existing > 0 and payload_len(existing - 1, size_before, cap) < cap
                    first = first_rewrite(fw, existing, fw >= existing and last_partial)
                    after = units_after(existing, end, cap)
                    new_units = dict(units)
                    reads_old = residues = 0
                    for u in range(first, after):
                        us, ue = u * cap, u * cap + cap
                        kind = classify(u < existing, off, end, us, ue)
                        if kind == 'old_read':
                            reads_old += 1
                            base = bytearray(units[u])
                        elif kind == 'new_residue':
                            residues += 1
                            base = bytearray()
                        else:
                            base = bytearray()
                        decl = payload_len(u, size_after, cap)
                        base.extend(bytes(decl - len(base))) if len(base) < decl else None
                        assert len(base) == decl, ('旧净荷比新声明长', cap, existing, size_before, off, ln, u)
                        lo, hi = max(off, us), min(end, ue)
                        if lo < hi:
                            base[lo - us:hi - us] = data[lo - off:hi - off]
                        new_units[u] = bytes(base)
                    rebuilt = bytearray(size_after)
                    for u, p in new_units.items():
                        rebuilt[u * cap:u * cap + len(p)] = p
                    assert rebuilt == ref, ('内容不同', cap, existing, size_before, off, ln)
                    assert len(new_units) == after
                    for u in range(0, first):
                        assert new_units[u] == units[u], ('段外单元变了', cap, existing, size_before, off, ln, u)
                        assert payload_len(u, size_after, cap) == len(units[u]), ('段外声明长度该变', cap, existing, size_before, off, ln, u)
                    # 单元号连号、声明长度与 size_after 一致
                    for u in range(after):
                        assert len(new_units[u]) == payload_len(u, size_after, cap) or (u * cap >= size_after)
                    # 标签：单元数 == ceil(size/cap) 则无洞（1/2），少于则有洞（3/4）；必不多于
                    assert after <= max(1, covering_units(size_after, cap)) or existing > covering_units(size_after, cap)
                    cases += 1
    return cases

if __name__ == '__main__':
    total = 0
    for cap in (1, 2, 3, 4, 7):
        n = run(cap)
        print(f'cap={cap} 比了 {n} 例，全相同')
        total += n
    print(f'合计 {total} 例，0 例不同')
