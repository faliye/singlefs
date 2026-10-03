import re,subprocess,json,sys,hashlib
def src(path,rev='HEAD'):
    return subprocess.run(['git','show',f'{rev}:{path}'],capture_output=True,cwd='/home/fy5090/code/singlefs').stdout.decode('utf8')
def tokens(s):
    """yield (kind,text,start_offset,end_offset); skips comments/strings/chars; kinds: id, p (punct), str"""
    i=0;n=len(s);out=[]
    while i<n:
        c=s[i]
        if c.isspace(): i+=1;continue
        if s.startswith('//',i):
            j=s.find('\n',i); i=n if j<0 else j; continue
        if s.startswith('/*',i):
            d=1;j=i+2
            while j<n and d:
                if s.startswith('/*',j): d+=1;j+=2
                elif s.startswith('*/',j): d-=1;j+=2
                else: j+=1
            i=j;continue
        m=re.match(r'b?r(#*)"',s[i:i+300]) if c in 'br' else None
        if m:
            h=m.group(1); end='"'+h
            j=s.find(end,i+m.end())
            j=n if j<0 else j+len(end)
            out.append(('str','',i,j));i=j;continue
        if c=='"' or (c=='b' and s.startswith('b"',i)):
            j=i+(2 if c=='b' else 1)
            while j<n and s[j]!='"':
                j+=2 if s[j]=='\\' else 1
            out.append(('str','',i,j+1));i=j+1;continue
        if c=="'":
            m=re.match(r"'(\\.[^']*|[^\\'])'",s[i:i+12])
            if m:
                out.append(('str','',i,i+m.end()));i+=m.end();continue
            m=re.match(r"'[A-Za-z_]\w*",s[i:])
            if m: i+=m.end();continue
            i+=1;continue
        m=re.match(r'[A-Za-z_]\w*',s[i:i+200])
        if m:
            out.append(('id',m.group(0),i,i+m.end()));i+=m.end();continue
        m=re.match(r'\d[\w.]*',s[i:i+100])
        if m:
            out.append(('num',m.group(0),i,i+m.end()));i+=m.end();continue
        out.append(('p',c,i,i+1));i+=1
    return out
def fn_spans(s):
    """list of dict(name,is_const,start,end(offset of body end),hdr) for fn items with body; nested included"""
    t=tokens(s);res=[]
    n=len(t)
    for k,(kind,tx,a,b) in enumerate(t):
        if kind=='id' and tx=='fn' and k+1<n and t[k+1][0]=='id':
            name=t[k+1][1]
            # is_const: scan back over qualifiers
            q=k-1;consts=False
            while q>=0 and t[q][0]=='id' and t[q][1] in ('const','async','unsafe','extern','pub','default') or (q>=0 and t[q][0]=='str' and q>=1 and t[q-1][1]=='extern'):
                if t[q][1]=='const': consts=True
                q-=1
            # pub(crate)
            # find body
            j=k+2;depth=0
            while j<n:
                kd,x,_,_=t[j]
                if kd=='p':
                    if x in '([': depth+=1
                    elif x in ')]': depth-=1
                    elif x=='{' and depth==0: break
                    elif x==';' and depth==0: j=None;break
                j+=1
            if j is None or j>=n: continue
            # match braces
            d=0;e=j
            while e<n:
                if t[e][0]=='p':
                    if t[e][1]=='{': d+=1
                    elif t[e][1]=='}':
                        d-=1
                        if d==0: break
                e+=1
            h=k
            while True:
                p=h-1
                if p>=0 and t[p][0]=='id' and t[p][1] in ('const','async','unsafe','extern','pub','default'): h=p;continue
                if p>=0 and t[p][0]=='str' and p>=1 and t[p-1][1]=='extern': h=p-1;continue
                if p>=0 and t[p][0]=='p' and t[p][1]==')':
                    q2=p;dd=0
                    while q2>=0:
                        if t[q2][0]=='p' and t[q2][1]==')': dd+=1
                        elif t[q2][0]=='p' and t[q2][1]=='(':
                            dd-=1
                            if dd==0: break
                        q2-=1
                    if q2>=1 and t[q2-1][0]=='id' and t[q2-1][1]=='pub': h=q2-1;continue
                    break
                if p>=0 and t[p][0]=='p' and t[p][1]==']':
                    q2=p;dd=0
                    while q2>=0:
                        if t[q2][0]=='p' and t[q2][1]==']': dd+=1
                        elif t[q2][0]=='p' and t[q2][1]=='[':
                            dd-=1
                            if dd==0: break
                        q2-=1
                    if q2>=1 and t[q2-1][0]=='p' and t[q2-1][1]=='#': h=q2-1;continue
                    break
                break
            res.append(dict(name=name,is_const=consts,start=t[j][2],end=t[e][3] if e<n else len(s),hdr=t[h][2]))
    return res
def line_starts(s):
    ls=[0]
    for m in re.finditer('\n',s): ls.append(m.end())
    return ls
def off(ls,line,col,s):
    # col is 1-based in chars (cargo-mutants uses chars? columns in chars)
    return ls[line-1]+col-1
