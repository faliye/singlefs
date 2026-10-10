import json,hashlib,re,sys,os
root=sys.argv[1]
files=['.claude/kb/decisions/16-发布语义.md','.claude/kb/checks-owed.md','.claude/kb/invariants.md','.claude/kb/decisions/13-验证路线.md']
text={f:open(f,encoding='utf-8').read() for f in files}
def undo(f,old,new,label):
    if f not in text: return
    c=text[f].count(new)
    if c!=1: print('MISS',label,f,c); return
    text[f]=text[f].replace(new,old)
def md_spec(p):
    s=open(p,encoding='utf-8').read()
    out=[]
    for m in re.finditer(r'文件：(\S+)\n旧串：\n(~{3,})\n(.*?)\n\2\n新串：\n\2\n(.*?)\n\2\n',s,re.S):
        out.append(dict(file=m.group(1),old=m.group(3),new=m.group(4)))
    return out
# 1 peer extra C159
undo('.claude/kb/checks-owed.md','C160（记账臂的真实代价不在天平上）','C159（E112 的臂集缺三条且天平不含挂不上的代价）、C160（记账臂的真实代价不在天平上）','C159')
# 2 peer scribe-b inserted rows
for row in json.load(open('/tmp/claude-1000/m3-writeback3/scribe-b/newrows.json'))['rows']:
    undo('.claude/kb/checks-owed.md','',row+'\n','newrow')
# 3 peer scribe-b replacements
for e in reversed(json.load(open('/tmp/claude-1000/m3-writeback3/scribe-b/replace-spec.json'))):
    undo(e['file'],e['old'],e['new'],'peerB')
# 4 peer scribe-a
for e in reversed(md_spec('research/prompts/m3-candidate-hole-r1-snapshot/peer-deletion-writeback3/scribe-a/spec.md')):
    undo(e['file'],e['old'],e['new'],'peerA')
# 5 my C342 backfill
undo('.claude/kb/checks-owed.md','会红的检查是 S2-1 新立的那条（见 invariants.md）','会红的检查是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）','C342')
# 6 writeback B
for e in reversed(json.load(open('/tmp/claude-1000/m3-r1/batch9-writeback-b/spec.json'))):
    undo(e['file'],e['old'],e['new'],'wbB')
# 7 writeback A
for e in reversed(json.load(open('/tmp/claude-1000/m3-r1/batch9-writeback-a/spec.json'))):
    undo(e['file'],e['old'],e['new'],'wbA')
for f in files:
    p=os.path.join(root,f); os.makedirs(os.path.dirname(p),exist_ok=True)
    open(p,'w',encoding='utf-8').write(text[f])
    print(hashlib.sha256(text[f].encode()).hexdigest(),f)
