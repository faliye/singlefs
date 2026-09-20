"""geo-table.py: per mutant x geometry, new-finding count and per-signature seed counts from logs/<id>/geo-G*.log."""
import re,pathlib,sys
ids=sys.argv[1:]
for i in ids:
    for g in sorted(pathlib.Path(f'logs/{i}').glob('geo-G*.log')):
        t=g.read_text(errors='replace')
        h=re.search(r'^历史 (\d+) 段：.*新发现 (\d+)',t,re.M)
        sig={}
        for m in re.finditer(r'^新发现 (\S+ \{[^}]*\})：第一个种子 \d+（同签名的种子 \[([^\]]*)\]）',t,re.M):
            sig[m[1]]=len(m[2].split(','))
        print(i,g.stem[4:],f'{h[2]}/{h[1]}' if h else '?','; '.join(f'{k.replace("ModelDisagreement","MD")}:{v}' for k,v in sig.items()))
