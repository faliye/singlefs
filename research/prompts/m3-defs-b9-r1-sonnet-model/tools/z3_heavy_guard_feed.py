#!/usr/bin/env python3
"""Z3：喂 heavy-test-guard.sh 钩子入口（stdin JSON）。python3 z3_feed.py <树> <工作根> ；树 = 冻结副本拷贝（钩子所在仓）。
在 <工作根>/copy-same 与 copy-body 造两个仓副本（只放 .claude/hooks 登记表与一份 light 脚本），copy-body 里的脚本正文多一行。"""
import json, os, shutil, subprocess, sys
tree, wr = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
key = "scripts/gate/apply-writer-patch.py"
hook = os.path.join(tree, ".claude/hooks/heavy-test-guard.sh")
for name in ("copy-same", "copy-body"):
    root = os.path.join(wr, name)
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(os.path.join(root, "scripts/gate")); os.makedirs(os.path.join(root, ".claude/hooks"))
    shutil.copy(os.path.join(tree, key), os.path.join(root, key))
    shutil.copy(os.path.join(tree, ".claude/hooks/selftest-registry.tsv"), os.path.join(root, ".claude/hooks/selftest-registry.tsv"))
with open(os.path.join(wr, "copy-body", key), "a") as handle:
    handle.write("# 副本里多的一行\n")
env = dict(os.environ, SINGLEFS_WORK_ROOT=wr)
def feed(label, command, agent_type=None, cwd=None):
    payload = {"tool_name": "Bash", "cwd": cwd or wr, "tool_input": {"command": command}}
    if agent_type:
        payload["agent_type"] = agent_type
    r = subprocess.run(["bash", hook], capture_output=True, text=True, env=env, input=json.dumps(payload))
    print(f"## {label}\n命令：{command}\n退出码：{r.returncode}\nstderr：{r.stderr.strip()}\n")
copy_same, copy_body = os.path.join(wr, "copy-same", key), os.path.join(wr, "copy-body", key)
feed("子 agent 在没改脚本的副本里跑自证", f"python3 {copy_same} --selftest", "tooling-writer")
feed("子 agent 在改过正文的副本里跑自证", f"python3 {copy_body} --selftest", "tooling-writer")
feed("主 agent 不带前缀在改过正文的副本里跑自证", f"python3 {copy_body} --selftest")
feed("主 agent 带 =commit 前缀在改过正文的副本里跑自证", f"SINGLEFS_HEAVY_TESTS=commit python3 {copy_body} --selftest")
feed("主 agent 带 =user-request 前缀在改过正文的副本里跑自证", f"SINGLEFS_HEAVY_TESTS=user-request python3 {copy_body} --selftest")
feed("主 agent 不带前缀在没改脚本的副本里跑自证", f"python3 {copy_same} --selftest")
feed("主 agent 不带前缀在主仓里跑同一份登记成 light 的自证", f"python3 {os.path.join(tree, key)} --selftest")
