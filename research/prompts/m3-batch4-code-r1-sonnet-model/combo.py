#!/usr/bin/env python3
import sys,shutil,hashlib
sys.argv=[sys.argv[0],'none']+sys.argv[1:]
exec(open('/tmp/claude-1000/m3-batch4-code-r1-sonnet/drive.py').read().split("tkey=sys.argv[1]")[0])
mode=sys.argv[2]; labels=sys.argv[3:]
T='/tmp/claude-1000/m3-batch4-code-r1-sonnet/tree/'; FZ='/tmp/claude-1000/m3-batch5-freeze/tree/'
if mode=='restore':
    for rel in {WALK,IMG}:
        shutil.copyfile(FZ+rel,T+rel)
    print('restored')
else:
    for label in labels:
        f,o,r=P[label]
        s=open(T+f,encoding='utf-8').read()
        assert s.count(o)==1,(label,s.count(o))
        open(T+f,'w',encoding='utf-8').write(s.replace(o,r))
    print('applied',labels)
