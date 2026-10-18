#!/usr/bin/env bash
# 辩方腿的几个小核对（K1、K4 用）：
#   一、gate-staged.sh 第 20 行的参数摘除：拿仓里真的 preflight.sh（不起任何门禁阶段、不起 gate-staged.sh），对一个空脚本喂 `--force`，看摘完剩什么、交给里层的参数里有没有 --force。
#   二、里层门禁用的 git worktree 看得到主仓的 refs/sop/staged-green（庚二只改取法就够，不用另传）。
#   三、gate.sh 开头清掉的那一组 git 环境变量里有没有 GIT_INDEX_FILE（外壳想用私有索引把暂存区冻住再交给里层，行不行）。
# 用法：bash small-checks.sh <仓根> <工作目录（新建）>
set -uo pipefail
REPO="$1"; WORK="$2"; mkdir -p "$WORK" || exit 2
cat > "$WORK/dummy.sh" <<'DUMMY'
#!/usr/bin/env bash
# admission: always 演示参数摘除用的空脚本，每次都判
# run-condition: none 没有环境要求，只是个空壳
DUMMY
echo "== 一、preflight 摘 --force"
( cd "$REPO" && bash -c 'source .claude/singlefs-ai-sop/scripts/preflight.sh; preflight "$1" --force /some/root extra; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}; echo "after-strip: n=$# args=[$*] force_given=$PREFLIGHT_FORCE_GIVEN"; echo "inner would run: gate.sh --staged $*"' _ "$WORK/dummy.sh" 2>&1 )
echo "== 二、worktree 与主仓共用 refs/sop"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
git init -q -b master "$WORK/main" && git -C "$WORK/main" -c user.name=m -c user.email=m@example.invalid commit -q --allow-empty -m a \
  && git -C "$WORK/main" update-ref refs/sop/staged-green HEAD && git -C "$WORK/main" worktree add -q --detach "$WORK/wt" HEAD \
  && echo "worktree_sees_equal_main=$([[ "$(git -C "$WORK/wt" rev-parse refs/sop/staged-green)" == "$(git -C "$WORK/main" rev-parse refs/sop/staged-green)" ]] && echo yes || echo no)"
echo "== 三、git rev-parse --local-env-vars 里的索引变量"
git rev-parse --local-env-vars | tr ' ' '\n' | grep -n '^GIT_INDEX_FILE$' | sed 's/^/found_at_position=/'
