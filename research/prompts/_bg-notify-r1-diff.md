# 附录二：后台任务通知这批改动（`git diff HEAD -- .claude/agent-common.md .claude/main-agent.md .claude/hooks/bash-command-detector.sh research/scripts/agent-watch.py research/scripts/check-staged.sh` 原样；基准 HEAD 00c9d4f1b865a6a1a54d62393528332c524d763c，生成于 2026-09-18 14:58:23 UTC）

这五份文件都是已跟踪文件的原地修改（`git status --porcelain` 五行都是 `M`），没有新增文件，因此本附录只有一节 diff，没有「新文件全文」一节。

## 一、diff（相对 HEAD `00c9d4f` 的工作区改动）

```diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index df6f3de..40578a3 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -37,7 +37,7 @@
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
-- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你；前台命令的 `timeout` 不超过 240000 毫秒。
+- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起（命令照前台的写法原样交给它，不再加 `nohup`、`setsid`、`disown`，也不在末尾加 `&`；并行起几个再 `wait` 的照常写），起完结束本轮，完成时会通知你；结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
   **结束本轮去等之前，再用 `run_in_background` 起一次 `bash research/scripts/cache-keepalive.sh`**：它 230 秒后退出，那条完成通知把你叫醒。被它叫醒就看一眼等的东西跑完没有：没跑完再起一次、结束本轮接着等，跑完了接着干、不用再起。自己已经交回或被停的不用管。预计单段要等 15 分钟以上的（层 0 全量、E152（按里程碑对比六家文件系统的文件性能） 那类），不起计时器。
   等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）只记不拦，把没超时的等待循环、按模式找进程这类命令交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
 
diff --git a/.claude/hooks/bash-command-detector.sh b/.claude/hooks/bash-command-detector.sh
index 06c6558..988107a 100755
--- a/.claude/hooks/bash-command-detector.sh
+++ b/.claude/hooks/bash-command-detector.sh
@@ -5,14 +5,18 @@
 # 主 agent 查进度才发现（records/2026-09-17-已分配口径三方与两个实验.md 第六节）。用户定：hook 不能终止任务和脚本，
 # 只负责检出问题，交给主 agent 去判断；子 agent 与脚本结不结束由主 agent 定。
 #
-# 检出两种，写进检出记录（默认 /tmp/claude-1000/agent-hook-detections.jsonl，环境变量 AGENT_HOOK_DETECTIONS 可改），每条一行 JSON：
+# 检出三种，写进检出记录（默认 /tmp/claude-1000/agent-hook-detections.jsonl，环境变量 AGENT_HOOK_DETECTIONS 可改），每条一行 JSON：
 #   ① 按模式匹配的进程命令（`pgrep -f`、`pkill -f`、`killall`）：模式串会命中自己所在的 shell（command-safety.md）；
-#   ② 没有 `timeout` 的 `until` / `while` 等待循环（里面有 `sleep`）：等的条件不成立时会一直等。
+#   ② 没有 `timeout` 的 `until` / `while` 等待循环（里面有 `sleep`）：等的条件不成立时会一直等；
+#   ③ `run_in_background` 起的命令里又自己放后台（`nohup`、`setsid`、`disown`、后面没有 `wait` 的单独 `&`）：外层 shell 当场退出，
+#      harness 的完成通知当场发出，真正在跑的东西跑完不再叫醒谁（2026-09-18 一个门禁分诊员这样起门禁与三个缓存计时器，
+#      records/2026-09-16-subagent拆分提案.md 第三十三节）。
 # 主 agent 的看门狗（research/scripts/agent-watch.py watch）每 15 秒读一次检出记录，读到本会话的检出就退出、叫醒主 agent。
 # 只看顶层命令文本；命令里只是带着这些字（heredoc 里的测试数据）也会被记一条，主 agent 看了判断即可。
 #
 #   bash-command-detector.sh             # 从 stdin 读 hook 的 JSON，永远退出 0
-#   bash-command-detector.sh --selftest  # 走一遍检出与不检出；BASH_COMMAND_DETECTOR_DISABLE_CHECK=1 时自检必须判红
+#   bash-command-detector.sh --selftest  # 走一遍检出与不检出；BASH_COMMAND_DETECTOR_DISABLE_CHECK=1 或
+#                                        # BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1（只关第三种）时自检必须判红
 set -uo pipefail
 HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
 # python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
@@ -22,9 +26,17 @@ from datetime import datetime, timezone
 
 PATTERN_PROCESS_COMMAND = re.compile(r"\bpgrep\s+(?:-\w+\s+)*-\w*f|\bpkill\s+(?:-\w+\s+)*-\w*f|\bkillall\b")
 WAIT_LOOP = re.compile(r"\b(until|while)\b[^\n]*?;\s*do\b[\s\S]*?\bsleep\b")
+SELF_DETACH = re.compile(r"\bnohup\b|\bsetsid\b|\bdisown\b")
+# 单独的 `&`：不是 `&&`、`>&`、`&>`、`|&`、`<&`（重定向与逻辑与都不算放后台）；它后面有 `wait` 的是外层等着它，不算
+LONE_AMPERSAND = re.compile(r"(?<![&>|<])&(?![&>])")
+
+def puts_itself_in_background(command):
+    if SELF_DETACH.search(command):
+        return True
+    return any(not re.search(r"\bwait\b", command[match.end():]) for match in LONE_AMPERSAND.finditer(command))
 DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"
 
-def findings_for(command):
+def findings_for(command, run_in_background=False):
     if os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1":
         return []
     findings = []
@@ -32,13 +44,15 @@ def findings_for(command):
         findings.append("按模式匹配的进程命令（pgrep -f / pkill -f / killall）")
     if WAIT_LOOP.search(command) and not re.search(r"\btimeout\b", command):
         findings.append("没有 timeout 的等待循环")
+    if run_in_background and puts_itself_in_background(command) and os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND") != "1":
+        findings.append("run_in_background 里又自己放后台（nohup / setsid / disown / 后面没有 wait 的 &）：完成通知当场发出，跑完的那个不会叫醒你")
     return findings
 
 def record(hook_input, detections_path):
     """检出就追加一行；返回写了几行。永远不拦命令。"""
     tool_input = hook_input.get("tool_input") or {}
     command = tool_input.get("command") or ""
-    findings = findings_for(command)
+    findings = findings_for(command, bool(tool_input.get("run_in_background")))
     if not findings:
         return 0
     entry = {
@@ -67,10 +81,18 @@ def selftest(hook_dir):
         ("pgrep -f", "pgrep -f e154-binary", 1),
         ("pkill -f", "pkill -f cargo", 1),
         ("killall", "killall cargo", 1),
+        ("后台起的 nohup … & 加 disown", "nohup nice -n 19 bash gate.sh > gate.log 2>&1 & echo $!; disown", 1, True),
+        ("后台起的命令末尾单独一个 &", "bash cache-keepalive.sh > keepalive.log 2>&1 &", 1, True),
+        ("后台起的命令只有 2>&1、|& 与 &&", "cargo build 2>&1 | tail -3 && echo ok; make |& tee out", 0, True),
+        ("后台起的普通命令", "bash cache-keepalive.sh", 0, True),
+        ("后台起的并行加 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", 0, True),
+        ("后台起的 & 在 wait 之后", "wait; bash late.sh &", 1, True),
+        ("前台命令里的 & 不算（不是 run_in_background）", "sleep 1 & wait", 0),
     ]
     results = []
-    for label, command, want in cases:
-        written = record({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": command}}, detections)
+    for label, command, want, *background in cases:
+        tool_input = {"command": command, "run_in_background": bool(background and background[0])}
+        written = record({"tool_name": "Bash", "session_id": "s", "tool_input": tool_input}, detections)
         results.append((label, want, written))
     # 走真实入口：从标准输入喂 JSON，退出码必须是 0（不拦），检出要落进文件
     script = os.path.join(hook_dir, "bash-command-detector.sh")
@@ -85,9 +107,9 @@ def selftest(hook_dir):
     for label, want, got in failures:
         print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
     if failures:
-        print("    → 看 findings_for() 与 record()；BASH_COMMAND_DETECTOR_DISABLE_CHECK 设着的话这里本来就该红")
+        print("    → 看 findings_for() 与 record()；BASH_COMMAND_DETECTOR_DISABLE_CHECK 或 _DISABLE_SELF_BACKGROUND 设着的话这里本来就该红")
         return 1
-    print(f"  ✓ 自检通过：按模式找进程与没超时的等待循环记进检出记录，普通命令不记，入口一律放行（查了 {len(results)} 种）")
+    print(f"  ✓ 自检通过：按模式找进程、没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 与前台的 & 不记，入口一律放行（查了 {len(results)} 种）")
     return 0
 
 def main():
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index 504823e..9373bb2 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -26,6 +26,8 @@
 
 判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。
 
+交回按子 agent 的 SubagentHandback 消息判。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。
+
 ## 派发提示怎么写
 
 在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。
diff --git a/research/scripts/agent-watch.py b/research/scripts/agent-watch.py
index 7af76bb..5144e06 100644
--- a/research/scripts/agent-watch.py
+++ b/research/scripts/agent-watch.py
@@ -16,6 +16,8 @@
 它只看、只报，不停任何子 agent、不杀任何进程：子 agent 与脚本不强制结束，结束不结束由主 agent 看了报告定（用户 2026-09-17）。
 退出码：0 被看的子 agent 全部交回或被停、没有告警；3 有告警；4 定时回报（没有告警、子 agent 还在跑，到点叫醒主 agent 看一眼）；2 用法错。
 「交回」按交回工具（SubagentHandback）成功返回判：只结束本轮、没交回的子 agent 还在等后台任务（缓存续命见 research/scripts/cache-keepalive.sh），不算结束。
+「被停」认两处：子 agent 会话记录里的 `[Request interrupted`（停在工具调用中途）；主会话记录（<会话 id>.jsonl）里 TaskStop 对它的成功结果，
+时刻不早于它自己最后一条记录减 30 秒（停在结束本轮、等后台任务的时候，子 agent 的会话记录里一个字都不写；停了之后又被续做的不算）。
 
 hook 的检出也在这里汇给主 agent：`.claude/hooks/bash-command-detector.sh` 只记不拦，把检出写进检出记录；
 watch 每 15 秒读一次，读到被看子 agent 所在会话的检出就退出、叫醒主 agent。
@@ -40,12 +42,39 @@ DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"
 WAIT_LOOP_PATTERN = re.compile(r"\b(until|while)\b[\s\S]*\bsleep\b")
 BANNED_COMMAND_PATTERN = re.compile(r"\bpgrep\s+-f\b|\bpkill\s+-f\b|\bkillall\b")
 BROKEN_DETECTION = os.environ.get("AGENT_WATCH_BREAK", "")
+STOP_AFTER_LAST_RECORD_SECONDS = 30  # 停在工具中途时，被停之后还可能落几条收尾记录
 
 
 def parse_timestamp(text):
     return datetime.fromisoformat(text.replace("Z", "+00:00"))
 
 
+def parent_session_transcript(transcript_path):
+    """…/<会话 id>/subagents/agent-<id>.jsonl → …/<会话 id>.jsonl（派它的那个会话的记录）。"""
+    return os.path.dirname(os.path.dirname(transcript_path)) + ".jsonl"
+
+
+def task_stop_time(agent_id, session_transcript_path):
+    """主会话记录里 TaskStop 停掉这个子 agent 的最晚一次成功结果的时刻；没有就 None。"""
+    if not os.path.isfile(session_transcript_path):
+        return None
+    stopped_at = None
+    for line in open(session_transcript_path, encoding="utf-8", errors="replace"):
+        if agent_id not in line or "Successfully stopped task" not in line:
+            continue
+        try:
+            record = json.loads(line)
+        except ValueError:
+            continue
+        result = record.get("toolUseResult")
+        if not isinstance(result, dict) or result.get("task_id") != agent_id or "Successfully stopped task" not in str(result.get("message", "")):
+            continue
+        if record.get("timestamp"):
+            timestamp = parse_timestamp(record["timestamp"])
+            stopped_at = timestamp if stopped_at is None else max(stopped_at, timestamp)
+    return stopped_at
+
+
 def format_duration(seconds):
     seconds = int(max(seconds, 0))
     if seconds >= 3600:
@@ -88,6 +117,10 @@ class AgentTranscript:
         self.pending_tool = None         # (时间, 工具名, 命令或路径) —— 发出了还没收到结果
         self.state = "思考中"
         self._read()
+        stopped_at = task_stop_time(agent_id, parent_session_transcript(transcript_path))
+        if (stopped_at is not None and BROKEN_DETECTION != "taskstop" and self.state != "已交回"
+                and (self.last_timestamp is None or (self.last_timestamp - stopped_at).total_seconds() <= STOP_AFTER_LAST_RECORD_SECONDS)):
+            self.state = "被停"
 
     def _read(self):
         seen_message_ids = set()
@@ -563,6 +596,20 @@ def write_transcript(directory, agent_id, records, meta=None):
         json.dump(meta, open(path[: -len(".jsonl")] + ".meta.json", "w", encoding="utf-8"), ensure_ascii=False)
 
 
+def write_session_stops(session_directory, minutes_ago_by_agent):
+    """主会话记录里 TaskStop 的调用与成功结果，结果的形态照 2026-09-18 主会话记录里的原样。"""
+    with open(session_directory + ".jsonl", "w", encoding="utf-8") as handle:
+        for index, (agent_id, minutes_ago) in enumerate(minutes_ago_by_agent.items()):
+            use = record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": f"s{index}", "name": "TaskStop", "input": {"task_id": agent_id}}],
+                            stop_reason="tool_use", message_id=f"ms{index}")
+            message = f"Successfully stopped task: {agent_id} (样本)"
+            result = record_at(minutes_ago - 0.01, "user", [{"tool_use_id": f"s{index}", "type": "tool_result",
+                                                             "content": json.dumps({"message": message, "task_id": agent_id}, ensure_ascii=False)}])
+            result["toolUseResult"] = {"message": message, "task_id": agent_id, "task_type": "local_agent", "command": "样本"}
+            handle.write(json.dumps(use, ensure_ascii=False) + "\n")
+            handle.write(json.dumps(result, ensure_ascii=False) + "\n")
+
+
 def iso_minutes_ago(minutes):
     return (datetime.now(timezone.utc).timestamp() - minutes * 60)
 
@@ -631,6 +678,13 @@ def selftest():
     write_transcript(session, "idle", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1")])
     write_transcript(session, "interrupted", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1"),
                                               record_at(39, "user", [{"type": "text", "text": "[Request interrupted by user]"}])])
+    waiting_then_idle = [bash_use(25, "t1", "bash gate.sh --staged", "m1"), bash_result(24.9, "t1"),
+                         record_at(24.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")]
+    write_transcript(session, "stoppedidle", waiting_then_idle,
+                     {"agentType": "gate-triage", "description": "结束本轮等后台任务时被 TaskStop 停掉"})
+    write_transcript(session, "stoppedcontinued", [*waiting_then_idle, record_at(2, "user", "续做：再跑一遍"), bash_use(0.5, "t2", "cargo build", "m3")],
+                     {"agentType": "gate-triage", "description": "被停之后又被续做"})
+    write_session_stops(session, {"stoppedidle": 20, "stoppedcontinued": 20})
     write_transcript(session, "costed", [
         record_at(30, "assistant", [{"type": "tool_use", "id": "c1", "name": "Read", "input": {"file_path": "/x/e999-preregistration.md"}}],
                   stop_reason="tool_use", message_id="k1",
@@ -657,6 +711,7 @@ def selftest():
         failures.append(f"工具结果分类不对：{costed.tool_result_characters}")
     expectations = {
         "finished": set(), "healthy": set(), "boundedloop": set(), "interrupted": set(), "repeatchanging": set(), "waiting": set(), "continued": set(),
+        "stoppedidle": set(), "stoppedcontinued": set(),
         "waitloop": {"等待循环"}, "longtool": {"工具调用过长"}, "repeat": {"同一命令反复且输出不变"}, "banned": {"禁用命令"}, "idle": {"无动静"},
     }
     now = datetime.now(timezone.utc)
@@ -665,7 +720,8 @@ def selftest():
         got = {name for name, _, _ in agent_alerts(transcript, now, thresholds)}
         if got != wanted:
             failures.append(f"子 agent {agent_id}：应当告警 {sorted(wanted) or '无'}，实际 {sorted(got) or '无'}")
-    wanted_done = {"finished": True, "interrupted": True, "waiting": False, "continued": False, "healthy": False}
+    wanted_done = {"finished": True, "interrupted": True, "stoppedidle": True, "waiting": False, "continued": False, "healthy": False,
+                   "stoppedcontinued": False}
     for agent_id, wanted in wanted_done.items():
         transcript = AgentTranscript(agent_id, find_transcript(agent_id, session))
         if transcript.is_done() != wanted:
@@ -690,7 +746,7 @@ def selftest():
         return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", *watch_arguments, "--session-dir", session, "--process-root-pid", "0"],
                               capture_output=True, text=True, timeout=timeout)
 
-    watch_exit = run_watch(["--agents", "finished,interrupted", "--interval-seconds", "1", "--max-minutes", "1"]).returncode
+    watch_exit = run_watch(["--agents", "finished,interrupted,stoppedidle", "--interval-seconds", "1", "--max-minutes", "1"]).returncode
     if watch_exit != 0:
         failures.append(f"被看的子 agent 都交回或被停时看门狗应当退出码 0，实际 {watch_exit}")
     watch_exit = run_watch(["--agents", "waitloop", "--interval-seconds", "1", "--max-minutes", "1"]).returncode
@@ -734,7 +790,8 @@ def selftest():
         print("    → 看 agent_alerts() / process_alerts() / run() 的判法；AGENT_WATCH_BREAK 设着的话这里本来就该红")
         return 1
     print(f"  ✓ agent-watch 自检通过：等待循环、工具过长、同一命令反复且输出不变、禁用命令、无动静、进程无输出六种告警都报得出，"
-          f"交回、被停、带超时的循环、输出在变的复检与健康的子 agent 不误报，交回按交回工具成功判（只结束本轮、交回后又被续做的都不算），看门狗三种退出码对、定时回报不被复检间隔拖后并列出还没交回的子 agent，本会话的 hook 检出会叫醒主 agent、别的会话的与只检出等待循环的不立刻叫醒，用量与工具结果分类对（查了 {len(expectations) + 1} 个子 agent、1 个进程、{len(watch_runs)} 次看门狗）")
+          f"交回、被停、带超时的循环、输出在变的复检与健康的子 agent 不误报，交回按交回工具成功判（只结束本轮、交回后又被续做的都不算），"
+          f"被停认会话记录里的打断与主会话记录里的 TaskStop（停在等后台任务时的算、停了又被续做的不算），看门狗三种退出码对、定时回报不被复检间隔拖后并列出还没交回的子 agent，本会话的 hook 检出会叫醒主 agent、别的会话的与只检出等待循环的不立刻叫醒，用量与工具结果分类对（查了 {len(expectations) + 1} 个子 agent、1 个进程、{len(watch_runs)} 次看门狗）")
     return 0
 
 
diff --git a/research/scripts/check-staged.sh b/research/scripts/check-staged.sh
index 1bb607c..eb02c2f 100755
--- a/research/scripts/check-staged.sh
+++ b/research/scripts/check-staged.sh
@@ -9,7 +9,8 @@
 # 规范副本 .claude/singlefs-ai-sop/ 不进 git，worktree 里没有它；有就原样拷进去，doc-lint 与链接检查才跑得起来。
 # 自证会红：--selftest 在临时仓里放一个「文件里有 BAD 就红」的阶段，确认工作区里没暂存的 BAD 不算、暂存了的 BAD 判红；
 # 再用 CHECK_STAGED_USE_WORKTREE=1 改成拿工作区原样去跑，确认没暂存的 BAD 也被算进来、selftest 判红；
-# CHECK_STAGED_NO_TRAP=1 关掉打断时的清理，确认 selftest 判红（临时 worktree 留在仓里）。
+# CHECK_STAGED_NO_TRAP=1 关掉打断时的清理，确认 selftest 判红（临时 worktree 留在仓里）；
+# CHECK_STAGED_77_IS_RED=1 把退出码 77（无对象可判）照旧算红，确认 selftest 判红。
 set -uo pipefail
 
 DEFAULT_STAGES=(doc 10 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 42 43 44 45 50 60 61 85 86)
@@ -25,6 +26,7 @@ run_isolated() {
   local repo="$1"; shift
   local stages=("$@")
   local base wt patch red=0 ran=0
+  local -a not_run=()
   base="$(mktemp -d)"; wt="$base/wt"; patch="$base/staged.patch"
   git -C "$repo" diff --cached --binary > "$patch" || { echo "  ✗ 取不到暂存区的 diff"; echo "    → 在仓里跑，并确认 git 可用"; rm -rf "${base:?}"; return 2; }
   git -C "$repo" worktree add --detach "$wt" HEAD >/dev/null 2>&1 || { echo "  ✗ 建临时 worktree 失败"; echo "    → git worktree prune 之后再试"; rm -rf "${base:?}"; return 2; }
@@ -58,7 +60,10 @@ run_isolated() {
       [[ -f "$script" ]] || continue
       matched=1; ran=$((ran + 1))
       out="$(cd "$wt" && bash "$script" 2>&1)"; rc=$?
-      if [[ $rc -eq 0 ]]; then echo "  ok  $(basename "$script")"; else red=$((red + 1)); echo "  RED $(basename "$script")"; echo "$out" | grep -E "✗|→" | head -8; fi
+      # 77 = 这一轮无对象可判（show-me-test.md「门禁不许假装通过」）：不算红，也不算通过，单独列名
+      if [[ $rc -eq 77 && "${CHECK_STAGED_77_IS_RED:-0}" != 1 ]]; then
+        not_run+=("$(basename "$script")"); echo "  77  $(basename "$script")（无对象可判，没跑）"
+      elif [[ $rc -eq 0 ]]; then echo "  ok  $(basename "$script")"; else red=$((red + 1)); echo "  RED $(basename "$script")"; echo "$out" | grep -E "✗|→" | head -8; fi
     done
     [[ $matched -eq 1 ]] || echo "  ! 阶段号 $stage 没有对应的 .claude/gate.d/$stage-*.sh（没跑）"
   done
@@ -69,12 +74,18 @@ run_isolated() {
     echo "    → 阶段号写成 .claude/gate.d/ 下文件名的前缀（例：34），doc-lint 写 doc"
     return 1
   fi
+  local not_run_note=""
+  [[ ${#not_run[@]} -gt 0 ]] && not_run_note="；无对象没跑 ${#not_run[@]} 个：${not_run[*]}"
   if [[ $red -gt 0 ]]; then
-    echo "  ✗ HEAD + 暂存区上跑了 $ran 个阶段，红 $red 个"
+    echo "  ✗ HEAD + 暂存区上跑了 $ran 个阶段，红 $red 个${not_run_note}"
     echo "    → 红的是这一次提交带进来的（别人的未提交改动不在这棵树里）；先查它在不在 HEAD 上就红：暂存区清空再跑一次"
     return 1
   fi
-  echo "  ✓ HEAD + 暂存区上跑了 $ran 个阶段，全绿"
+  if [[ ${#not_run[@]} -eq $ran ]]; then
+    echo "  ! HEAD + 暂存区上点名的 $ran 个阶段全是无对象可判，一个都没判（不记通过）"
+    return 77
+  fi
+  echo "  ✓ HEAD + 暂存区上跑了 $ran 个阶段，判了的 $((ran - ${#not_run[@]})) 个全绿${not_run_note}"
   return 0
 }
 
@@ -89,6 +100,12 @@ selftest() {
   echo "BAD（别人没收尾的）" >> "$repo/kb/a.md"
   echo "我的" >> "$repo/kb/b.md" && git -C "$repo" add kb/b.md
   run_isolated "$repo" 20 >/dev/null; rc_clean=$?
+  # 无对象可判（77）的阶段：与判得了的阶段一起点名时应当通过并在成功行里列名；只点名它时应当报 77、不报通过
+  printf '#!/usr/bin/env bash\n# gate-stage: selftest-none\necho "  ! 无对象"; exit 77\n' > "$repo/.claude/gate.d/40-none.sh"
+  git -C "$repo" add .claude/gate.d/40-none.sh
+  local out_mixed rc_mixed rc_none
+  out_mixed="$(run_isolated "$repo" 20 40 2>&1)"; rc_mixed=$?
+  run_isolated "$repo" 40 >/dev/null 2>&1; rc_none=$?
   if [[ "${CHECK_STAGED_USE_WORKTREE:-0}" == 1 ]]; then
     rm -rf "${repo:?}"
     if [[ $rc_clean -eq 0 ]]; then echo "selftest: 拿工作区原样去跑，没暂存的 BAD 却没算进来 —— 检查坏了"; return 1; fi
@@ -122,7 +139,11 @@ selftest() {
   if [[ "$registered_after_interrupt" != 1 ]]; then echo "selftest: 跑到一半被打断，临时 worktree 留在仓里（登记 $registered_after_interrupt 个）"; return 1; fi
   if [[ $rc_clean -ne 0 ]]; then echo "selftest: 只有工作区里没暂存的 BAD，却判红了 —— 别人的改动漏进来了"; return 1; fi
   if [[ $rc_bad -ne 1 ]]; then echo "selftest: 暂存了 BAD 却没判红"; return 1; fi
-  echo "selftest: 通过（没暂存的 BAD 不算、暂存了的 BAD 判红、跑到一半被打断也清掉临时 worktree）"
+  if [[ $rc_mixed -ne 0 || "$out_mixed" != *"无对象没跑 1 个：40-none.sh"* ]]; then
+    echo "selftest: 无对象可判（退出码 77）的阶段被当成红、或没在成功行里列名（退出码 $rc_mixed）"; return 1
+  fi
+  if [[ $rc_none -ne 77 ]]; then echo "selftest: 点名的阶段全是无对象可判时应当退出码 77，实际 $rc_none"; return 1; fi
+  echo "selftest: 通过（没暂存的 BAD 不算、暂存了的 BAD 判红、无对象可判的阶段不算红也不算通过、跑到一半被打断也清掉临时 worktree）"
   return 0
 }
 
```
