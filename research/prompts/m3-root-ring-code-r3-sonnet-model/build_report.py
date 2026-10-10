# 把草稿里的 ⟦路径|原文片段⟧ 换成 「原文片段」（`路径:行号`）：在被引文件里逐行找片段（比对前去掉 ** 与反引号、并空白），必须恰好命中一行，
# 命中 0 行或多行就停（不许写成引文）。⟦cat|文件⟧ 换成那份文件的原样内容（围栏块）。用法：build_report.py 草稿 输出
import re,sys,os
REPO='/home/fy5090/code/singlefs'
def norm(t): return re.sub(r'\s+',' ',t.replace('**','').replace('`','')).strip()
cache={}
def lines_of(path):
    if path not in cache:
        cache[path]=open(os.path.join(REPO,path),encoding='utf-8').read().split('\n')
    return cache[path]
def hit(path,frag):
    f=norm(frag); out=[]
    for i,l in enumerate(lines_of(path),1):
        if f in norm(l): out.append(i)
    return out
src=open(sys.argv[1],encoding='utf-8').read()
err=[]
def sub(m):
    inner=m.group(1)
    kind,rest=inner.split('|',1)
    if kind=='cat':
        body=open(rest,encoding='utf-8').read().rstrip('\n')
        return '```text\n'+body+'\n```'
    h=hit(kind,rest)
    if len(h)!=1:
        err.append((kind,rest[:60],h)); return '【引文未定】'
    return '「%s」（`%s:%d`）'%(rest,kind,h[0])
res=re.sub(r'⟦([^⟧]+)⟧',sub,src)
if err:
    for e in err: print('✗',e)
    sys.exit(1)
open(sys.argv[2],'w',encoding='utf-8').write(res)
print('ok lines',len(res.split('\n')))
