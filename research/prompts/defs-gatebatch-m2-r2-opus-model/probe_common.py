"""defs-gatebatch-m2-r2 云端攻方（Opus）探针的共用部分：按路径导入被判的 admission.py / lib_heavy_tests.py，搭小仓，逐格记。
被判的文件只读；改动只落在临时目录（/tmp/claude-1000/defs-gatebatch-m2-r2-opus/ 下）。
每格写成「期望的行为」：ATTACK 格打中则不成立（BROKEN），CONTROL 格是对照、今天应当成立。"""
import importlib.util
import json
import os
import subprocess
import sys
import tempfile

REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
# 默认攻仓里被判的那一份；K1_ADMISSION / K1_HOOK 指到副本时攻副本（量我提的改法用，副本上的数不算入库装置上的数）
ADMISSION = os.environ.get("K1_ADMISSION") or os.path.join(REPOSITORY, "research/scripts/admission.py")
HOOK = os.environ.get("K1_HOOK") or os.path.join(REPOSITORY, ".claude/hooks/heavy-test-guard.sh")
SCRATCH_ROOT = "/tmp/claude-1000/defs-gatebatch-m2-r2-opus"
GIT_IDENTITY = {"GIT_AUTHOR_NAME": "probe", "GIT_AUTHOR_EMAIL": "probe@example.invalid",
                "GIT_COMMITTER_NAME": "probe", "GIT_COMMITTER_EMAIL": "probe@example.invalid"}


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def scratch(prefix):
    os.makedirs(SCRATCH_ROOT, exist_ok=True)
    return tempfile.mkdtemp(prefix=prefix, dir=SCRATCH_ROOT)


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def run(command, cwd=None, extra_environment=None):
    environment = dict(os.environ, **GIT_IDENTITY)
    environment.update(extra_environment or {})
    completed = subprocess.run(command, cwd=cwd, capture_output=True, text=True, env=environment)
    return completed.returncode, completed.stdout, completed.stderr


def hook(command, cwd, agent_type="implementation-writer", detections=None):
    """把一条 Bash 命令当 JSON 喂给重型测试闸（不执行命令）：交 (退出码, stderr 第一行)。"""
    payload = {"tool_name": "Bash", "cwd": cwd, "tool_input": {"command": command}}
    if agent_type:
        payload["agent_type"] = agent_type
    environment = dict(os.environ, CLAUDE_PROJECT_DIR=REPOSITORY,
                       AGENT_HOOK_DETECTIONS=detections or os.path.join(SCRATCH_ROOT, "detections.jsonl"))
    environment.pop("SINGLEFS_HEAVY_TESTS", None)
    completed = subprocess.run(["bash", HOOK], input=json.dumps(payload), capture_output=True, text=True, env=environment)
    return completed.returncode, completed.stderr.strip().split("\n")[0][:160]


class Cells:
    def __init__(self, title):
        self.title = title
        self.rows = []

    def expect(self, kind, label, holds, detail):
        self.rows.append((kind, label, bool(holds), detail))
        print(f"{kind}\t{'holds' if holds else 'BROKEN'}\t{label}\t{detail}", flush=True)

    def finish(self):
        attacks = [row for row in self.rows if row[0] == "ATTACK" and not row[2]]
        controls = [row for row in self.rows if row[0] == "CONTROL" and not row[2]]
        print(f"SUMMARY {self.title}: cells={len(self.rows)} attack_cells_broken={len(attacks)} control_cells_broken={len(controls)}")
        return 2 if controls else (1 if attacks else 0)
