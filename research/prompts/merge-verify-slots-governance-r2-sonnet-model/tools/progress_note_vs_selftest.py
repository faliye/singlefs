#!/usr/bin/env python3
"""把 progress_note 读进度文件的各支逐个弄坏，量 `agent-watch.py --selftest` 红不红（第一轮判决说 M2、M4、M5、M6 归这份自证罩）。
用法：progress_note_vs_selftest.py <副本根> [--dry-run]
  <副本根> 是 minimal-copy.py 建的、带 scripts/agents、.claude/hooks、.claude/scripts、.claude/singlefs-ai-sop 的仓副本；只改副本里的 agent_watch_alerts.py，
  每个弄坏项跑完照原字节换回，收尾核哈希。--dry-run：只核每项的旧串恰好命中，不起自证。
每一项一行：项名<TAB>预期<TAB>结局（GREEN=自证退 0、RED=退非 0 且有「✗ 自检」行，红在哪一行跟在后面；TIMEOUT / OTHER 照写）。预期是推的（读代码），不同的行末加 DIFFERENT。
退出码：0 全部与预期相同；1 有不同；2 用法错或旧串没命中。每趟自证用 start_new_session 起、超时只对它自己的那一组发信号。"""
import hashlib
import os
import signal
import subprocess
import sys
import time

ALERTS = "scripts/agents/agent_watch_alerts.py"
MESSAGE = "；progress.md 在检查期间已消失或不可读"
TIMEOUT_SECONDS = 1500


def replace_nth(text, old, new, nth, expect_count):
    if text.count(old) != expect_count:
        raise SystemExit(f"旧串命中 {text.count(old)} 次，要 {expect_count} 次：{old!r}")
    position = -1
    for _ in range(nth):
        position = text.index(old, position + 1)
    return text[:position] + new + text[position + len(old):]


def mutations():
    """(项名, 预期, 环境变量, 改法函数或 None)"""
    return [
        ("M0 基线：不弄坏", "GREEN", {}, None),
        ("M1 moment 为 None 那一支的句子改字（单元用例看得见的那一支）", "GREEN", {}, lambda t: replace_nth(t, f'return "{MESSAGE}"', 'return "；X"', 1, 2)),
        ("M2 except OSError 那一支的句子改字", "GREEN", {}, lambda t: replace_nth(t, f'return "{MESSAGE}"', 'return "；X"', 2, 2)),
        ("M3 去掉 if moment is None 那一支", "GREEN", {}, lambda t: replace_nth(t, "        if moment is None:\n", "        if False:\n", 1, 1)),
        ("M4 readable_mtime 读不到时交 0.0 不交 None（输出与原来相同的等价改坏）", "GREEN", {},
         lambda t: replace_nth(t, "    except OSError:\n        return None\n", "    except OSError:\n        return 0.0\n", 1, 1)),
        ("M5 except OSError 改成 except KeyError", "GREEN", {},
         lambda t: replace_nth(t, "    except OSError:\n        return \"" + MESSAGE + "\"\n    return (f", "    except KeyError:\n        return \"" + MESSAGE + "\"\n    return (f", 1, 1)),
        ("M6 末几行取头不取尾（自证造的进度文件恰好 3 行、PROGRESS_TAIL_LINES 是 3）", "GREEN", {}, lambda t: replace_nth(t, "[-PROGRESS_TAIL_LINES:]", "[:PROGRESS_TAIL_LINES]", 1, 1)),
        ("M7 告警里「改于」改字", "GREEN", {}, lambda t: replace_nth(t, "改于 {format_duration", "改了 {format_duration", 1, 1)),
        ("C1 对照：只取头两行（末行「第三格在跑」不在了）", "RED", {}, lambda t: replace_nth(t, "[-PROGRESS_TAIL_LINES:]", "[:PROGRESS_TAIL_LINES - 1]", 1, 1)),
        ("C2 对照：弄坏开关 AGENT_WATCH_BREAK=progressnote（文件头写它红在这份自证）", "RED", {"AGENT_WATCH_BREAK": "progressnote"}, None),
    ]


def run_selftest(root, environment):
    command = [sys.executable, "-B", "scripts/agents/agent-watch.py", "--selftest"]
    env = {key: value for key, value in os.environ.items() if not key.endswith("_BREAK")}
    env.update(environment)
    started = time.time()
    process = subprocess.Popen(command, cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
    try:
        output, _ = process.communicate(timeout=TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        output, _ = process.communicate()
        return "TIMEOUT", output, time.time() - started
    failures = [line for line in output.splitlines() if "✗ 自检" in line]
    if process.returncode == 0 and not failures:
        return "GREEN", output, time.time() - started
    if process.returncode != 0 and failures:
        return "RED", output, time.time() - started
    return f"OTHER(rc={process.returncode})", output, time.time() - started


def main():
    if len(sys.argv) < 2 or sys.argv[1].startswith("--"):
        raise SystemExit(2)
    root = os.path.realpath(sys.argv[1])
    dry = "--dry-run" in sys.argv[2:]
    target = os.path.join(root, ALERTS)
    with open(target, "rb") as handle:
        original = handle.read()
    digest = hashlib.sha256(original).hexdigest()
    print(f"name=progress_note_vs_selftest file={ALERTS} sha256={digest}")
    different = 0
    for name, expected, environment, edit in mutations():
        try:
            if edit is not None:
                changed = edit(original.decode("utf-8"))
                if changed == original.decode("utf-8"):
                    raise SystemExit(f"{name}：改动没改变文本")
                if dry:
                    print(f"{name}\t预期 {expected}\t（dry-run：旧串命中核过）")
                    continue
                with open(target + ".new", "w", encoding="utf-8") as handle:
                    handle.write(changed)
                os.replace(target + ".new", target)
            elif dry:
                print(f"{name}\t预期 {expected}\t（dry-run）")
                continue
            outcome, output, seconds = run_selftest(root, environment)
        finally:
            if not dry:
                with open(target + ".new", "wb") as handle:
                    handle.write(original)
                os.replace(target + ".new", target)
        flag = "" if outcome == expected else "\tDIFFERENT"
        different += bool(flag)
        first_red = next((line.strip()[:160] for line in output.splitlines() if "✗ 自检" in line), "")
        print(f"{name}\t预期 {expected}\t结局 {outcome}（{seconds:.0f} 秒）{flag}")
        if first_red:
            print(f"    | 第一条红：{first_red}")
        print(f"    | 末行：{(output.strip().splitlines() or [''])[-1][:160]}")
    with open(target, "rb") as handle:
        if hashlib.sha256(handle.read()).hexdigest() != digest:
            print(f"✗ {ALERTS} 没照原字节换回")
            raise SystemExit(1)
    print("收尾：agent_watch_alerts.py 的哈希与开跑时相同")
    raise SystemExit(1 if different else 0)


main()
