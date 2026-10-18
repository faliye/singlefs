#!/usr/bin/env bash
# 合总表：十张事实表 → facts.tsv（去掉 F609 去向列、F121 与 F215 两个重复的宽词）→ check-facts → candidates-all.tsv
# → candidates.tsv（去掉 records/ 里只按欠账编号搜到的行，那些交第二轮探针按「引已还清的欠账当开着」机械判）+ 额外载体候选 → segments.tsv
set -euo pipefail
W=/home/fy5090/.local/share/singlefs/work/rot-2026-10-17
cd /home/fy5090/code/singlefs
{ head -1 $W/agents/甲/facts-甲.tsv
  for n in 甲 乙一 乙二甲 乙二乙 乙二丙; do tail -n +2 $W/agents/$n/facts-$n.tsv; done
  for n in 丙甲 丙乙 丙丙 丁甲 丁乙; do tail -n +2 /home/fy5090/.local/share/singlefs/work/rot-2026-10-17-$n/facts-$n.tsv; done
} | awk -F'\t' '$1!="F609" && $1!="F121" && $1!="F215"' > $W/facts.tsv
echo "事实表 $(($(wc -l <$W/facts.tsv)-1)) 行；重复编号：$(cut -f1 $W/facts.tsv | sort | uniq -d | tr '\n' ' ')"
python3 scripts/kb/stale-candidates.py --check-facts $W/facts.tsv --base ce1457c3 | tail -1
python3 scripts/kb/stale-candidates.py --facts $W/facts.tsv --base ce1457c3 --out $W/candidates-all.tsv | tail -1
python3 $W/extra-candidates.py $W/facts.tsv $W/candidates-extra.tsv
python3 - <<'PY'
import csv,re
W='/home/fy5090/.local/share/singlefs/work/rot-2026-10-17'
facts={r['编号']:r for r in csv.DictReader(open(f'{W}/facts.tsv'),delimiter='\t')}
cgroups={k for k,r in facts.items() if re.match(r'^(\\b)?C[\(\d]',r['检索词'].strip())}
rows=[l.rstrip('\n').split('\t') for l in open(f'{W}/candidates-all.tsv')][1:]
extra=[l.rstrip('\n').split('\t') for l in open(f'{W}/candidates-extra.tsv')][1:]
keep=[r for r in rows if not (r[3].startswith('records/') and r[0] in cgroups)]
dropped=[r for r in rows if (r[3].startswith('records/') and r[0] in cgroups)]
with open(f'{W}/candidates.tsv','w') as f:
    f.write('组\t旧事实\t新事实\t载体\t原文\n')
    for r in keep+extra: f.write('\t'.join(r)+'\n')
with open(f'{W}/candidates-records-owed-groups.tsv','w') as f:
    f.write('组\t旧事实\t新事实\t载体\t原文\n')
    for r in dropped: f.write('\t'.join(r)+'\n')
print(f'候选表 candidates.tsv：{len(keep)+len(extra)} 行（主 {len(keep)} + 额外载体 {len(extra)}），不同的行 {len({r[3] for r in keep+extra})}；交第二轮探针的 records 欠账编号行 {len(dropped)} 行（{len({r[3] for r in dropped})} 个不同的行）→ candidates-records-owed-groups.tsv')
PY
python3 $W/segment.py $W/candidates.tsv $W/segments.tsv 300
