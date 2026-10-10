import itertools
faults = ["racct","racct,rlin","racct,rall","racct,rtomb","racct,oldroot","racct,oldacct","racct,rlin,oldroot","rall"]
out=[]
i=0
for slots in (384,1536):
  for r in ["b1","b2","b3","b4","d","n","o","s"]:
    for mid in ["","o","n","b1"]:
      i+=1
      fg=";".join(faults)
      out.append(f"d{i}-{slots}-{r}-{mid or 'x'} s2 SINGLEFS_PROBE_ARM=diff S2_SLOTS={slots} S2_PREFIXES=oooo;oooon S2_MID={mid} S2_RS={r} S2_FAULTS={fg} S2_COUNT=1 S2_ALPHABET=onsFb S2_SEED={10+i} S2_SHORTEST=14 S2_LONGEST=22 S2_PREROOT=1 S2_LABEL=d{i}")
open("scripts/jobs-1.txt","w").write("\n".join(out)+"\n")
print(len(out))
