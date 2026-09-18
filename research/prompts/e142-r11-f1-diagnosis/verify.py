# 独立核对：按位 CRC32C（不查表，不共用两份 Rust 代码），逐个校验和字段按 layout/01-first-txn.md 的覆盖口径重算
import sys, struct
def crc32c(data):
    c = 0xFFFFFFFF
    for b in data:
        c ^= b
        for _ in range(8):
            c = (c >> 1) ^ 0x82F63B78 if c & 1 else c >> 1
    return c ^ 0xFFFFFFFF
assert crc32c(b"123456789") == 0xE3069283
def u32(b, o): return struct.unpack_from('<I', b, o)[0]
def wide_ok(b, cover_end, off):
    z = bytearray(b[:cover_end]); z[off:off+32] = bytes(32)
    return u32(b, off) == crc32c(z) and b[off+4:off+32] == bytes(28), u32(b, off)
d = sys.argv[1]
R = lambda n: open(f'{d}/{n}.bin', 'rb').read()
unit = {n: R(n+'_0') for n in ['data_unit','extent_root','inode_leaf','inode_root','allocation_root','accounting_root','mapping_root','tree_table']}
whole = {n: crc32c(v) for n, v in unit.items()}
rows = []
def row(region, field, off, value, ok): rows.append(f"{d} {region} off={off} {field} value=0x{value:08x} ok={ok}")
# data unit
b = unit['data_unit']; ok, v = wide_ok(b, 105, 10); row('data_unit','header_csum[0,105)',10,v,ok)
v = u32(b,101); row('data_unit','payload_crc[105,32768)',101,v, v==crc32c(b[105:]))
row('data_unit','payload_first4', 134, u32(b,134), True)
# code-2 nodes
for n in ['extent_root','inode_root','allocation_root','accounting_root','mapping_root','tree_table']:
    b = unit[n]; k = b[51]; he = 86 + 2*k
    ok, v = wide_ok(b, he, 10); row(n, f'header_csum[0,{he})', 10, v, ok)
    v = u32(b, 76+2*k); row(n, f'payload_crc[{he},16384)', 76+2*k, v, v == crc32c(b[he:]))
def ptr_locs(region, b, p, target):
    for i, lo in enumerate((p+50, p+64)):
        dev = u32(b, lo); v = u32(b, lo+10)
        row(region, f'ptr@{p}.loc{i}(dev{dev}).crc->{target}', lo+10, v, v == whole[target])
# extent root entry0 pointer at 163+24
ptr_locs('extent_root', unit['extent_root'], 163+24, 'data_unit')
# mapping root entries: 6 x 55 from 169; value (2 loc entries) at key end +27
mb = unit['mapping_root']
tgt_by_key = {}
for e in range(6):
    base = 169 + 55*e; cls = mb[base]; tree = struct.unpack_from('<Q', mb, base+1)[0]
    for i in range(2):
        lo = base + 27 + 14*i; v = u32(mb, lo+10)
        match = [n for n, w in whole.items() if w == v]
        row('mapping_root', f'entry{e}(class{cls},tree{tree}).loc{i}.crc matches={match}', lo+10, v, bool(match))
# tree table entries: 7 x 200 from 131; root pointer at +14
tb = unit['tree_table']
for e in range(7):
    base = 131 + 200*e; tree = struct.unpack_from('<Q', tb, base)[0]; p = base + 14
    for i in range(2):
        lo = p + 50 + 14*i; v = u32(tb, lo+10)
        match = [n for n, w in whole.items() if w == v]
        row('tree_table', f'entry{e}(tree{tree}).root_ptr.loc{i}.crc matches={match}', lo+10, v, bool(match) or v == 0)
# root record
rr = R('root_record_0'); ok, v = wide_ok(rr, 512, 138); row('root_record','self_csum[0,512)',138,v,ok)
ptr_locs('root_record', rr, 36, 'tree_table'); ptr_locs('root_record', rr, 256, 'mapping_root')
for dev in (0, 1):
    j = R(f'journal_record_{dev}'); cnt = u32(j, 12)
    ok, v = wide_ok(j, 4096, 46); row(f'journal_record{dev}','header_csum[0,4096)',46,v,ok)
    v = u32(j, 91); row(f'journal_record{dev}', f'payload_crc[307,{307+56*cnt})', 91, v, v == crc32c(j[307:307+56*cnt]))
    ptr_locs(f'journal_record{dev}', j, 95, 'tree_table'); ptr_locs(f'journal_record{dev}', j, 181, 'mapping_root')
    for e in range(cnt):
        base = 307 + 56*e
        for i in range(2):
            lo = base + 14*i; v = u32(j, lo+10); match = [n for n, w in whole.items() if w == v]
            row(f'journal_record{dev}', f'named{e}(class{j[base+28]}).loc{i}.crc matches={match}', lo+10, v, bool(match))
print('\n'.join(rows)); print(d, 'rows', len(rows), 'all_ok', all(r.endswith('ok=True') for r in rows))
