"""家族 G（函数 / 别名 / 间接启动）、N（按名字判的重型脚本）、B（花括号与 cat 源文件不在）。{B} 是桩 cargo 所在目录。"""
from corpus_base import add

MEM = "bash {R}/scripts/hosts/run-with-memory-cap.sh 4G"
BAD = "cargo test --workspace"
OKC = MEM + " cargo test -p singlefs-harness --test t1"

G = [
    ("fn-launcher", "run() { \"$@\"; }; run {x}"), ("fn-launcher-logged", "run_logged() { local l=$1; shift; \"$@\" > \"$l\" 2>&1; }; run_logged {D}/g.log {x}"),
    ("fn-launcher-env", "r() { env \"$@\"; }; r {x}"), ("fn-launcher-nice", "w() { nice \"$@\"; }; w {x}"), ("fn-launcher-mem", "m() { " + MEM + " \"$@\"; }; m cargo test --workspace"),
    ("fn-launcher-mem-ok", "m() { " + MEM + " \"$@\"; }; m cargo test -p singlefs-harness --test t1"),
    ("fn-keyword-launcher", "function run { \"$@\"; }; run {x}"), ("alias", "shopt -s expand_aliases; alias ct='{x}'; ct"),
    ("bashc-args", "bash -c 'cargo \"$@\"' _ test --workspace"), ("bashc-dollar0", "bash -c '$0 test --workspace' cargo"),
    ("bashc-dollar12", "bash -c 'cargo \"$1\" \"$2\"' _ test --workspace"), ("sh-exec-args", "sh -c 'exec \"$@\"' sh cargo test --workspace"),
    ("xargs-0", "printf '%s\\0' cargo test --workspace | xargs -0 echo >/dev/null; printf '%s\\0' cargo test --workspace | xargs -0 -n3 env"),
    ("set-dollar-at", "set -- cargo test --workspace; \"$@\""), ("abs-cargo", "{B}/cargo test --workspace"), ("abs-cargo-ckr", "{B}/cargo test -p singlefs-checker-tier"),
    ("abs-cargo-mem", MEM + " {B}/cargo test --workspace"), ("command-v", "\"$(command -v cargo)\" test --workspace"), ("which", "$(which cargo) test --workspace"),
("rustflags", "RUSTFLAGS=-Awarnings cargo test --workspace"),
    ("target-dir", "CARGO_TARGET_DIR={D}/tg cargo test --workspace"), ("toolchain", "cargo +nightly test --workspace"), ("quiet", "cargo -q test --workspace"),
    ("offline", "cargo --offline test --workspace"), ("alias-t", "cargo t --workspace"), ("all-flag", "cargo test --all"), ("nextest", "cargo nextest run --workspace"),
    ("nextest-ckr", "cargo nextest run -p singlefs-checker-tier"), ("llvm-cov", "cargo llvm-cov --workspace"), ("miri", "cargo miri test --workspace"),
    ("mutants", "cargo mutants -p singlefs-checker-tier"), ("hack", "cargo hack test --workspace"), ("pkg-eq", "cargo test --package=singlefs-checker-tier"),
    ("pkg-glued", "cargo test -psinglefs-checker-tier"), ("pkg-at", "cargo test -p singlefs-checker-tier@0.1.0"), ("pkg-glob", "cargo test -p 'singlefs-checker*'"),
    ("manifest", "cargo test --manifest-path {C}/crates/singlefs-checker-tier/Cargo.toml"), ("cd-crate", "cd {C}/crates/singlefs-checker-tier && cargo test"),
    ("exclude-ws", "cargo test --workspace --exclude singlefs-checker-tier"), ("two-pkgs", "cargo test -p singlefs-harness -p singlefs-checker-tier"),
    ("bare-copy", "cargo test"), ("all-targets", "cargo test --all-targets"), ("lib-only", "cargo test --lib"), ("no-run", "cargo test --workspace --no-run"),
    ("list", "cargo test -p singlefs-checker-tier -- --list"), ("run-bin", "cargo run --bin e1"), ("run-bin-mem", MEM + " cargo run --bin e1"), ("bench", "cargo bench -p singlefs-harness"),
    ("build", "cargo build -p singlefs-harness"), ("clippy", "cargo clippy --workspace"), ("fmt", "cargo fmt --all"), ("check", "cargo check --workspace"),
    ("doc", "cargo doc --no-deps"), ("quoted-word", "cargo 'test' \"--workspace\""), ("split-flag", "cargo test --work\"space\""), ("backslash-cmd", "\\cargo test --workspace"),
    ("quoted-cmd", "'cargo' test --workspace"), ("dq-cmd", "\"cargo\" test --workspace"), ("mid-quote-cmd", "car\"go\" test --workspace"), ("dollar-quote-cmd", "$'cargo' test --workspace"),
    ("brace-flag", "cargo test --{workspace,all}"), ("brace-pkg", "cargo test -p singlefs-{checker-tier,harness}"), ("tilde", "cargo test --workspace ~"), ("glob-arg", "cargo test --workspace {D}/*.nothing"),
    ("var-flag", "F=--workspace; cargo test $F"), ("var-flag-default", "cargo test ${F:---workspace}"), ("var-pkg", "P=singlefs-checker-tier; cargo test -p $P"),
    ("var-pkg-dq", "P=singlefs-checker-tier; cargo test -p \"$P\""), ("var-pkg-cd", "d={C}; cd $d && cargo test -p singlefs-checker-tier"), ("cmdsub-flag", "cargo test --$(echo workspace)"),
    ("cmdsub-sub", "cargo $(echo test) --workspace"), ("cmdsub-all", "cargo $(echo test --workspace)"), ("for-flag", "for f in --workspace; do cargo test $f; done"),
    ("export-then", "export RUST_LOG=x; cargo test --workspace"), ("env-var-prefix-only", "RUST_LOG=x cargo test -p singlefs-harness --test t1"),
]
for name, tmpl in G:
    inner = BAD if "{x}" in tmpl else None
    cmd = tmpl.replace("{x}", BAD)
    add(f"G-{name}", cmd, roles=("sub", "main"), cwd="C", prefix_style="export")

# N：按名字判的重型脚本
NAMED = [".claude/scripts/check.sh", ".claude/scripts/gate.sh", "scripts/gate/gate-staged.sh", ".claude/gate.d/checker-tier-lkmm.sh",
         ".claude/gate.d/checker-tier-qemu-device-streams.sh", ".claude/gate.d/checker-tier-formal-verification.sh", ".claude/gate.d/checker-tier-crates-mutation-replay.sh",
         ".claude/gate.d/checker-tier-crash-point-replay.sh", "scripts/crash/crash-point-replay.sh", "scripts/crash/crash-amplification.sh",
         "scripts/experiment-apparatus/replay.sh", "scripts/mutation/mutate.sh", "scripts/devices/build-patched-qemu.sh"]
for script in NAMED:
    for arg in ("", " --quick"):
        short = script.split("/")[-1].replace(".sh", "")
        for fname, tmpl in (("bash", "bash {R}/{s}{a}"), ("mem", MEM + " bash {R}/{s}{a}"), ("rel", "bash {s}{a}")):
            add(f"N-{short}{arg.replace(' --', '-') or '-bare'}-{fname}", tmpl.replace("{s}", script).replace("{a}", arg), roles=("sub", "main"), cwd="R", prefix_style="export")
for arg in ("", " --version", " -version"):
    add(f"N-herd7{arg.replace(' ', '')}", "herd7" + arg, roles=("sub", "main"), cwd="R", prefix_style="export")
add("N-qemu", "qemu-system-x86_64 -version", roles=("sub", "main"), cwd="R", prefix_style="export")

# B：花括号、cat 源文件不在
BR = [
    ("redir-brace-two", "echo hi > {D}/o.{a,b}"), ("redir-brace-one", "echo hi > {D}/o.{a}"), ("tee-brace", "echo hi | tee {D}/o.{a,b} >/dev/null"),
    ("cp-brace", "echo hi > {D}/c0.txt; cp {D}/c0.txt {D}/c1.{a,b}"), ("redir-brace-in-quote", "echo hi > '{D}/o.{a,b}'"), ("redir-brace-num", "echo hi > {D}/o{1..2}.txt"),
    ("cat-missing-src", "cat {U}/nosuch.sh > {U}/t.sh; bash {U}/t.sh"), ("cat-missing-src-ok", "cat {U}/nosuch.sh > {U}/t.sh; echo done"),
    ("cat-two-one-missing", "echo ':' > {U}/a.sh; cat {U}/a.sh {U}/nosuch.sh > {U}/t.sh; bash {U}/t.sh"), ("cat-then-cargo", "cat {U}/nosuch.sh > {U}/t.sh 2>/dev/null; cargo test -p singlefs-harness --test t1 2>/dev/null; true"),
    ("cat-to-file-then-other", "cat {U}/nosuch.txt > {U}/t.txt 2>/dev/null; bash {U}/other.sh 2>/dev/null; true"),
    ("brace-body-data", "echo '{'; echo '}'"), ("brace-in-dq-then-cmd", "echo \"{a,b}\"; echo ok"), ("fn-then-quoted-braces", "f() { :; }; echo '{ x }'; f"),
    ("fn-def-quoted-brace-call", "f() { echo \"{\"; }; f"), ("fn-def-brace-expansion-call", "f() { echo {a,b}; }; f"), ("unclosed-fn-in-quote", "echo 'f() { cargo test --workspace; }'"),
]
for name, cmd in BR:
    add(f"B-{name}", cmd.replace("{a,b}", "{a,b}"), roles=("sub", "editor", "main"), cwd="R", setup="mkdir -p {U}; echo ':' > {U}/other.sh", prefix_variants=False)
