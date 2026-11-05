import re,subprocess,sys
root=sys.argv[1]; f=sys.argv[2]
d=subprocess.run(['git','diff','HEAD','--word-diff=plain','--word-diff-regex=[^，。；：、（）「」`]+|[，。；：、（）「」`]','-U0','--',f],capture_output=True,text=True,cwd=root).stdout
n=0
for m in re.finditer(r'\{\+(.*?)\+\}',d,re.S):
    t=m.group(1)
    if len(t.strip())<3: continue
    n+=1
    print(f'[{n}] {t[:900]}')
