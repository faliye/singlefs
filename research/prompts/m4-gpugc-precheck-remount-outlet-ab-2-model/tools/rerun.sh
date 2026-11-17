#!/usr/bin/env bash
# 卸载重挂出路判别子探针（第二批候选）的安全重跑包装：一族一次。
#
# 用法（整条交 task-server 跑；不套内存包装、不写线程：上限、线程与机器都由服务定）：
#   task-server/task submit -- bash <模型目录>/tools/rerun.sh --family <random-history|fill-then-fragment> --repo <仓根> --frozen <冻结副本> --work <运行目录>
#                              [--model <模型目录>] [--cost-tier-extra-slots <槽数>] [--cost-tier-configurations <个数>]
#     --family  random-history：第一族（随机历史，310 个配置、每个 400 步）；fill-then-fragment：第二族（先填满再打碎，155 个配置、每个 250 步）
#     --repo    仓根：要有 .git 与 task-server/task；只读它的 .claude/heavy-thread-default
#     --frozen  冻结副本：拷进这台机器的 $TMPDIR 下编，拷贝退出时删掉
#     --work    运行目录：要在工作根（~/.local/share/singlefs/work/）下、名字以 -run 收尾；服务撞内存上限重排同一条命令时它可能非空，先整个清掉再跑
#     --model   模型目录：默认是本脚本的上一层
#     --cost-tier-extra-slots     代价那一档两块盘在 1007 + 384 槽之外各多几槽（默认 4096）
#     --cost-tier-configurations  代价那一档跑几个配置（默认 31；0 不跑）
# 输出只写进 <运行目录>/out/：patch.out（拼拷贝）、build.log（编译）、oracle.out（探针的标准输出）；探针每跑完一个配置往标准错误打一行 PROGRESS（进任务日志）。
# 跑完在标准输出打四格表（DISCRIMINATOR_TABLE）、走那一遍重挂的挂钟（WALK_COST）、代价那一档的汇总（cost-tier RESULT 或 SKIPPED）、
# 自证（name=self_check…、name=self_checks bad=…）与 name=rerun_done。
# 退出码：0 跑完且自证全过；1 探针自证有不合；2 用法错或运行目录不合；3 探针估时超过 100 分钟、没跑完；4 编译失败；5 拼拷贝失败；
#         别的是探针被信号打断等（照原样交回）。
set -euo pipefail

refuse() {
  echo "  ✗ $1" >&2
  echo "用法：rerun.sh --family <random-history|fill-then-fragment> --repo <仓根> --frozen <冻结副本> --work <运行目录> [--model <模型目录>] [--cost-tier-extra-slots <槽数>] [--cost-tier-configurations <个数>]" >&2
  exit 2
}

FAMILY="" REPO_ROOT="" FROZEN="" WORK="" MODEL="" COST_TIER_EXTRA_SLOTS=4096 COST_TIER_CONFIGURATIONS=31
while (($#)); do
  case "$1" in
    --family|--repo|--frozen|--work|--model|--cost-tier-extra-slots|--cost-tier-configurations)
      (($# >= 2)) || refuse "$1 后面要跟一个值"
      case "$1" in
        --family) FAMILY="$2" ;;
        --repo) REPO_ROOT="$2" ;;
        --frozen) FROZEN="$2" ;;
        --work) WORK="$2" ;;
        --model) MODEL="$2" ;;
        --cost-tier-extra-slots) COST_TIER_EXTRA_SLOTS="$2" ;;
        --cost-tier-configurations) COST_TIER_CONFIGURATIONS="$2" ;;
      esac
      shift 2 ;;
    *) refuse "认不出参数：$1" ;;
  esac
done
case "$FAMILY" in
  random-history) CONFIGURATIONS=310 STEPS=400 ;;
  fill-then-fragment) CONFIGURATIONS=155 STEPS=250 ;;
  *) refuse "--family 要是 random-history 或 fill-then-fragment，给的是「$FAMILY」" ;;
esac
[[ "$COST_TIER_EXTRA_SLOTS" =~ ^[0-9]+$ ]] || refuse "--cost-tier-extra-slots 要是非负整数，给的是「$COST_TIER_EXTRA_SLOTS」"
[[ "$COST_TIER_CONFIGURATIONS" =~ ^[0-9]+$ ]] || refuse "--cost-tier-configurations 要是非负整数，给的是「$COST_TIER_CONFIGURATIONS」"
[[ -n "$REPO_ROOT" && -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/task-server/task" ]] || refuse "--repo「$REPO_ROOT」不像本仓（要有 .git 与 task-server/task）"
[[ -n "$FROZEN" && -d "$FROZEN/crates/singlefs-core" ]] || refuse "--frozen「$FROZEN」不像冻结副本（缺 crates/singlefs-core）"
case "$WORK" in
  */.local/share/singlefs/work/*-run) ;;
  *) refuse "--work「$WORK」不在工作根下或名字不以 -run 收尾：不清、不跑" ;;
esac
MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"
[[ -d "$MODEL/core" && -d "$MODEL/$FAMILY-family/probe" ]] || refuse "模型目录「$MODEL」缺 core/ 或 $FAMILY-family/probe/"
REPO_ROOT="$(cd "$REPO_ROOT" && pwd)"
FROZEN="$(cd "$FROZEN" && pwd)"
MODEL="$(cd "$MODEL" && pwd)"

# 运行目录非空（服务重排了同一条命令）先整个清掉，输出不在旧的上面接着写。
rm -rf -- "${WORK:?}"
mkdir -p "$WORK/out"
WORK="$(cd "$WORK" && pwd)"

# 仓副本拷到这台本地的临时目录，不放共享工作根（第二台往共享根写小文件慢）；输出照旧写 $WORK/out。退出时删掉。
COPY="${TMPDIR:?}/remount-outlet-probe-copy.$$"
trap 'rm -rf -- "${COPY:?}"' EXIT
mkdir -p "$COPY"
rsync -a --exclude target --exclude .git "$FROZEN/" "$COPY/"
if ! python3 "$MODEL/$FAMILY-family/tools/patch_the_copy.py" "$COPY" "$MODEL/$FAMILY-family/probe" > "$WORK/out/patch.out" 2>&1; then
  cat "$WORK/out/patch.out"
  exit 5
fi
if ! python3 "$MODEL/tools/add_the_discriminator_arm.py" "$COPY" "$MODEL" >> "$WORK/out/patch.out" 2>&1; then
  cat "$WORK/out/patch.out"
  exit 5
fi
cat "$WORK/out/patch.out"
# 编译目录：服务给了 CARGO_TARGET_DIR 就用它（相对路径按拷贝的根算），没给用拷贝里的 target（随拷贝删）。
if [[ -n "${CARGO_TARGET_DIR:-}" && "$CARGO_TARGET_DIR" != /* ]]; then
  export CARGO_TARGET_DIR="$COPY/$CARGO_TARGET_DIR"
fi
if ! (cd "$COPY" && cargo build --offline --release -p refusal-precheck-oracle-probe) > "$WORK/out/build.log" 2>&1; then
  tail -n 80 "$WORK/out/build.log"
  exit 4
fi
binary="${CARGO_TARGET_DIR:-$COPY/target}/release/refusal-precheck-oracle-probe"
echo "name=build done family=$FAMILY binary=$binary"
rc=0
"$binary" --configurations "$CONFIGURATIONS" --seed-base 1 --steps "$STEPS" --registration "$REPO_ROOT/.claude/heavy-thread-default" \
  --cost-tier-extra-slots "$COST_TIER_EXTRA_SLOTS" --cost-tier-configurations "$COST_TIER_CONFIGURATIONS" \
  > "$WORK/out/oracle.out" || rc=$?
echo "name=exit_code oracle=$rc"
grep -h -e "ESTIMATE" -e "WALL_SECONDS" -e "PROBE_RESULT" -e "DISCRIMINATOR_TABLE" -e "WALK_COST" -e "cost-tier RESULT" -e "cost-tier SKIPPED" \
  -e "name=self_check" -e "name=misrefusal_base_zero" -e "name=conjunct_fired_zero" "$WORK/out/oracle.out" || true
if [[ "$rc" -ne 0 ]]; then
  echo "  ✗ 探针没跑完或自证不合（退出码 $rc；3 是估时超过 100 分钟，1 是自证不合）"
  exit "$rc"
fi
echo "name=rerun_done family=$FAMILY work=$WORK model=$MODEL"
