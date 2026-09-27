#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的测试源码与 harness 档的耗时表，除了跑门禁本身就要的 bash、python3 之外没有环境要求
# gate-stage: 用例住对档、粒度对：崩溃枚举住 checker 档，harness 档一条用例一个场景、耗时用例标记与耗时表对得上，checker 档测试文件声明它测哪几个模块
# gate-similar: 47-research-script-selftests.sh 只跑 crash-case-check.py 与 harness-test-timing.py 的 --selftest（判别力），不在真仓上判；这里在真仓上判
# gate-similar: 94-checker-implementation-disjoint.sh 判三个包之间的依赖方向，对象是 Cargo.toml 与 use 语句；这里判的是用例本身的归属与粒度
# gate-similar: 13-vague-names.sh 判函数名与类型名只由空泛词拼成，对象是全部 .rs 的声明；这里不看名字
#
# 判据（两个脚本，任一个红整道红）：
#   ① research/scripts/crash-case-check.py <项目根>：崩溃枚举用例（调 enumerate_layer0 一族、或自己逐个造崩溃状态）都住在
#      checker 档包、标了 #[ignore] 的都登记了；harness 档一条用例不在循环里每轮新建池；checker 档每个测试文件第一行的
#      「//! checker 档模块：…」与它导入的模块逐个相同。
#   ② research/scripts/harness-test-timing.py check <项目根>：标了「harness 耗时用例」的用例在 crates/singlefs-harness/test-timing.tsv 里
#      量过、单线程耗时过门槛；表里过门槛的都标了。表里没有的用例只报条数，不判红。
# 管不到的：耗时表是不是最近量的（表头写着量的那次提交）；一条用例里不建池的多场景（改的是同一个池）；认不出的写法照
# crash-case-check.py 文件头。
# 样本：fixtures/14-test-granularity.sh/red 有一条每轮新建池的 harness 用例、一份没声明模块的 checker 档测试文件、一条快用例标了耗时用例；
# green 各样都对。
#
#   bash .claude/gate.d/14-test-granularity.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
SCRIPTS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts"
if [[ ! -d crates/singlefs-harness && ! -d crates/singlefs-checker-tier ]]; then
  echo "  ! 没有 crates/singlefs-harness 与 crates/singlefs-checker-tier：无对象可判"
  exit 77
fi
if python3 "$SCRIPTS/crash-case-check.py" "$ROOT"; then placement_exit_code=0; else placement_exit_code=$?; fi
if python3 "$SCRIPTS/harness-test-timing.py" check "$ROOT"; then timing_exit_code=0; else timing_exit_code=$?; fi
if (( placement_exit_code != 0 || timing_exit_code != 0 )); then
  echo "  ✗ 用例的归属与粒度：crash-case-check 退 $placement_exit_code、harness-test-timing check 退 $timing_exit_code（逐处列在上面）"
  echo "     → 怎么办：照上面各自那一句出路改；两份脚本的判据写在各自文件头，规则在 .claude/rules/verification.md「harness 档里再分轻用例与耗时用例」与「崩溃枚举用例住哪、怎么登记」。"
  exit 1
fi
echo "  ✓ 用例的归属与粒度都对（两份脚本各自报的查了多少条在上面两行）"
