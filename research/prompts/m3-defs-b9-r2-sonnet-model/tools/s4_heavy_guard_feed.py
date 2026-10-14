#!/usr/bin/env python3
"""S4：重型测试闸钩子入口（stdin JSON）。第一轮正推的 7 例（z3_heavy_guard_feed.py）之外再加：
  A 子 agent 在改过正文的副本里，自己给命令加 SINGLEFS_HEAVY_TESTS=commit / =user-request 前缀（拒绝说明现在写着『命令带 =commit』，子 agent 照抄会不会被放）
  B 主 agent 带 =commit 在『副本里正文改过、改成会起重活』的 light 自证上跑（闸不读副本脚本正文）
  C 主 agent 在副本里跑登记成 heavy 的自证（vm-bench.sh）：不带前缀 / 带 =commit / 带 =user-request
  D 子 agent 在没改正文的副本里跑登记成 heavy 的自证
用法：s4_heavy_guard_feed.py <树> <工作根>。树＝冻结副本拷贝（钩子所在仓）。"""
import json, os, shutil, subprocess, sys
tree, wr = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
light = "scripts/gate/apply-writer-patch.py"; heavy = "scripts/devices/vm-bench.sh"
hook = os.path.join(tree, ".claude/hooks/heavy-test-guard.sh")
def make(name, files, changed=()):
    root = os.path.join(wr, name); shutil.rmtree(root, ignore_errors=True)
    for f in files:
        os.makedirs(os.path.dirname(os.path.join(root, f)), exist_ok=True); shutil.copy(os.path.join(tree, f), os.path.join(root, f))
    os.makedirs(os.path.join(root, ".claude/hooks"), exist_ok=True)
    shutil.copy(os.path.join(tree, ".claude/hooks/selftest-registry.tsv"), os.path.join(root, ".claude/hooks/selftest-registry.tsv"))
    for f, extra in changed:
        with open(os.path.join(root, f), "a") as handle: handle.write(extra)
    return root
same = make("copy-same", [light, heavy]); body = make("copy-body", [light, heavy], [(light, "# 副本里多的一行\n")])
heavy_body = make("copy-body-heavy", [light, heavy], [(light, "\nimport subprocess\nsubprocess.run(['cargo', 'test', '--release', '-p', 'singlefs-checker-tier'])  # 副本里改成起重活，闸不读正文\n")])
env = dict(os.environ, SINGLEFS_WORK_ROOT=wr)
def feed(label, command, agent_type=None):
    payload = {"tool_name": "Bash", "cwd": wr, "tool_input": {"command": command}}
    if agent_type: payload["agent_type"] = agent_type
    r = subprocess.run(["bash", hook], capture_output=True, text=True, env=env, input=json.dumps(payload))
    first = (r.stderr.strip().split("\n") or [""])[0][:14]   # 只留开头几个字：后面带草稿目录路径，换占位之前截断会随路径长短变
    print(f"{label}\t退出码={r.returncode}\t{first}")
P = lambda root, f: os.path.join(root, f)
feed("A1 子 agent 在改过正文的副本里自加 =commit 前缀", f"SINGLEFS_HEAVY_TESTS=commit python3 {P(body, light)} --selftest", "tooling-writer")
feed("A2 子 agent 在改过正文的副本里自加 =user-request 前缀", f"SINGLEFS_HEAVY_TESTS=user-request python3 {P(body, light)} --selftest", "tooling-writer")
feed("A3 子 agent 在没改正文的副本里不带前缀（对照）", f"python3 {P(same, light)} --selftest", "tooling-writer")
feed("B1 主 agent 带 =commit，副本正文被改成起 cargo test（闸不读正文）", f"SINGLEFS_HEAVY_TESTS=commit python3 {P(heavy_body, light)} --selftest")
feed("B2 主 agent 不带前缀，同一份", f"python3 {P(heavy_body, light)} --selftest")
feed("C1 主 agent 不带前缀，副本里登记成 heavy 的自证", f"python3 {P(same, heavy)} --selftest")
feed("C2 主 agent 带 =commit，副本里登记成 heavy 的自证", f"SINGLEFS_HEAVY_TESTS=commit bash {P(same, heavy)} --selftest")
feed("C3 主 agent 带 =user-request，副本里登记成 heavy 的自证", f"SINGLEFS_HEAVY_TESTS=user-request bash {P(same, heavy)} --selftest")
feed("D1 子 agent 在没改正文的副本里跑登记成 heavy 的自证", f"bash {P(same, heavy)} --selftest", "tooling-writer")
feed("E1 主 agent 带 =commit 在主仓里跑登记成 heavy 的自证", f"SINGLEFS_HEAVY_TESTS=commit bash {P(tree, heavy)} --selftest")
