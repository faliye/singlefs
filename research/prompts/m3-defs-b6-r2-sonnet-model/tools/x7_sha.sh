#!/usr/bin/env bash
# X7：豁免表带 sha256 之后，对豁免表里一份 E171 入口做各种改动，判 exempt-check / lib_long_run.py script / detached-run.sh long-run-check。
# 在 <仓根> 的拷贝（不含 research）里做，每例从干净拷贝重来。用法：x7_sha.sh <仓根（after）> <草稿目录（要不存在）>
set -u
src=$1; work=$2
[[ ! -e "$work" ]] || { echo "  ✗ $work 已存在" >&2; exit 2; }
mkdir -p "$work"
E=scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes
S=$E/run_segment_one_revision_11_multi_host.sh
fresh() { rm -rf "$work/t"; mkdir -p "$work/t"; rsync -a --exclude research --exclude multi-host.env "$src/" "$work/t/"; cp "$work/t/.claude/long-run-live-hosts-exempt" "$work/base-table"; }
judge() {   # judge <标签>
  local label=$1 a b c
  (cd "$work/t" && python3 -B .claude/hooks/lib_long_run.py exempt-check "$work/t" --base-table "$work/base-table" >"$work/ec.out" 2>&1); a=$?
  (cd "$work/t" && python3 -B .claude/hooks/lib_long_run.py script "$work/t" -- bash "$work/t/$S" >/dev/null 2>"$work/sc.err"); b=$?
  (cd "$work/t" && bash scripts/hosts/detached-run.sh long-run-check -- bash "$S" >/dev/null 2>&1); c=$?
  printf '%s\texempt-check=%s\tlib_long_run_script=%s\tlong-run-check=%s\t%s\n' "$label" "$a" "$b" "$c" "$(grep -m1 -E '✗|✓' "$work/ec.out" | cut -c1-150)"
}
fresh; judge "0 原样"
fresh; chmod +x "$work/t/$S"; judge "1 只改执行位 chmod +x"
fresh; chmod -x "$work/t/$S"; judge "2 只改执行位 chmod -x"
fresh; printf '\n' >> "$work/t/$S"; judge "3 文件末尾多一个换行"
fresh; sed -i 's/$/\r/' "$work/t/$S"; judge "4 换行符改 CRLF"
fresh; python3 - "$work/t/$S" <<'PY'
import sys
p = sys.argv[1]; data = open(p, "rb").read()
open(p, "wb").write(data.rstrip(b"\n"))
PY
judge "5 去掉文件末尾换行"
fresh; python3 -I - "$work/t/$S" "$work/t/.claude/long-run-live-hosts-exempt" <<'PY'
import hashlib, re, sys
script, table = sys.argv[1:3]
open(script, "ab").write("# 改动一行，同时把表里的 sha256 换成新值\n".encode("utf-8"))
new = hashlib.sha256(open(script, "rb").read()).hexdigest()
text = open(table, encoding="utf-8").read()
name = "run_segment_one_revision_11_multi_host.sh"
text = re.sub(r"(" + re.escape(name) + r"[ \t]{2,})[0-9a-f]{64}", r"\g<1>" + new, text)
open(table, "w", encoding="utf-8").write(text)
PY
judge "6 改脚本并同步改表里的 sha256（与基准表比：路径没涨）"
fresh; python3 -I - "$work/t/$S" "$work/t/.claude/long-run-live-hosts-exempt" <<'PY'
import hashlib, re, sys
script, table = sys.argv[1:3]
text = open(script, encoding="utf-8").read().replace("现算", "现算（改）", 1)
open(script, "w", encoding="utf-8").write(text)
new = hashlib.sha256(open(script, "rb").read()).hexdigest()
t = open(table, encoding="utf-8").read()
name = "run_segment_one_revision_11_multi_host.sh"
open(table, "w", encoding="utf-8").write(re.sub(r"(" + re.escape(name) + r"[ \t]{2,})[0-9a-f]{64}", r"\g<1>" + new, t))
PY
judge "7 改脚本正文里的 现算 说法并同步 sha256"
fresh; cp "$work/t/$S" "$work/t/$E/copy_of_rev11.sh"; (cd "$work/t" && bash scripts/hosts/detached-run.sh long-run-check -- bash "$E/copy_of_rev11.sh" >/dev/null 2>&1); echo "8 同内容拷到表外 long-run-check=$?"
fresh; mv "$work/t/$S" "$work/t/$E/renamed_rev11.sh"; judge "9 脚本改名（表里路径对不上）"
fresh; python3 -I - "$work/t/.claude/long-run-live-hosts-exempt" <<'PY'
import re, sys
p = sys.argv[1]; t = open(p, encoding="utf-8").read()
open(p, "w", encoding="utf-8").write(re.sub(r"(run_segment_one_revision_11_multi_host\.sh[ \t]{2,})[0-9a-f]{64}", r"\g<1>" + "0" * 64, t))
PY
judge "10 表里 sha256 写成全 0"
fresh; python3 -I - "$work/t/.claude/long-run-live-hosts-exempt" <<'PY'
import re, sys
p = sys.argv[1]; t = open(p, encoding="utf-8").read()
open(p, "w", encoding="utf-8").write(re.sub(r"(run_segment_one_revision_11_multi_host\.sh[ \t]{2,})([0-9a-f]{64})", lambda m: m.group(1) + m.group(2).upper(), t))
PY
judge "11 表里 sha256 写成大写十六进制"
fresh; ln -s run_segment_one_revision_11_multi_host.sh "$work/t/$E/link_to_rev11.sh"; (cd "$work/t" && bash scripts/hosts/detached-run.sh long-run-check -- bash "$E/link_to_rev11.sh" >/dev/null 2>&1); echo "12 符号链接指到表内脚本 long-run-check=$?"
fresh; (cd "$work/t" && python3 -B .claude/hooks/lib_long_run.py exempt-check "$work/t" --base-table "$work/t/.claude/long-run-live-hosts-exempt" >/dev/null 2>&1); echo "13 基准表就是现表（含 sha256 的新写法）exempt-check=$?"
