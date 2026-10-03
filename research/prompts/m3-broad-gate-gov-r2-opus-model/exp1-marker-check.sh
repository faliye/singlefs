#!/usr/bin/env bash
# H1 第 3 条：写标记那一侧核了，查标记那一侧仍按工作区算指纹。用法：exp1-marker-check.sh <冻结副本树根> <草稿目录>
# ① 拷 marker-clean-written 样本、setup.sh；② cd 进样本根照样本档起法跑阶段（写标记 A）；③ 暂存 lib.rs 的另一份 B、再把工作区写回 A；
# ④ admission.py stage-marker-check（commit-stages 第 1 步那一句）；⑤ 同一现场再起阶段（第一格判暂存树）。
set -uo pipefail
tree="$1"; work="$2"
root="$work/exp1-root"; rm -rf "${root:?}"; mkdir -p "$root"
cp -a "$tree/.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/library-version-is-staged-tree-marker-clean-written/." "$root/"
( cd "$root" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash setup.sh >/dev/null 2>&1 ); echo "setup exit=$?"
( cd "$root" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash "$tree/.claude/gate.d/checker-tier-broad-mutation.sh" "$root" --force ); echo "stage(A) exit=$?"
cp "$root/crates/sample/src/lib.rs" "$work/exp1-lib-A.rs"
printf 'pub fn add(left: u32, right: u32) -> u32 {\n    left - right\n}\n' > "$root/crates/sample/src/lib.rs"
git -C "$root" add crates/sample/src/lib.rs
cp "$work/exp1-lib-A.rs" "$root/crates/sample/src/lib.rs"
echo "--- 暂存区与工作区（crates）"; git -C "$root" status --short -- crates
echo "--- 暂存区 lib.rs 第 2 行：$(git -C "$root" show :crates/sample/src/lib.rs | sed -n 2p)；工作区 lib.rs 第 2 行：$(sed -n 2p "$root/crates/sample/src/lib.rs")"
echo "--- stage-marker-check（标记里的跑完时刻换成「时刻略」：报告与产物只写日期）"; { python3 "$tree/research/scripts/admission.py" stage-marker-check "$root" checker-tier-broad-mutation.sh; echo "marker-check exit=$?"; } | sed -E "s/T[0-9]{2}:[0-9]{2}:[0-9]{2}Z/T（时刻略）Z/"
echo "--- 同一现场再起阶段"; ( cd "$root" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash "$tree/.claude/gate.d/checker-tier-broad-mutation.sh" "$root" --force ); echo "stage(B staged) exit=$?"
rm -rf "${root:?}" "$work/exp1-lib-A.rs"
