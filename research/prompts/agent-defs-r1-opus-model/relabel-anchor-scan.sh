#!/usr/bin/env bash
# 模型：copy-repo-sequences.sh ① 只翻了 D22 已定项 7 一个取样点。这里把「变异表锚点里出现过的每一个已定项」逐个翻回未定，
# 各在一份新副本里跑 relabel-item.py（改写逻辑原样，只把那一条的状态在内存里改成「未」，决策正文不动），再跑 33 号。
# 另数 replay.sh 钉住的 E142 产物里印着这个标签（去掉空白的写法）的行数——那几行在源码字符串被改写之后复跑对不上。
#   SINGLEFS_ROOT=/home/fy5090/code/singlefs WORK_PARENT=/tmp/claude-1000/agent-defs-r1-opus bash relabel-anchor-scan.sh
set -uo pipefail
ROOT="${SINGLEFS_ROOT:-/home/fy5090/code/singlefs}"
PARENT="${WORK_PARENT:-/tmp/claude-1000/agent-defs-r1-opus}"
WORK="$(mktemp -d "$PARENT/anchor-scan.XXXXXX")"
trap 'rm -rf "${WORK:?}"' EXIT
items="$(grep -ohE 'D[0-9]+ 已定项 [0-9]+' "$ROOT"/research/mutations/*.tsv | sort -u)"
echo "name=items count=$(printf '%s\n' "$items" | grep -c .)"
product="$(awk -F'|' '$1 == "E142" { print $4 }' "$ROOT/research/scripts/replay.sh")"
while read -r decision _ item; do
  [[ -z "$decision" ]] && continue
  rm -rf "${WORK:?}/repo"
  rsync -a --exclude target --exclude .git "$ROOT/" "$WORK/repo/"
  (
    cd "$WORK/repo" || exit 2
    python3 - "$WORK/repo" "$decision" "$item" <<'PY'
import importlib.util, io, contextlib, sys
root, decision, item = sys.argv[1], sys.argv[2], int(sys.argv[3])
spec = importlib.util.spec_from_file_location("relabel", root + "/research/scripts/relabel-item.py")
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
real_loader = module.load_library
def flipped_loader(path):
    library = real_loader(path)
    real_map = library.load_map
    def load_map():
        item_map, names = real_map()
        item_map[decision][item] = "未"
        return item_map, names
    library.load_map = load_map
    return library
module.load_library = flipped_loader
with contextlib.redirect_stdout(io.StringIO()) as buffer:
    code, _ = module.relabel(root, decision, item, False)
rs = sum(1 for line in buffer.getvalue().split("\n") if line.strip().startswith("research/") and ".rs：" in line)
print(f"relabel_rc={code} rs_files={rs}", end=" ")
PY
    out33="$(bash .claude/gate.d/33-mutation-tables.sh 2>&1)"; rc33=$?
    rotted="$(grep -cE '^\s+e[0-9]+_.* 的 .*命中 0 次' <<<"$out33")"
    compact="${decision}已定项${item}"
    spaced="${decision} 已定项 ${item}"
    pinned_hits=0
    while read -r pinned; do
      [[ -n "$pinned" && -f "research/results/$pinned" ]] || continue
      if grep -qF -e "$compact" -e "$spaced" "research/results/$pinned"; then pinned_hits=$((pinned_hits + 1)); fi
    done < <(awk -F'|' '/^E[0-9]+\|/ { print $4 }' research/scripts/replay.sh)
    printf 'stage33_rc=%s rotted_entries=%s e142_product_lines=%s replay_pinned_products_with_label=%s\n' "$rc33" "$rotted" "$(grep -c "$compact" "research/results/$product")" "$pinned_hits"
  ) | sed "s/^/name=scan item=${decision}_${item} /"
done <<<"$items"
echo "name=done"
