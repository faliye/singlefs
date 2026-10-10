import re,sys,os
src=open(sys.argv[1],encoding='utf8').read().split('\n')
def block(start_pat,end_pat):
    s=next(i for i,l in enumerate(src) if start_pat in l and i>6000)
    e=next(i for i,l in enumerate(src) if i>s and end_pat in l)
    return src[s:e]
fam={}
cur=None
for l in block('const fn family(self) -> Family {','/// T-表与 T-行判的是')[2:]:
    for n in re.findall(r'Self::(\w+)',l): fam.setdefault('_pending',[]).append(n)
    m=re.search(r'=> Family::(\w+),',l)
    if m:
        for n in fam.pop('_pending'): fam[n]=m.group(1)
crit=list(fam)
lab={}
for i,l in enumerate(src):
    m=re.match(r'\s+(\w+) => "([^"]+)",\s*$',l)
    if m and 6000<i<6200: lab[m.group(1)]=m.group(2)
J7={'J7aInstanceTable','J7aAllocationRecordTreeRoot','J7aFilesystemIdentifier','J7bUnmountMarkerOfARebuiltRoot','J7cUnmountMarkerWhenNothingIsApplied'}
J8={'J8aFieldsFromTheNewRootSegment','J8aUnmountMarkerFromTheNewRootSegment','J8bFilesystemIdentifierOfTheChosenRoot','J8cNothingAppliedReturnsTheChosenRoot'}
Q={'J6QnFixedPoint','MultiDiskAnyCopyIsOnDisk','MultiDiskReturnedRoot','MultiDiskFarthestVersion','MultiDiskAppliedSetAndLandedVersion'}
T={'TableComparisonSeesADifferentTable','TableComparisonSeesADifferentRow'}
def judged(c,p,both):
    crit_set=p
    j6qn=p!='r1'; j7=p in('r2','r3'); j8=p=='r5'; split=p in('r3','r5')
    if c=='J6QnSplitByI812': return split
    if c in T: return both
    if c in J7: return j7
    if c in J8: return j8
    if c in Q: return j6qn
    return True
def unit(f,p,both):
    return {lab[c] for c in crit if (fam[c]==f or c in T) and judged(c,p,both)}
import json
print(len(crit),'criteria; fam counts',{k:sum(1 for c in crit if fam[c]==k) for k in sorted(set(fam.values()))})
units={'r1 legal':('LegalHistory','r1',False),'r1 adv':('Adversarial','r1',False),'r2 md':('MultiDisk','r2',False),
'r2b legal':('LegalHistory','r2',True),'r2b adv':('Adversarial','r2',True),'r2b md':('MultiDisk','r2',True),
'r3 adv':('Adversarial','r3',False),'r5 legal':('LegalHistory','r5',False),'r5 adv':('Adversarial','r5',False),'r5 md':('MultiDisk','r5',False)}
pr=os.environ['RESULTS']+'/e165-replay-bounded-enumeration-2026-10-04-r5-'
prod={'r1 legal':'segment1-r1-legal-family','r1 adv':'segment1-r1-adversarial-family','r2 md':'segment1-r2-multi-disk','r2b legal':'segment1-r2-table-both-legal-family','r2b adv':'segment1-r2-table-both-adversarial-family','r2b md':'segment1-r2-table-both-multi-disk','r3 adv':'segment1-r3-adversarial-family','r5 legal':'segment3-r5-legal-family','r5 adv':'segment3-r5-adversarial-family','r5 md':'segment3-r5-multi-disk'}
for k,(f,p,b) in units.items():
    got={re.search(r'criterion=(\S+) ',l).group(1) for l in open(pr+prod[k]+'.out',encoding='utf8') if 'name=control ' in l}
    print(k,len(unit(f,p,b)),'model==product',unit(f,p,b)==got)
# not-in-14 units
for k,(f,p,b) in {'r2 legal direct':('LegalHistory','r2',False),'r2 adv direct':('Adversarial','r2',False),'r3 legal (impossible-if-no-run)':('LegalHistory','r3',False)}.items():
    print(k,len(unit(f,p,b)))
for k,pf in {'r2 legal direct':'segment1-r2-legal-family','r2 adv direct':'segment1-r2-adversarial-family','r2 odd legal':'segment1-r2-odd-marker-legal-family','r2 odd adv':'segment1-r2-odd-marker-adversarial-family','r1 light':'segment1-r1-light-legal-family'}.items():
    got={re.search(r'criterion=(\S+) ',l).group(1) for l in open(pr+pf+'.out',encoding='utf8') if 'name=control ' in l}
    print(k,len(got))

print('--- family-move mutants against the 14-unit test')
U14=[('LegalHistory','r1',False),('Adversarial','r1',False),('MultiDisk','r2',False),('LegalHistory','r2',True),('Adversarial','r2',True),('MultiDisk','r2',True),
('LegalHistory','r2',True),('MultiDisk','r2',True),('Adversarial','r3',False),('LegalHistory','r5',False),('Adversarial','r5',False),('MultiDisk','r5',False),('LegalHistory','r5',False),('Adversarial','r5',False)]
def unit_m(f,p,b,famof,every=lambda c:c in T):
    return {c for c in crit if (famof[c]==f or every(c)) and judged(c,p,b)}
base=[unit_m(*u,fam) for u in U14]
fams=['LegalHistory','Adversarial','MultiDisk']
caught=0;miss=[]
for c in crit:
    for nf in fams:
        if nf==fam[c]: continue
        f2=dict(fam);f2[c]=nf
        red=any(unit_m(*u,f2)!=b for u,b in zip(U14,base))
        if red: caught+=1
        else: miss.append((lab[c],nf))
print('mutants',len(crit)*2,'caught',caught,'missed',miss)
# old r2 test: union families
P5=[(['LegalHistory','Adversarial'],'r1',False),(['LegalHistory','Adversarial','MultiDisk'],'r2',False),(['LegalHistory','Adversarial','MultiDisk'],'r2',True),(['LegalHistory','Adversarial','MultiDisk'],'r3',False),(['LegalHistory','Adversarial','MultiDisk'],'r5',False)]
def un5(fs,p,b,famof): return {c for c in crit if (famof[c] in fs or c in T) and judged(c,p,b)}
b5=[un5(*u,fam) for u in P5]
c5=0
for c in crit:
    for nf in fams:
        if nf==fam[c]: continue
        f2=dict(fam);f2[c]=nf
        if any(un5(*u,f2)!=b for u,b in zip(P5,b5)): c5+=1
print('old 5-profile test catches',c5,'of',len(crit)*2)
# every_family mutants
for name,ev in {'false':lambda c:False,'all-true':lambda c:True}.items():
    red=any(unit_m(*u,fam,ev)!=b for u,b in zip(U14,base)); red5=any({c for c in crit if (fam[c] in fs or ev(c)) and judged(c,p,bb)}!=b for (fs,p,bb),b in zip(P5,b5))
    print('every_family=',name,'new red',red,'old red',red5)

print('--- predicate mutants (is_judged_under arm swapped)')
PRED={'true':lambda p,b:True,'false':lambda p,b:False,'judges_j6qn':lambda p,b:p!='r1','judges_j7':lambda p,b:p in('r2','r3'),'judges_j8':lambda p,b:p=='r5','splits':lambda p,b:p in('r3','r5'),'compares':lambda p,b:b}
orig={}
for c in crit:
    if c=='J6QnSplitByI812': orig[c]='splits'
    elif c in T: orig[c]='compares'
    elif c in J7: orig[c]='judges_j7'
    elif c in J8: orig[c]='judges_j8'
    elif c in Q: orig[c]='judges_j6qn'
    else: orig[c]='true'
def unit_p(f,p,b,po):
    return {c for c in crit if (fam[c]==f or c in T) and PRED[po[c]](p,b)}
# 14 units formal run (distinct run configs; disk-order/parity irrelevant)
bases=[unit_p(*u,orig) for u in U14]
# r2 verdict's "old 5 profile" test
P5b=[(['LegalHistory','Adversarial'],'r1',False),(['LegalHistory','Adversarial','MultiDisk'],'r2',False),(['LegalHistory','Adversarial','MultiDisk'],'r2',True),(['LegalHistory','Adversarial','MultiDisk'],'r3',False),(['LegalHistory','Adversarial','MultiDisk'],'r5',False)]
def u5(fs,p,b,po): return {c for c in crit if (fam[c] in fs or c in T) and PRED[po[c]](p,b)}
base5=[u5(*u,orig) for u in P5b]
# formal-run: actual runs (family,profile,table) : 
tot=0;changed=0;unch=[];newred_unch=0;oldred_unch=0;new_miss_changed=0
res={'changed':0,'unchanged':0,'unch_new_red':0,'unch_old5_red':0,'changed_new_red':0}
nine=[]
for c in crit:
    for po in PRED:
        if po==orig[c]: continue
        m=dict(orig);m[c]=po
        ch=any(unit_p(*u,m)!=b for u,b in zip(U14,bases))
        # formal-run changes (same as 14-unit)
        old5=any(u5(*u,m)!=b for u,b in zip(P5b,base5))
        if ch: res['changed']+=1; res['changed_new_red']+=1
        else:
            res['unchanged']+=1
            if old5: res['unch_old5_red']+=1; nine.append((lab[c],po))
print(res); print(len(nine),nine)

print('--- with 4 registered formal runs that have no replay.sh row (r5 prereg compare table rows 4-7)')
EXTRA=[('LegalHistory','r2',False),('Adversarial','r2',False)]  # odd variants are same sets
U18=U14+EXTRA
b18=[unit_p(*u,orig) for u in U18]
miss=[];allch=0;inc=0
for c in crit:
    for po in PRED:
        if po==orig[c]: continue
        m=dict(orig);m[c]=po
        ch18=any(unit_p(*u,m)!=b for u,b in zip(U18,b18))
        ch14=any(unit_p(*u,m)!=b for u,b in zip(U14,bases))
        if ch18: allch+=1
        if ch18 and not ch14: miss.append((lab[c],po))
print('mutants changing some of 18 registered formal runs',allch,'; missed by the 14-unit test',len(miss),miss)
# family-move mutants vs 18
fm=[]
for c in crit:
    for nf in fams:
        if nf==fam[c]: continue
        f2=dict(fam);f2[c]=nf
        ch18=any(unit_m(*u,f2)!=unit_m(*u,fam) for u in U18); ch14=any(unit_m(*u,f2)!=unit_m(*u,fam) for u in U14)
        if ch18 and not ch14: fm.append((lab[c],nf))
print('family-move mutants missed by 14 but changing 18:',fm)
