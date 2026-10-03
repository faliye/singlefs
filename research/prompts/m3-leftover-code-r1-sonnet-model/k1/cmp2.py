import re,difflib
exec(open('cmp.py').read().split('for (path,name),body in secs.items():')[0])
body=secs[('crates/singlefs-checker/src/walk.rs','check_pool_image')]
new=fn_text('crates/singlefs-checker/src/walk.rs','check_pool_image')
strip=lambda L:[re.sub(r'\s+','',x) for x in L if not x.lstrip().startswith('//') and x.strip()]
a=strip(body);b=strip(new)
sm=difflib.SequenceMatcher(None,a,b,autojunk=False)
for tag,i1,i2,j1,j2 in sm.get_opcodes():
    if tag!='equal':
        print(tag,i1,i2,j1,j2)
        print('  OLD:',' '.join(a[i1:i2])[:400])
        print('  NEW:',' '.join(b[j1:j2])[:600])
