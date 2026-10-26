from cases_a import C, CASES as _ignore  # noqa
import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
CAP = cases_a.CAP; SC = cases_a.SC
ALL = [IW, ER, TF, CL, INV, TW, GP, TA, MT]
# ---------- 子 agent 的正当命令（应放行）：基础只读命令，每种身份各一遍 ----------
basics = [
 ('B01', 'ls -la; wc -l .claude/hooks/lib_shell_words.py', ['ac_read']),
 ('B02', "grep -nF 'def ' .claude/hooks/lib_shell_words.py | head -5", ['ac_read','cmdpos']),
 ('B03', 'sha256sum .claude/hooks/lib_shell_words.py .claude/hooks/lib_heavy_tests.py', ['ac_read']),
 ('B04', "nice -n 19 python3 scripts/kb/kb-sections.py .claude/kb/decisions.md > {D}/sections.txt 2>&1; echo rc=$?; tail -n 20 {D}/sections.txt", ['ac_rc']),
 ('B05', "cat > {D}/n.sh <<'EOF'\necho hi\nEOF\nbash {D}/n.sh", ['written']),
 ('B06', "python3 - <<'EOF'\nimport json\nprint(json.dumps({'a': 1}))\nEOF", ['heredoc_strip']),
 ('B07', 'git diff --stat; git log --oneline -3; git show HEAD:README.md | head -3', ['cmdpos']),
 ('B08', 'python3 scripts/agents/cite-check.py research/prompts/x-output.md', ['cmdpos']),
 ('B09', "bash scripts/hosts/work-directory-ownership.sh claim {D} --purpose 'shell-words-r2 草稿'", ['cmdpos']),
 ('B10', 'bash {D}/model/tools/rerun.sh --repo {D}/repo --frozen {D}/frozen', ['script_read']),
 ('B11', 'cp -r crates/singlefs-core {D}/core-copy && ls {D}/core-copy', ['b_scope_land']),
 ('B12', 'mkdir -p {D}/tmp-m && cp scripts/hosts/capped.sh {D}/tmp-m/', ['b_scope_land']),
 ('B13', 'rm -rf {D}/tmp-m', ['b_rm_ok']),
 ('B14', "sed -i 's/a/b/' {D}/f.txt", ['b_sedi']),
 ('B15', "python3 - <<'EOF'\nopen('{D}/out.txt','w').write('x')\nEOF", ['b_py_open']),
 ('B16', 'for n in a b c; do echo x > {D}/$n.txt; done', ['b_scope_loop']),
 ('B17', 'command -v claude; which claude; grep -n claude README.md', ['b_claude_ok']),
 ('B18', 'ls scripts/{hosts,crash}/*.sh | head', ['brace']),
 ('B19', "env -C scripts bash -c 'ls | head -3'", ['env_opts','bash_c']),
 ('B20', 'timeout 60 bash -c "ls | head"', ['prefix_wrap']),
]
for n, cmd, cites in basics:
    for k, ident in enumerate(ALL):
        # B14/B15（sed -i / python 写）只给有 Edit 的身份跑也没问题，工作根下一律放行
        C(f'{n}-{ident}', ident, cmd, 0, 0, cites)
