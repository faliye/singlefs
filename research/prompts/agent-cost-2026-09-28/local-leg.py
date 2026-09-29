"""本地腿的采纳率与有效率（2026-09-28 那份分析用）。在仓根跑：python3 research/prompts/agent-cost-2026-09-28/local-leg.py
读工作区里的判决文件 research/prompts/*-main-verification.md 与 git 里 SINCE 起新增的本地腿产物文件名。"""
import glob, re, subprocess, collections, os, sys
SINCE = os.environ.get('AGENT_COST_SINCE', '2026-09-20')
verdicts = sorted(glob.glob('research/prompts/*-main-verification.md'))
with_local = 0; summaries = []
for p in verdicts:
    t = open(p, errors='replace').read()
    if re.search(r'本地(攻方|辩方|腿|样本)', t): with_local += 1
    for m in re.finditer(r'本地(?:腿|攻方|辩方)小结[：:]([^|\n]{0,160})', t): summaries.append((os.path.basename(p)[:30], m.group(1).replace('**', '').strip()))
print('# local-leg.py')
print('判决文件', len(verdicts), '；提到本地腿的', with_local, '；带「本地腿小结」句的', len(summaries))
print('\n## 全部小结句（主 agent 逐句判打中还是没打中，判法写在报告里）')
for p, s in summaries: print(f'  {p} | {s}')
def added(pattern): return sorted(set(subprocess.run(['git', 'log', f'--since={SINCE}', '--diff-filter=A', '--name-only', '--format=', '--', pattern], capture_output=True, text=True).stdout.split()))
clean = [f for f in added('research/prompts/*local*output*.md') if 'void' not in f]; void = added('research/prompts/*local*void*.md')
print('\n## 有效率：git 里', SINCE, '起新增的本地腿产物文件')
print('干净或通读待判的样本文件', len(clean), '；损坏闸作废的 void 文件', len(void), f'；作废占 {len(void) / max(1, len(clean) + len(void)):.0%}')
