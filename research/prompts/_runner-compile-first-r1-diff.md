# 附录二：runner-compile-first-r1 被判的改动（基准 HEAD `faf255e2`，生成于 2026-09-27 /）

## 一、`git diff HEAD -- .claude/hooks/write-guard.sh .claude/hooks/bash-command-detector.sh`（原样，工作区相对 HEAD `faf255e2` 的改动；`git diff --stat` 核过为 244 行增、20 行删，与主 agent 给的数一致）

```diff
diff --git a/.claude/hooks/bash-command-detector.sh b/.claude/hooks/bash-command-detector.sh
index 5e4344cd..fabff104 100755
--- a/.claude/hooks/bash-command-detector.sh
+++ b/.claude/hooks/bash-command-detector.sh
@@ -4,7 +4,7 @@
 # PreToolUse hook（Bash）：检出可能出问题的命令，记下来交给主 agent 判断；起看门狗的错误写法、前台没超时的等待循环、把活放出追踪的写法、run_in_background 里后面没有 wait 的单独 `&`、整份覆盖 `research/results/` 下未跟踪产物的写法、打得到别人进程的终止写法与在同一个 inode 上改已有脚本的写法在执行前拒绝，其余只记不拦，不停任何在跑的命令与脚本。
 # hook-events: PreToolUse:Bash
 # gate-similar: heavy-test-guard.sh 同挂 PreToolUse[Bash]、按同一个切词模块认命令位置，但它只判重型测试与内存包装；这里判等待循环、放出追踪、单独的 &、覆盖产物、终止进程、同 inode 改脚本、就地改仓内文件，判据没有一条相同
-# gate-similar: write-guard.sh 管 Write / Edit 的写范围；这里 ⑧ 管有 Edit 的子 agent 绕到 Bash 里写仓内文件，两边判的工具不同
+# gate-similar: write-guard.sh 管 Write / Edit 的写范围与先编后换；这里 ⑧ 管有 Edit 的子 agent 绕到 Bash 里写仓内文件、⑨ 管执行员在 Bash 里写主工作区 crates/ 下的 .rs，两边判的工具不同
 # gate-similar: pattern-process-guard.sh（上游）同挂 Bash、拒按模式找进程；这里不判那一类
 #
 # 按模式找进程（`pgrep -f`、`pkill -f`、`killall`）不在这里判：上游 SOP 的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`
@@ -54,6 +54,18 @@
 # ⑧ 有 Edit 工具的项目子 agent（`.claude/agents/<agent_type>.md` 的 tools 里有 Edit）在 Bash 里就地改仓内文件：命令位置上的 `sed -i`（目标不在 /tmp/ 下），
 #   或 python 代码里 open(<仓内路径字面量>, 'w' / 'a' / 'x')、Path(<仓内路径字面量>).write_text( / .write_bytes(。写范围闸只看 Write / Edit，这样写它看不见；
 #   草稿目录照写，定义点名的脚本（replace-once.py 这类）不在这一条里。路径放在变量里的认不出。
+# ⑨ 先编后换：experiment-runner（名单是 COMPILE_FIRST_AGENTS）在 Bash 里写主工作区 crates/ 下的 .rs：出路是在草稿目录的副本里改、
+#   经 research/scripts/compile-then-swap.py 编过再整份换进来。write-guard.sh「四、先编后换」判 Write / Edit 那一半，这里判 Bash 那一半；
+#   agent 名单与路径判据两边各写一份、改的时候一起改。排在 ⑧ 之前判，执行员拿到的出路是这条脚本。
+#   为什么：执行员直接在主工作区改入库装置，改到一半编不过，别的会话带 --all-targets 的编译一起卡住（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 51 行）。
+#   认的写法：⑤ ⑦ 那一段 overwrite_steps 认的（>、>|、&>、>& 文件、不带 -a 的 tee、cp / mv / install 的目标、dd of=、truncate），every_write 时另认
+#   >>、&>>、tee -a、sed -i、perl -i、rsync 的目标，mv 挪走与 rm 删掉的源（连同 crates/ 里的目录），喂给 python 的代码里 open(…, 带 w / a / x / + 的模式)、
+#   Path.write_text / write_bytes、shutil.copyfile / copy / copy2 / move 与 os.replace / os.rename 的目标。cd 与前面赋过值的变量跟着算，每一层都判；
+#   目标里有这一刻算不出的段的，按通配对主工作区已有的文件，对得上 crates/ 下的 .rs 就拒。前台、run_in_background 一样拒。
+#   放行：主 agent、implementation-writer 与别的 agent；草稿目录里的写；把主工作区那份拷进草稿目录；crates/ 下不是 .rs 的（crates/mutations.tsv）；
+#   `python3 research/scripts/compile-then-swap.py …`（脚本文件里的写这里本来就判不到）；引号里当数据写的、写进文件的 heredoc 正文、注释里的。
+#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、patch 与 git apply / checkout / restore、目标算不出又是新文件名的、
+#   python 里从运行期值来的路径与 os.open / os.write、经 subprocess 起的写法。
 #   「外面有 timeout」按包装的层次认：`timeout N bash -c '…'`、`capped.sh N timeout N bash -c '…'`、`timeout N bash <<EOF` 喂进去的正文里的循环算有；
 #   只在循环条件或循环体里的 timeout（`until timeout 5 grep …; do sleep 10; done`）不算，循环照样能一直转下去。
 #   只认命令位置（按 shell 的规矩切词），引号里当数据写的、写进文件的 heredoc 正文、注释里的循环不拒；
@@ -156,6 +168,7 @@
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_UNWAITED_AMPERSAND=1（run_in_background 里后面没有 wait 的单独 & 也放行）或
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_BACKGROUND_TASK_OUTPUT_WAIT=1（run_in_background 里轮询 tasks/*.output 也放行）或
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_REPOSITORY_IN_PLACE_EDIT=1（有 Edit 的子 agent 在 Bash 里就地改仓内文件也放行）或
+#                                        # BASH_COMMAND_DETECTOR_ALLOW_COMPILE_FIRST_BYPASS=1（⑨ experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs 也放行）或
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_RESULTS_OVERWRITE=1（整份覆盖 research/results/ 下未跟踪产物也放行、也不记检出）或
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_PROCESS_SIGNALS=1（⑥ 终止进程的写法也放行）或
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_SCRIPT_IN_PLACE_WRITE=1（⑦ 在同一个 inode 上改已有脚本也放行）或
@@ -486,6 +499,57 @@ def repository_in_place_edit_refusal(command, agent_type, repository_root):
         found.append(f"python 写 {match.group('open') or match.group('path')}")
     return list(dict.fromkeys(found))
 
+# ⑨ 先编后换：experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs。写法沿用 ⑤ ⑦ 的 overwrite_steps（every_write=True），
+#   mv 挪走、rm 删掉的源也算改。agent 名单与路径判据与 write-guard.sh「四、先编后换」各写一份、改的时候一起改。
+COMPILE_FIRST_AGENTS = ("experiment-runner",)
+COMPILE_THEN_SWAP_SCRIPT = "research/scripts/compile-then-swap.py"
+
+def is_main_workspace_crate_source(path, repository_root):
+    return path.endswith(".rs") and is_inside(path, os.path.join(repository_root, "crates"))
+
+def compile_first_verdict(command, repository_root):
+    """整条命令里写主工作区 crates/ 下 .rs 的每一步，交回 `写法 相对仓库根的路径`。"""
+    crates_directory = os.path.join(repository_root, "crates")
+    refused = []
+    for step in overwrite_steps(command, repository_root, every_write=True):
+        base = step.directory if step.directory is not None else repository_root
+        targets = []
+        if isinstance(step, PreservingStep):
+            if step.kind in ("mv", "rm"):
+                for path in preserved_paths(step, base):
+                    if is_main_workspace_crate_source(path, repository_root) or (
+                            os.path.isdir(path) and (os.path.normpath(path) == crates_directory or is_inside(path, crates_directory))):
+                        refused.append(f"{step.kind} {os.path.relpath(path, repository_root)}")
+            continue
+        if isinstance(step, PythonCode):
+            for write in python_in_place_writes(step.code, every_write=True):
+                path = shell_words.resolve_path(base, write.path)
+                if write.source is not None and os.path.isdir(path):
+                    path = os.path.join(path, os.path.basename(write.source))
+                targets.append((write.form, path))
+        else:
+            for word in destination_words(step, base):
+                pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
+                path = shell_words.resolve_path(base, pattern)
+                if has_unknown_piece or has_glob:
+                    # `"$变量"/crates/…` 这类变量在前的，另从 crates/ 那一段起按仓库根接（与 ⑤ 认 research/results/ 同一个办法）
+                    anchor = pattern.find("crates/")
+                    patterns = [path] + ([os.path.join(repository_root, pattern[anchor:])] if anchor > 0 and "*" in pattern[:anchor] else [])
+                    targets += [(step.form, match) for candidate in patterns for match in sorted(glob.glob(candidate))]
+                else:
+                    targets.append((step.form, path))
+        for form, path in targets:
+            if is_main_workspace_crate_source(path, repository_root):
+                refused.append(f"{form} {os.path.relpath(path, repository_root)}")
+    return list(dict.fromkeys(refused))
+
+def compile_first_refusal(command, agent_type, repository_root):
+    """交回要拒的写法与目标；只判 COMPILE_FIRST_AGENTS 里的 agent，前台、run_in_background 一样判。"""
+    if (agent_type not in COMPILE_FIRST_AGENTS or os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1"
+            or os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_COMPILE_FIRST_BYPASS") == "1"):
+        return []
+    return compile_first_verdict(command, repository_root)
+
 # run_in_background 里等本会话后台任务输出文件（<会话目录>/tasks/<id>.output）的循环：等自己起的后台任务要结束本轮等完成通知，不写轮询
 TASK_OUTPUT_PATH = re.compile(r"(?:^|/)tasks/[^/\s]+\.output\b")
 
@@ -574,6 +638,7 @@ def unwaited_ampersand_refusal(command, run_in_background):
 
 # ⑤ 整份覆盖 research/results/ 下未跟踪的产物：按命令位置认会整份覆盖文件的写法，跟着 cd 与这条命令里赋过的变量算目标
 OVERWRITING_REDIRECTS = {">", ">|", "&>", ">&"}   # >> 与 &>> 是追加；>& 后面跟 fd 号或 - 时是复制、关闭 fd，不写文件
+APPENDING_REDIRECTS = {">>", "&>>"}                # 只有 ⑨（every_write）认：追加也改了那一份
 FILE_DESCRIPTOR_WORD = re.compile(r"^(?:\d+-?|-)$")
 ASSIGNED_VARIABLE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")
 # 目标词里这一刻算不出的段：变量（含 ${…} 的各种展开与特殊参数）、花括号展开；命令替换另由共用模块的 substitution_span 认
@@ -754,7 +819,23 @@ def truncate_files(arguments):
                 position += 1
     return files
 
-def follow_simple_command(words, directory, variables, steps, depth, heredoc_bodies=()):
+def in_place_editor_files(name, arguments):
+    """sed -i / perl -i 改的文件（⑨ 用）；不带 -i 的交 []。sed 沿用 ⑧ 的 sed_in_place_targets（/tmp/ 下的不交）。"""
+    if name == "sed":
+        return sed_in_place_targets(arguments)
+    if not any(re.match(r"^-[a-zA-Z]*i", argument) for argument in arguments):
+        return []
+    files, skip_next, script_given = [], False, False
+    for argument in arguments:
+        if skip_next:
+            skip_next = False
+        elif re.match(r"^-[a-zA-Z]*[eE]$", argument):
+            skip_next, script_given = True, True
+        elif not argument.startswith("-"):
+            files.append(argument)
+    return files if script_given else files[1:]
+
+def follow_simple_command(words, directory, variables, steps, depth, heredoc_bodies=(), every_write=False):
     """一条简单命令：会整份覆盖文件的写法（tee、cp、mv、install、dd、truncate；⑦ 另认喂给 python 的代码）与让旧字节留住的一步
     （git add、mv 挪走源；⑦ 另认 rm）追加进 steps；跟着 cd / pushd 换目录，跟着独立的赋值与 export 记变量（改 variables）；
     bash -c 的那段代码按这一刻的目录与变量递归进去。heredoc_bodies 是喂给这一条的、被当数据剥掉的 heredoc 正文。交回这条之后的当前目录。"""
@@ -782,7 +863,16 @@ def follow_simple_command(words, directory, variables, steps, depth, heredoc_bod
     elif name == "popd":
         return None
     elif name == "tee":
-        steps += [OverwriteCandidate("tee", file, directory) for file in tee_files(arguments)]
+        files = tee_files(arguments)
+        if every_write and not files:
+            files = [argument for argument in arguments if argument != "-" and not argument.startswith("-")]
+        steps += [OverwriteCandidate("tee", file, directory) for file in files]
+    elif every_write and name in ("sed", "perl"):
+        steps += [OverwriteCandidate(f"{name} -i", file, directory) for file in in_place_editor_files(name, arguments)]
+    elif every_write and name == "rsync":
+        operands = [argument for argument in arguments if not argument.startswith("-")]
+        if len(operands) >= 2:
+            steps.append(OverwriteCandidate("rsync", operands[-1], directory, tuple(operands[:-1]), None))
     elif name in COPY_SHORT_OPTIONS_WITH_VALUE:
         parsed = parse_copy_arguments(name, arguments)
         if parsed.creates_directories:
@@ -814,7 +904,7 @@ def follow_simple_command(words, directory, variables, steps, depth, heredoc_bod
     elif name in shell_words.SHELL_NAMES:
         code = shell_words.shell_invocation(arguments).code_string
         if code is not None:
-            steps += overwrite_steps(code, directory, variables, depth + 1)
+            steps += overwrite_steps(code, directory, variables, depth + 1, every_write)
     return directory
 
 def command_pieces(tokens):
@@ -842,10 +932,11 @@ def command_pieces(tokens):
         pieces.append(piece)
     return pieces
 
-def overwrite_steps(text, directory, variables=None, depth=0):
+def overwrite_steps(text, directory, variables=None, depth=0, every_write=False):
     """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换与进程替换）命令位置上，会整份覆盖文件的写法
     （OverwriteCandidate：重定向、tee、cp、mv、install、dd、truncate）与让旧字节留住的一步（PreservingStep），按执行的先后交回。
-    喂给 shell 的 heredoc 正文留在原位按行切（共用模块的 strip_data_heredocs 只剥喂给别的命令的）；directory 是开头的当前目录。"""
+    喂给 shell 的 heredoc 正文留在原位按行切（共用模块的 strip_data_heredocs 只剥喂给别的命令的）；directory 是开头的当前目录。
+    every_write（⑨ 用）另交追加与别的改法：>>、&>>、tee -a、sed -i、perl -i、rsync 的目标。"""
     if depth > shell_words.MAXIMUM_NESTING:
         return []
     variables = dict(variables or {})
@@ -854,16 +945,16 @@ def overwrite_steps(text, directory, variables=None, depth=0):
     for piece in command_pieces(shell_words.shell_tokens(shell_words.strip_data_heredocs(text, stripped_bodies=data_heredocs))):
         for is_operator, substitution in piece:
             if is_operator is None:
-                steps += overwrite_steps(substitution, directory, variables, depth + 1)
+                steps += overwrite_steps(substitution, directory, variables, depth + 1, every_write)
         for (is_operator, operator), (next_is_operator, target) in zip(piece, piece[1:]):
-            if (is_operator and operator in OVERWRITING_REDIRECTS and next_is_operator is False
+            if (is_operator and (operator in OVERWRITING_REDIRECTS or (every_write and operator in APPENDING_REDIRECTS)) and next_is_operator is False
                     and not (operator == ">&" and FILE_DESCRIPTOR_WORD.match(target))):
                 steps.append(OverwriteCandidate(operator, substitute_assigned_variables(target, variables), directory))
         commands, _, _, redirections = shell_words.simple_commands(piece)
         if commands and commands[0]:
             fed_heredocs = [data_heredocs[int(marker.group(1))] for operator, target in redirections[0] if operator in ("<<", "<<-")
                             for marker in [shell_words.HEREDOC_BODY_MARKER.match(target)] if marker and int(marker.group(1)) < len(data_heredocs)]
-            directory = follow_simple_command(commands[0], directory, variables, steps, depth, fed_heredocs)
+            directory = follow_simple_command(commands[0], directory, variables, steps, depth, fed_heredocs, every_write)
     return steps
 
 def target_pattern(word):
@@ -1090,15 +1181,17 @@ def mode_writes_in_place(mode):
     """'w' 截断重写、'r+' 从头改写，都在原 inode 上；'a' 追加、'x' 只建新文件，不算。"""
     return mode is not None and ("w" in mode or ("r" in mode and "+" in mode))
 
-def python_in_place_writes(code):
+def python_in_place_writes(code, every_write=False):
     """python 代码里在原 inode 上写文件的调用：open(路径, 带 w 或 r+ 的模式)、Path(路径).open(同上)、Path(路径).write_text / write_bytes、
-    shutil.copyfile / copy / copy2 的目标。路径与模式算不出的不判，解析不了的代码不判。"""
+    shutil.copyfile / copy / copy2 的目标。路径与模式算不出的不判，解析不了的代码不判。
+    every_write（⑨ 用）另交带 a、x、+ 的模式与换 inode 的写：shutil.move、os.replace、os.rename 的目标。"""
     try:
         tree = ast.parse(code)
     except (SyntaxError, ValueError):
         return []
     assignments = single_assignments(tree)
     writes = []
+    mode_writes = (lambda mode: mode is not None and any(letter in mode for letter in "wax+")) if every_write else mode_writes_in_place
     for node in ast.walk(tree):
         if not isinstance(node, ast.Call):
             continue
@@ -1107,12 +1200,12 @@ def python_in_place_writes(code):
         if name == "open" and (receiver is None or (isinstance(receiver, ast.Name) and receiver.id in PYTHON_OPEN_MODULES)):
             mode = opening_mode(call_argument(node, 1, "mode"))
             path = python_literal_path(call_argument(node, 0, "file"), assignments)
-            if path is not None and mode_writes_in_place(mode):
+            if path is not None and mode_writes(mode):
                 writes.append(PythonWrite(f"python open(…, '{mode}')", path, None))
         elif name == "open":
             mode = opening_mode(call_argument(node, 0, "mode"))
             path = python_literal_path(receiver, assignments)
-            if path is not None and mode_writes_in_place(mode):
+            if path is not None and mode_writes(mode):
                 writes.append(PythonWrite(f"python Path.open('{mode}')", path, None))
         elif name in PYTHON_PATH_WRITERS and receiver is not None:
             path = python_literal_path(receiver, assignments)
@@ -1123,6 +1216,11 @@ def python_in_place_writes(code):
             source = python_literal_path(call_argument(node, 0, "src"), assignments) if name != "copyfile" else None
             if path is not None:
                 writes.append(PythonWrite(f"python shutil.{name}", path, source))
+        elif every_write and ((name == "move" and owner == "shutil") or (name in ("replace", "rename") and owner == "os")):
+            path = python_literal_path(call_argument(node, 1, "dst"), assignments)
+            source = python_literal_path(call_argument(node, 0, "src"), assignments)
+            if path is not None:
+                writes.append(PythonWrite(f"python {owner}.{name}", path, source))
     return writes
 
 def is_existing_script(path, repository_root, scratch_root):
@@ -1807,6 +1905,45 @@ def selftest_in(hook_dir, work):
         ]
         for label, agent, command, want in repository_edit_cases:
             results.append((f"就地改仓内文件:{label}", want, 1 if repository_in_place_edit_refusal(command, agent, edit_repository) else 0))
+        # ⑨ 先编后换：experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs：(说明, agent 类型, 命令, 该不该拒)
+        harness_bin = os.path.join(edit_repository, "crates", "singlefs-harness", "src", "bin")
+        os.makedirs(harness_bin)
+        open(os.path.join(harness_bin, "e161_x.rs"), "w").write("fn main() {}\n")
+        open(os.path.join(edit_repository, "crates", "mutations.tsv"), "w").write("# 表头\n")
+        bin_path = "crates/singlefs-harness/src/bin/e161_x.rs"
+        draft_path = "/tmp/claude-1000/runner-x/e161_x.rs"
+        compile_first_cases = [
+            ("执行员 cp 草稿副本盖主工作区的入库装置", "experiment-runner", f"cp {draft_path} {bin_path}", 1),
+            ("执行员 cat > 新建入库装置", "experiment-runner", "cat > crates/singlefs-harness/src/bin/e999_new.rs <<'EOF'\nfn main() {}\nEOF", 1),
+            ("执行员 >> 追加", "experiment-runner", f"echo '// x' >> {bin_path}", 1),
+            ("执行员 tee -a", "experiment-runner", f"echo x | tee -a {bin_path}", 1),
+            ("执行员 mv 草稿副本进 bin 目录", "experiment-runner", f"mv {draft_path} crates/singlefs-harness/src/bin/", 1),
+            ("执行员 cd 进 bin 目录之后 cp 到 .", "experiment-runner", f"cd crates/singlefs-harness/src/bin && cp {draft_path} .", 1),
+            ("执行员 sed -i", "experiment-runner", f"sed -i 's/a/b/' {bin_path}", 1),
+            ("执行员 perl -pi -e", "experiment-runner", f"perl -pi -e 's/a/b/' {bin_path}", 1),
+            ("执行员 rsync", "experiment-runner", f"rsync -a {draft_path} {bin_path}", 1),
+            ("执行员 rm 入库装置", "experiment-runner", f"rm {bin_path}", 1),
+            ("执行员 mv 把入库装置挪走", "experiment-runner", f"mv {bin_path} /tmp/claude-1000/runner-x/", 1),
+            ("执行员 bash -c 里 cp", "experiment-runner", f"bash -c 'cp {draft_path} {bin_path}'", 1),
+            ("执行员变量里的目录", "experiment-runner", f"d=crates/singlefs-harness/src/bin; cp {draft_path} \"$d/e161_x.rs\"", 1),
+            ("执行员目标算不出、对得上已有的入库装置", "experiment-runner", f"cp {draft_path} \"$DIR\"/crates/singlefs-harness/src/bin/e161_x.rs", 1),
+            ("执行员 python shutil.copy", "experiment-runner",
+             f"python3 -c \"import shutil; shutil.copy('{draft_path}', '{bin_path}')\"", 1),
+            ("执行员 python open(…, 'a')", "experiment-runner", f"python3 -c \"open('{bin_path}', 'a').write('x')\"", 1),
+            ("执行员 python os.replace", "experiment-runner", f"python3 -c \"import os; os.replace('{draft_path}', '{bin_path}')\"", 1),
+            ("执行员经先编后换脚本换进来放行", "experiment-runner",
+             f"bash research/scripts/capped.sh 16 python3 {COMPILE_THEN_SWAP_SCRIPT} {draft_path} {bin_path} --scratch /tmp/claude-1000/runner-x", 0),
+            ("执行员把主工作区那份拷进草稿目录放行", "experiment-runner", f"cp {bin_path} /tmp/claude-1000/runner-x/", 0),
+            ("执行员在草稿目录里改放行", "experiment-runner", f"sed -i 's/a/b/' {draft_path}", 0),
+            ("执行员追加 crates/mutations.tsv 放行", "experiment-runner", "printf 'x\\n' >> crates/mutations.tsv", 0),
+            ("执行员 grep 读入库装置放行", "experiment-runner", f"grep -n main {bin_path}", 0),
+            ("执行员 python 只读入库装置放行", "experiment-runner", f"python3 -c \"print(open('{bin_path}').read())\"", 0),
+            ("执行员引号里当数据写的放行", "experiment-runner", f"echo 'cp {draft_path} {bin_path}'", 0),
+            ("实现员同样的 cp 放行", "implementation-writer", f"cp {draft_path} {bin_path}", 0),
+            ("主 agent 同样的 cp 放行", None, f"cp {draft_path} {bin_path}", 0),
+        ]
+        for label, agent, command, want in compile_first_cases:
+            results.append((f"先编后换:{label}", want, 1 if compile_first_refusal(command, agent, edit_repository) else 0))
     finally:
         shutil.rmtree(edit_repository, ignore_errors=True)
     # 把活放出追踪的写法：(说明, 命令, 该不该拒绝)；前台、run_in_background 一样拒，detaching_refusal 不看 run_in_background
@@ -2287,6 +2424,18 @@ def selftest_in(hook_dir, work):
     results.append(("stdin:就地改脚本拒了的不记检出", 0, in_place_recorded))
     python_in_place = through_entry(f"python3 - <<'EOF'\nopen('{running}', 'w').write('x')\nEOF", True)[0]
     results.append(("stdin:run_in_background 里喂给 python 的 open(…, 'w') 同样拒绝（退出码 2）", 2, python_in_place.returncode))
+    # 走真实入口：experiment-runner 在 Bash 里 cp 进主工作区 crates/ 下的 .rs 拒绝（退出码 2）、stderr 点名先编后换那条脚本；implementation-writer 放行
+    def entry_as(agent_type, command):
+        return subprocess.run(["bash", os.path.join(repository_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
+                              env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections),
+                              input=json.dumps({"tool_name": "Bash", "session_id": "s", "agent_type": agent_type,
+                                                "tool_input": {"command": command, "run_in_background": False}}))
+    runner_copy = entry_as("experiment-runner", "cp /tmp/claude-1000/runner-x/e161_x.rs crates/singlefs-harness/src/bin/e161_x.rs")
+    results.append(("stdin:experiment-runner cp 进主工作区的入库装置拒绝（退出码 2）", 2, runner_copy.returncode))
+    results.append(("stdin:先编后换的拒绝点名那条脚本与出路", 1,
+                    int("✗" in runner_copy.stderr and "→" in runner_copy.stderr and COMPILE_THEN_SWAP_SCRIPT in runner_copy.stderr)))
+    writer_copy = entry_as("implementation-writer", "cp /tmp/claude-1000/runner-x/e161_x.rs crates/singlefs-harness/src/bin/e161_x.rs")
+    results.append(("stdin:implementation-writer 同样的 cp 放行（退出码 0）", 0, writer_copy.returncode))
     renamed, renamed_recorded = through_entry(f"cp /tmp/new.sh research/scripts/.running.sh.installing && mv research/scripts/.running.sh.installing {running}")
     results.append(("stdin:写到临时文件再 mv 换上放行（退出码 0）、不记检出", (0, 0), (renamed.returncode, renamed_recorded)))
     # 同走 run_in_background 的那一条：只记检出
@@ -2398,7 +2547,8 @@ def selftest_in(hook_dir, work):
               "_ALLOW_FOREGROUND_WAIT_LOOP、_REFUSE_EVERY_WAIT_LOOP、_ALLOW_DETACHING、_ALLOW_UNWAITED_AMPERSAND、_ALLOW_RESULTS_OVERWRITE、_ALLOW_PROCESS_SIGNALS "
               "或 _ALLOW_SCRIPT_IN_PLACE_WRITE 设着的话这里本来就该红；"
               "「终止进程:」「扫脚本:」红的看 process_signal_refusal() / termination_findings() / call_findings() / scan_scripts()；"
-              "「就地改脚本:」红的看 script_in_place_refusal() / script_in_place_verdict() / python_in_place_writes() / is_existing_script()")
+              "「就地改脚本:」红的看 script_in_place_refusal() / script_in_place_verdict() / python_in_place_writes() / is_existing_script()；"
+              "「先编后换:」红的看 compile_first_refusal() / compile_first_verdict() / overwrite_steps(every_write=True)，_ALLOW_COMPILE_FIRST_BYPASS 设着的话本来就该红")
         return 1
     summary = ("没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 、前台的 & 、写进文件的 heredoc 正文与按模式找进程（归上游钩子）不记；"
                "起看门狗不用 run_in_background、或带 &、nohup、setsid、disown、丢进 /dev/null 的拒绝（退出码 2），run_in_background 只写 watch.sh 的、把名字当参数的、"
@@ -2422,7 +2572,10 @@ def selftest_in(hook_dir, work):
                "dd of=、truncate、python 的 open(…, 'w' / 'r+' / mode='wb')、Path.open('w')、write_text、write_bytes、shutil.copyfile / copy 写的拒绝"
                "（bash -c、heredoc、命令替换、cd 与变量跟着算，python 的只赋过一次的名字、Path / 、os.path.join 算得出），"
                ">>、&>>、tee -a、python 'a' / 'x' / 读、新文件名、临时文件再 mv、mv、install、cp --remove-destination / -l / -n / -b、sed -i、先 rm 或挪走再写、"
-               "os.replace、非脚本文件、仓外与 scratch 根外、当数据写的与算不出的放行；dd、truncate 现在 ⑤ 也认（同一段 overwrite_steps），⑤ 仍不认 python 的写法")
+               "os.replace、非脚本文件、仓外与 scratch 根外、当数据写的与算不出的放行；dd、truncate 现在 ⑤ 也认（同一段 overwrite_steps），⑤ 仍不认 python 的写法；"
+               "先编后换：experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs（cp、cat >、>>、tee -a、mv 进目录、cd 之后 cp 到 .、sed -i、perl -pi、rsync、rm、mv 挪走、"
+               "bash -c、变量里的目录、算不出而对得上已有装置的、python shutil.copy / open 'a' / os.replace）拒绝并点名 compile-then-swap.py，"
+               "经那条脚本换进来、拷进草稿目录、改草稿目录、追加 crates/mutations.tsv、只读、当数据写的与 implementation-writer、主 agent 放行")
     print(f"  ✓ 自检通过（查了 {len(results)} 种）：{summary}")
     return 0
 
@@ -2547,6 +2700,19 @@ def main():
                   "或者用 `research/scripts/replace-once.py` / `insert-row.py` 定点改。追加（`>>`、`tee -a`）与新建不拦；"
                   "Edit / Write 工具本来就换 inode。这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
             return 2
+        try:
+            crate_writes = compile_first_refusal(tool_input.get("command") or "", hook_input.get("agent_type"), repository_root_of(hook_dir))
+        except Exception as error:
+            print(f"  ! bash-command-detector.sh 没判成绕过先编后换的写法（{error!r}），这条命令照常执行", file=sys.stderr)
+            crate_writes = []
+        if crate_writes:
+            print(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里写主工作区 crates/ 下的 .rs（认出的写法与目标：{'；'.join(crate_writes)}）："
+                  "入库装置先在草稿目录的副本里改，编过再整份换进主工作区，主工作区里任何时候都只放编得过的版本", file=sys.stderr)
+            print(f"     → 怎么办：在草稿目录那一份上改完，跑 `bash research/scripts/capped.sh <线程上限> python3 {COMPILE_THEN_SWAP_SCRIPT} "
+                  "<草稿副本> crates/<crate>/src/bin/<名>.rs --scratch <草稿目录>`，它在仓副本里编过才整份换进来，编不过一个字节不写；"
+                  "把主工作区那份拷进草稿目录（`cp crates/… <草稿目录>/`）不拦。目标不是 crates/<crate>/src/bin/ 下的装置的，不归你改，"
+                  "写进报告交回主 agent。这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
+            return 2
         try:
             repository_edits = repository_in_place_edit_refusal(tool_input.get("command") or "", hook_input.get("agent_type"), repository_root_of(hook_dir))
         except Exception as error:
diff --git a/.claude/hooks/write-guard.sh b/.claude/hooks/write-guard.sh
index ae82cb9a..881a4576 100755
--- a/.claude/hooks/write-guard.sh
+++ b/.claude/hooks/write-guard.sh
@@ -1,7 +1,7 @@
 #!/usr/bin/env bash
 # admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
 # run-condition: command python3
-# PreToolUse hook（Write、Edit）：写文件之前的三道判定，合在一个 hook 里；拒绝的同时把它记进检出记录，交主 agent 看。
+# PreToolUse hook（Write、Edit）：写文件之前的四道判定，合在一个 hook 里；拒绝的同时把它记进检出记录，交主 agent 看。
 #
 # 一、整份覆盖未跟踪文件（只管 Write）：仓里已存在、又没进 git 的文件，拒绝用 Write 整份覆盖。
 #    为什么：几个会话同时在一个仓里干活。2026-09-11 一个会话用 Write 新建 `research/prompts/e137-preregistration.md`，
@@ -15,6 +15,11 @@
 #    出现撇号类字符之一就拒绝；只在 old_string 里出现（删掉它的改动）放行，ASCII 单引号不判。主 agent 与所有子 agent 一样判，目标在不在仓里都判。
 #    为什么：门禁 12 号只在收尾跑整轮门禁时判，2026-09-24 实验设计员照样把这类名字写进三份重跑登记、执行员又抄进源码。
 #    字符集与 12 号同一份：../gate.d/lib-prime-marks.py，不在这里另抄；读不到就拒绝（不静默放行）。
+# 四、先编后换（Write、Edit）：experiment-runner（名单是 COMPILE_FIRST_AGENTS）写主工作区 crates/ 下的 .rs（改已有的、新建的都算）一律拒绝，
+#    出路是在草稿目录的副本里改、经 research/scripts/compile-then-swap.py 编过再整份换进来。排在写范围那一道之前判，拒绝信息给的是这条出路。
+#    implementation-writer、主 agent、别的 agent 不判；草稿目录里的副本与 crates/ 下不是 .rs 的（crates/mutations.tsv）不判。
+#    为什么：执行员直接在主工作区改入库装置，改到一半编不过，别的会话带 --all-targets 的编译一起卡住
+#    （records/2026-09-16-subagent拆分提案.md 第四十节那张表第 51 行）。Bash 里的同一类写法归 bash-command-detector.sh 的 ⑨。
 #
 # 三道原本是两个 hook（refuse-overwrite-untracked.sh、agent-write-scope.sh），2026-09-17 用户定合并；第三道 2026-09-24 加进来。
 # 它们拒的是一次写，不停任务和脚本；拒绝时退出码 2、stderr 交给做这次写的模型，同时往检出记录
@@ -24,7 +29,7 @@
 # 2026-09-24 写这一道时实测，Write 的 content 里写的反斜杠 u 转义落盘成了字符本身，这道闸也会拒这样的写。
 #
 #   write-guard.sh             # 从 stdin 读 hook 的 JSON
-#   write-guard.sh --selftest  # 走一遍三道判定的放行与拒绝；WRITE_GUARD_DISABLE_OVERWRITE=1 或 WRITE_GUARD_DISABLE_SCOPE=1 时自检必须判红
+#   write-guard.sh --selftest  # 走一遍四道判定的放行与拒绝；WRITE_GUARD_DISABLE_OVERWRITE=1、WRITE_GUARD_DISABLE_COMPILE_FIRST=1 或 WRITE_GUARD_DISABLE_SCOPE=1 时自检必须判红
 set -uo pipefail
 source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
 preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
@@ -107,6 +112,28 @@ def decide_scope(hook_input, project_root, table_path):
                f"→ 它的写范围：{'、'.join(patterns)}（.claude/hooks/agent-write-scope.tsv）。\n"
                f"→ 怎么办：范围外的改动不自己做，写进报告交回主 agent，由主 agent 改或派该写的 agent。")
 
+# 四、先编后换：这几个项目 subagent 不直接写主工作区 crates/ 下的 .rs，出路是 research/scripts/compile-then-swap.py。
+# Bash 里的同一类写法由 bash-command-detector.sh 的 ⑨ 判，agent 名单与路径判据两边各写一份、改的时候一起改。
+COMPILE_FIRST_AGENTS = ("experiment-runner",)
+COMPILE_THEN_SWAP_SCRIPT = "research/scripts/compile-then-swap.py"
+
+def decide_compile_first(hook_input, project_root):
+    """四、登记在 COMPILE_FIRST_AGENTS 里的 agent 写主工作区 crates/ 下的 .rs。"""
+    agent_type = hook_input.get("agent_type")
+    if agent_type not in COMPILE_FIRST_AGENTS or os.environ.get("WRITE_GUARD_DISABLE_COMPILE_FIRST") == "1":
+        return 0, None
+    absolute = absolute_target(hook_input, project_root)
+    root = os.path.normpath(project_root)
+    if not absolute or not absolute.endswith(".rs") or not absolute.startswith(os.path.join(root, "crates") + os.sep):
+        return 0, None
+    relative = os.path.relpath(absolute, root)
+    return 2, (f"✗ {agent_type} 不直接写主工作区的 {relative}：入库装置先在草稿目录的副本里改，编过再整份换进主工作区，"
+               "主工作区里任何时候都只放编得过的版本。\n"
+               f"→ 怎么办：cp {relative} <草稿目录>/ 拷一份（新建的装置直接在草稿目录里建），在草稿目录那一份上用 Edit / Write 改；"
+               f"改完跑 bash research/scripts/capped.sh <线程上限> python3 {COMPILE_THEN_SWAP_SCRIPT} <草稿目录>/{os.path.basename(relative)} "
+               f"{relative} --scratch <草稿目录>，它编过才整份换进来，编不过一个字节不写。\n"
+               "→ 目标不是 crates/<crate>/src/bin/ 下的装置（crate 的库、测试）的，不归你改，写进报告交回主 agent。")
+
 def prime_marks_library_path(hook_dir):
     return os.path.join(os.path.dirname(hook_dir), "gate.d", "lib-prime-marks.py")
 
@@ -165,6 +192,9 @@ def decide(hook_input, project_root, table_path, prime_marks_library):
     code, message = decide_overwrite(hook_input, project_root)
     if code:
         return code, message, "整份覆盖未跟踪文件"
+    code, message = decide_compile_first(hook_input, project_root)
+    if code:
+        return code, message, "绕过先编后换"
     code, message = decide_scope(hook_input, project_root, table_path)
     if code:
         return code, message, "越出写范围"
@@ -237,6 +267,11 @@ def selftest(hook_dir):
             if agent:
                 hook_input["agent_type"] = agent
             return (label, want, decide(hook_input, work, table, library)[0])
+        def finding_case(label, tool_name, agent, path, want):
+            hook_input = {"tool_name": tool_name, "tool_input": {"file_path": path}}
+            if agent:
+                hook_input["agent_type"] = agent
+            return (label, want, decide(hook_input, work, table, library)[2])
         def prime_case(label, tool_name, agent, tool_input, want, library_path=library):
             hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=f"{work}/crates/a.rs")}
             if agent:
@@ -266,6 +301,22 @@ def selftest(hook_dir):
             case("范围:experiment-runner mutate.sh", "Edit", "experiment-runner", f"{work}/research/scripts/mutate.sh", 2),
             case("范围:未登记的项目 agent", "Edit", "unscoped-writer", f"{work}/anything.md", 2),
             case("两道都中时先报覆盖", "Write", "kb-scribe", f"{work}/untracked.md", 2),
+            # 四、实验执行员对主工作区 crates/ 下 .rs 的写：拒的那一道要是「绕过先编后换」，不是写范围那一道（出路不同）
+            finding_case("先编后换:experiment-runner Edit 主工作区的入库装置", "Edit", "experiment-runner",
+                         f"{work}/crates/singlefs-harness/src/bin/e161_x.rs", "绕过先编后换"),
+            finding_case("先编后换:experiment-runner Write 新建入库装置", "Write", "experiment-runner",
+                         f"{work}/crates/singlefs-harness/src/bin/e999_new.rs", "绕过先编后换"),
+            finding_case("先编后换:experiment-runner 改 crates 里 bin 以外的 .rs", "Edit", "experiment-runner",
+                         f"{work}/crates/singlefs-core/src/lib.rs", "绕过先编后换"),
+            finding_case("先编后换:.. 绕路进 crates 也算", "Edit", "experiment-runner",
+                         f"{work}/research/../crates/singlefs-harness/src/bin/e161_x.rs", "绕过先编后换"),
+            finding_case("先编后换:implementation-writer 同一处放行", "Edit", "implementation-writer",
+                         f"{work}/crates/singlefs-harness/src/bin/e161_x.rs", None),
+            finding_case("先编后换:主 agent 同一处放行", "Edit", None, f"{work}/crates/singlefs-harness/src/bin/e161_x.rs", None),
+            finding_case("先编后换:experiment-runner 改草稿目录里的副本放行", "Edit", "experiment-runner",
+                         "/tmp/claude-1000/runner-x/e161_x.rs", None),
+            finding_case("先编后换:experiment-runner 追加 crates/mutations.tsv 放行", "Edit", "experiment-runner",
+                         f"{work}/crates/mutations.tsv", None),
             # 三、撇号类角标：五个字符各写死一例（共用字符集少了哪一个，这里就红）
             prime_case("角标:Write 内容含 B2 加一撇（U+2032）", "Write", None, {"content": "取 B2\u2032 的读法\n"}, 2),
             prime_case("角标:Edit new_string 含 P1 加两撇（U+2033）", "Edit", None, {"old_string": "P1", "new_string": "P1\u2033"}, 2),
@@ -298,12 +349,17 @@ def selftest(hook_dir):
             stdin_case("stdin:kb-scribe 范围内", {"tool_name": "Edit", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{work}/.claude/kb/x.md"}}, 0),
             stdin_case("stdin:大内容越界", {"tool_name": "Write", "agent_type": "implementation-writer", "tool_input": {"file_path": f"{work}/research/big.md", "content": "y" * 300000}}, 2),
         ]
+        runner_entry = run_entry({"tool_name": "Edit", "agent_type": "experiment-runner",
+                                  "tool_input": {"file_path": f"{work}/crates/singlefs-harness/src/bin/e161_x.rs", "old_string": "a", "new_string": "b"}})
+        cases.append(("stdin:experiment-runner 改主工作区的入库装置拒绝", 2, runner_entry.returncode))
+        cases.append(("stdin:先编后换的拒绝点名那条脚本与草稿目录", True,
+                      all(fragment in runner_entry.stderr for fragment in ("research/scripts/compile-then-swap.py", "草稿目录", "→ 怎么办"))))
         prime_entry = run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/crates/a.rs", "old_string": "B2", "new_string": "取 B2\u2032 的读法"}})
         cases.append(("stdin:Edit 写进角标", 2, prime_entry.returncode))
         cases.append(("stdin:角标的拒绝带字符、前后几个字与出路", True,
                       all(fragment in prime_entry.stderr for fragment in ("撇号类角标 \u2032", "取 B2\u2032 的读法", "→ 怎么办", "path-moves.md"))))
         recorded = sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
-        cases.append(("stdin:四次拒绝都落进检出记录", 4, recorded))
+        cases.append(("stdin:五次拒绝都落进检出记录", 5, recorded))
         os.unlink(outside.name)
     finally:
         shutil.rmtree(work)
@@ -311,10 +367,12 @@ def selftest(hook_dir):
     for label, want, got in failures:
         print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
     if failures:
-        print("    → 看 decide_overwrite() / decide_scope() / decide_prime_marks() 与入口；WRITE_GUARD_DISABLE_OVERWRITE / WRITE_GUARD_DISABLE_SCOPE 设着的话这里本来就该红")
+        print("    → 看 decide_overwrite() / decide_compile_first() / decide_scope() / decide_prime_marks() 与入口；"
+              "WRITE_GUARD_DISABLE_OVERWRITE / WRITE_GUARD_DISABLE_COMPILE_FIRST / WRITE_GUARD_DISABLE_SCOPE 设着的话这里本来就该红")
         return 1
     print(f"  ✓ 自检通过（查了 {len(cases)} 种情形）：未跟踪的已有文件整份覆盖拒绝，已跟踪 / 不存在 / 仓外 / Edit 放行；主 agent 与内置 agent 放行、范围内放行、"
-          "范围外与 .. 绕路与未登记的项目 agent 拒绝；Write 内容、Edit 与 MultiEdit 的 new_string 里有撇号类角标（五个字符各一例，主 agent 与子 agent 一样）拒绝，"
+          "范围外与 .. 绕路与未登记的项目 agent 拒绝；experiment-runner 写主工作区 crates/ 下的 .rs（改、新建、.. 绕路）按先编后换拒绝并点名 compile-then-swap.py，"
+          "implementation-writer 与主 agent 写同一处、执行员写草稿目录与 crates/mutations.tsv 放行；Write 内容、Edit 与 MultiEdit 的 new_string 里有撇号类角标（五个字符各一例，主 agent 与子 agent 一样）拒绝，"
           "只在 old_string 里有、ASCII 单引号放行，共用字符集读不到拒绝；拒绝都记进检出记录")
     return 0
 
```

## 二、新文件 `research/scripts/compile-then-swap.py` 全文（未跟踪，`git status --porcelain` 为 `?? research/scripts/compile-then-swap.py`；412 行）

```python
#!/usr/bin/env python3
# admission: always 每一次换装置都要现编现判，判的是这一刻的草稿副本与主工作区，上一次编过不替这一次作保
# run-condition: command cargo nice
"""先编后换：草稿目录里改好的入库装置（crates/<crate>/src/bin/ 下的 .rs）先在仓副本里编过，再整份换进主工作区。

用法：
    compile-then-swap.py <草稿副本> <目标> --scratch <草稿目录> [--root 仓根] [--release]
    compile-then-swap.py --clean --scratch <草稿目录>     # 删掉这条脚本在草稿目录里建的仓副本与编译目录
    compile-then-swap.py --selftest                        # COMPILE_THEN_SWAP_BREAK=<项> 时必须判红

<目标> 写相对仓根的路径（crates/singlefs-harness/src/bin/e<号>_<英文名>.rs）或绝对路径；只收 crates/<crate>/src/bin/ 下的
<名>.rs 与 <名>/main.rs。<草稿副本> 与 <草稿目录> 都要在仓外。

做法：
  1. 把主工作区此刻的 Cargo.toml、Cargo.lock、.cargo/ 与 crates/（不带 target/）拷进 <草稿目录>/compile-then-swap/tree，
     把草稿副本放到目标在副本里的位置；主工作区一个字节不碰。
  2. 在副本里 `nice -n 19 cargo build --offline -p <crate> --bin <名>` 与 `cargo test --offline --no-run -p <crate> --bin <名>`
     （后一条编 bin 里的 #[cfg(test)]），CARGO_TARGET_DIR 放 <草稿目录>/compile-then-swap/target。线程数照环境里的
     CARGO_BUILD_JOBS（经 research/scripts/capped.sh N 起时就是 N）。编译输出落 <草稿目录>/compile-then-swap/build-*.log。
  3. 两条都编过，再核主工作区那份在编的这段时间里没被改过（开跑时记的字节与此刻相同，或开跑时与此刻都不存在）。
  4. 换上：目标已有的，同目录临时文件写完、fsync、照原权限位改名盖上（research/scripts/lib_atomic_replace.py）；目标还没有的，
     同目录临时文件写完再硬链接到目标名（目标名这时被别人占了就失败）、删临时文件。换上之后回读，与草稿副本逐字节相同。
  5. 删掉仓副本，编译目录留着给下一次增量编；收工前 --clean。
编不过、目标被改过、参数不对，都在换上之前停下：主工作区那份一个字节不动。

退出码：0 换上了；1 编不过（一个字节没写）；2 参数或路径不对（没编、一个字节没写）；3 编的时候主工作区那份被改了（一个字节没写）；
4 换上时失败或回读对不上（看提示行）。

弄坏开关（只给自证判红用，生效时往 stderr 打一行）：COMPILE_THEN_SWAP_BREAK=
  swap-on-failure   编不过也换上
  skip-race-check   不核主工作区那份在编的时候改没改
  in-place-write    不改名换上，直接在目标原来的 inode 上截断重写
不认识的值直接拒绝，免得拼错的开关被当成「没开」。
"""
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402

BREAK_VARIABLE = "COMPILE_THEN_SWAP_BREAK"
KNOWN_BREAK_MODES = ("swap-on-failure", "skip-race-check", "in-place-write")
WORK_DIRECTORY_NAME = "compile-then-swap"
COPIED_ROOT_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain", "rust-toolchain.toml", "crates")
ERROR_LINES_SHOWN = 40
EXIT_SWAPPED, EXIT_BUILD_FAILED, EXIT_BAD_INPUT, EXIT_CHANGED_DURING_BUILD, EXIT_SWAP_FAILED = 0, 1, 2, 3, 4


class BadInput(Exception):
    """参数或路径不对：str 是两行，第二行以「→」开头写下一步。"""


def break_mode():
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAK_MODES:
        raise BadInput(f"{BREAK_VARIABLE}={mode} 不认识，没编、一个字节没写\n"
                       f"→ 怎么办：弄坏开关只认 {'、'.join(KNOWN_BREAK_MODES)}；平时不设这个变量")
    if mode:
        print(f"  ! {BREAK_VARIABLE}={mode} 生效：故意走坏的写法，只给自证判红用", file=sys.stderr)
    return mode


def is_inside(path, directory):
    return os.path.realpath(path).startswith(os.path.realpath(directory) + os.sep)


def read_bytes_or_none(path):
    try:
        with open(path, "rb") as handle:
            return handle.read()
    except FileNotFoundError:
        return None


def sha256_of(data):
    return "不存在" if data is None else hashlib.sha256(data).hexdigest()


def locate_bin(target, root):
    """目标 → (crate 目录, 包名, bin 名)；不是 crates/<crate>/src/bin/ 下的装置就抛 BadInput。"""
    crates_directory = os.path.join(root, "crates")
    if not target.endswith(".rs") or not is_inside(os.path.dirname(target), crates_directory):
        raise BadInput(f"目标 {target} 不是主工作区 crates/ 下的 .rs，这条脚本不换它\n"
                       "→ 怎么办：目标写 crates/<crate>/src/bin/<名>.rs（相对仓根）；crates/ 以外的文件照原来的办法改")
    crate_directory = os.path.dirname(target)
    while not os.path.isfile(os.path.join(crate_directory, "Cargo.toml")):
        crate_directory = os.path.dirname(crate_directory)
        if not is_inside(crate_directory, crates_directory):
            raise BadInput(f"目标 {target} 往上找不到 crate 的 Cargo.toml\n"
                           "→ 怎么办：目标要在 crates/<crate>/ 里面；先 ls crates/ 核 crate 名")
    try:
        with open(os.path.join(crate_directory, "Cargo.toml"), "rb") as handle:
            manifest = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise BadInput(f"读不了 {crate_directory}/Cargo.toml（{error}）\n"
                       "→ 怎么办：主工作区的 Cargo.toml 此刻坏着，交主 agent；修好之前不换") from error
    package = (manifest.get("package") or {}).get("name")
    relative = os.path.relpath(target, crate_directory)
    declared = [entry.get("name") for entry in manifest.get("bin") or [] if os.path.normpath(entry.get("path", "")) == relative]
    parts = relative.split(os.sep)
    if declared and declared[0]:
        bin_name = declared[0]
    elif len(parts) == 3 and parts[:2] == ["src", "bin"]:
        bin_name = parts[2][:-len(".rs")]
    elif len(parts) == 4 and parts[:2] == ["src", "bin"] and parts[3] == "main.rs":
        bin_name = parts[2]
    else:
        raise BadInput(f"目标 {target} 不是 {os.path.relpath(crate_directory, root)}/src/bin/ 下的装置（<名>.rs 或 <名>/main.rs）\n"
                       "→ 怎么办：这条脚本只换入库装置；crate 里别的源文件归 implementation-writer，写进报告交主 agent")
    if not package:
        raise BadInput(f"{crate_directory}/Cargo.toml 里没有 [package] name\n"
                       "→ 怎么办：这个目录不是一个包；先 ls crates/ 核目标路径")
    return crate_directory, package, bin_name


def copy_workspace(root, tree):
    if os.path.lexists(tree):
        shutil.rmtree(tree)
    os.makedirs(tree)
    for entry in COPIED_ROOT_ENTRIES:
        source = os.path.join(root, entry)
        if os.path.isdir(source):
            shutil.copytree(source, os.path.join(tree, entry), symlinks=True, ignore=shutil.ignore_patterns("target", ".git"))
        elif os.path.isfile(source):
            shutil.copy2(source, os.path.join(tree, entry))


def build(tree, target_directory, package, bin_name, release, log_path):
    """两条 cargo 依次跑；交回 (都编过没有, 失败那条的错误行)。"""
    environment = dict(os.environ, CARGO_TARGET_DIR=target_directory)
    profile = ["--release"] if release else []
    commands = [["nice", "-n", "19", "cargo", "build", "--offline", "-p", package, "--bin", bin_name, *profile],
                ["nice", "-n", "19", "cargo", "test", "--offline", "--no-run", "-p", package, "--bin", bin_name, *profile]]
    with open(log_path, "a", encoding="utf-8") as log:
        for command in commands:
            log.write(f"$ {' '.join(command)}\n")
            log.flush()
            completed = subprocess.run(command, cwd=tree, env=environment, stdin=subprocess.DEVNULL,
                                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace")
            log.write(completed.stdout)
            log.write(f"退出码 {completed.returncode}\n")
            if completed.returncode != 0:
                lines = completed.stdout.splitlines()
                shown = [line for index, line in enumerate(lines)
                         if line.startswith("error") or (line.lstrip().startswith("-->") and index > 0)]
                return False, [f"$ {' '.join(command[3:])}（退出码 {completed.returncode}）", *shown[:ERROR_LINES_SHOWN]]
    return True, []


def create_new_file(target, data):
    """目标还不存在：同目录临时文件写完、fsync，再硬链接到目标名（被占了就失败），删临时文件。"""
    directory = os.path.dirname(target)
    descriptor, temporary = tempfile.mkstemp(prefix="." + os.path.basename(target) + ".", suffix=".swapping", dir=directory)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fchmod(handle.fileno(), 0o644)
            os.fsync(handle.fileno())
        os.link(temporary, target)
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def swap(draft, target, scratch, root, release=False, after_build=None):
    """交回 (退出码, 提示行)。after_build 只给自证用：编完、核改没改之前调一次。"""
    mode = break_mode()
    root = os.path.realpath(root)
    draft = os.path.abspath(draft)
    target = os.path.normpath(target if os.path.isabs(target) else os.path.join(root, target))
    scratch = os.path.abspath(scratch)
    if not os.path.isfile(draft):
        raise BadInput(f"草稿副本 {draft} 不存在或不是文件\n"
                       "→ 怎么办：先把主工作区那份拷进草稿目录（cp crates/…/<名>.rs <草稿目录>/），在那一份上改，再把它的路径交给这条脚本")
    for label, path in (("草稿副本", draft), ("草稿目录", scratch)):
        if os.path.realpath(path) == root or is_inside(path, root):
            raise BadInput(f"{label} {path} 在主工作区里\n"
                           "→ 怎么办：放进派发提示给的草稿目录（/tmp/claude-1000/<轮>/ 这类仓外位置）")
    crate_directory, package, bin_name = locate_bin(target, root)
    try:
        draft_bytes = open(draft, "rb").read()
        draft_bytes.decode("utf-8")
    except UnicodeDecodeError as error:
        raise BadInput(f"草稿副本 {draft} 不是 UTF-8 文本（{error}）\n"
                       "→ 怎么办：Rust 源文件必须是 UTF-8；核一下给的是不是那份 .rs") from error
    before = read_bytes_or_none(target)
    work = os.path.join(scratch, WORK_DIRECTORY_NAME)
    tree = os.path.join(work, "tree")
    target_directory = os.path.join(work, "target")
    os.makedirs(work, exist_ok=True)
    log_path = os.path.join(work, f"build-{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-{os.getpid()}.log")
    lines = [f"目标 {os.path.relpath(target, root)}（包 {package}，bin {bin_name}）；草稿副本 {draft}",
             f"主工作区那份开跑时 sha256 {sha256_of(before)}；草稿副本 sha256 {sha256_of(draft_bytes)}"]
    try:
        copy_workspace(root, tree)
        tree_target = os.path.join(tree, os.path.relpath(target, root))
        os.makedirs(os.path.dirname(tree_target), exist_ok=True)
        with open(tree_target, "wb") as handle:
            handle.write(draft_bytes)
        built, errors = build(tree, target_directory, package, bin_name, release, log_path)
    finally:
        shutil.rmtree(tree, ignore_errors=True)
    lines.append(f"编译输出：{log_path}；编译目录 {target_directory} 留着给下一次增量编，收工前 --clean --scratch {scratch}")
    if not built:
        lines += [f"✗ 草稿副本在仓副本里编不过，主工作区 {os.path.relpath(target, root)} 一个字节没写：", *errors,  # gate-lint:summary
                  f"→ 怎么办：照上面的错误改草稿副本 {draft}，再跑这条脚本；错误落在草稿副本以外的文件（别的会话在改的），"
                  "把错误行写进报告交主 agent，不自己去改那些文件"]
        if mode != "swap-on-failure":
            return EXIT_BUILD_FAILED, lines
    if after_build is not None:
        after_build()
    now = read_bytes_or_none(target)
    if now != before and mode != "skip-race-check":
        lines += [f"✗ 编的这段时间里主工作区 {os.path.relpath(target, root)} 被改了（开跑时 sha256 {sha256_of(before)}，此刻 {sha256_of(now)}），"
                  "一个字节没写",
                  "→ 怎么办：先看是谁改的（git diff 那一份、问主 agent）；把那一版的改动并进草稿副本之后再跑这条脚本，不盖掉别人的改动"]
        return EXIT_CHANGED_DURING_BUILD, lines
    try:
        if mode == "in-place-write":
            with open(target, "wb") as handle:
                handle.write(draft_bytes)
        elif now is None:
            create_new_file(target, draft_bytes)
        else:
            def unchanged_since_check():
                if read_bytes_or_none(target) != now:
                    raise ReplaceRefused(f"{target} 在换上前一刻又被改了\n→ 怎么办：重跑这条脚本")
            lines += replace_file_contents_by_rename(target, draft_bytes.decode("utf-8"), check_before_rename=unchanged_since_check)
    except (OSError, ReplaceRefused) as error:
        lines += [f"✗ 换上 {os.path.relpath(target, root)} 时失败：{error}",
                  "→ 怎么办：核主工作区那份此刻的字节（sha256sum）是不是还是开跑时那份；是就查目录权限与剩余空间后重跑，不是就交主 agent"]
        return EXIT_SWAP_FAILED, lines
    after = read_bytes_or_none(target)
    if after != draft_bytes:
        lines += [f"✗ 换上之后回读 {os.path.relpath(target, root)}，sha256 {sha256_of(after)} 与草稿副本不同",
                  "→ 怎么办：有别的进程同时在写这一份；把现状写进报告交主 agent，不再重跑"]
        return EXIT_SWAP_FAILED, lines
    lines.append(f"✓ 编过（cargo build 与 cargo test --no-run，-p {package} --bin {bin_name}），已整份换进主工作区 "
                 f"{os.path.relpath(target, root)}，回读 sha256 {sha256_of(after)}")
    return EXIT_SWAPPED, lines


def clean(scratch):
    work = os.path.join(os.path.abspath(scratch), WORK_DIRECTORY_NAME)
    if not os.path.isdir(work):
        return [f"  ! {work} 不存在，没有要删的"]
    shutil.rmtree(work)
    return [f"  ✓ 删了 {work}（仓副本、编译目录与编译输出）"]


SELFTEST_MANIFEST = '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n'
GOOD_BIN = "fn main() {\n    println!(\"{}\", demo::value());\n}\n"
GOOD_BIN_CHANGED = "fn main() {\n    println!(\"{}\", demo::value() + 1);\n}\n"
BAD_BIN = "fn main() {\n    println!(\"{}\", unit_check_fields);\n}\n"
BAD_TEST_MODULE = GOOD_BIN + "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        missing_helper();\n    }\n}\n"


def selftest():
    work = tempfile.mkdtemp(prefix="compile-then-swap-selftest-")
    failures, checked = [], 0
    try:
        root = os.path.join(work, "repo")
        bin_directory = os.path.join(root, "crates", "demo", "src", "bin")
        os.makedirs(bin_directory)
        with open(os.path.join(root, "Cargo.toml"), "w") as handle:
            handle.write('[workspace]\nresolver = "2"\nmembers = ["crates/demo"]\n')
        with open(os.path.join(root, "crates", "demo", "Cargo.toml"), "w") as handle:
            handle.write(SELFTEST_MANIFEST)
        with open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w") as handle:
            handle.write("pub fn value() -> u32 {\n    1\n}\n")
        target = os.path.join(bin_directory, "e1_demo.rs")
        with open(target, "w") as handle:
            handle.write(GOOD_BIN)
        scratch = os.path.join(work, "scratch")
        os.makedirs(scratch)

        def draft(name, text):
            path = os.path.join(scratch, name)
            with open(path, "w") as handle:
                handle.write(text)
            return path

        def run(label, draft_path, target_path, want_code, want_bytes, after_build=None, want_new_inode=False):
            nonlocal checked
            checked += 1
            inode_before = os.stat(target_path).st_ino if os.path.exists(target_path) else None
            try:
                code, lines = swap(draft_path, target_path, scratch, root, after_build=after_build)
            except BadInput as error:
                code, lines = EXIT_BAD_INPUT, str(error).splitlines()
            got_bytes = read_bytes_or_none(target_path)
            leftovers = [name for name in os.listdir(os.path.dirname(target_path)) if name.startswith(".")] \
                if os.path.isdir(os.path.dirname(target_path)) else []
            problems = []
            if code != want_code:
                problems.append(f"退出码 {code}（应当 {want_code}）")
            if got_bytes != want_bytes:
                problems.append(f"目标 sha256 {sha256_of(got_bytes)}（应当 {sha256_of(want_bytes)}）")
            if leftovers:
                problems.append(f"目标目录里留下临时文件 {leftovers}")
            if want_new_inode and inode_before is not None and got_bytes is not None and os.stat(target_path).st_ino == inode_before:
                problems.append("换上之后 inode 没变：在原 inode 上写的，正在读它的进程会读到半份")
            if code not in (EXIT_SWAPPED, EXIT_BAD_INPUT) and not any(line.lstrip().startswith("→") for line in lines):
                problems.append("拒绝没带「→」出路")
            if problems:
                failures.append(f"{label}：{'；'.join(problems)}；输出 {lines[-3:]}")

        good = GOOD_BIN.encode()
        run("编不过的草稿（E0425 那一类）不写主工作区", draft("bad.rs", BAD_BIN), target, EXIT_BUILD_FAILED, good)
        run("bin 编过而 #[cfg(test)] 编不过的也不写", draft("bad-test.rs", BAD_TEST_MODULE), target, EXIT_BUILD_FAILED, good)
        new_target = os.path.join(bin_directory, "e2_demo.rs")
        run("新 bin 编不过就不建", draft("bad-new.rs", BAD_BIN), new_target, EXIT_BUILD_FAILED, None)

        def someone_edits():
            with open(target, "w") as handle:
                handle.write(GOOD_BIN + "// 别的会话的改动\n")
        run("编的时候主工作区那份被改了就不盖", draft("good-race.rs", GOOD_BIN_CHANGED), target, EXIT_CHANGED_DURING_BUILD,
            (GOOD_BIN + "// 别的会话的改动\n").encode(), after_build=someone_edits)
        with open(target, "w") as handle:
            handle.write(GOOD_BIN)
        run("编过的草稿整份换上、换的是新 inode", draft("good.rs", GOOD_BIN_CHANGED), target, EXIT_SWAPPED, GOOD_BIN_CHANGED.encode(),
            want_new_inode=True)
        run("编过的新 bin 建出来", draft("good-new.rs", GOOD_BIN), new_target, EXIT_SWAPPED, good)
        library = os.path.join(root, "crates", "demo", "src", "lib.rs")
        run("crate 里 bin 以外的源文件不收", draft("lib.rs", "pub fn value() -> u32 { 2 }\n"), library, EXIT_BAD_INPUT,
            b"pub fn value() -> u32 {\n    1\n}\n")
        inside = os.path.join(root, "draft-inside.rs")
        with open(inside, "w") as handle:
            handle.write(GOOD_BIN)
        run("草稿副本放在主工作区里不收", inside, target, EXIT_BAD_INPUT, GOOD_BIN_CHANGED.encode())
        outside_crates = os.path.join(root, "tools", "x.rs")
        run("crates/ 以外的 .rs 不收", draft("x.rs", GOOD_BIN), outside_crates, EXIT_BAD_INPUT, None)
        checked += 1
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME, "tree")):
            failures.append("跑完之后仓副本还在草稿目录里：交回闸会拦")
        checked += 1
        clean(scratch)
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME)):
            failures.append("--clean 之后编译目录还在")
    except BadInput as error:
        failures.append(f"自检现场本身被拒：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 swap() / build() / locate_bin() 的判法；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ compile-then-swap 自检通过（查了 {checked} 种）：编不过的（bin 本身、#[cfg(test)]、新 bin）一个字节不写，"
          "编的时候主工作区那份被改了不盖，编过的整份换上且换的是新 inode、新 bin 建得出来，bin 以外的源文件、crates/ 以外的 .rs、"
          "放在主工作区里的草稿都不收，仓副本跑完就删、--clean 删得掉编译目录")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    usage = ("compile-then-swap.py <草稿副本> <目标> --scratch <草稿目录> [--root 仓根] [--release]，"
             "或 compile-then-swap.py --clean --scratch <草稿目录>")
    options = {"--scratch": None, "--root": REPOSITORY_ROOT}
    flags = {"--release": False, "--clean": False}
    positional = []
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in options and position + 1 < len(arguments):
            options[argument] = arguments[position + 1]
            position += 2
            continue
        if argument in flags:
            flags[argument] = True
        else:
            positional.append(argument)
        position += 1
    if options["--scratch"] is None:
        print("✗ 没给 --scratch <草稿目录>")
        print(f"→ 怎么办：{usage}")
        return EXIT_BAD_INPUT
    if flags["--clean"]:
        print("\n".join(clean(options["--scratch"])))
        return 0
    if len(positional) != 2:
        print(f"✗ 要给草稿副本与目标两个路径，给了 {len(positional)} 个")
        print(f"→ 怎么办：{usage}")
        return EXIT_BAD_INPUT
    try:
        code, lines = swap(positional[0], positional[1], options["--scratch"], options["--root"], release=flags["--release"])
    except BadInput as error:
        problem, _, next_step = str(error).partition("\n")
        print(f"✗ {problem}")
        print(f"→ {next_step.lstrip('→ ') or usage}")
        return EXIT_BAD_INPUT
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
```

## 三、`.claude/agents/experiment-runner.md` 第 30 行（第 2 步整段挤在同一行；只摘这两处，理由见正文「四、分工」与主 agent 派发提示，其余改动不进这份附录）

### 三条改四条那一处（现文）

```markdown
**另有四条只对入库装置生效**：
```

### 第 ④ 条整段（从「④」起，到它自己那一句结束，即「…删掉它建的仓副本与编译目录）」为止；这句末尾的「）」闭合的是「三条」那一处更早开启的外层括注，不是本段自己开的括号，原文如此）

```markdown
④ **入库装置先在草稿目录的副本里改，编过再整份换进主工作区**：主工作区那份先 `cp` 进草稿目录（新建的装置直接建在草稿目录里），只在草稿目录那一份上改；改完跑 `bash research/scripts/capped.sh <线程上限> python3 research/scripts/compile-then-swap.py <草稿副本> crates/singlefs-harness/src/bin/e<号>_<英文名>.rs --scratch <草稿目录>`，它在仓副本里编过才整份换进主工作区，编不过一个字节不写，照它报的错误改草稿副本再跑；主工作区里任何时候都只放编得过的版本，不用 Edit、Write 或 Bash 直接写主工作区 `crates/` 下的 `.rs`。交回之前跑 `python3 research/scripts/compile-then-swap.py --clean --scratch <草稿目录>` 删掉它建的仓副本与编译目录）
```
