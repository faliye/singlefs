"""家族 S：脚本文件的写出与执行。现场（setup）在 {U} 里放 s.sh（里面是全量测试）、ok.sh（经内存包装的自己那一份）。"""
from corpus_base import add

SETUP = ("printf '#!/usr/bin/env bash\\ncd {C} && cargo test --workspace\\n' > {U}/s.sh; "
         "printf '#!/usr/bin/env bash\\ncd {C} && bash {R}/scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test t1\\n' > {U}/ok.sh; "
         "cp {U}/s.sh {U}/s2.sh; cp {U}/ok.sh {U}/ok2.sh; chmod +x {U}/*.sh")
MEM = "bash {R}/scripts/hosts/run-with-memory-cap.sh 4G"
BADL = "cd {C} && cargo test --workspace"
OKL = "cd {C} && " + MEM + " cargo test -p singlefs-harness --test t1"

EXEC = [
    ("bash", "bash {U}/{n}"), ("sh", "sh {U}/{n}"), ("source", "source {U}/{n}"), ("dot", ". {U}/{n}"), ("direct", "{U}/{n}"),
    ("cd-direct", "cd {U} && ./{n}"), ("cd-bash", "cd {U} && bash {n}"), ("exec", "exec {U}/{n}"), ("mem-bash", MEM + " bash {U}/{n}"),
    ("mem-direct", MEM + " {U}/{n}"), ("nice-bash", "nice bash {U}/{n}"), ("env-bash", "env FOO=1 bash {U}/{n}"),
    ("timeout-bash", "timeout 100 bash {U}/{n}"), ("bash-args", "bash {U}/{n} a b"), ("bash-x", "bash -x {U}/{n}"),
    ("bash-dd", "bash -- {U}/{n}"), ("bash-s-redir", "bash -s < {U}/{n}"), ("bash-redir", "bash < {U}/{n}"),
    ("cat-pipe-bash", "cat {U}/{n} | bash"), ("sh-s-redir", "sh -s < {U}/{n}"), ("bash-c-source", "bash -c '. {U}/{n}'"),
    ("bash-c-dollar-cat", "bash -c \"$(cat {U}/{n})\""), ("eval-cat", "eval \"$(cat {U}/{n})\""), ("source-procsub", "source <(cat {U}/{n})"),
    ("var-path", "S={U}/{n}; bash $S"), ("var-path-dq", "S={U}/{n}; bash \"$S\""), ("var-dir", "d={U}; bash $d/{n}"),
    ("glob-q", "bash {U}/*{n}"), ("brace-path", "bash {U}/{{n},zz}"), ("subshell-path", "bash $(echo {U}/{n})"),
    ("path-lookup", "PATH={U}:$PATH {n}"), ("relative-up", "cd {U}/.. && bash u/$(basename {U})/{n}"),
    ("for-script", "for f in {U}/{n}; do bash $f; done"), ("xargs-bash", "echo {U}/{n} | xargs bash"), ("find-exec-bash", "find {U} -name '{n}' -exec bash {} \\;"),
    ("bash-env-file", "BASH_ENV={U}/{n} bash -c true"), ("mem-bash-c-script", MEM + " bash -c 'bash {U}/{n}'"),
    ("two-scripts", "bash {U}/{n}; bash {U}/{n}"), ("script-via-ln", "ln -s {U}/{n} {U}/lnk.sh; bash {U}/lnk.sh"),
    ("script-via-cp", "cp {U}/{n} {U}/cp.sh; bash {U}/cp.sh"), ("script-via-cp-dir", "mkdir -p {U}/d; cp {U}/{n} {U}/d/; bash {U}/d/{n}"),
    ("script-via-mv", "cp {U}/{n} {U}/m0.sh; mv {U}/m0.sh {U}/m1.sh; bash {U}/m1.sh"),
    ("script-via-install", "install -m755 {U}/{n} {U}/ins.sh; bash {U}/ins.sh"),
    ("script-via-cat", "cat {U}/{n} > {U}/cat.sh; bash {U}/cat.sh"), ("script-via-cat-app", "cat {U}/{n} >> {U}/cat.sh; bash {U}/cat.sh"),
    ("script-via-cat-amp", "cat {U}/{n} &> {U}/cat.sh; bash {U}/cat.sh"), ("script-via-cat-gtamp", "cat {U}/{n} >& {U}/cat.sh; bash {U}/cat.sh"),
    ("script-via-tee-pipe", "cat {U}/{n} | tee {U}/t.sh >/dev/null; bash {U}/t.sh"), ("script-via-tee-stdin", "tee {U}/t.sh < {U}/{n} >/dev/null; bash {U}/t.sh"),
    ("script-via-tee-a", "cat {U}/{n} | tee -a {U}/t.sh >/dev/null; bash {U}/t.sh"), ("script-via-dd", "dd if={U}/{n} of={U}/dd.sh 2>/dev/null; bash {U}/dd.sh"),
    ("script-via-head", "head -n 9 {U}/{n} > {U}/h.sh; bash {U}/h.sh"), ("script-via-sed", "sed 's/x/x/' {U}/{n} > {U}/se.sh; bash {U}/se.sh"),
    ("script-via-two-cat", "cat {U}/{n} {U}/{n} > {U}/c2.sh; bash {U}/c2.sh"), ("script-via-cp-then-cat-app", "cp {U}/{n} {U}/a.sh; cat {U}/{n} >> {U}/a.sh; bash {U}/a.sh"),
]
for name, tmpl in EXEC:
    for tag, n in (("bad", "s.sh"), ("ok", "ok.sh")):
        add(f"S-{name}-{tag}", tmpl.replace("{n}", n), roles=("sub", "main"), cwd="R", setup=SETUP, prefix_style="export")

HERE_WRITES = [
    ("cat-gt", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"), ("cat-heredoc-first", "cat <<'EOF' > {U}/w.sh\n{l}\nEOF\nbash {U}/w.sh"),
    ("cat-gt-nospace", "cat >{U}/w.sh<<'EOF'\n{l}\nEOF\nbash {U}/w.sh"), ("cat-unquoted", "cat > {U}/w.sh <<EOF\n{l}\nEOF\nbash {U}/w.sh"),
    ("cat-app", "cat >> {U}/w.sh <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"), ("cat-amp", "cat &> {U}/w.sh <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"),
    ("cat-gtamp", "cat >& {U}/w.sh <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"), ("cat-1gt", "cat 1> {U}/w.sh <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"),
    ("cat-clobber", "cat >| {U}/w.sh <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"), ("cat-fd3", "cat 3>{U}/w.sh >&3 <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"),
    ("tee-heredoc", "tee {U}/w.sh >/dev/null <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"), ("tee-a", "tee -a {U}/w.sh >/dev/null <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"),
    ("tee-first", "tee {U}/w.sh <<'EOF' >/dev/null\n{l}\nEOF\nbash {U}/w.sh"), ("cat-pipe-tee", "cat <<'EOF' | tee {U}/w.sh >/dev/null\n{l}\nEOF\nbash {U}/w.sh"),
    ("cat-pipe-tee-a", "cat <<'EOF' | tee -a {U}/w.sh >/dev/null\n{l}\nEOF\nbash {U}/w.sh"), ("echo-gt", "echo '{l}' > {U}/w.sh; bash {U}/w.sh"),
    ("printf-gt", "printf '%s\\n' '{l}' > {U}/w.sh; bash {U}/w.sh"), ("echo-app", "echo '{l}' >> {U}/w.sh; bash {U}/w.sh"),
    ("echo-e", "echo -e '{l}' > {U}/w.sh; bash {U}/w.sh"), ("dd-heredoc", "dd of={U}/w.sh 2>/dev/null <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"),
    ("install-stdin", "install -m755 /dev/stdin {U}/w.sh <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"),
    ("cat-then-source", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\nsource {U}/w.sh"), ("cat-then-direct", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\nchmod +x {U}/w.sh; {U}/w.sh"),
    ("cat-then-mem", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\n" + MEM + " bash {U}/w.sh"),
    ("cat-then-cp", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\ncp {U}/w.sh {U}/w2.sh; bash {U}/w2.sh"),
    ("cat-then-cp-app", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\ncat {U}/w.sh >> {U}/w2.sh; bash {U}/w2.sh"),
    ("cat-then-cat-amp", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\ncat {U}/w.sh &> {U}/w2.sh; bash {U}/w2.sh"),
    ("cat-then-mv", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\nmv {U}/w.sh {U}/w2.sh; bash {U}/w2.sh"),
    ("cat-then-ln", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\nln -s {U}/w.sh {U}/w2.sh; bash {U}/w2.sh"),
    ("cat-then-xargs", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\necho {U}/w.sh | xargs bash"),
    ("cat-then-bash-s", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\nbash -s < {U}/w.sh"),
    ("cat-then-pipe-bash", "cat > {U}/w.sh <<'EOF'\n{l}\nEOF\ncat {U}/w.sh | bash"),
    ("python-write", "python3 -c \"open('{U}/w.sh','w').write('{l}')\"; bash {U}/w.sh"),
    ("python-heredoc-write", "python3 - <<'PY'\nopen('{U}/w.sh','w').write('{l}')\nPY\nbash {U}/w.sh"),
    ("function-write", "w() { cat > {U}/w.sh <<'EOF'\n{l}\nEOF\n}; w; bash {U}/w.sh"),
    ("var-target", "t={U}/w.sh; cat > $t <<'EOF'\n{l}\nEOF\nbash $t"),
    ("subshell-write", "(cat > {U}/w.sh <<'EOF'\n{l}\nEOF\n); bash {U}/w.sh"),
    ("two-heredocs", "cat > {U}/a.sh <<'EOF'\n:\nEOF\ncat > {U}/w.sh <<'EOF'\n{l}\nEOF\nbash {U}/w.sh"),
]
for name, tmpl in HERE_WRITES:
    for tag, line in (("bad", BADL), ("ok", OKL)):
        add(f"SW-{name}-{tag}", tmpl.replace("{l}", line), roles=("sub", "main"), cwd="R", setup=SETUP, prefix_style="export")
