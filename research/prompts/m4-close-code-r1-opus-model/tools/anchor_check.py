#!/usr/bin/env python3
"""攻方腿（m4-close-code-r1）T10：变异表里落在这一轮射程文件上的行，原文在冻结副本里是不是恰好命中一次，并对照 HEAD 那一份。
用法：python3 anchor_check.py <冻结副本根> <HEAD 那一份的根>
末行 `name=mutation_anchor_check checked=<行数> anchors_not_exactly_once=<数>`。"""
import re
import sys

frozen, head = sys.argv[1].rstrip('/') + '/', sys.argv[2].rstrip('/') + '/'
scope = re.compile(r'singlefs-core/src/(bytes|checksum|pointer|unit|root_record|records|logged_operations|journal|container_tree|'
                   r'deleted_inodes|inode_tree|lineage_tree|retirement_tree|tombstone_tree|write_request_split|truncate|transaction|'
                   r'mounted_session|recovery|system_configuration|block_device)\.rs$')
checked = bad = 0
for line_number, line in enumerate(open(frozen + 'crates/mutations.tsv', encoding='utf-8'), 1):
    if line.startswith('#') or not line.strip():
        continue
    fields = line.rstrip('\n').split('\t')
    if len(fields) < 6 or not scope.search(fields[1]):
        continue
    checked += 1
    anchor = fields[2].replace('\\n', '\n')
    in_frozen = open(frozen + fields[1], encoding='utf-8').read().count(anchor)
    if in_frozen != 1:
        bad += 1
        in_head = open(head + fields[1], encoding='utf-8').read().count(anchor)
        print(f"anchor line={line_number} file={fields[1]} frozen_count={in_frozen} head_count={in_head} test={fields[5]} name={fields[0]}")
print(f"name=mutation_anchor_check checked={checked} anchors_not_exactly_once={bad}")
