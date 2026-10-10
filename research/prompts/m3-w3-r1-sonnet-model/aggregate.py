import re,sys,collections
def fields(line):
    d={}
    for tok in line.split(' '):
        if '=' in tok:
            k,v=tok.split('=',1); d[k]=v
    return d
def load(path):
    ev={};own={};summ={}
    for line in open(path,encoding='utf-8'):
        line=line.rstrip('\n')
        if line.startswith('E7RESULT name=event '):
            f=fields(line); ev[(f['index'],f['position'],f['inode'],f['new_birth'])]=f
        elif line.startswith('E7RESULT name=w3step '):
            f=fields(line)
            if f['own']!='-':
                for item in f['own'].split(','):
                    nb,flags=item.split(':'); n,b=nb.split('/')
                    own[(f['index'],f['position'],n,b)]=flags
        elif line.startswith('E7RESULT name=w3summary '):
            f=fields(line); summ[f['index']]=f
    return ev,own,summ
names=['c1a','c1b','c2i','c2all','c3','c4']
for path in sys.argv[1:]:
    ev,own,summ=load(path)
    print('==',path,'histories',len(summ),'events',len(ev))
    tot=collections.Counter(); gone=collections.Counter()
    for k,f in ev.items():
        flags=own.get(k)
        assert flags is not None,k
        tot['events']+=1
        isgone = f['q9']=='Gone'
        if isgone: gone['events']+=1
        for i,nm in enumerate(names):
            if flags[i]=='R':
                tot[nm]+=1
                if isgone: gone[nm]+=1
        if f['kind']=='q3b_across_abandoned': tot['q3b']+=1
    print(' all events :',dict(tot))
    print(' Q9=Gone    :',dict(gone))
    # false positives: reds while no same-timeline conflict so far
    fp=collections.Counter()
    for f in summ.values():
        for nm in names:
            fp[nm]+=int(f['untainted_red_'+nm])
        fp['steps']+=int(f['steps'])
    print(' untainted red steps:',dict(fp))
