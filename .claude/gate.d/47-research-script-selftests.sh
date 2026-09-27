#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: research/scripts/ 里声称有 --selftest 的脚本，自证都有阶段在跑、而且还会红
# gate-similar: 73-research-gate-lint.sh 同样扫 research/scripts/，但它判拒绝带不带出路与 shell 纪律，不跑任何脚本的自证
# gate-similar: 63-agent-write-scope.sh 跑的是 .claude/hooks/ 下钩子的自证，这里跑 research/scripts/ 下脚本的自证，两边的被测对象不重叠
#
# 判据：下面 runner 表里的每一条自证都要通过；research/scripts/ 里出现 `--selftest` 的脚本，要么有阶段在调用它
# （本阶段的 runner 表，或 15 号那种先赋给变量再调的写法），要么登记进 NOT_RUN_HERE 并写明为什么。份数不在注释里写死，成功行现算。
# 为什么：这几份自证此前都写着，却没有任何门禁阶段在跑（2026-09-12 现查 gate.d 与 .claude/scripts 零处调用）——自证只在写它的那天被跑过一次，
# 之后脚本改坏了也没人知道。2026-09-12 实测的两个坑都住在这里：
# ask-local.sh 判红时正文照样打到 stdout（一份作废输出顶着 -output-s1.md 落盘），
# 以及取法用不加引号的 $SPECS 传过 shell 被拆词（checklist-specs.py 就是为它写的）。
#
# check-segment-registry.py 不是三方论证脚本，但同一个道理成立：52 号阶段（段序列登记表与 E142 产物逐字比对）的
# 判别力样本只放三样合成输入，钉活代码的那几句与真产物的解析靠它自己的 --selftest 拿真文件测；
# 那份 --selftest 同样要有人在门禁里替它复跑，不然只在写它的那天跑过一次。
#
# 「有阶段在跑它」只认代码：注释行不算，出路文字也不算——echo / printf / howto / bad / die / say / print( 后面带引号的那几段
# 先去掉再认，一句「单跑 python3 research/scripts/foo.py --selftest 看它报什么」不是在跑它。
# 样本：fixtures/47-research-script-selftests.sh/red 放一份只在某个阶段的 echo 出路里被提到的 --selftest 脚本，必须报它没人跑；
# green 放一份真被阶段调用的与一份登记在 NOT_RUN_HERE 的，判绿并报对数。样本目录里放一个 .selftest-coverage-only 标记，
# 本阶段就不跑 runner 表（那几十份自证要真仓里的脚本，装不进样本），只判覆盖那一半；标记只在被判的仓不是本阶段所在的仓时认，
# 真仓与 --staged 的临时 worktree 里放了它也照跑 runner 表。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$ROOT" || exit 1
# 判别力样本只判覆盖那一半：样本目录里有标记、而且被判的不是本阶段所在的仓时，不跑 runner 表
coverage_only=0
if [[ -f .selftest-coverage-only && "$(pwd -P)" != "$STAGE_REPOSITORY" ]]; then coverage_only=1; fi

failed=0
checked=0
for runner in "bash research/scripts/ask-local-selftest.sh" "python3 research/scripts/checklist-specs.py --selftest" \
              "python3 research/scripts/quote-kb.py --selftest" "python3 research/scripts/kb-sections.py --selftest" \
              "python3 research/scripts/check-segment-registry.py --selftest" "python3 research/scripts/replace-once.py --selftest" \
              "python3 research/scripts/replace-batch.py --selftest" "python3 research/scripts/e152-tables.py --selftest" \
              "python3 research/scripts/agent-watch.py --selftest" "python3 research/scripts/quote-rust-items.py --selftest" \
              "python3 research/scripts/sweep-term.py --selftest" "python3 research/scripts/archive-past-rounds.py --selftest" \
              "python3 research/scripts/insert-row.py --selftest" \
              "bash research/scripts/cache-keepalive.sh --selftest" \
              "bash research/scripts/watch.sh --selftest" \
              "python3 research/scripts/stale-candidates.py --selftest" "python3 research/scripts/stale-candidates.py --benchmark" \
              "python3 research/scripts/test-environment-check.py --selftest" \
              "bash research/scripts/change-touches-crates.sh --selftest" \
              "bash research/scripts/verify-citations.sh --selftest" \
              "python3 research/scripts/agent-handover.py --selftest" "python3 research/scripts/decision-slim-check.py --selftest" \
              "python3 research/scripts/pdf-text.py --selftest" "python3 research/scripts/rules-sweep-audit.py --selftest" \
              "bash research/scripts/stage-must-run.sh --selftest" "bash research/scripts/changed-paths.sh --selftest" "bash research/scripts/gate-staged.sh --selftest" \
              "python3 research/scripts/admission.py --selftest" \
              "bash research/scripts/capped.sh --selftest" "bash research/scripts/stage-run-or-skip.sh --selftest" \
              "bash research/scripts/layer0-shard-run.sh --selftest" \
              "bash research/scripts/run-with-memory-cap.sh --selftest" "bash research/scripts/mutate.sh --selftest" \
              "python3 research/scripts/cite-check.py --selftest" "python3 research/scripts/kb-spec-check.py --selftest" \
              "python3 research/scripts/crash-case-check.py --selftest" "bash research/scripts/prove-red.sh --selftest" \
              "python3 research/scripts/apply-writer-patch.py --selftest" "python3 research/scripts/closeout-status.py --selftest" \
              "python3 research/scripts/compile-then-swap.py --selftest" \
              "python3 research/scripts/corruption-check.py --selftest"; do
  ((coverage_only)) && continue
  checked=$((checked + 1))
  output="$($runner 2>&1)"; rc=$?
  if [[ $rc -ne 0 ]]; then
    echo "  ✗ $runner 没过（退出码 $rc）："
    printf '%s\n' "$output" | sed 's/^/    /'   # gate-lint:detail
    echo "  → 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。"
    failed=1
  fi
done
((failed)) && exit 1

# ── 覆盖：research/scripts/ 里出现 `--selftest` 的脚本，都要有门禁阶段在跑它，或者登记进豁免表 ──
# 被扫集合现算，不手抄份数（show-me-test.md「跳过清单要与被扫集合出自同一份数据、现算」）。
# 认两种调用写法：脚本名后面直接跟 --selftest，或先赋给变量再 "$变量" --selftest（15 号那样）；注释行不算调用。
NOT_RUN_HERE=(
  "research/scripts/vm-bench.sh	自证要连起三次虚机，挂钟太重，不进每轮门禁"
)
coverage="$(python3 - "${NOT_RUN_HERE[@]}" <<'PY_COVERAGE'
import glob, os, re, sys
exempt = dict(row.split("\t", 1) for row in sys.argv[1:])
# 出路文字：echo / printf / howto / bad / die / say / print( 后面带引号的参数（可以连着几段）。里面提到的「x.py --selftest」不是在跑它
OUTPUT_TEXT = re.compile(r'''(?:\b(?:echo|printf|howto|bad|die|say)\b(?:\s+-\w+)*|\bprint\()(?:\s*f?(?:"(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*'))+''')
claimed = sorted(path for path in glob.glob("research/scripts/*")
                 if os.path.isfile(path) and "--selftest" in open(path, encoding="utf-8", errors="replace").read())
run = set()
for stage in sorted(glob.glob(".claude/gate.d/[0-9][0-9]-*.sh")):
    code = "\n".join(line for line in open(stage, encoding="utf-8").read().splitlines() if not line.lstrip().startswith("#"))
    code = OUTPUT_TEXT.sub(" ", code)
    variables = dict(re.findall(r'^\s*(\w+)="[^"\n]*?([\w.-]+\.(?:py|sh))"', code, re.M))
    for script in claimed:
        base = os.path.basename(script)
        if re.search(re.escape(base) + r'["\']?\s+--selftest', code):
            run.add(script)
        for variable, target in variables.items():
            if target == base and re.search(r'"\$' + variable + r'"\s+--selftest', code):
                run.add(script)
print(f"CLAIMED\t{len(claimed)}\tRUN\t{len(run)}")
for script in claimed:
    if script not in run and script not in exempt:
        print(f"MISSING\t{script}")
for path, why in exempt.items():
    if path not in claimed or path in run:
        print(f"STALE\t{path}\t{'已经有阶段在跑它' if path in run else '它已经不声称有 --selftest，或文件不在了'}")
    else:
        print(f"EXEMPT\t{path}\t{why}")
PY_COVERAGE
)" || { echo "  ✗ 自证覆盖的对账脚本没跑成"; echo "  → 怎么办：看上面 python 的报错修这一段；对账没跑等于这一格没验。"; exit 1; }
if grep -q '^MISSING' <<<"$coverage"; then
  echo "  ✗ 这些 research 脚本声称有 --selftest，却没有任何门禁阶段在跑它："
  grep '^MISSING' <<<"$coverage" | cut -f2 | sed 's/^/      /'   # gate-lint:detail
  echo "  → 怎么办：把它加进上面的 runner 表；确实不该每轮跑的，登记进 NOT_RUN_HERE 并写明为什么——没人跑的自证只在写它的那天跑过一次。"
  exit 1
fi
if grep -q '^STALE' <<<"$coverage"; then
  echo "  ✗ NOT_RUN_HERE 里有过期的豁免："
  grep '^STALE' <<<"$coverage" | cut -f2,3 | sed 's/\t/：/; s/^/      /'   # gate-lint:detail
  echo "  → 怎么办：从 NOT_RUN_HERE 里删掉这一行；留着它会让人以为那份脚本还被绕开着。"
  exit 1
fi
read -r _ claimed_count _ run_count < <(grep '^CLAIMED' <<<"$coverage")
echo "  ✓ research 脚本的自证都通过（本阶段跑了 $checked 条；research/scripts/ 里声称有 --selftest 的 $claimed_count 份中 $run_count 份有门禁阶段在跑）"
if ((coverage_only)); then
  echo "    runner 表没跑：$ROOT 是判别力样本（有 .selftest-coverage-only 标记、不是本阶段所在的仓），只判覆盖那一半"
fi
echo "    没跑的 $(grep -c '^EXEMPT' <<<"$coverage") 份（登记在本阶段的 NOT_RUN_HERE）："
grep '^EXEMPT' <<<"$coverage" | cut -f2,3 | sed 's/\t/：/; s/^/      /'
