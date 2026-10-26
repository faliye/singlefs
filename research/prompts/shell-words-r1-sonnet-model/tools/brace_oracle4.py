import sys, random, subprocess, os, json
sys.path.insert(0, os.path.join(os.environ["SW_ROOT"], "tree", ".claude", "hooks"))
import lib_shell_words as L
random.seed(int(sys.argv[1])); N=int(sys.argv[2])
atoms=["{","}",",","a","b","..","1","3","-2","0","01","10",".. 2","x","{1..3}","{a..c}","{,}","{a,b}","{1..4..2}","{c..a}","{-1..1}","{a..e..2}","{3..1}","{1..3..-1}","{a..c..0}","{A..C}","{1..3..2}","{z..x}","'q'","\\{","\\,","\\}"]
def gen(): return "".join(random.choice(atoms) for _ in range(random.randint(1,6)))
def bash_words(word):
    p=subprocess.run(["bash","-c","set -f\nprintf '[%s]' "+word+"\necho\n"],capture_output=True,text=True,timeout=5)
    if p.returncode!=0: return None
    out=p.stdout.rstrip("\n")
    if out=="[]": return []
    return out[1:-1].split("][") if out.startswith("[") and out.endswith("]") else None
seen=set(); diffs=[]; n=0
while n<N:
    w=gen().replace(" ","")
    if w in seen or (len(w)-len(w.rstrip(chr(92))))%2==1: continue
    seen.add(w); bw=bash_words(w)
    if bw is None: continue
    if len(bw)>64: continue
    n+=1
    hw=[t for op,t in L.shell_tokens(w) if op is False]
    hwn=[x for x in hw if x]; bwn=[x for x in bw if x]
    if hw==bw: continue
    kind="empties-only" if hwn==bwn else "other"
    diffs.append((kind,w,bw,hw))
from collections import Counter
print(Counter(k for k,*_ in diffs), "of", n)
for d in [d for d in diffs if d[0]=="other"][:40]: print(json.dumps(d,ensure_ascii=False))
