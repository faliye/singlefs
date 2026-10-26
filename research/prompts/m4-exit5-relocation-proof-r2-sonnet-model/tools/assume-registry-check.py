#!/usr/bin/env python3
"""登记式 assume_specification 的演示判法（草稿，不是门禁）。

用法：assume-registry-check.py <演示 crate 目录> <工作目录> <cargo 命令前缀…>
做四件事：
  1 源码里的 assume_specification 对象集合 == registry.tsv 的集合；
  2 好世界：cargo verus focus 过、cargo test 过（差分测试在真实 std 上逐值核规格谓词）；
  3 必红世界：把规格谓词改成错的（登记表点名的那一个），同时往 CLAIMS-MARKER 处塞一个只在错规格下才证得出的假命题：
    cargo verus focus 仍然过（Verus 把 assume_specification 当前提），cargo test 红；
  4 打出每步的退出码与摘要行。
cargo 命令前缀整条经调用方给的包装（capped.sh、run-with-memory-cap.sh）起；本脚本只拼 cargo 的子命令。
"""
import os
import re
import shutil
import subprocess
import sys

crate, work, *prefix = sys.argv[1:]
env = dict(os.environ)
env["PATH"] = os.path.expanduser("~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux") + ":" + env["PATH"]


def run(directory, *cargo_args):
    # 每个世界一个自己的编译目录：副本里的文件保留了修改时间，共用编译目录时 cargo 会把内容不同的两份当成同一份，拿旧的二进制来跑
    own_env = dict(env, CARGO_TARGET_DIR=os.path.join(work, "target-" + os.path.basename(directory)))
    out = subprocess.run([*prefix, "cargo", *cargo_args[0].split(), "--offline", *cargo_args[1:]], cwd=directory, capture_output=True, text=True, env=own_env)
    text = out.stdout + out.stderr
    with open(os.path.join(work, "log-" + os.path.basename(directory) + "-" + cargo_args[0].split()[0] + ".txt"), "w", encoding="utf-8") as handle:
        handle.write(text)
    return out.returncode, text


def summary(text, pattern):
    lines = [line.strip() for line in text.splitlines() if re.search(pattern, line)]
    if lines:
        return lines[-1]
    tail = [line.strip() for line in text.splitlines() if line.strip()]
    return "(无匹配行；输出末行：" + (tail[-1][:160] if tail else "空") + ")"


TEST_NAME = "differential::the_registered_spec_agrees_with_std_on_the_bounded_domain"
source = open(os.path.join(crate, "src/lib.rs"), encoding="utf-8").read()
found = sorted(set(re.findall(r"assume_specification\s*(?:<[^\[]*?>)?\s*\[\s*([\w:<>& ,]+?)\s*\]", source)))
found = sorted({re.sub(r"::<[^>]*>", "", re.sub(r"\s+", "", f)) for f in found})
registered = []
rows = {}
for line in open(os.path.join(crate, "registry.tsv"), encoding="utf-8"):
    if line.startswith("#") or line.startswith("std函数") or not line.strip():
        continue
    parts = line.rstrip("\n").split("\t")
    registered.append(parts[0])
    rows[parts[0]] = parts
print(f"name=assume_set source={found} registry={sorted(registered)} same={'yes' if found == sorted(registered) else 'NO'}")

good = os.path.join(work, "good")
shutil.copytree(crate, good, ignore=shutil.ignore_patterns("target"))
rc, text = run(good, "verus focus")
print(f"name=good_verus rc={rc} {summary(text, 'verification results')}")
rc, text = run(good, "test", "--lib", "--", TEST_NAME, "--exact")
print(f"name=good_test rc={rc} {summary(text, 'test result')}")

mutations = {
    "copied-returns-none-for-some": [("Some(x) => output == Some(x),", "Some(x) => output == None::<T>,", 1),
                                     ("Some(x) => output == Some(x),", "Some(x) => output == None::<u64>,", 1)],
}
claim = """pub fn claims_none_for_a_nonempty_vector(v: &Vec<u64>) -> (r: Option<u64>)
    requires v@.len() > 0,
    ensures r is None,
{
    v.first().copied()
}
"""
for name, edits in mutations.items():
    bad = os.path.join(work, "bad-" + name)
    shutil.copytree(crate, bad, ignore=shutil.ignore_patterns("target"))
    path = os.path.join(bad, "src/lib.rs")
    text = open(path, encoding="utf-8").read()
    # 规格谓词与它的可执行孪生一起改错（孪生是从谓词导出的，Verus 证两者相等，所以错规格下两者一起错）
    for old, replacement in (("Some(x) => output == Some(x),", "Some(x) => output == None::<T>,"),
                             ("Some(y) => x == y,", "Some(_y) => false,"),
                             ("None => false,\n        },\n        None => output.is_none(),", "None => true,\n        },\n        None => output.is_none(),")):
        assert text.count(old) == 1, old
        text = text.replace(old, replacement)
    text = text.replace("// CLAIMS-MARKER", claim)
    # 错规格下 first_or_none 自己的 ensures 会证不出（它是对的）；演示的是「错规格 + 一条只在错规格下才证得出的假命题」Verus 全接受，所以先去掉它
    text = re.sub(r"pub fn first_or_none.*?\n}\n", "", text, flags=re.S)
    open(path, "w", encoding="utf-8").write(text)
    rc, out = run(bad, "verus focus")
    print(f"name=bad_verus world={name} rc={rc} {summary(out, 'verification results')}")
    rc, out = run(bad, "test", "--lib", "--", TEST_NAME, "--exact")
    print(f"name=bad_test world={name} rc={rc} {summary(out, 'test result')}")
