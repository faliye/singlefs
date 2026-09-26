#!/usr/bin/env bash
# G2：崩溃验证员第 1 步两条核对命令（改后：未跟踪文件三道共用 `crates litmus .lkmm-static-only`）
# 与改前（未跟踪文件按每道自己的输入）在几种「别的会话留下的未跟踪文件」上各停哪几道；
# 另列今天的仓里哪一道真读那个文件。临时仓建在 <草稿目录> 下。
# 用法：bash g2-untracked.sh <仓根> <草稿目录>
set -uo pipefail
real="$1"; draft="$2"; repo="$draft/g2-repo"; rm -rf "$repo"; mkdir -p "$repo"; cd "$repo" || exit 1
git init -q .; git config user.email m@x; git config user.name m
mkdir -p crates/singlefs-harness/tests crates/singlefs-harness/src/bin litmus .claude/gate.d .claude/scripts research/scripts research/results
for f in Cargo.toml Cargo.lock crates/singlefs-harness/tests/a.rs litmus/first-txn-root-implies-units.litmus .claude/gate.d/55-qemu-first-transaction.sh .claude/gate.d/57-lkmm.sh .claude/gate.d/59-crates-mutation-replay.sh .claude/scripts/lkmm.sh research/scripts/run-with-memory-cap.sh research/scripts/replay.sh research/results/e142.out; do echo x > "$f"; done
cp "$real/.claude/gate.d/stage-inputs.tsv" .claude/gate.d/stage-inputs.tsv
git add -A; git commit -qm base
inputs_of() {  # 定义第 1 步：55、59 取 stage-inputs.tsv 那一行，57 取 litmus crates .claude/scripts/lkmm.sh，都加阶段脚本
  case "$1" in
    57-lkmm.sh) echo "litmus crates .claude/scripts/lkmm.sh .claude/gate.d/$1" ;;
    *) echo "$(awk -F'\t' -v s="$1" '$1==s{print $2}' .claude/gate.d/stage-inputs.tsv) .claude/gate.d/$1" ;;
  esac
}
scenario() {
  local label="$1" path="$2"
  mkdir -p "$(dirname "$path")"; echo wip > "$path"
  echo "== $label：别的会话留下未跟踪的 $path"
  for s in 55-qemu-first-transaction.sh 57-lkmm.sh 59-crates-mutation-replay.sh; do
    # shellcheck disable=SC2046
    git diff --quiet -- $(inputs_of "$s"); d=$?
    new="$(git ls-files --others --exclude-standard -- crates litmus .lkmm-static-only)"
    # shellcheck disable=SC2046
    old="$(git ls-files --others --exclude-standard -- $(inputs_of "$s"))"
    printf '  %s\tgit diff 退 %s\t改后未跟踪那条：%s\t改前（按这道自己的输入）：%s\n' "$s" "$d" "$([[ -n "$new" ]] && echo 停 || echo 照跑)" "$([[ -n "$old" ]] && echo 停 || echo 照跑)"
  done
  rm -f "$path"
}
scenario 'U1 实现员在别的会话里起草一条新 litmus' litmus/wip-fsync-order.litmus
scenario 'U2 别的会话调 57 号判别力时在仓根放了标记' .lkmm-static-only
scenario 'U3 别的会话新建一个测试文件（该停）' crates/singlefs-harness/tests/wip.rs
echo "== 今天的仓里谁读 litmus/ 下的文件"
printf '  55 号脚本里 litmus 出现次数：%s\n' "$(grep -c litmus "$real/.claude/gate.d/55-qemu-first-transaction.sh")"
printf '  crates/ 里在运行时读 litmus 的测试文件：%s\n' "$(grep -rl --include='*.rs' '\.\./\.\./litmus' "$real/crates" | sed "s#$real/##" | tr '\n' ' ')"
printf '  crates/mutations.tsv 里跑这个测试文件的行数：%s\n' "$(grep -c 'publish_order_matches_litmus' "$real/crates/mutations.tsv")"
printf '  那个测试读哪几个 litmus：%s\n' "$(grep -o '"first-txn-[a-z-]*\.litmus"' "$real/crates/singlefs-harness/tests/publish_order_matches_litmus.rs" | tr '\n' ' ')"
printf '  57 号读仓根标记：%s\n' "$(grep -n 'lkmm-static-only' "$real/.claude/gate.d/57-lkmm.sh" | sed -n 2p)"
