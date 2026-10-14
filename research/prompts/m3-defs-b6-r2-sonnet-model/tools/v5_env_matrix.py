#!/usr/bin/env python3
"""X3：env 前缀拆法矩阵。implementation-writer 在 <仓>/crates/singlefs-harness 里起的命令，喂 heavy-test-guard.sh 入口（Bash 工具的 hook JSON），
每例打 期望（D 应拒 / A 应放）与 after / before / pre0 的退出码。期望是按「命令里 cargo 真正落在哪个目录、点名哪个包」手判的：落到 checker 档包的测试 = D，别的 = A。
用法：v5_env_matrix.py --root after=<仓> --root before=<仓> [--root pre0=<仓>]"""
import argparse, json, os, subprocess

ap = argparse.ArgumentParser()
ap.add_argument("--root", action="append", required=True)
args = ap.parse_args()
ROOTS = [tuple(item.split("=", 1)) for item in args.root]
W = "bash ../../scripts/hosts/run-with-memory-cap.sh 8G"
CK = "../singlefs-checker-tier"
CASES = [
    ("D", "env cargo test -p singlefs-checker-tier", f"{W} env cargo test -p singlefs-checker-tier"),
    ("D", "env -- cargo test -p checker", f"{W} env -- cargo test -p singlefs-checker-tier"),
    ("D", "env -i cargo test -p checker", f"{W} env -i cargo test -p singlefs-checker-tier"),
    ("D", "env -i -C <ck> cargo test", f"{W} env -i -C {CK} cargo test"),
    ("D", "env -iC <ck> cargo test", f"{W} env -iC {CK} cargo test"),
    ("D", "env -iC<ck> 贴写", f"{W} env -iC{CK} cargo test"),
    ("D", "env -u RUSTFLAGS cargo test -p checker", f"{W} env -u RUSTFLAGS cargo test -p singlefs-checker-tier"),
    ("D", "env -uRUSTFLAGS -C <ck>", f"{W} env -uRUSTFLAGS -C {CK} cargo test"),
    ("D", "env -u RUSTFLAGS -C <ck>", f"{W} env -u RUSTFLAGS -C {CK} cargo test"),
    ("D", "env -S 'cargo test -p checker'", f"{W} env -S 'cargo test -p singlefs-checker-tier'"),
    ("D", "env --split-string='cargo test -p checker'", f"{W} env --split-string='cargo test -p singlefs-checker-tier'"),
    ("D", "env --split-string 'cargo test -p checker' 分词", f"{W} env --split-string 'cargo test -p singlefs-checker-tier'"),
    ("D", "env -S 'env -C <ck> cargo test' 里再嵌 env -C", f"{W} env -S 'env -C {CK} cargo test'"),
    ("D", "env -S '-C <ck> cargo test' 选项在串里", f"{W} env -S '-C {CK} cargo test'"),
    ("D", "env -S 'FOO=1 cargo test -p checker' 赋值在串里", f"{W} env -S 'FOO=1 cargo test -p singlefs-checker-tier'"),
    ("D", "env --chd <ck> cargo test 缩写", f"{W} env --chd {CK} cargo test"),
    ("D", "env --chdir <ck> cargo test", f"{W} env --chdir {CK} cargo test"),
    ("D", "env --chdir=<ck> cargo test", f"{W} env --chdir={CK} cargo test"),
    ("D", "env - cargo test -p checker 单独的 -", f"{W} env - cargo test -p singlefs-checker-tier"),
    ("D", "env -0 cargo test -p checker", f"{W} env -0 cargo test -p singlefs-checker-tier"),
    ("D", "env -S\"cargo test -p checker\" 贴写", f"{W} env -S\"cargo test -p singlefs-checker-tier\""),
    ("D", "env -S 'sh -c \"cargo test -p checker\"'", f"{W} env -S 'sh -c \"cargo test -p singlefs-checker-tier\"'"),
    ("D", "env -- env -C <ck> cargo test", f"{W} env -- env -C {CK} cargo test"),
    ("D", "env -iS 'cargo test -p checker'", f"{W} env -iS 'cargo test -p singlefs-checker-tier'"),
    ("D", "env --unset=X --chdir=<ck> cargo test", f"{W} env --unset=X --chdir={CK} cargo test"),
    ("D", "env --ignore-environment cargo test -p checker", f"{W} env --ignore-environment cargo test -p singlefs-checker-tier"),
    ("D", "env --ignore cargo test -p checker 缩写", f"{W} env --ignore cargo test -p singlefs-checker-tier"),
    ("D", "env -C <ck> -S 'cargo test'", f"{W} env -C {CK} -S 'cargo test'"),
    ("A", "env -S 'cargo test' -C <ck>（-S 串在前：拆开后是 env cargo test -C 目录，-C 落在 cargo 的参数里，不是 env 的选项）", f"{W} env -S 'cargo test' -C {CK}"),
    ("D", "env -S '' -C <ck> cargo test（空串）", f"{W} env -S '' -C {CK} cargo test"),
    ("D", "env -S '\\_ cargo test' -C <ck> 反斜杠转义", f"{W} env -C {CK} -S 'cargo\\_test'"),
    ("A", "env FOO=1 cargo test -p singlefs-harness --lib", f"{W} env FOO=1 cargo test -p singlefs-harness --lib"),
    ("A", "env -i cargo build -p singlefs-harness", f"{W} env -i cargo build -p singlefs-harness"),
    ("A", "env -u X cargo fmt", f"{W} env -u X cargo fmt"),
    ("A", "env -S 'cargo build'", f"{W} env -S 'cargo build'"),
    ("A", "env -S 'cargo test -p singlefs-harness --lib'", f"{W} env -S 'cargo test -p singlefs-harness --lib'"),
    ("A", "env -C ../singlefs-harness cargo test --lib", f"{W} env -C ../singlefs-harness cargo test --lib"),
    ("A", "env -iC ../singlefs-harness cargo test --lib", f"{W} env -iC ../singlefs-harness cargo test --lib"),
    ("A", "env -S 'cargo test' （当前目录 harness）", f"{W} env -S 'cargo test'"),
    ("A", "env -C（缺值，畸形）", f"{W} env -C"),
    ("A", "env -S（缺值，畸形）", f"{W} env -S"),
    ("A", "env -u（缺值）cargo build", f"{W} env -u"),
    ("A", "env -iC ../singlefs-harness cargo clippy", f"{W} env -iC ../singlefs-harness cargo clippy"),
    ("A", "env -S 'ls -la'", f"{W} env -S 'ls -la'"),
    ("A", "env -S 'echo singlefs-checker-tier'（只是字符串）", f"{W} env -S 'echo singlefs-checker-tier'"),
    ("A", "env X='a b' cargo build（赋值带空格）", f"{W} env X='a b' cargo build"),
    ("A", "env -- cargo build", f"{W} env -- cargo build"),
    ("A", "env 查版本 cargo --version", f"env -S 'cargo --version'"),
]
for expect, label, command in CASES:
    row = []
    for name, root in ROOTS:
        cwd = os.path.join(root, "crates/singlefs-harness")
        payload = json.dumps({"tool_name": "Bash", "cwd": cwd, "agent_type": "implementation-writer", "tool_input": {"command": command}})
        env = dict(os.environ, CLAUDE_PROJECT_DIR=root, PYTHONDONTWRITEBYTECODE="1")
        done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/heavy-test-guard.sh")], input=payload, capture_output=True, text=True, env=env, timeout=60)
        row.append(f"{name}={done.returncode}")
    ok = lambda text: ("拒" if text.endswith("=2") else "放")
    verdict = "".join(f"[{item.split('=')[0]}:{ok(item)}{'✓' if (ok(item)=='拒')==(expect=='D') else '✗'}]" for item in row)
    print(f"{expect}\t{label}\t" + "\t".join(row) + "\t" + verdict)
