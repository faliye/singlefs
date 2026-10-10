#!/usr/bin/env python3
"""b30 正推腿 M3：独立（不经 core 的校验和函数）核两份样本槽的新摘要。
读两份 cargo 输出（旧 = T_time 写 5 的那一版，新 = 入库版）里的 B30SLOT 行，逐样本：
 ① 新槽与旧槽逐字节只差 [433,437)（5 → 5000）与整槽校验和 [155,159)；
 ② 新槽 [155,187) 里 CRC-32C（纯 Python 现算，字段自身按 0 参与）等于槽里存的，其余 28 字节 0；
 ③ 旧槽照同一条规则改 5000 重封得到的字节的 SHA-256 等于新槽的、也等于测试里的钉值（命令行第三、四个参数）。"""
import hashlib, re, sys

def crc32c(data):
    table = []
    for i in range(256):
        c = i
        for _ in range(8):
            c = (c >> 1) ^ (0x82F63B78 if c & 1 else 0)
        table.append(c)
    crc = 0xFFFFFFFF
    for b in data:
        crc = table[(crc ^ b) & 0xFF] ^ (crc >> 8)
    return crc ^ 0xFFFFFFFF

def slots(path):
    out = {}
    for line in open(path, encoding='utf-8'):
        m = re.match(r'B30SLOT (\S+) sha256=(\S+) t_time_bytes_433_437=\[[^\]]*\] hex=(\S+)', line)
        if m: out[m.group(1)] = (m.group(2), bytes.fromhex(m.group(3)))
    return out

old, new = slots(sys.argv[1]), slots(sys.argv[2])
pins = {'at_mkfs': sys.argv[3], 'after_the_new_pool_file_creation': sys.argv[4]}
ok = True
for name in ('at_mkfs', 'after_the_new_pool_file_creation'):
    old_sha, o = old[name]; new_sha, n = new[name]
    diff = [i for i in range(len(o)) if o[i] != n[i]]
    only_expected = all(155 <= i < 159 or 433 <= i < 437 for i in diff)
    covered = bytearray(n); covered[155:187] = bytes(32)
    stored_ok = int.from_bytes(n[155:159], 'little') == crc32c(bytes(covered)) and n[159:187] == bytes(28)
    patched = bytearray(o); patched[433:437] = (5000).to_bytes(4, 'little'); patched[155:187] = bytes(32)
    patched[155:159] = crc32c(bytes(patched)).to_bytes(4, 'little')
    sha_patched = hashlib.sha256(bytes(patched)).hexdigest()
    t_old = int.from_bytes(o[433:437], 'little'); t_new = int.from_bytes(n[433:437], 'little')
    line = (f'SLOTCHECK {name} t_time_old={t_old} t_time_new={t_new} differing_byte_positions={len(diff)} only_433_437_and_155_159={only_expected} '
            f'stored_crc_ok_by_python={stored_ok} old_sha_matches_old_pin_in_log={old_sha[:8]} new_sha={new_sha[:8]} sha_of_old_slot_patched_by_python={sha_patched[:8]} equals_new={sha_patched == new_sha} equals_pin={sha_patched == pins[name]}')
    print(line)
    ok = ok and only_expected and stored_ok and sha_patched == new_sha == pins[name]
print('SLOTCHECK_RESULT', 'OK' if ok else 'FAIL')
sys.exit(0 if ok else 1)
