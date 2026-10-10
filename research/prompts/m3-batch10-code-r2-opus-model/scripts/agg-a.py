import re,glob,sys
D='/tmp/claude-1000/m3-batch10-code-r2-opus/a'
rows={}
for arm in ['on','off','slot']:
    for f in glob.glob(f'{D}/{arm}-*.log'):
        for line in open(f):
            line=re.sub(r'^test a_one\S* \.\.\. ','',line.strip())
            m=re.match(r'OPUS-A \S+ point=(\S+) fault=(\S+) reads=(\d+) tally=(.*)',line)
            if m: rows.setdefault((m.group(1),m.group(2)),{})[arm]=(m.group(3),m.group(4))
            m=re.match(r'OPUS-A-IMAGE \S+ attribute_by_slot=\S+ point=(\S+) image_after_a_mount_without_faults=(\S+)',line)
            if m: rows.setdefault((m.group(1),'image'),{})[arm]=m.group(2)
            m=re.match(r'OPUS-A-BASE \S+ point=(\S+) roots=(.*) outcome=(\S+) hook_calls=(\d+) reads=(\d+)',line)
            if m: rows.setdefault((m.group(1),'base'),{})[arm]=f'outcome={m.group(3)} hooks={m.group(4)} reads={m.group(5)}'
total={a:0 for a in ['on','off','slot']}
for k in sorted(rows):
    v=rows[k]
    if k[1] in ('fails','flip'):
        for a in v: total[a]+=sum(int(x) for x in re.findall(r': (\d+)',v[a][1]))
    same = len(set(map(str,v.values())))==1
    print(k[0],k[1],'SAME' if same else 'DIFF', v if not same else list(v.values())[0])
print('mounts',total)
