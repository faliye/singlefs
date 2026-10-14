import sys
root=sys.argv[1]; probes=open(sys.argv[2],encoding='utf-8').read()
p=f'{root}/crates/singlefs-core/src/mount.rs'
s=open(p,encoding='utf-8').read()
i=s.rstrip().rfind('}')
open(p,'w',encoding='utf-8').write(s[:i]+probes+s[i:])
print('probes appended into', p)
