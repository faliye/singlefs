#!/usr/bin/env python3
"""打出 W0 W0 C W0 这段历史在 wal_K9 / wal_K9p / jia 下每一步的槽、记录与恢复结果（报告第二节的逐步历史）。"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import model
for arm in ['wal_K9', 'wal_K9p', 'jia']:
    for hist, fault in [(('W0', 'W0', 'C', 'W0'), ('slot', 1)), (('W0', 'W0', 'C', 'W0'), ('disk', 1))]:
        p = model.Pool(arm, 8)
        print(f'== {arm} {"".join(hist)} fault={fault}')
        for op in hist:
            if op == 'C':
                p.op_checkpoint()
            else:
                p.op_write(int(op[1]))
            print(f'  after {op}: txg={p.txg} units={dict(sorted(p.units.items()))} occupied={sorted(p.occupied)} defer={p.defer}')
        for r in p.journal:
            print(f'  record jsn={r["jsn"]} txg={r["txg"]} kind={r["kind"]} names={r["names"]}')
        print(f'  roots={sorted(p.roots.values(), key=lambda x: x[0])}')
        st, root = p.recover(fault)
        print(f'  recovered root={root} state={st} acked={p.acked}')
