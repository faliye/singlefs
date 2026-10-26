#!/usr/bin/env python3
"""对 JUDGING_UNITS_ENTRY_GROUPS 每一组，在 --old 与 --new 两棵树上各算一次判法单元键（与构建脚本同样的特性与工具链），打 组名 / 旧键 / 新键 / 是否变。
用法：group_keys.py --unit-graph <二进制> --old <树> --new <树>"""
import argparse, re, subprocess, sys
p = argparse.ArgumentParser()
p.add_argument('--unit-graph', required=True); p.add_argument('--old', required=True); p.add_argument('--new', required=True); p.add_argument('--gpu', action='store_true')
a = p.parse_args()
text = open(f'{a.new}/crates/singlefs-checker-tier/src/judging_units_entries.rs', encoding='utf-8').read()
head = text.split('pub const JUDGING_UNITS_ENTRY_GROUPS')[1].split('pub struct ObserverEntry')[0]
groups = re.findall(r'group_name:\s*"([^"]+)",\s*target:\s*"([^"]+)",\s*entry_functions:\s*&\[(.*?)\],\s*\}', head, re.S)
def key(root, target, entries):
    cmd = ['nice', '-n', '19', 'bash', '/home/fy5090/code/singlefs/scripts/hosts/capped.sh', '31', 'bash',
           '/home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh', '8G', a.unit_graph, 'key', root, *entries,
           '--target', target, '--no-default-features', '--cfg', 'feature="default"', '--cfg', 'feature="verdict-store"', *(['--cfg', 'feature="gpu"'] if a.gpu else []),
           '--toolchain-of-the-build', 'dev', '--lock-file', f'{root}/Cargo.lock']
    r = subprocess.run(cmd, capture_output=True, text=True, env={**__import__('os').environ,
        'SINGLEFS_RUN_SCOPE': '种类=target; 跑什么=unit-graph key 命令行; 范围=判法单元每一组入口的键; 为什么=正推腿核第三批改动后各组判法单元键是否变化'})
    for line in r.stdout.splitlines():
        if line.startswith('key\t'):
            return line.split('\t')[1][:16]
    return 'ERR:' + (r.stdout + r.stderr).strip().splitlines()[-1][:80] if (r.stdout + r.stderr).strip() else 'ERR'
for name, target, body in groups:
    entries = re.findall(r'"([^"]+)"', body)
    old, new = key(a.old, target, entries), key(a.new, target, entries)
    print(f'group={name} target={target} old={old} new={new} changed={old != new}')
