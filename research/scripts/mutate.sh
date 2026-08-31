#!/usr/bin/env bash
# 证明「这些测试会红」：逐条注入一个已知的破坏，跑测试，记下红了哪几个，然后还原。
#
#   mutate.sh <bin 名> <源文件> <变异表>
#
# 变异表每行三段，用制表符分隔：变异名 <TAB> 原文 <TAB> 替换文（原文/替换文里的 \n 表示换行）
# 替换必须命中，命中数为 0 直接报错退出 —— 静默失效的替换会让整份证明变成假的
# （singlefs-ai-sop/rules/command-safety.md「脚本改文件之后要回读确认」）。
set -uo pipefail
BIN="$1"; SRC="$2"; TABLE="$3"

# ⚠️ **先确认「改的文件」与「跑的二进制」对得上。**
# 不确认的话，变异会改 X 而跑 Y 的测试：一条真能被抓的破坏被报成盲区
# （2026-08-29 实测踩过：`e7_index.rs` 配上 crate 主二进制的名字，
# 三条变异全被误报成盲区，换对名字后三条全被抓）。
# 反方向更坏：Y 的测试恰好因别的原因红了，会被记成「变异被抓」。
_manifest="$(dirname "$SRC")/../../Cargo.toml"
[[ -f "$_manifest" ]] || _manifest="e7-index-bench/Cargo.toml"
_declared="$(awk -v src="$SRC" '
  /^\[\[bin\]\]/ { name=""; path=""; next }
  /^name *=/ { gsub(/.*= *"|"/,""); name=$0; next }
  /^path *=/ { gsub(/.*= *"|"/,""); path=$0;
               if (src ~ path"$") print name; next }
' "$_manifest" 2>/dev/null | head -1)"
if [[ -n "$_declared" ]]; then
  if [[ "$_declared" != "$BIN" ]]; then
    echo "mutate: 源文件 $SRC 在 Cargo.toml 里声明的二进制是 '$_declared'，不是 '$BIN'" >&2
    echo "        改的文件与跑的测试对不上，整份证明作废。用： mutate.sh $_declared $SRC $TABLE" >&2
    exit 6
  fi
else
  # 没有显式 [[bin]] ⇒ cargo 自动发现，名字就是文件名去掉扩展名
  _stem="$(basename "$SRC" .rs)"
  if [[ "$_stem" != "$BIN" ]]; then
    echo "mutate: $SRC 没有显式 [[bin]]，自动发现的名字是 '$_stem'，不是 '$BIN'" >&2
    echo "        改的文件与跑的测试对不上，整份证明作废。用： mutate.sh $_stem $SRC $TABLE" >&2
    exit 6
  fi
fi

BAK="$(mktemp)"; cp "$SRC" "$BAK"
restore() { cp "$BAK" "$SRC"; }
trap restore EXIT

# 基线必须全绿，否则后面「红了」分不清是变异造成的还是本来就红
if ! cargo test --release --bin "$BIN" >/dev/null 2>&1; then
  echo "mutate: 基线就是红的，先修好再来" >&2; exit 2
fi
echo "基线：全绿"

fail=0
while IFS=$'\t' read -r name from to; do
  [[ -z "${name:-}" || "${name:0:1}" == "#" ]] && continue
  restore
  NAME="$name" FROM="$from" TO="$to" SRC="$SRC" python3 - <<'PY' || { echo "mutate: [$name] 替换没命中，证明作废" >&2; exit 3; }
import os,sys
src=os.environ["SRC"]; s=open(src).read()
f=os.environ["FROM"].replace("\\n","\n"); t=os.environ["TO"].replace("\\n","\n")
n=s.count(f)
if n!=1:
    sys.stderr.write("命中 %d 次（要求恰好 1 次）：%r\n"%(n,f)); sys.exit(1)
open(src,"w").write(s.replace(f,t))
PY
  out="$(cargo test --release --bin "$BIN" 2>&1)"
  # ⚠️ **编译失败不等于「没抓到」。** 一个改坏了语法或穷尽性的变异根本跑不到测试，
  # 那时既不能记成「测试抓到了」，也不能记成「测试没抓到」——它是一条**无效变异**。
  # 混成一类的话，一个编译不过的变异会被报成测试盲区，把人引去改测试（实测踩过）。
  # ⚠️ 判据是「测试进程有没有跑起来」，**不是「输出里有没有 error」**——
  # `cargo test` 在测试变红时也会打 `error: test failed`，
  # 拿它当编译失败的判据会把每一条成功的变异都误判成无效（实测踩过，全套复跑才发现）。
  # ⚠️ **不许写成 `printf ... | grep -q ...`。** `grep -q` 命中后立刻退出并关闭管道，
  # 上游 printf 收到 SIGPIPE；本脚本开了 `pipefail` ⇒ **整条管道判失败，命中被读成没命中**。
  # 实测：那样写会让每一条「测试确实变红了」的变异都被误报成「编译失败」。
  # 这正是 `.claude/singlefs-ai-sop/rules/command-safety.md`「管道里的退出码不是你想要的那个」。
  # ⇒ 用 here-string，根本不建管道。
  if ! grep -q '^running [0-9]* test' <<<"$out"; then
    echo "⏭  [$name] 变异导致编译失败，本条无效（不计入盲区，也不算命中）"
    continue
  fi
  # ⚠️ **第四种结局：测试进程根本没跑完**（挂死被 OOM 杀、段错误、abort）。
  # 它既不是「编译失败」也不是「一个测试都没红」——报成后者会把人引去改测试，
  # 而实际发生的事是「这条破坏让被测代码不终止了」。实测踩过：
  # 摘掉一条循环终止条件之后 recover 无限接受记录，进程 SIGKILL，
  # 当时被报成「没有被任何检查看见」。
  if grep -q "process didn't exit successfully" <<<"$out"; then
    sig="$(sed -n 's/.*(signal: \([0-9]*\).*/\1/p' <<<"$out" | head -1)"
    echo "💥 [$name] 测试进程没跑完（signal ${sig:-?}）——破坏被看见了，但不是断言抓到的"
    continue
  fi
  # ⚠️ **不许枚举字符类——这个坑犯过两次，形状一模一样。**
  # 2026-08-29 写成 `[a-z_]*`，名字带数字的测试匹配不上；
  # 2026-08-31 写成 `[A-Za-z0-9_]*`，**中文测试名**匹配不上——E57 的 9 条变异
  # 全被报成「一个测试都没红」，实际 9 条全红。两次都是**谎报盲区**（把人引去补
  # 一条本来就有的检查），而不是漏报命中，所以两次都很难自己发现。
  # ⇒ 改成「`...` 之前的都算名字」：Rust 的测试名可以是任意 Unicode 标识符，
  #   枚举允许的字符永远追不上。下面那条旧注释留着，它记的是同一个坑的第一次。
  # ⚠️ **字符类必须含数字与大写。** 写成 `[a-z_]*` 时，任何名字里带数字的测试
  # （例如 `..._analytic_io_per_op_of_1_9375`、`..._reads_exactly_122`）匹配不上，
  # `red` 为空 ⇒ **一条确实红了的变异被报成「一个测试都没红」**。
  # 方向是谎报盲区，会把人引去补一条本来就有的检查。2026-08-29 实测踩过并修。
  red="$(sed -n 's/^test tests::\(.*\) \.\.\. FAILED$/\1/p' <<<"$out" | paste -sd, -)"
  if [[ -z "$red" ]]; then
    echo "❌ [$name] 一个测试都没红 —— 这条破坏没有被任何检查看见"
    fail=1
  else
    echo "✅ [$name] 红：$red"
  fi
done < "$TABLE"

restore
cargo test --release --bin "$BIN" >/dev/null 2>&1 || { echo "mutate: 还原后没回到全绿" >&2; exit 4; }
echo "已还原，基线仍全绿"
exit $fail
