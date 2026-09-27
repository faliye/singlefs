#!/usr/bin/env bash
# G3 出口：仓里换进之后的自证与门禁（每件退出码进自己的文件）
set -u
D=/tmp/claude-1000/gate-batch-m2-g3; O=$D/final; R=/home/fy5090/code/singlefs
cd $R; rm -f $O/*.exit
job() { local name=$1; shift; { nice -n 19 "$@" > $O/$name.log 2>&1; echo $? > $O/$name.exit; } & }
job admission env SINGLEFS_GATE_FULL=1 python3 research/scripts/admission.py --selftest
job lib python3 .claude/hooks/lib_heavy_tests.py --selftest
job guard bash .claude/hooks/heavy-test-guard.sh --selftest
job shard bash research/scripts/layer0-shard-run.sh --selftest
wait
job stage54 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $D/stage54-final/.claude/gate.d
job g47 bash .claude/gate.d/47-research-script-selftests.sh
job g62 bash .claude/gate.d/62-stage-owners.sh
job g63 bash .claude/gate.d/63-agent-write-scope.sh
wait
job g72 bash .claude/gate.d/72-agent-def-adversarial-review.sh
job g73 bash .claude/gate.d/73-research-gate-lint.sh
job doclint bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .
job ruleslint env RULES_LINT_DIR=.claude/rules "RULES_LINT_FILES=CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md" bash .claude/singlefs-ai-sop/scripts/rules-lint.sh .
wait
job gatelint env GATE_LINT_DIR=.claude/gate.d bash .claude/singlefs-ai-sop/scripts/gate-lint.sh .
job shelllint env SHELL_LINT_DIR=.claude/gate.d bash .claude/singlefs-ai-sop/scripts/shell-lint.sh .
job preflightlint python3 .claude/singlefs-ai-sop/scripts/preflight-lint.py
job overlap python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py
wait
echo "files=$(ls $O/*.exit | wc -l)"
for n in admission lib guard shard stage54 g47 g62 g63 g72 g73 doclint ruleslint gatelint shelllint preflightlint overlap; do printf '%s exit=%s\n' $n "$(cat $O/$n.exit)"; done
