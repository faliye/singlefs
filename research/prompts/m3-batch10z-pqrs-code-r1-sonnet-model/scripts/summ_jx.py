# 汇总 logs/final/jx-inject-with-abandoned-root.log：每个偏移 k 一行（独立数出的次数、推前推后全部隔离槽数、环里还剩几条被抛弃根、池级 checker 的 first_red）
import re, os
root = os.path.dirname(os.path.abspath(__file__)) + '/../logs/final/'
rows, cur = [], None
for line in open(root + 'jx-inject-with-abandoned-root.log', errors='replace'):
    if line.startswith('## J_OFFSETS='):
        cur = line.split('=')[1].strip(); continue
    m = re.search(r'P\[cur,rOcc=[^]]*needs=Some\((\d+)\),hold=(\d+),isoC=(\d+),isoAll=(\d+)->(\d+)\]pushed(\d+)\[n=\d+,stillIsoC=(\d+),hold_after=(\d+),ab_left=(\d+),(\w+),txg\+(\d+)\]', line)
    if m: rows.append([cur] + list(m.groups()))
    f = re.search(r'first_red=(\S+)', line)
    if f and 'S2 label' in line: rows[-1].append(f.group(1))
print('k needs 注入的C槽数 推前全部隔离->推后 推了几次 推后C里还隔离几槽 环里剩被抛弃根 F 池级first_red')
for r in rows:
    print(r[0], r[1], r[3], f'{r[4]}->{r[5]}', r[6], r[7], r[9], r[10], r[12])
