#!/usr/bin/env bash
# 关键反例：用真的钩子入口（bash <钩子> 读标准输入的 JSON）逐条喂改前、改后两棵树，原样打出退出码与第一条 ✗ 行（截 150 字）
trunc() { python3 -I -c 'import sys; print(sys.stdin.read().rstrip("\n")[:int(sys.argv[1])])' "$1"; }   # 按字符截（GNU cut -c 其实按字节，会把汉字截在中间）
R="$SW_ROOT"; HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export SINGLEFS_WORK_ROOT=/var/empty/swr
one() {  # one <钩子> <agent> <cwd:S|T> <命令>
  local hook="$1" agent="$2" cwdk="$3" cmd="$4" t out
  echo "命令（${hook}/${agent}）：$(printf '%s' "$cmd" | sed ':a;N;$!ba;s/\n/\\n/g')"
  for t in tree-orig tree; do
    if [[ $cwdk == S ]]; then export CWD=$R/scratch; else export CWD=$R/$t; fi
    out=$(TREE=$t "$HERE/feed.sh" "$hook" "$agent" "$cmd" 2>&1 | sed "s#$R#<work>#g")   # 草稿根的路径先换成占位再截，输出才不随草稿目录的长短变
    echo "  $([[ $t == tree-orig ]] && echo 改前 || echo 改后)：$(printf '%s\n' "$out" | head -1) $(printf '%s\n' "$out" | grep -m1 '✗' | trunc 120)"
  done
}
one guard main S '{cargo,} test --all'
one guard main S '{,} cargo test --all'
one guard main S 'cargo {,test} --all'
one guard implementation-writer S '{bash,} scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier'
one guard main S $'FOO="$(pwd)" bash <<\'EOF\'\ncargo test --all\nEOF'
one guard main S $'cd "$(git rev-parse --show-toplevel)" && bash <<\'EOF\'\ncargo test --all\nEOF'
one det main S $'cd "$(git rev-parse --show-toplevel)" && bash <<\'EOF\'\nuntil grep -q x f; do sleep 10; done\nEOF'
one guard main S $'x=$(bash <<\'EOF\'\ncargo test --all\nEOF\n)'
one guard main S 'cat s.sh > d1.sh 2>/dev/null && bash d1.sh'
one guard main S 'cat s.sh > d2.sh && bash d2.sh'
one det clerk T 'cat scripts/hosts/detached-run.sh > /tmp/shw-dr-b.sh 2>/dev/null && bash /tmp/shw-dr-b.sh start --label m -- sleep 1'
one det clerk T 'cat scripts/hosts/detached-run.sh > /tmp/shw-dr-a.sh && bash /tmp/shw-dr-a.sh start --label m -- sleep 1'
one guard main S 'env =1 cargo test --all'
one guard main S "bash -c -- 'cargo test --all'"
one guard main S 'ln -s ../s.sh sub/l.sh && bash sub/l.sh'
one guard main S 'ln -s s.sh sub/l2.sh && bash sub/l2.sh'
one guard main S 'bash -s -- s.sh'
# ---- 第二轮（shell-words-r2）追加：修补之后判错的反例，以及修补前后都错的几条 ----
one guard main S $'function foo { cargo test --all; }; "fo"o'
one guard main S $'function foo { cargo test --all; }; fo{o,}'
one guard main S $'function foo { cargo test --all; }; foo'
one guard main S 'cat s.sh 01> d.sh && bash d.sh'
one guard main S 'cat s.sh "2"> d.sh ; bash d.sh'
one guard main S 'cat s.sh 3>d.sh >&3; bash d.sh'
one guard main S "bash -c --posix 'cargo test --all'"
one guard main S "bash -c -n 'cargo test --all'"
one guard main S 'env =1 -u X cargo test --all'
one guard main S 'env A=1 -u X cargo test --all'
one guard main S $'cat <<\'EOS\' 2>d.sh\ncargo test --all\nEOS\nbash d.sh'
one guard main S 'cp s.sh s{01..100..50}.sh && bash s001.sh'
one guard main S $'echo $\'a\\\'b\'; x=$(bash <<\'EOF\'\ncargo test --all\nEOF\n)'
one guard main S 'cat s.sh >> d.sh && bash d.sh'
one guard main S 'cat s.sh &> d.sh && bash d.sh'
