"""崩溃状态枚举：前面的段全持久、当前段每次写各取几态任意组合（去掉整段全持久那一个）、后面的段一个都没持久；
加最后「全部持久」那一个。可撕的写取三态（没持久 / 撕裂 / 持久），其余两态。纯 sys 段逐态给盘面，别的段（单元、journal、根槽）
只数态数（它们不改系统配置槽，盘面同该段起点）。"""
import itertools
from slot_flow_model import Z, T, ZERO_TORN, valid, tearable, torn_contents


def apply_whole(disk, w):
    if w.kind == 'sys':
        disk[w.dev][w.slot] = w.content
    elif w.kind == 'zerofill' and w.covers_slots:
        disk[w.dev] = [Z, Z]


def dead_devices(disk):
    return tuple(d for d in sorted(disk) if not any(valid(c) for c in disk[d]))


def states(rec):
    """逐态交回 (段号, 域, 盘面, 死盘元组, 权重)。非 sys 段按一个代表盘面交回，权重 = 该段态数（2^n − 1）。
    域：'whole' 没有任何撕裂落法；'torn' 持久集合里有撕裂镜像。最后一个状态段号 -1，权重 1。"""
    torn = torn_contents(rec)
    disk = {d: list(rec.base[d]) for d in rec.devices}
    for si, seg in enumerate(rec.segments):
        sys_idx = [i for i in seg if rec.writes[i].kind == 'sys' or (rec.writes[i].kind == 'zerofill' and rec.writes[i].covers_slots)]
        if not sys_idx:
            n = len(seg)
            yield si, 'whole', {d: list(disk[d]) for d in disk}, dead_devices(disk), (1 << n) - 1
            for i in seg:
                apply_whole(disk, rec.writes[i])
            continue
        others = [i for i in seg if i not in sys_idx]
        radices = [3 if i in torn else 2 for i in sys_idx]
        # 同段内各 sys 写落在互不相同的槽上（本模型里的协议满足；断言防写错）
        assert len({(rec.writes[i].dev, rec.writes[i].slot) for i in sys_idx if rec.writes[i].kind == 'sys'}) \
            == len([i for i in sys_idx if rec.writes[i].kind == 'sys'])
        for combo in itertools.product(*[range(r) for r in radices]):
            whole_all = all(c == (r - 1) for c, r in zip(combo, radices))
            for o in range(1 << len(others)):
                if whole_all and o == (1 << len(others)) - 1:
                    continue  # 整段全持久：并进下一段起点
                dsk = {d: list(disk[d]) for d in disk}
                has_torn = False
                for c, r, i in zip(combo, radices, sys_idx):
                    w = rec.writes[i]
                    if c == r - 1:
                        apply_whole(dsk, w)
                    elif c == 1 and r == 3:
                        if torn[i] == ZERO_TORN:
                            dsk[w.dev] = [Z, T]
                        else:
                            dsk[w.dev][w.slot] = torn[i]
                        has_torn = True
                yield si, ('torn' if has_torn else 'whole'), dsk, dead_devices(dsk), 1
        for i in seg:
            apply_whole(disk, rec.writes[i])
    yield -1, 'whole', {d: list(disk[d]) for d in disk}, dead_devices(disk), 1


def tally(rec):
    """态数、按域分、死盘分类（单盘全废 / 全部盘全废），权重已计。"""
    out = {'states': 0, 'whole': 0, 'torn': 0, 'single_dead': {'whole': 0, 'torn': 0},
           'all_dead': {'whole': 0, 'torn': 0}, 'images': set()}
    n_dev = len(rec.devices)
    for si, dom, dsk, dead, wgt in states(rec):
        out['states'] += wgt
        out[dom] += wgt
        out['images'].add(tuple((d, tuple(dsk[d])) for d in sorted(dsk)))
        if dead:
            key = 'all_dead' if len(dead) == n_dev else 'single_dead'
            out[key][dom] += wgt
    return out
