#!/usr/bin/env python3
# 用法：python3 -I control.py <仓副本根> <包装脚本所在仓根> —— 对照：把 C 的落点隔离循环里的候选豁免拿掉（`referenced_by_candidates.contains(&key) ||` 只在水位之上记录点名那一段），
# 再跑探针 p23，打 B / C / D / E 四形的「R 的单元副本空闲几份、被写掉几份、写行根之前断电后恢复落到哪一版」。跑完还原。
import os, re, subprocess, sys
repo, scripts_repo = sys.argv[1], sys.argv[2]
p = os.path.join(repo, 'crates/singlefs-core/src/mount.rs')
orig = open(p, encoding='utf-8').read()
old = ('        let key = (placement.device.0, placement.slot.0);\n'
       '        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {\n'
       '            continue;\n        }\n        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);')
new = old.replace('referenced_by_candidates.contains(&key) || ', '')
assert orig.count(old) == 1 and old != new
try:
    open(p, 'w', encoding='utf-8').write(orig.replace(old, new))
    cmd = ['nice', '-n', '19', 'bash', scripts_repo + '/research/scripts/run-with-memory-cap.sh', '4G', 'bash', scripts_repo + '/research/scripts/capped.sh', '10',
           'cargo', 'test', '-p', 'singlefs-harness', '--test', 'sonnet_probe_b33t', '--', '--nocapture', '--test-threads=1', 'p23']
    r = subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=220)
finally:
    open(p, 'w', encoding='utf-8').write(orig)
for l in (r.stdout + r.stderr).splitlines():
    m = re.search(r'PROBE23 (\S+) MOUNTED.*?units_of_R_overwritten_before_row_root=(\[.*?\]) recover_after_that: effective_root=Some\(\((\S+), (\S+)\)\)', l)
    if m:
        print('CONTROL', m.group(1), 'free_copies', len(re.findall(r', true\)', l)), 'overwritten_copies', len(re.findall('DeviceIdentity', m.group(2))),
              'recover_lands_on', m.group(3), m.group(4))
