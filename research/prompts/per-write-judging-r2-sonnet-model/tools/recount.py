# 从任务 8321、8322 的段表日志重算：全量态数（与日志里量到的总数比）与候选甲、乙在「单元段全部过判据」前提下的态数。
import re, sys
def parse(path, pattern, key):
    flows = {}
    order = []
    for line in open(path, encoding='utf-8').read().splitlines():
        m = re.search(pattern, line)
        if m:
            name = m.group('flow') if 'flow' in m.groupdict() else 'unlink'
            if name not in flows:
                flows[name] = []
                order.append(name)
            flows[name].append((int(m.group('segment')), int(m.group('writes')), m.group('kinds')))
    return flows, order
unlink, _ = parse(sys.argv[1], r'PROBE_SEGMENT segment=(?P<segment>\d+) writes=(?P<writes>\d+) kinds=\{(?P<kinds>.*?)\} tearable', 'x')
pairing, order = parse(sys.argv[2], r'PROBE_FLOW_SEGMENT flow=(?P<flow>\S+) segment=(?P<segment>\d+) writes=(?P<writes>\d+) kinds=\[(?P<kinds>.*?)\] over', 'x')
measured = {
    'unlink': 1463812286,
    'second-deletion-batch-quarantined-data-unit-records-carry-the-mark': 1178599580,
    'three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first': 1107689646,
    'deletion-through-the-last-batch': 1195376807,
}
allflows = dict(unlink)
allflows.update(pairing)
for name, segs in allflows.items():
    full = 1; jia = 1; yi = 1; unit_segments = []
    for s, w, k in segs:
        if 'UnitWrite' in k:
            unit_segments.append((s, w))
            full += 2 ** w - 1
            jia += w + 1
            yi += 1 + 3 * (w // 2) if w % 2 == 0 else 10 ** 9
        elif 'SystemConfigurationSlot' in k:
            full += 3 ** w - 1; jia += 3 ** w - 1; yi += 3 ** w - 1
        else:
            full += 2 ** w - 1; jia += 2 ** w - 1; yi += 2 ** w - 1
    print(f'RECOUNT flow={name[:60]} segments={len(segs)} unit_segments={unit_segments}')
    print(f'RECOUNT   full={full} measured={measured[name]} same={full == measured[name]} jia={jia} yi={yi}')
for seg in (12, 16, 20):
    w = dict((s, w) for s, w, k in unlink['unlink'])[seg]
    print(f'RECOUNT unlink segment={seg} writes={w} full={2**w-1} jia={w+1} yi={1+3*(w//2)}')
