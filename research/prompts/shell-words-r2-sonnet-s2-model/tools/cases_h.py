import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
# ---------- 文件头 lib_shell_words.py 写明的切词写法：正当的放行，藏着重型命令的照拒 ----------
legit = [   # 命令，引的判据，身份都放行
 ('Y01', 'echo "$(echo ok)" && ls', ['redir']),
 ('Y02', 'echo "a $(echo "b") c"; cargo fmt --check', ['redir']),
 ('Y03', 'diff <(ls) <(ls -a) | head -3', ['redir']),
 ('Y04', 'x=$(bash <<\'EOF\'\nls\nEOF\n); echo "$x"', ['redir']),
 ('Y05', 'ls 2> {D}/err.txt', ['redir']),
 ('Y06', 'ls 2>&1 | head; ls 1>&2 2>/dev/null', ['redir']),
 ('Y07', 'ln -s heavy-all.sh {D}/y07-link.sh', ['redir']),
 ('Y08', 'ls {a,}', ['brace_w']),
 ('Y09', 'echo {1..3} {01..03}', ['brace_w']),
 ('Y10', 'echo ${HOME} "${USER:-x}"', ['brace_w']),
 ('Y11', "bash -s -- cargo test --all <<'EOF'\nls\nEOF", ['bash_c']),
 ('Y12', 'bash -c -x "ls -la"', ['bash_c']),
 ('Y13', 'env -i -u HOME PATH=/usr/bin ls', ['env_opts']),
 ('Y14', 'env -- a=1 ls', ['env_opts']),
 ('Y15', 'function f { cargo test --all; }', ['func_def']),
 ('Y16', 'f() { echo hi; }; f', ['func_def']),
 ('Y17', 'for f in a b; do echo $f; done', ['cmdpos']),
 ('Y18', 'if [ -f README.md ]; then echo y; else echo n; fi', ['cmdpos']),
 ('Y19', 'case "$1" in a) echo a;; *) echo other;; esac', ['brace_w']),
 ('Y20', 'cat <<\'EOF\' > {D}/note.md\ncargo test --all\nbash scripts/gate/gate-staged.sh\nEOF', ['heredoc_strip','written']),
 ('Y21', 'cat <<\'EOF\' | grep -c cargo\ncargo test --all\nEOF', ['heredoc_strip']),
 ('Y22', 'python3 - <<\'EOF\'\nprint("cargo test --all")\nEOF', ['heredoc_strip']),
 ('Y23', 'echo "cargo test --all" > {D}/y23.txt', ['cmdpos']),
 ('Y24', "git commit -m 'note: cargo test --all is heavy'", ['cmdpos']),
 ('Y25', "sort -u {D}/list | uniq -c | head -3", ['cmdpos']),
 ('Y26', "[[ -d scripts ]] && echo ok", ['brace_w']),
 ('Y27', "echo $((1+2))", ['redir']),
 ('Y28', "ls -la \\\n  scripts | head -3", ['redir']),
]
for n, cmd, cites in legit:
    for ident in (M, IW, TF, GP):
        C(f'{n}-{ident[:3]}', ident, cmd, 0, 0, cites)
# 藏在这些写法里的重型命令，子 agent 照拒；主 agent 不带前缀照拒
sneaky = [
 ('Y40', 'echo "$(cargo test --all)"', ['redir','sub_refuse']),
 ('Y41', 'x=$(bash <<\'EOF\'\ncargo test --all\nEOF\n)', ['redir','stdin_shell']),
 ('Y42', 'diff <(bash fx/light.sh) <(bash fx/heavy-all.sh)', ['redir','script_read']),
 ('Y43', 'cargo test --all 2> {D}/err.txt', ['redir','full']),
 ('Y44', 'cargo 2>{D}/err.txt test --all', ['redir','full']),
 ('Y45', '2>{D}/err.txt cargo test --all', ['redir','full']),
 ('Y46', 'env =1 cargo test --all', ['env_opts','full']),
 ('Y47', 'env -i cargo test --all', ['env_opts','full']),
 ('Y48', 'bash -c -- \'cargo test --all\'', ['bash_c','full']),
 ('Y49', 'bash -c -x \'cargo test --all\'', ['bash_c','full']),
 ('Y50', 'function f { cargo test --all; }; f', ['func_def','full']),
 ('Y51', 'ln -s heavy-all.sh fx/y51-link.sh; bash fx/y51-link.sh', ['redir','script_read']),
 ('Y52', 'cp fx/heavy-all.sh {D}/y52.sh && bash {D}/y52.sh', ['written','script_read']),
 ('Y53', '{cargo,} test --all', ['brace_w','full']),
 ('Y55', 'cargo test --{all,}', ['brace_w','full']),
 ('Y56', 'cargo test --workspace{,}', ['brace_w','full']),
 ('Y57', 'echo "x" && cargo test --all', ['cmdpos','full']),
 ('Y58', 'cargo test --all || true', ['cmdpos','full']),
 ('Y59', '(cd crates && cargo test --workspace)', ['cmdpos','full']),
 ('Y60', 'if true; then cargo test --all; fi', ['cmdpos','full']),
 ('Y61', 'echo $(cargo test --workspace) # x', ['cmdpos','full']),
 ('Y62', 'builtin command cargo test --all', ['builtin','full']),
 ('Y63', 'command cargo test --all', ['cmd_v','full']),
 ('Y64', 'exec cargo test --all', ['cmdpos','full']),
 ('Y65', 'sudo cargo test --all', ['cmdpos','full']),
 ('Y66', 'stdbuf -oL cargo test --all', ['cmdpos','full']),
 ('Y67', 'ionice -c3 cargo test --all', ['cmdpos','full']),
 ('Y68', 'taskset -c 0 cargo test --all', ['cmdpos','full']),
 ('Y69', 'eval "cargo test --all"', ['cmdpos','full']),
 ('Y70', "bash -c 'echo ok; cargo test --workspace'", ['bash_c','full']),
 ('Y71', "bash -lc 'cargo test --all'", ['bash_c','full']),
 ('Y72', "sh -c 'cargo test --all'", ['bash_c','full']),
]
for n, cmd, cites in sneaky:
    C(f'{n}-IW', IW, cmd, 2, 0, cites)
    C(f'{n}-M', M, cases_a.RB + cmd if cmd.startswith(('cargo','{cargo','ca{')) else cmd, 2, 0, cites + ['prefix_need'])

C('Y54-IW', IW, 'ca{r,r}go test --all', 0, 0, ['brace_w'], note='bash 展开成 cargo cargo test --all，cargo 报没有子命令 cargo，不是测试')
