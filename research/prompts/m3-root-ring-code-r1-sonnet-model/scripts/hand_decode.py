#!/usr/bin/env python3
# 手算腿：不经 core、不经 checker，按 kb D23 已定项 4 / 15 与 D22 已定项 7 的字段偏移表手解 build_pool 盘上的三条记录与三条根槽。
import struct, sys
D = '/tmp/claude-1000/m3-root-ring-code-r1-sonnet/dump/'
POLY = 0x82F63B78
TABLE = []
for i in range(256):
    c = i
    for _ in range(8):
        c = (c >> 1) ^ POLY if c & 1 else c >> 1
    TABLE.append(c)
def crc32c(b):
    c = 0xFFFFFFFF
    for x in b:
        c = TABLE[(c ^ x) & 0xFF] ^ (c >> 8)
    return c ^ 0xFFFFFFFF
assert crc32c(b'123456789') == 0xE3069283
rec = {n: open(D + f'record-{n}-dev0.bin', 'rb').read() for n in (1, 2, 3)}
rec1 = {n: open(D + f'record-{n}-dev1.bin', 'rb').read() for n in (1, 2, 3)}
root = {n: open(D + f'root-txg{n}.bin', 'rb').read() for n in (1, 2, 3)}
for n in (1, 2, 3):
    assert rec[n] == rec1[n], f'record {n} 两盘不同'
OFF = dict(magic=0, typ=4, algo=6, flags=7, length=8, named=12, jsn=16, txg=26, nonce=34, hcsum=46, tx=78, commit=86, ordinal=87, back=91, payload=95, seg=99, fsid=463, mac=471, end=487)
SEG = dict(tree_table=(0, 86), mapping=(86, 172), watermark=(172, 180), floor=(180, 188), instance_table=(188, 274), alloc=(274, 360), marker=(360, 364))
assert OFF['seg'] + 364 == OFF['fsid'] and OFF['fsid'] + 8 == OFF['mac'] and OFF['mac'] + 16 == OFF['end'] == 487
print('named entries per 4096 record:', (4096 - 487) // 56)
for n in (1, 2, 3):
    r = rec[n]
    assert r[:4] == b'SFSJ'
    typ, = struct.unpack_from('<H', r, 4)
    length, named = struct.unpack_from('<II', r, 8)
    inst, = struct.unpack_from('<I', r, 16)
    counter = int.from_bytes(r[20:26], 'little')
    txg, = struct.unpack_from('<Q', r, 26)
    # header_csum: CRC32C over [0,4096) with the 32 bytes zeroed, stored as 4 bytes LE + 28 zero
    z = bytearray(r); z[46:78] = bytes(32)
    hc = crc32c(bytes(z))
    assert struct.unpack_from('<I', r, 46)[0] == hc and r[50:78] == bytes(28), f'record {n} header_csum 不符'
    pay = crc32c(r[487:487 + 56 * named])
    assert struct.unpack_from('<I', r, 95)[0] == pay, f'record {n} payload_csum 不符'
    seg = r[99:463]
    flags = r[7]
    back = struct.unpack_from('<I', r, 91)[0]
    marker, = struct.unpack_from('<I', seg, 360)
    print(f'record {n}: type={typ} len={length} named={named} instance={inst} counter={counter} txg={txg} flags={flags} tx={struct.unpack_from("<Q", r, 78)[0]} commit={r[86]} ordinal={struct.unpack_from("<I", r, 87)[0]} back={back} marker={marker:#x}')
    # 与同 txg 根槽逐段比（D22 已定项 7 偏移表）
    rt = root.get(txg)
    if rt is not None and rt[:4] == b'SFSR':
        rootmap = dict(tree_table=(36, 122), watermark=(122, 130), floor=(130, 138), instance_table=(170, 256), mapping=(256, 342), alloc=(342, 428))
        for name, (a, b) in rootmap.items():
            sa, sb = SEG[name]
            same = rt[a:b] == seg[sa:sb]
            print(f'   segment {name:15s} == root txg{txg}[{a}:{b}] : {same}')
        rflags, = struct.unpack_from('<I', rt, 20)
        print(f'   marker {marker:#x} == root flags {rflags:#x} : {marker == rflags}')
        print(f'   root fsid[4:20][:8] vs record fsid: {rt[4:12] == r[463:471]}  (record fsid = low 8 bytes of root fsid? {rt[4:12].hex()} / {r[463:471].hex()})')
    else:
        print(f'   (no root slot decoded for txg {txg})')
# back chain
for n in (2, 3):
    prev = bytearray(rec[n - 1][:487]); prev[46:78] = bytes(32)
    chain = crc32c(bytes(prev))
    got = struct.unpack_from('<I', rec[n], 91)[0]
    print(f'record {n} back_chain on disk {got} ; CRC32C(record {n-1} 487-byte header, csum zero) = {chain} ; equal={got == chain}')
print('pin E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION = 2660362816 ; record 3 back chain =', struct.unpack_from('<I', rec[3], 91)[0])
# 旧宽度的对照：把同一条记录当 311 字节头会得到什么（说明 2213468031 是不是旧值）
prev = bytearray(rec[2][:311]); prev[46:78] = bytes(32)
print('control: CRC32C of first 311 bytes of record 2 (csum zero) =', crc32c(bytes(prev)), '(不是 2213468031 也不应是：字节已变)')
