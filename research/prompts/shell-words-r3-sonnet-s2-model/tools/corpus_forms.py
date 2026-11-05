"""家族 F：语法形态 × 内层命令。{x} 是内层命令。BAD 是子 agent 不许跑的（全量测试），OK 是子 agent 经内存包装跑的自己那一份。"""
from corpus_base import add

MEM = "bash {R}/scripts/hosts/run-with-memory-cap.sh 4G"
BAD = "cargo test --workspace"
OK = MEM + " cargo test -p singlefs-harness --test t1"

FORMS = [
    ("seq-before", "echo a; {x}"), ("seq-after", "{x}; echo a"), ("and", "true && {x}"), ("or", "false || {x}"),
    ("pipe-tee", "{x} | tee {D}/o.txt"), ("pipe-into", "echo a | {x}"), ("and-echo", "{x} && echo ok"), ("bang", "! {x}"),
    ("amp-wait", "{x} & wait"), ("or-true", "{x} || true"), ("redir-2-1-tail", "{x} 2>&1 | tail -n1"),
    ("redir-gt", "{x} > {D}/o.txt 2>&1"), ("redir-append", "{x} >> {D}/o.txt"), ("redir-amp-gt", "{x} &> {D}/o.txt"),
    ("redir-gt-amp", "{x} >& {D}/o.txt"), ("redir-devnull", "{x} 2>/dev/null"), ("redir-in", "{x} < /dev/null"),
    ("redir-first", "> {D}/o.txt {x}"), ("redir-fd3", "{x} 3>{D}/o.txt"), ("redir-pipe-err", "{x} |& tail -n1"),
    ("cmdsub", "$({x})"), ("cmdsub-echo", "echo $({x})"), ("backtick", "echo `{x}`"), ("cmdsub-dq", "echo \"$({x})\""),
    ("procsub", "cat <({x})"), ("procsub-out", "{x} > >(cat)"), ("procsub-tee", "{x} 2> >(tee {D}/e.txt >&2)"),
    ("subshell", "( {x} )"), ("brace-group", "{ {x}; }"), ("subshell-cd", "(cd {C}; {x})"), ("cd-and", "cd {C} && {x}"),
    ("fn-called", "f() { {x}; }; f"), ("fn-nested", "f() { {x}; }; g() { f; }; g"), ("fn-keyword", "function f { {x}; }; f"),
    ("fn-never-called", "f() { {x}; }; echo done"), ("fn-oneline", "f(){ {x};};f"), ("fn-call-args", "f() { {x} \"$@\"; }; f"),
    ("fn-then-brace-data", "f() { echo '{'; }; f; {x}"), ("brace-in-quote-then-call", "echo '{ not a body }'; {x}"),
    ("fn-def-and-brace-quote", "f() { :; }; echo \"{a,b}\"; {x}"),
    ("if-true", "if true; then {x}; fi"), ("if-false", "if false; then {x}; fi"), ("if-else", "if false; then :; else {x}; fi"),
    ("and-false", "false && {x}"), ("for-run", "for i in 1 2; do {x}; done"), ("for-empty", "for i in; do {x}; done"),
    ("while-false", "while false; do {x}; done"), ("until-true", "until true; do {x}; done"),
    ("case-hit", "case a in a) {x};; esac"), ("case-miss", "case a in b) {x};; esac"), ("dbl-bracket", "[[ a == a ]] && {x}"),
    ("while-read", "while read l; do {x}; done <<< foo"), ("test-and", "[ -d {C} ] && {x}"), ("time-kw", "time {x}"),
    ("group-redir", "{ {x}; } 2>&1"), ("comment-tail", "{x} # --all"), ("comment-only", "echo ok # {x}"),
    ("echo-data", "echo \"{x}\""), ("echo-sq", "echo '{x}'"), ("grep-data", "grep -n \"{x}\" /dev/null"),
    ("heredoc-cat-data", "cat <<'EOF'\n{x}\nEOF"), ("heredoc-bash", "bash <<EOF\n{x}\nEOF"), ("heredoc-bash-q", "bash <<'EOF'\n{x}\nEOF"),
    ("heredoc-bash-s", "bash -s <<'EOF'\n{x}\nEOF"), ("heredoc-sh", "sh <<'EOF'\n{x}\nEOF"), ("herestring", "bash <<< '{x}'"),
    ("printf-pipe-bash", "printf '%s\\n' '{x}' | bash"), ("echo-pipe-bash", "echo '{x}' | bash"), ("echo-pipe-sh-s", "echo '{x}' | sh -s"),
    ("echo-pipe-bash-dash", "echo '{x}' | bash -"), ("cat-heredoc-pipe-bash", "cat <<'EOF' | bash\n{x}\nEOF"),
    ("eval-sq", "eval '{x}'"), ("eval-dq", "eval \"{x}\""), ("eval-var", "c='{x}'; eval \"$c\""), ("bash-c-dollar", "bash -c \"$(printf '%s' '{x}')\""),
    ("bash-procsub", "bash <(echo '{x}')"), ("source-procsub", "source <(echo '{x}')"),
    ("var-cmd", "c='{x}'; $c"), ("var-cmd-dq", "c='{x}'; \"$c\""), ("array-cmd", "a=({x}); \"${a[@]}\""),
    ("lineno", "echo a\n{x}"), ("continuation", "{x} \\\n  --quiet"), ("tab", "{x}\t"), ("semi-end", "{x};"),
    ("env-prefix-var", "FOO=1 {x}"), ("env-two", "FOO=1 BAR=2 {x}"), ("export-then", "export FOO=1; {x}"), ("unset-then", "unset FOO; {x}"),
    ("trap", "trap 'echo x' EXIT; {x}"), ("set-e", "set -e; {x}"), ("set-x", "set -x; {x}"), ("ulimit", "ulimit -v 8000000; {x}"),
    ("cd-fail", "cd /nonexistent 2>/dev/null; {x}"), ("pushd", "pushd {C} >/dev/null; {x}"),
]
for fname, tmpl in FORMS:
    for tag, inner in (("bad", BAD), ("ok", OK)):
        add(f"F-{fname}-{tag}", tmpl.replace("{x}", inner), roles=("sub", "main"), cwd="C", prefix_style="export")
