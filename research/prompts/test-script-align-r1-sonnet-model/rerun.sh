#!/usr/bin/env bash
# test-script-align-r1 正推腿（Sonnet）的复跑：只读，grep / awk / git show / 一段 python 读标题；不编译、不跑任何自证。
# 复跑挂钟：不到 5 秒。在仓根跑：bash research/prompts/test-script-align-r1-sonnet-model/rerun.sh
# 报告里每一段「rerun.sh 第 N 段」对应下面的「== 段 N ==」。
cd "$(git rev-parse --show-toplevel)" || exit 2

echo "== 段 1：并行线号与来历写进文件头（T1-1、T1-1a、T1-1b）"
echo -n "HEAD verification.md 并行线号: "; git show HEAD:.claude/rules/verification.md | grep -c '并行线号'
echo -n "工作区 verification.md 并行线号: "; grep -c '并行线号' .claude/rules/verification.md
echo -n "规则与 CLAUDE.md 里的 parallel_line 或并行线号命中: "; grep -rn '并行线号\|parallel_line' .claude/rules CLAUDE.md | wc -l
grep -nF '(^|_)parallel_line_(one|two|three)(_|$)' scripts/crash/crash-case-check.py | cut -c1-60
echo -n "crash-case-check.py 里「来历写进文件头」命中: "; grep -cF '来历写进文件头' scripts/crash/crash-case-check.py
grep -nF '来历进 commit message 或 .claude/kb/' scripts/crash/crash-case-check.py | cut -c1-60
grep -nF 'file-names 格出路还叫人把来历写进文件头' .claude/kb/checks-owed.md | cut -c1-40
echo -n "HEAD 多线程旧句、修完代码旧句命中: "; git show HEAD:.claude/rules/implementation-workflow.md | grep -c '在同一份代码上核过一次'; git show HEAD:.claude/rules/verification.md | grep -c '修完代码也只重跑红的那几条'

echo "== 段 2：指向上游节名是否逐字存在（T1-2）"
python3 - <<'PY'
import re,subprocess
heads=[l[3:].strip() for l in open('.claude/singlefs-ai-sop/rules/test-script-discipline.md',encoding='utf-8') if l.startswith('## ')]
files=subprocess.run("grep -rlI 'test-script-discipline' .claude CLAUDE.md scripts crates --include=*.md --include=*.sh --include=*.py --include=*.tsv --include=*.rs",shell=True,capture_output=True,text=True).stdout.split()
bad=tot=0
for f in files:
    if 'singlefs-ai-sop/' in f: continue
    for n,line in enumerate(open(f,encoding='utf-8',errors='ignore'),1):
        for m in re.finditer(r'test-script-discipline\.md`?[」）]?\s*[：:]?\s*「([^」]+)」',line):
            tot+=1
            ok=m.group(1) in heads
            bad+=0 if ok else 1
            print(('OK  ' if ok else 'BAD ')+f"{f}:{n}: 「{m.group(1)}」")
print('total',tot,'bad',bad)
PY

echo "== 段 3：夜间残留（T1-3）"
grep -rnI '夜间' .claude CLAUDE.md scripts crates --exclude-dir=singlefs-ai-sop | grep -v 'decisions-history.md' | cut -c1-150
echo -n "合并前 在规则与定义里的命中: "; grep -rn '合并前' .claude/rules .claude/agents .claude/main-agent.md .claude/agent-common.md | wc -l

echo "== 段 4：自证登记表各类的条数（T1-5）"
awk -F'\t' '!/^#/{print $2}' .claude/hooks/selftest-registry.tsv | sort | uniq -c

echo "== 段 5：C610 文件头日期行的量与 C613 / C620 的现场（T4）"
echo -n "带日期的 //! 行: "; grep -rhE '^//!.*20[0-9]{2}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs | wc -l
echo -n "  其中没有用户也没有定: "; grep -rhE '^//!.*20[0-9]{2}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs | grep -vcE '用户|定'
echo -n "日期之后跟用户或定的行: "; grep -rhE '^//!.*20[0-9]{2}-[0-9]{2}-[0-9]{2}.*(用户|定)' crates/*/tests/*.rs | wc -l
echo -n "  其中同一行带 kb 指路: "; grep -rhE '^//!.*20[0-9]{2}-[0-9]{2}-[0-9]{2}.*(用户|定)' crates/*/tests/*.rs | grep -cE '已定项|D[0-9]+（|C[0-9]{3}|E[0-9]{2,3}'
echo -n "NOT_RUN_NO_GPU_ADAPTER 在 crates 里的命中: "; grep -rn 'NOT_RUN_NO_GPU_ADAPTER' crates | wc -l
echo -n "decide_gpu_test_start 在 crates 里的命中: "; grep -rn 'decide_gpu_test_start' crates | wc -l
echo -n "states_reused 在 crates 里的命中: "; grep -rn 'states_reused' crates | wc -l
echo -n "mutations.tsv 里 wgsl 的命中: "; grep -c wgsl crates/mutations.tsv
echo -n "mutations.tsv 里开 gpu 的行: "; awk -F'\t' '!/^#/ && NF>=5 && $5 ~ /gpu/' crates/mutations.tsv | wc -l
echo -n "mutations.tsv 里点名 each_attribute_changes_the_reuse_key 的行: "; grep -c each_attribute_changes_the_reuse_key crates/mutations.tsv
echo -n "mutate.sh 里 252 或 254 的命中: "; grep -c '252\|254' scripts/mutation/mutate.sh
echo -n "crash-amplification.sh 里 item_output= 的命中: "; grep -c 'item_output=' scripts/crash/crash-amplification.sh
echo "本机 vendor 0x10de 的 PCI 设备："
for d in /sys/bus/pci/devices/*; do [ "$(cat "$d/vendor")" = 0x10de ] && echo "$(basename "$d") class=$(cat "$d/class")"; done

echo "== 段 6：D13 与同批 kb 改动的词级 diff（T4-D13）"
git diff HEAD -U0 --word-diff=plain -- .claude/kb/decisions/13-验证路线.md .claude/kb/decisions/06-快照实现模型.md .claude/kb/decisions/16-发布语义.md | grep -c '^\[-\|{+'
