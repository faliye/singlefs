import sys,re,collections
files=sys.argv[1:]
hist=0; red=0; cu=0; ho=0; pu=0; skipped=collections.Counter(); crashbad=0; crashs=0
cls=collections.Counter(); flags=collections.Counter(); mount_err=collections.Counter()
ex={}
for f in files:
    for line in open(f,errors='replace'):
        if line.startswith('S2 ') or ' S2 label' in line:
            line=line[line.index('S2 label'):]
            if 'record=skipped' in line:
                skipped[re.search(r'record=skipped:(\S+)',line).group(1)]+=1; continue
            hist+=1
            m=dict(re.findall(r'(\w+)=(\S+)',line))
            if m.get('first_red','-')!='-': red+=1; ex.setdefault('red',[]).append(line[:400])
            if m.get('c_unblocked_max','0')!=m.get('c_unblocked_mount','0'): cu+=1; ex.setdefault('cu',[]).append(line[:300])
            if m.get('c_handed_out_max','0')!='0': ho+=1; ex.setdefault('ho',[]).append(line[:300])
            if int(m.get('pu_max','0'))>0: pu+=1; ex.setdefault('pu',[]).append(line[:300])
            crashs+=int(m.get('crash_samples','0')); crashbad+=int(m.get('crash_bad','0'))
            if m.get('selected') is None or 'mount=' in line: mount_err[re.search(r'(mount=\S+)',line).group(1)[:80] if re.search(r'(mount=\S+)',line) else 'x']+=1
        if 'DIAG' in line:
            d=line[line.index('DIAG'):]
            for mm in re.finditer(r'NS\[(\w+);(S\(.*?,r(?:Eq|Ne)\)|N);hold=(\d+);isoC=(\d+);needs=(\S+?)\]',d):
                stop,wide,hold,isoc,needs=mm.groups()
                kind='Stuck' if wide.startswith('S') else 'NotWide'
                key=(stop,kind,'holdPos' if int(hold)>0 else 'hold0','isoPos' if int(isoc)>0 else 'iso0')
                cls[key]+=1
                if kind=='Stuck':
                    for fl in ('!needs','!iso','rNe'):
                        if fl in wide: flags[fl]+=1; ex.setdefault(fl,[]).append(d[:500])
            r=re.search(r'\|(pushed\d+|pushRef:[^|]*)\|stillIso=(\d+)/(\d+)\|(retry\S*)',d)
            if r:
                push,left,tot,retry=r.groups()
                tag=('push:'+re.sub(r'\d+','N',push)[:25],'stillIso>0' if int(left)>0 else 'stillIso0',re.sub(r'\d+','N',retry)[:60])
                cls[tag]+=1
                if int(left)>0: ex.setdefault('stillIso',[]).append(d[:500])
            if 'X:' in d: mount_err[re.sub(r'\d+','N',re.search(r'(X:\S+)',d).group(1))[:90]]+=1
print('histories',hist,'first_red',red,'c_unblocked_grew',cu,'c_handed_out',ho,'pu_pos',pu,'crash_samples',crashs,'crash_bad',crashbad)
print('skipped',dict(skipped))
for k,v in sorted(cls.items(),key=lambda x:-x[1]): print(v,k)
print('flags',dict(flags)); print('X/mount',dict(mount_err))
for k,v in ex.items(): print('EX',k,len(v)); print('   ',v[0][:480])
