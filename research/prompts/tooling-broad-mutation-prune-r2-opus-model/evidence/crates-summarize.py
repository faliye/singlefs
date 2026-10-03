import re, sys
rows=[l for l in open(sys.argv[1]) if l.startswith("SEQ") and "build-failed" not in l]
bp=sum("baseline-pass=True" in l for l in rows); mf=sum("mutant-pass=False" in l for l in rows)
nf=[l for l in rows if "files-only-in-mutant=none" not in l]
nfun=[l for l in rows if not re.search(r"functions-only-in-mutant=0 ", l)]
br=[l for l in rows if re.search(r"replace (==|!=|<|>|<=|>=|&&|\|\|) with|delete !|match guard|delete match arm|-> bool with", l)]
print(f"rows={len(rows)} baseline-pass={bp} mutant-fails-T={mf} mutant-executes-a-file-outside-T-baseline-files={len(nf)} mutant-executes-a-function-outside-T-baseline-functions={len(nfun)} branchy={len(br)} branchy-new-file={sum(l in nf for l in br)} branchy-new-function={sum(l in nfun for l in br)}")
for l in nf: print("NEWFILE", l[:500].rstrip())
