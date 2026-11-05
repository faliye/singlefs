#!/usr/bin/env python3
"""只从两份定义（.claude/agent-common.md、.claude/main-agent.md）与两个钩子的文件头写明的判据出发，
对"真 bash 跑出来的观测"（oracle.py 的返回）算一条命令该不该拒。不读钩子的实现代码。
返回 [(编号, 原因)]；空表示该放行。"""
import os, re

CHECKER = "singlefs-checker-tier"
TEST_SUBS = {"test", "t", "nextest", "miri", "llvm-cov", "hack", "mutants"}
RUN_SUBS = {"test", "t", "run", "r", "bench", "nextest", "miri", "llvm-cov", "hack", "mutants"}
BUILD_SUBS = {"build", "b", "check", "c", "test", "t", "bench", "run", "r", "doc", "clippy", "rustc", "rustdoc", "fix", "nextest"}
HEAVY_NAMED = {  # 桩名 -> 判法（True 恒重型；函数按参数判）
    ".claude/scripts/check.sh": lambda a: True,
    ".claude/scripts/gate.sh": lambda a: True,
    "scripts/gate/gate-staged.sh": lambda a: True,
    ".claude/gate.d/checker-tier-lkmm.sh": lambda a: not (set(a) & {"--list", "--list-items", "--tool-versions"}),
    ".claude/gate.d/checker-tier-qemu-device-streams.sh": lambda a: not (set(a) & {"--list", "--list-items", "--tool-versions"}),
    ".claude/gate.d/checker-tier-broad-mutation.sh": lambda a: not (set(a) & {"--list", "--tool-versions"}),
    ".claude/gate.d/checker-tier-research-build-and-replay.sh": lambda a: not (set(a) & {"--list", "--list-items", "--tool-versions"}),
    ".claude/gate.d/checker-tier-formal-verification.sh": lambda a: "--quick" not in a and not (set(a) & {"--list", "--list-items", "--tool-versions"}),
    ".claude/gate.d/checker-tier-crates-mutation-replay.sh": lambda a: "--quick" not in a and not (set(a) & {"--list", "--list-items", "--tool-versions"}),
    ".claude/gate.d/checker-tier-crash-point-replay.sh": lambda a: not (set(a) & {"--list", "--list-items", "--tool-versions"}),
    "scripts/crash/crash-point-replay.sh": lambda a: "--quick" not in a and a != ["--list-items"],
    "scripts/crash/crash-amplification.sh": lambda a: a != ["--list-items"],
    "scripts/experiment-apparatus/replay.sh": lambda a: a != ["--list-items"],
    "scripts/mutation/mutate.sh": lambda a: any("crates/mutations.tsv" in x for x in a),
    "scripts/mutation/mutation-shard-run.sh": lambda a: True,
    "scripts/devices/build-patched-qemu.sh": lambda a: a != ["--selftest"],
    "qemu-system-x86_64": lambda a: True,
    "herd7": lambda a: not (set(a) & {"-version", "--version"}),
}


LIGHT_QUICK = {"scripts/crash/crash-point-replay.sh", ".claude/gate.d/checker-tier-formal-verification.sh", ".claude/gate.d/checker-tier-crates-mutation-replay.sh"}


def cargo_facts(ev, repo, copy):
    args = ev["args"]
    cut = args.index("--") if "--" in args else len(args)
    head = args[:cut]
    sub = next((a for a in head if not a.startswith("-") and not a.startswith("+")), "")
    if sub == "miri" or sub == "hack" or sub == "llvm-cov":
        sub = "test"
    manifest = None
    for i, a in enumerate(head):
        if a == "--manifest-path" and i + 1 < len(head):
            manifest = head[i + 1]
        elif a.startswith("--manifest-path="):
            manifest = a.split("=", 1)[1]
    directory = os.path.dirname(os.path.abspath(os.path.join(ev["cwd"], manifest))) if manifest else ev["cwd"]
    packages = []
    for i, a in enumerate(head):
        if a in ("-p", "--package") and i + 1 < len(head):
            packages.append(head[i + 1])
        elif a.startswith("--package="):
            packages.append(a.split("=", 1)[1])
    narrowing = bool(packages) or any(a in ("--test", "--lib", "--bin", "--bins", "--example", "--doc", "--tests", "--benches", "--bench") for a in head)
    workspace_flag = "--workspace" in head or "--all" in head
    return {"sub": sub, "head": head, "packages": packages, "directory": directory, "narrow": narrowing,
            "workspace_flag": workspace_flag, "no_run": "--no-run" in head,
            "list_only": "--list" in args[cut:] and cut < len(args)}


def in_tree(path, root):
    path, root = os.path.realpath(path), os.path.realpath(root)
    return path == root or path.startswith(root + os.sep)


def prefix_given(obs, ev):
    """前缀算带了：这条记录自己带，或日志里更早的任何一条桩记录带（env -i 之类在中间清掉的，前缀已经明明白白写过了）"""
    ok = ("commit", "user-request")
    if ev["env"].get("HEAVY") in ok:
        return True
    for other in obs["events"]:
        if other is ev:
            break
        if other["env"].get("HEAVY") in ok:
            return True
    return False


def expected_rejections(role, background, obs, repo, copy):
    """role: 'main' | 'sub' | 'editor'。返回 [(编号, 原因)]"""
    reasons = []
    sub_agent = role in ("sub", "editor")
    for ev in obs["events"]:
        name, env = ev["name"], ev["env"]
        if name == "cargo":
            f = cargo_facts(ev, repo, copy)
            sub = f["sub"]
            heavy = None
            if sub in TEST_SUBS and not f["list_only"] and not f["no_run"]:
                import fnmatch
                ckr = any(p.startswith(CHECKER) or fnmatch.fnmatch(CHECKER, p.split("@")[0]) for p in f["packages"]) or \
                    (not f["packages"] and os.path.realpath(f["directory"]).endswith(os.path.join("crates", CHECKER)))
                at_root = os.path.realpath(f["directory"]) in (os.path.realpath(repo), os.path.realpath(copy))
                if f["workspace_flag"]:
                    heavy = "全量测试：--workspace/--all"
                elif ckr:
                    heavy = "checker 档包的测试"
                elif at_root and not f["narrow"]:
                    heavy = "工作区根上不挑包不挑目标的 cargo test"
            if heavy:
                if sub_agent:
                    reasons.append(("H1", "子 agent 跑重型：" + heavy))
                elif not prefix_given(obs, ev):
                    reasons.append(("H1", "主 agent 没带前缀跑重型：" + heavy))
            if sub_agent and sub in RUN_SUBS and not f["no_run"] and env.get("MEMCAP") != "1":
                reasons.append(("H2", "子 agent 不经内存包装跑编译出来的代码：cargo " + sub))
            if sub in BUILD_SUBS and in_tree(f["directory"], repo) and not in_tree(f["directory"], copy) \
                    and env.get("BUILDDIR") != "1" and not env.get("TARGET") \
                    and not any(a.startswith("--target-dir") for a in f["head"]):
                reasons.append(("H3", "主工作区直接起会编的 cargo 且没给编译目录"))
        elif name in LIGHT_QUICK and "--quick" in ev["args"] and sub_agent and env.get("MEMCAP") != "1":
            reasons.append(("H2", "子 agent 跑轻型三条要经内存包装：" + name))
        elif name in HEAVY_NAMED:
            if HEAVY_NAMED[name](ev["args"]):
                if sub_agent:
                    reasons.append(("H1", "子 agent 跑重型脚本：" + name))
                elif not prefix_given(obs, ev):
                    reasons.append(("H1", "主 agent 没带前缀跑重型脚本：" + name))
        elif name == "claude" and sub_agent:
            reasons.append(("D13", "子 agent 起 claude 会话"))
        elif name == "scripts/hosts/detached-run.sh" and sub_agent and ev["args"][:1] in (["start"], ["stop"]):
            reasons.append(("D3", "子 agent 用 detached-run.sh " + ev["args"][0]))
    if sub_agent and background:
        reasons.append(("D10", "子 agent 用 run_in_background"))
    return reasons
