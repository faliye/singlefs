"""把 L0 族里 WALK 与真值不同的状态逐个归类：WALK 装上而真值没装的那次发布，缺掉的记录点名的单元有没有被新根段引用。"""
import collections
import importlib.util
import os
spec = importlib.util.spec_from_file_location('m', os.path.join(os.path.dirname(os.path.abspath(__file__)), 'model.py'))
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
kinds = collections.Counter()
def classify(image, writer, walk, truth):
    # 找 WALK 装上而真值没装的那次发布，把它缺掉的记录按「点名的单元有没有被新根段引用」分类
    applied = set(walk.applied) - set(truth.applied)
    txgs = {r.txg for (i, c), r in image.records().items() if (i, c) in applied}
    out = []
    for write in writer.writes:
        if write[0] in ('rec', 'failed') and hasattr(write[1], 'txg') and isinstance(write[1], m.Rec) and write[1].txg in txgs:
            rec = write[1]
            if (rec.inst, rec.ctr) not in image.records():
                reachable = set(m.version_units(rec.seg))
                if set(rec.named) & reachable:
                    out.append('missing_record_names_reachable_unit')
                elif not rec.named:
                    out.append('missing_record_names_nothing')
                else:
                    out.append('missing_record_units_superseded')
    return tuple(sorted(set(out))) or ('no_missing_record',)
for pattern in m.patterns(3):
    base, writer, _ = m.first_mount_stream(pattern)
    for key, image, persisted in m.crash_states(base, writer.writes):
        walk = m.recover(image, 'WALK', 'prefix'); truth = m.recover(image, 'TRUTH6', 'prefix')
        if walk.outcome() != truth.outcome():
            kinds[classify(image, writer, walk, truth)] += 1
print(dict(kinds), sum(kinds.values()))
