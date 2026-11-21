#!/usr/bin/env bash
# 正推腿的模型：用真 cargo、真 rustc 编 make-workspace.sh 造的小工作区，读 target 里每个编译单元的 .d（rustc 的 dep-info），
# 逐个消费包问一句：provider 包里的文件列在它的 .d 里没有。门禁 crate-cross-references 的判法靠的就是这个前提（scripts/gate/crate-cross-references-check.py 文件头）。
# 用法：run-dep-info-model.sh <运行目录>（rerun.sh 的 rerun_steps 起它；运行目录是 rerun.sh 的 --work）
# 输出：<运行目录>/result.txt，与 <模型目录>/out/expected.txt 逐行比；末行 MODEL_RESULT identical（退 0）或 MODEL_RESULT differs（退 1）。
# 只用本地包、不联网、不读仓；编译目录在运行目录里。
set -euo pipefail
(($# == 1)) || { echo "用法：run-dep-info-model.sh <运行目录>" >&2; exit 2; }
WORK="$1"
TOOLS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODEL="$(cd "$TOOLS/.." && pwd)"
WS="$WORK/ws"
TARGET="$WORK/target"
mkdir -p "$TARGET"
bash "$TOOLS/make-workspace.sh" "$WS"
export CARGO_TARGET_DIR="$TARGET"
{ cargo --version; rustc -vV; } > "$WORK/toolchain.txt"

packages=(provider c00 c01 c02 c03 c04 c05 c06 c07 c08 c09 c10 c11 c12 c13 c14)
package_arguments=()
for package in "${packages[@]}"; do
  package_arguments+=(-p "$package")
done
if ! (cd "$WS" && cargo check --offline --all-targets --all-features "${package_arguments[@]}") > "$WORK/cargo-check-main.log" 2>&1; then
  echo "  ✗ 主体 cargo check 没编过（c15 之外的包都应当编得过）："
  tail -n 40 "$WORK/cargo-check-main.log"
  echo "MODEL_RESULT differs"
  exit 1
fi
if (cd "$WS" && cargo check --offline --all-targets --all-features -p c15) > "$WORK/cargo-check-c15.log" 2>&1; then c15_exit=0; else c15_exit=$?; fi

names=(
  "c00 same-package-control" "c01 path-attribute" "c02 cfg-attr-path" "c03 renamed-include-macro" "c04 bracket-delimiters" "c05 raw-string-path"
  "c06 inline-module-base" "c07 absolute-path" "c08 non-rs-intermediate" "c09 target-directory" "c10 hidden-directory" "c11 data-files"
  "c12 build-script-path-attribute" "c13 include-rs-in-library" "c14 build-script-runtime-read"
)
result="$WORK/result.txt"
: > "$result"
for entry in "${names[@]}"; do
  case_id="${entry%% *}"
  case_name="${entry#* }"
  # 这个消费包的每个 .d（库、测试、构建脚本单元）：里面写了「crates/<包>/」的就是它的
  units="$(grep -rl --include='*.d' "crates/$case_id/" "$TARGET" 2>/dev/null || true)"
  listed=no
  if [[ -n "$units" ]] && grep -qE '(crates|\.\.)/provider/' $units; then
    listed=yes
  fi
  line="$case_id $case_name provider-listed=$listed"
  if [[ "$case_id" == c14 ]]; then
    reported=no
    if grep -rqF 'rerun-if-changed=../provider/src/k.wgsl' "$TARGET"/debug/build/c14-*/output 2>/dev/null; then reported=yes; fi
    line="$line rerun-if-changed-in-build-output=$reported"
  fi
  echo "$line" >> "$result"
done
failed=no
if ((c15_exit != 0)); then failed=yes; fi
echo "c15 compile-failure cargo-exit-nonzero=$failed" >> "$result"

echo "---- result.txt"
cat "$result"
echo "---- 与 out/expected.txt 比"
if diff -u "$MODEL/out/expected.txt" "$result"; then
  echo "MODEL_RESULT identical"
else
  echo "MODEL_RESULT differs"
  exit 1
fi
