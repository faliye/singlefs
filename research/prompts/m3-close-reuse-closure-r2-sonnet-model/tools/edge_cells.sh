#!/usr/bin/env bash
# W2 自证格的原型跑法：在已打上「声明式排边」补丁的树上（make_trees.py 的 edge-decl）量五格。用法：edge_cells.sh <树根> <工作目录>
# 每一格在 <工作目录> 里拷一份树再改，不动 <树根>。entry 是 verus-proof-check 的 check_proof。
set -u
tree="$1"; work="$2"
entry=scripts/gate/verus-proof-check.py::check_proof
mkdir -p "$work"
# 键 = 闭包单元（种类、限定名、文件、指纹，行号不算）加拒算行排序后的摘要；键本身要外部程序的 sha，这里不带，与 closure_key 的单元部分同一批输入
key_of() { (cd "$1" && nice -n 19 python3 scripts/gate/lib_function_units.py closure . "$2" 2>&1 | awk -F'\t' '$1=="refused"{print "refused\t"$2"\t"$4; next} $1!="key" && NF>=4 {split($3,a,":"); print $1"\t"$2"\t"a[1]"\t"$4}' | sort | sha256sum | cut -c1-16; (cd "$1" && nice -n 19 python3 scripts/gate/lib_function_units.py closure . "$2" 2>&1 | awk -F'\t' '$1=="refused"{print $2}' | sort | uniq -c | tr '\n' ' ')); }
copy() { rm -rf "$work/$1"; nice -n 19 rsync -a "$tree/" "$work/$1/"; }
base="$(key_of "$tree" "$entry")"
echo "CELL 基线键：$base"
copy callee; python3 - "$work/callee/scripts/hosts/host-resources.py" <<'PY'
import os, sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = "HOST_TIMEOUT_SECONDS = 12.0"
assert text.count(old) == 1
open(path + ".new", "w", encoding="utf-8").write(text.replace(old, "HOST_TIMEOUT_SECONDS = 13.0"))
os.replace(path + ".new", path)
PY
echo "CELL 改被调方 host-resources.py 的一个常量：键$([[ "$(key_of "$work/callee" "$entry")" == "$base" ]] && echo 不变 || echo 变了)"
copy caller; python3 - "$work/caller/scripts/hosts/run-with-memory-cap.sh" <<'PY'
import os, sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = "HOST_LOOKUP_SECONDS = 15.0"
assert text.count(old) == 1
open(path + ".new", "w", encoding="utf-8").write(text.replace(old, "HOST_LOOKUP_SECONDS = 16.0"))
os.replace(path + ".new", path)
PY
echo "CELL 改调用方包装里的一个常量：键$([[ "$(key_of "$work/caller" "$entry")" != "$base" ]] && echo 变了 || echo 不变)"
copy decl-edited; python3 - "$work/decl-edited/scripts/hosts/run-with-memory-cap.sh" <<'PY'
import os, sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = "只打提示，出错只等于没有提示（试验声明）"
assert text.count(old) == 1
open(path + ".new", "w", encoding="utf-8").write(text.replace(old, "改过的理由"))
os.replace(path + ".new", path)
PY
echo "CELL 改声明的理由文字：键$([[ "$(key_of "$work/decl-edited" "$entry")" != "$base" ]] && echo 变了 || echo 不变)"
copy decl-stale; python3 - "$work/decl-stale/scripts/hosts/run-with-memory-cap.sh" <<'PY'
import os, sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = "closure-excluded-edge: HostLookup.query -> scripts/hosts/host-resources.py"
assert text.count(old) == 1
open(path + ".new", "w", encoding="utf-8").write(text.replace(old, "closure-excluded-edge: HostLookup.query -> scripts/hosts/host-nothing.py"))
os.replace(path + ".new", path)
PY
echo "CELL 声明指到不存在的边：$(key_of "$work/decl-stale" "$entry")"
copy other-edge; cat > "$work/other-edge/scripts/hosts/also-calls-resources.sh" <<'SH'
#!/usr/bin/env bash
# 另一条边：这个入口自己直接起 host-resources.py
run_probe() { python3 scripts/hosts/host-resources.py --json -- true; }
run_probe
bash scripts/hosts/run-with-memory-cap.sh 64M true
SH
(cd "$work/other-edge" && nice -n 19 python3 scripts/gate/lib_function_units.py closure . scripts/hosts/also-calls-resources.sh > "$work/other-edge.closure" 2>&1; echo "CELL 另一条边（入口自己直接起 host-resources.py，同时经包装）：闭包里 host-resources.py 的单元 $(awk -F'\t' '$3 ~ /^scripts\/hosts\/host-resources.py/' "$work/other-edge.closure" | wc -l) 个")
