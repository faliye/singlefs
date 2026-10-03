#!/usr/bin/env bash
# D1：崩溃验证员第 1 步的两条核对都过（工作区与暂存区在登记路径上相同、crates 与 e165 红例账没有未跟踪文件），
# 而 research/results/ 里另有一份别的会话的未跟踪产物时，它在工作区写的全绿标记，gate.sh --staged 的临时 worktree 里对不对得上。
# 只调 admission.py 的 stage-marker-write / stage-marker-check / gate-reuse 与只读 git；git 写只落在 d2-reuse.sh 建的草稿拷贝里。
# 用法：bash d1b-untracked-results.sh <草稿目录>（先跑过 d2-reuse.sh，拷贝在 <草稿目录>/repo）
set -uo pipefail
WORK="${1:?草稿目录}"; COPY="$WORK/repo"; STAGE=checker-tier-formal-verification.sh
G=(git -C "$COPY" -c user.name=scratch -c user.email=scratch@invalid)
COMMON="$("${G[@]}" rev-parse --path-format=absolute --git-common-dir)"
TREE="$("${G[@]}" write-tree)"
rm -f "$COMMON"/singlefs-stage-green."$STAGE".* "$COMMON/singlefs-gate-environment.$STAGE"
"${G[@]}" update-ref -d refs/sop/staged-green 2>/dev/null || true
c="$("${G[@]}" commit-tree "$TREE" -m "上一次整轮全绿")"; "${G[@]}" update-ref refs/sop/staged-green "$c"
export SINGLEFS_VERUS_ROOT="$WORK/verus-A"
untracked="$COPY/research/results/e167-other-session-in-progress.log"
printf 'another session is still appending here\n' > "$untracked"
paths="$(awk -F'\t' -v s="$STAGE" '$1 == s { print $2 }' "$COPY/.claude/gate.d/stage-inputs.tsv")"
echo "崩溃验证员第 1 步，照 crash-verifier.md 的两条命令（在拷贝里）："
# shellcheck disable=SC2086
"${G[@]}" diff --quiet -- $paths .claude/gate.d/$STAGE; echo "  git diff --quiet -- <登记路径> <阶段脚本> 退 $?"
echo "  git ls-files --others --exclude-standard -- crates 'research/results/e165-*-red-examples.tsv' 打出 $("${G[@]}" ls-files --others --exclude-standard -- crates 'research/results/e165-*-red-examples.tsv' | wc -l) 行"
echo "  （对照）同一条换成整个 research/results/ 打出 $("${G[@]}" ls-files --others --exclude-standard -- research/results/ | wc -l) 行"
echo "崩溃验证员在工作区判绿、写标记：$(env -u SINGLEFS_STAGED_TREE python3 "$COPY/research/scripts/admission.py" stage-marker-write "$COPY" "$STAGE" | sed "s#$COMMON#<common-dir>#")"
WT="$WORK/staged-worktree"
"${G[@]}" worktree remove --force "$WT" >/dev/null 2>&1 || true
"${G[@]}" worktree add -q --detach "$WT" HEAD
echo "gate.sh --staged 的临时 worktree（HEAD + 暂存区，这里暂存区与 HEAD 相同）："
out="$(python3 "$COPY/research/scripts/admission.py" stage-marker-check "$WT" "$STAGE")"; echo "  stage-marker-check 退 $? ：$(echo "$out" | head -1 | sed "s#$COMMON#<common-dir>#")"
out="$(SINGLEFS_STAGED_TREE="$TREE" python3 "$COPY/research/scripts/admission.py" gate-reuse "$WT" "$STAGE")"; echo "  gate-reuse 退 $? （0 = 要跑）：$out" | sed "s#$COMMON#<common-dir>#"
rm -f "$untracked"
out="$(python3 "$COPY/research/scripts/admission.py" stage-marker-check "$COPY" "$STAGE")"; echo "（对照）删掉那份未跟踪产物后，工作区 stage-marker-check 退 $? ：$(echo "$out" | head -1 | sed "s#$COMMON#<common-dir>#")"
"${G[@]}" worktree remove --force "$WT"
rm -f "$COMMON"/singlefs-stage-green."$STAGE".* "$COMMON/singlefs-gate-environment.$STAGE"
"${G[@]}" update-ref -d refs/sop/staged-green
