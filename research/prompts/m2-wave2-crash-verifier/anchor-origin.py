# 用法（仓库根）：python3 research/prompts/m2-wave2-crash-verifier/anchor-origin.py > research/prompts/m2-wave2-crash-verifier/anchor-origin.out
# 实现员开工前的副本在 /tmp/claude-1000/impl-m2-wave2-fixes/baseline，那个目录不留存；2026-09-18 跑出的结果在同目录 anchor-origin.out。
import os
base='/tmp/claude-1000/impl-m2-wave2-fixes/baseline'
print('# crates/mutations.tsv 里 59 号判锚点腐化的 6 行：原文在实现员开工前的副本（'+base+'）与今天的工作区里各命中几次')
print('# 原文按 59 号的读法把 \\n 换成换行；生成于 2026-09-18，命令见 research/prompts/m2-wave2-crash-verifier/anchor-origin.py')
lines=open('crates/mutations.tsv').read().split('\n')
for n in (32,37,41,42,69,107):
    name,path,anchor=lines[n-1].split('\t')[:3]
    anchor=anchor.replace('\\n','\n')
    b=open(os.path.join(base,path)).read().count(anchor)
    w=open(path).read().count(anchor)
    print(f'第 {n} 行\t{path}\t开工前副本 {b} 次\t今天 {w} 次\t{name}')
