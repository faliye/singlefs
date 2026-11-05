import re,sys,os,subprocess
root=sys.argv[1]
files=sys.argv[2:]
def heads(p):
    try:
        return [re.sub(r'^#+\s*','',l.rstrip('\n')) for l in open(os.path.join(root,p),encoding='utf-8') if l.startswith('#')]
    except FileNotFoundError:
        return None
tot=0;bad=0
for f in files:
    added=subprocess.run(['git','diff','HEAD','-U0','--',f],capture_output=True,text=True,cwd=root).stdout
    addedtext='\n'.join(l[1:] for l in added.split('\n') if l.startswith('+') and not l.startswith('+++'))
    for n,line in enumerate(open(os.path.join(root,f),encoding='utf-8'),1):
        if '开工先读' not in line and 'required' not in line: continue
        # sequence: `path` then 「t」「t」 until next backtick path
        for m in re.finditer(r'`(\.claude/[^`]+\.md)`((?:[ ]*「[^」]+」)*)',line):
            p=m.group(1); titles=re.findall(r'「([^」]+)」',m.group(2))
            h=heads(p)
            for t in titles:
                tot+=1
                if h is None: ok='NOFILE'
                else: ok="ok" if any(t==x for x in h) else "MISSING"
                if ok!='ok':
                    bad+=1; print(f'{f}:{n}: {p} 「{t}」 {ok}', '(new this round)' if t in addedtext else '')
print('refs',tot,'bad',bad)
