# 附录二：门禁批 G1 + G2 diff（defs-gatebatch-m2-r3，生成于 2026-09-27 12:33 JST / 2026-09-27 03:33 UTC）

基准：这一轮被判的是两份已归档的 diff，原样放进本附录——`research/prompts/defs-gatebatch-m2-r2-fixes.diff`（G1，1630 行）与 `research/prompts/defs-gatebatch-m2-g2-changes.diff`（G2，1498 行），两批 2026-09-27 00:32 UTC 一起进了提交 `ecdf8465`（门禁批 G1/G2：层 0 分片、崩溃枚举用例准入、重型测试闸与交回闸）。被判的就是这两批 diff 落在今天文件里的样子。

sha256（材料员现算，与正文第一节给的一致）：
```
a4c1ffecf1f7cf1a1612d3e591dc89063236d152114e306b86a739f76112715e  research/prompts/defs-gatebatch-m2-r2-fixes.diff
d2a94fafe7b200bf19ed455c389f28db80e2e04cb53c5bc737d1bc92422f5864  research/prompts/defs-gatebatch-m2-g2-changes.diff
```

## 一、G1 diff（`research/prompts/defs-gatebatch-m2-r2-fixes.diff`，1630 行，原样）

```diff
--- a/.claude/gate.d/54-layer0-replay.sh
+++ b/.claude/gate.d/54-layer0-replay.sh
@@ -9,16 +9,19 @@
 #   bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full [--start-over] <worktree>
 #     主 agent 暂存之后（提交时由崩溃验证员），在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
 #     逐条崩溃枚举用例（.claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的那几行，照登记表的次序）：先算这条用例这批输入的指纹
-#     （research/scripts/admission.py crash-case-manifest：登记路径下的文件减去用例读不到的文件，加判它的 54 号、准入模块 admission.py、
-#     工具链、构建环境与这条用例的登记行）；那一格全绿标记在、作数就复用，不跑；不在才跑 `cargo test --release -p <包> --test <测试目标> -- --include-ignored
-#     --exact <用例函数> --nocapture`，按登记行第三列判日志（crash-case-judge），开跑与跑完各算一次指纹，相同才写那一格（crash-case-record）。
+#     （research/scripts/admission.py crash-case-manifest：登记路径下的文件减去用例读不到的文件，加准入模块 admission.py 里崩溃枚举用例的判法摘要、
+#     工具链、构建环境与这条用例的登记行；这一份 54 号不进指纹）；那一格全绿标记在、作数就复用，不跑；不在才照 admission.py crash-case-command
+#     交出的命令与环境跑（`cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数> --nocapture`，续跑的变量与线程数也由它定，
+#     那一段在判法摘要里），按登记行第三列判日志（crash-case-judge），开跑与跑完各算一次指纹，相同才写那一格（crash-case-record）。
 #     全绿标记在 git common-dir：`singlefs-crash-case-green.<用例名>.<输入指纹>`，不进工作树，各 worktree 读写同一组；别的格不动。
 #     一条判红删它这批输入那一格（先绿后红，前一趟那一格不再作数），接着跑下一条；「跑的过程中输入变了」那一支判红不删：它说不出开跑那一批的好坏。
 #     两趟 --full 同时跑同一条用例、同一个指纹时这里不加锁：后跑完的那一趟判红会删掉先跑完的那一趟刚写的绿标记，只会假红、不会假绿
 #     （崩溃验证员的定义里「另有 --full 在跑时不起」挡着这一种）。
-#     断点续跑：跑用例时设 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=<common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>（不随 worktree 删掉）、
-#     SINGLEFS_LAYER0_INPUT_FINGERPRINT=<这条用例的输入指纹>；--start-over 设 SINGLEFS_LAYER0_START_OVER=1（丢掉进度文件、从头跑），
+#     断点续跑（crash-case-command 设）：跑用例时设 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=<common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>
+#     （不随 worktree 删掉）、SINGLEFS_LAYER0_INPUT_FINGERPRINT=<这条用例的输入指纹>；--start-over 设 SINGLEFS_LAYER0_START_OVER=1（丢掉进度文件、从头跑），
 #     不带它时从调用方的环境里清掉这个变量。续跑的判法（片方案、校验和、观察者计数、判红删进度文件）在 crates/singlefs-harness/src/layer0_progress.rs。
+#     这一份 54 号不进指纹：改它（出路句、快档、次序）不废旧标记。它里面还定着结论的只剩流程的次序（开跑与跑完各算一次指纹、先判日志再写标记、
+#     判红删那一格），由 admission.py --selftest 的「54 号」那几格核；改它时快档照样核标记（范围那一问不摘掉它）。
 #   bash .claude/gate.d/54-layer0-replay.sh [项目根]           整轮门禁的默认（gate.sh 只传项目根）
 #     快档先核登记的每一条路径 git 至少列得出一个文件（git ls-files -co --exclude-standard -- <那一条>），有一条列不出判红，之后才问复用与改动范围。
 #     两条流的测试二进制在 release 下只跑不标 ignored 的用例，一条都没通过判红；再逐条崩溃枚举用例算它这批输入的指纹、核那一格
@@ -35,7 +38,10 @@
 # 用例的判别力在用例自己：对着产物与闭式的计数断言，oracle 的判别力由同文件的靶向阳性对照证明。
 #
 # 多线程（增补 2 收口表第 41 行；2026-09-18 用户定：测试与崩溃检测优先多线程）：crash.rs 按状态序号区间切片、多线程跑，
-# 线程数由 SINGLEFS_LAYER0_THREADS 传进去——没设就取本机核数（nproc）。每跑完一片，用例打一行 `LAYER0_PROGRESS`，这里边跑边转到本阶段的输出里。
+# 线程数由 SINGLEFS_LAYER0_THREADS 传进去——调用方没设就取本机核数（nproc），取法在 admission.py 的 crash_case_worker_threads。
+# 每跑完一片，用例打一行 `LAYER0_PROGRESS`，这里边跑边转到本阶段的输出里。
+# 全绿标记里线程分两格记：configured_worker_threads= 是配的（SINGLEFS_LAYER0_THREADS 与它怎么来的、本机几核），started_worker_threads= 是
+# 登记了 threads= 的用例从 LAYER0_PARALLEL_FINISHED 读到的起了几个；没登记 threads= 的记「读不到」。
 # 登记了 threads=<前缀> 的用例：按那一行计数的状态数找 `LAYER0_PARALLEL_FINISHED`，这一趟真跑了至少两片、却只起了 1 个工作线程、本机多于 1 核、
 # SINGLEFS_LAYER0_THREADS 没显式设成 1，判红（多半是线程数没传进去）；全部片从进度文件读回（起 0 个线程）、只剩 1 片要跑（最多起 1 个）都不判红。
 #
@@ -92,8 +98,9 @@
 # 一问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
 # 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
 # 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
-# 二问这次改动碰没碰登记给本阶段的那几条路径，再加登记表本身、跑的这一份 54 号与准入模块：这三样都进每条崩溃枚举用例的指纹，
-# 只改它们的改动（新登记一条用例、改一条的第三列、改判法）同样要核标记，不能在这一问被摘掉。
+# 二问这次改动碰没碰登记给本阶段的那几条路径，再加登记表本身、跑的这一份 54 号与准入模块：登记行与准入模块的判法摘要进每条崩溃枚举用例的指纹，
+# 54 号定着流程的次序，只改它们的改动（新登记一条用例、改一条的第三列、改判法、改流程）同样要核标记，不能在这一问被摘掉；
+# 核下来标记照样作数（只改了准入模块判法之外的部分、只改了 54 号）就判绿。
 # 这是 C8（范围判定）的粗粒度前身：它只摘得掉「零行输入的改动」，摘不出别的，C8 照旧欠着。
 # 两问之前先核登记的每一条路径 git 至少列得出一个文件：写错的路径两问都拿它答「没碰」，这一道就一直退 77、一次都不跑。
 if [[ "$layer0_tier" == quick ]]; then
@@ -108,21 +115,13 @@
     echo "                在项目根跑 git ls-files -co --exclude-standard -- <那一条>，改到列得出文件为止。全部列不出时先看这里是不是 git 工作树。"
     exit 1
   fi
-  reuse_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "$ROOT" "$(basename "$0")")"
-  reuse_rc=$?
-  if [[ "$reuse_rc" != 0 ]]; then
-    echo "  ! 本阶段跳过（复用上一次整轮全绿的判定）：$reuse_reason"
-    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。"
-    exit 77
-  fi
+  source "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
+  stage_run_or_skip "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
+    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")"
   layer0_judge_paths=(.claude/gate.d/stage-inputs.tsv "$(realpath -m --relative-to="$ROOT" "$layer0_stage_script_path")" "$(realpath -m --relative-to="$ROOT" "$layer0_admission_module")")
-  scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" "${layer0_registered_input_paths[@]}" "${layer0_judge_paths[@]}")"
-  scope_rc=$?
-  if [[ "$scope_rc" != 0 ]]; then
-    echo "  ! 本阶段跳过（这次改动没碰它判的东西）：$scope_reason"
-    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${layer0_input_paths_text}，加 ${layer0_judge_paths[*]}，判据见 research/scripts/change-touches-crates.sh。"
-    exit 77
-  fi
+  stage_run_or_skip "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "这次改动没碰它判的东西" \
+    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${layer0_input_paths_text}，加 ${layer0_judge_paths[*]}，判据见 research/scripts/change-touches-crates.sh。" \
+    -- "$ROOT" "${layer0_registered_input_paths[@]}" "${layer0_judge_paths[@]}"
 fi
 [[ -f Cargo.toml && -d crates/singlefs-harness ]] || { echo "  ! 没有 crates/singlefs-harness，本阶段跳过（步 0 之前没有装置）"; exit 77; }
 # 前提（cargo、rustc）登记在 stage-inputs.tsv 本阶段那一行第三列，经准入模块判，没齐判红（不退 77）
@@ -138,17 +137,6 @@
 layer0_scratch_directory="$(mktemp -d)"
 trap 'rm -rf -- "${layer0_scratch_directory:?}"' EXIT
 
-machine_cores="$(nproc)"
-if [[ -n "${SINGLEFS_LAYER0_THREADS:-}" ]]; then
-  threads_origin="显式设的"
-  threads_origin_word="explicit"
-else
-  SINGLEFS_LAYER0_THREADS="$machine_cores"
-  threads_origin="没设，取本机核数"
-  threads_origin_word="default"
-fi
-export SINGLEFS_LAYER0_THREADS
-
 # 登记的崩溃枚举用例，一条一行「键、包、测试目标、用例函数」（制表符分隔），照登记表的次序；登记有错判红。
 if ! crash_case_listing="$(python3 "$layer0_admission_module" crash-cases "$ROOT")"; then
   printf '%s\n' "$crash_case_listing" | sed 's/^/       /'
@@ -170,8 +158,7 @@
 write_crash_case_manifest() {
   local manifest_summary
   if ! manifest_summary="$(python3 "$layer0_admission_module" crash-case-manifest "$ROOT" "$1" "$2" \
-      --extra-file "<判它的 54 号：$layer0_stage_file_name>" "$layer0_stage_script_path" \
-      --extra-file "<判它的准入模块：admission.py>" "$layer0_admission_module" --toolchain --build-environment)"; then
+      --judging-digest --toolchain --build-environment)"; then
     case_manifest_problem="$manifest_summary"
     return 1
   fi
@@ -200,15 +187,39 @@
   return "${PIPESTATUS[0]}"
 }
 
-# run_crash_case <包> <测试目标> <用例函数> <日志> <输入指纹>：--full 跑一条崩溃枚举用例，带续跑的三个环境变量；
-# cargo 的整段输出进日志，`LAYER0_PROGRESS` 行边跑边转到本阶段的输出（跑全量时这里一直在涨）。
+# read_crash_case_command <键> <输入指纹>：准入模块 crash-case-command 交出的这一条的命令与环境（以 NUL 分隔）读进 case_machine_cores、
+# case_threads、case_threads_origin、case_progress_directory 与数组 case_command。交不出、写法不对返回 1，原因放进 case_command_problem。
+read_crash_case_command() {
+  local command_file="$layer0_scratch_directory/command.${1#crash-case:}" command_output
+  local -a start_over_option=()
+  if [[ "$layer0_start_over" == 1 ]]; then start_over_option=(--start-over); fi
+  if ! python3 "$layer0_admission_module" crash-case-command "$ROOT" "$1" "$2" "${start_over_option[@]}" > "$command_file"; then
+    case_command_problem="$(tr '\0' ' ' < "$command_file")"
+    return 1
+  fi
+  local -a command_words=()
+  mapfile -d '' -t command_words < "$command_file"
+  if (( ${#command_words[@]} < 5 )) || [[ ! "${command_words[0]}" =~ ^[1-9][0-9]*$ || ! "${command_words[1]}" =~ ^[1-9][0-9]*$ \
+      || ! "${command_words[2]}" =~ ^(explicit|default)$ ]]; then
+    command_output="$(tr '\0' ' ' < "$command_file")"
+    case_command_problem="准入模块交的不是「核数、线程数、explicit|default、进度目录、命令…」：${command_output}"
+    return 1
+  fi
+  case_machine_cores="${command_words[0]}"
+  case_threads="${command_words[1]}"
+  case_threads_origin="${command_words[2]}"
+  case_progress_directory="${command_words[3]}"
+  case_command=("${command_words[@]:4}")
+  return 0
+}
+
+# run_crash_case <日志> <命令的词…>：--full 照准入模块交的命令跑一条崩溃枚举用例；
+# 整段输出进日志，`LAYER0_PROGRESS` 行边跑边转到本阶段的输出（跑全量时这里一直在涨）。
 run_crash_case() {
-  local -a progress_settings=(SINGLEFS_LAYER0_PROGRESS_DIRECTORY="$layer0_progress_root/$5" SINGLEFS_LAYER0_INPUT_FINGERPRINT="$5")
-  local -a start_over_setting=(-u SINGLEFS_LAYER0_START_OVER)
-  if [[ "$layer0_start_over" == 1 ]]; then start_over_setting=(SINGLEFS_LAYER0_START_OVER=1); fi
-  env "${start_over_setting[@]}" "${progress_settings[@]}" \
-    cargo test --release -p "$1" --test "$2" -- --include-ignored --exact "$3" --nocapture 2>&1 \
-    | tee "$4" \
+  local log_file="$1"
+  shift
+  "$@" 2>&1 \
+    | tee "$log_file" \
     | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
     | sed -u 's/^/    /'
   return "${PIPESTATUS[0]}"
@@ -328,7 +339,7 @@
   if ! write_crash_case_manifest "$case_key" "$manifest_at_start"; then
     echo "  ✗ $case_key：算不出这批输入的指纹：$case_manifest_problem"
     echo "     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- ${layer0_input_paths_text} 看列不列得出文件，再跑 cargo -V && rustc -V；"
-    echo "                单跑 python3 research/scripts/admission.py crash-case-manifest <根> $case_key <清单文件> --toolchain --build-environment 看它报什么。"
+    echo "                单跑 python3 research/scripts/admission.py crash-case-manifest <根> $case_key <清单文件> --judging-digest --toolchain --build-environment 看它报什么。"
     red_cases+=("$case_key")
     continue
   fi
@@ -341,11 +352,21 @@
     reused_cases+=("$case_key")
     continue
   fi
+  if ! read_crash_case_command "$case_key" "$fingerprint_at_start"; then
+    echo "  ✗ $case_key：准入模块交不出起用例的命令：$case_command_problem"
+    echo "     → 怎么办：单跑 python3 research/scripts/admission.py crash-case-command <根> $case_key <指纹> 看它报什么（取不到 git common-dir、"
+    echo "                nproc 起不来、SINGLEFS_LAYER0_THREADS 不是正整数都在这里报）；修好之后重跑 --full。"
+    red_cases+=("$case_key")
+    continue
+  fi
   case_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
   case_log="$layer0_scratch_directory/log.$case_label"
-  echo "  · $case_key 开跑（${case_started_utc}；cargo test --release -p $case_package --test $case_target -- --include-ignored --exact $case_function；这批输入的指纹 ${fingerprint_at_start:0:16}…，${file_count_at_start} 个文件，减去用例读不到的 ${excluded_count_at_start} 个）："
+  case_threads_origin_text="没设，取本机核数"
+  if [[ "$case_threads_origin" == explicit ]]; then case_threads_origin_text="显式设的"; fi
+  case_threads_note="SINGLEFS_LAYER0_THREADS=${case_threads}（${case_threads_origin_text}），本机 ${case_machine_cores} 核"
+  echo "  · $case_key 开跑（${case_started_utc}；${case_command[*]}；${case_threads_note}；这批输入的指纹 ${fingerprint_at_start:0:16}…，${file_count_at_start} 个文件，减去用例读不到的 ${excluded_count_at_start} 个）："
   sed 's/^/      /' <<< "$marker_check_output"
-  if ! run_crash_case "$case_package" "$case_target" "$case_function" "$case_log" "$fingerprint_at_start"; then
+  if ! run_crash_case "$case_log" "${case_command[@]}"; then
     tail -40 "$case_log"
     delete_crash_case_marker "$case_key" "$fingerprint_at_start"
     echo "  ✗ $case_key 判红：cargo test 退非 0（上面是它的尾部）"
@@ -356,7 +377,7 @@
   fi
   judged_lines_file="$layer0_scratch_directory/judged.$case_label"
   if ! judge_output="$(python3 "$layer0_admission_module" crash-case-judge "$ROOT" "$case_key" "$case_log" "$judged_lines_file" \
-      --machine-cores "$machine_cores" --threads "$SINGLEFS_LAYER0_THREADS" --threads-origin "$threads_origin_word")"; then
+      --machine-cores "$case_machine_cores" --threads "$case_threads" --threads-origin "$case_threads_origin")"; then
     delete_crash_case_marker "$case_key" "$fingerprint_at_start"
     printf '%s\n' "$judge_output" | sed 's/^/       /'
     echo "  ✗ $case_key 的用例跑过了，日志却判不绿（上面逐条列出）"
@@ -381,15 +402,16 @@
     red_cases+=("$case_key")
     continue
   fi
-  threads_text="${judge_output//$'\n'/；}${judge_output:+；}SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核"
+  threads_text="${judge_output//$'\n'/；}${judge_output:+；}${case_threads_note}"
   if ! record_output="$(python3 "$layer0_admission_module" crash-case-record "$ROOT" "$case_key" "$fingerprint_at_start" "$manifest_at_start" "$judged_lines_file" \
-      --files "$file_count_at_start" --excluded "$excluded_count_at_start" --started "$case_started_utc" --judged-root "$ROOT" --threads-text "$threads_text")"; then
+      --files "$file_count_at_start" --excluded "$excluded_count_at_start" --started "$case_started_utc" --judged-root "$ROOT" \
+      --machine-cores "$case_machine_cores" --threads "$case_threads" --threads-origin "$case_threads_origin")"; then
     echo "  ✗ $case_key 判绿，全绿标记却没写成：$record_output"
     echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full（没有标记，整轮门禁的快档会一直红）。"
     red_cases+=("$case_key")
     continue
   fi
-  rmdir -- "$layer0_progress_root/$fingerprint_at_start" 2>/dev/null
+  rmdir -- "$case_progress_directory" 2>/dev/null
   echo "  ✓ $case_key 判绿（${threads_text}）：全绿标记写进 ${record_output}，记下的行原样："
   sed 's/^/      /' "$judged_lines_file"
   green_cases+=("$case_key")
--- a/research/scripts/admission.py
+++ b/research/scripts/admission.py
@@ -20,7 +20,9 @@
   崩溃枚举用例行（键是 crash-case:<名>，门禁 54 号逐条按复用判定跑、逐条记全绿标记）：路径写整个 crates/ 与 Cargo 清单、锁，
     不按用例手列；算输入时自动减去用例读不到的文件（见「崩溃枚举用例」一节）。第三列认四种：
       test=<包>:<测试目标>:<用例函数>     恰好一条：跑的是 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数>；
-                                         用例函数要标 #[ignore]（不带 --ignored 的 cargo test 不跑它，重型测试闸按这一条认它）
+                                         用例函数的每一处定义都要标 #[ignore]（不带 --ignored 的 cargo test 不跑它，重型测试闸按这一条认它）：
+                                         同名的每一处 fn <用例函数>( 都算（cfg 二选一的、子模块里同名的也算），有一处没标就算没标；
+                                         # 与 [ 之间许空白；找不到字面的定义（宏生成的用例）判错
       count-line=<前缀>                  日志里以「<前缀> 」开头的行恰好一行，原样记进全绿标记
       exhaustive=<前缀>                  那一行带 exhaustive=true（前缀要先登记成 count-line=）
       threads=<前缀>                     按那一行的 states= 找 LAYER0_PARALLEL_FINISHED 行判工作线程（前缀要先登记成 count-line=）
@@ -34,7 +36,8 @@
   runner、wrapper 与编译器指的可执行文件另进一行：环境变量 CARGO_TARGET_*_RUNNER、RUSTC_WRAPPER、RUSTC_WORKSPACE_WRAPPER、
   CARGO_BUILD_RUSTC_WRAPPER、CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER，与上面那几份配置文件里的 target.<三元组或 cfg>.runner、
   build.rustc-wrapper、build.rustc-workspace-wrapper，指的那个程序按内容进（值的首词是程序；带斜杠的相对路径，环境变量从仓根起、
-  配置文件从 .cargo 所在的那一层起；不带斜杠的在 PATH 里找；找不到的进「找不到」一句）；CARGO_BUILD_RUSTC 与配置文件里的 build.rustc
+  配置文件从 .cargo 所在的那一层起；不带斜杠的在 PATH 里找；找不到的进「找不到」一句）；首词之后指到现存文件的参数（绝对路径，或从同一层起的
+  相对路径）接着按内容进（`runner = "bash tools/r.sh"` 这一类，定行为的是参数里的脚本）；CARGO_BUILD_RUSTC 与配置文件里的 build.rustc
   进它的 `-V`（与 RUSTC 同一种）。登记路径下指向目录的符号链接按链接指向的目录里的文件计入。
 
 准入条件分三类，这里管前两类：
@@ -61,16 +64,27 @@
   keys <根>                      列出登记表里的实验键，一行一个
   crash-cases <根>               核登记的崩溃枚举用例（第三列的写法、包与测试目标在不在、用例函数在不在、它标没标 #[ignore]），每条打一行
                                  「<键><制表符><包><制表符><测试目标><制表符><用例函数>」；登记有错退 2，原因打在 stdout（不带 ✗，阶段自己打 ✗ 与出路）
-  crash-case-manifest <根> <键> <清单文件> [--extra-file <名字> <文件>]... [--toolchain] [--build-environment]
+  crash-case-manifest <根> <键> <清单文件> [--extra-file <名字> <文件>]... [--judging-digest] [--toolchain] [--build-environment]
                                  这条用例的输入清单：登记路径下的文件减去别的测试目标独占的测试文件，末尾按参数次序加名字行，
-                                 最后一行是这条用例的登记行（键、路径与第三列）；stdout 打「<指纹> <文件数> <减去的文件数>」
+                                 最后一行是这条用例的登记行（键、路径与第三列）；stdout 打「<指纹> <文件数> <减去的文件数>」。
+                                 --judging-digest 加一行判法摘要：这一份准入模块按 ast 从判法入口（CRASH_CASE_JUDGING_ENTRIES，连同分派表 COMMANDS 里
+                                 CRASH_CASE_JUDGING_SUBCOMMANDS 那几项指的函数）顺着引用的模块级名字求闭包，闭包里每个定义的原文按名字排好，
+                                 加分派表那几项、main 与 `if __name__` 那一段；判法之外的部分改了摘要不变（门禁 54 号给它，不再给整份模块与 54 号）
+  crash-case-command <根> <键> <输入指纹> [--start-over]
+                                 --full 跑这一条的命令与环境（54 号原样执行）：stdout 以 NUL 分隔，本机核数、线程数、explicit|default、续跑的进度目录，
+                                 其后是整条命令（env 设 SINGLEFS_LAYER0_PROGRESS_DIRECTORY、SINGLEFS_LAYER0_INPUT_FINGERPRINT、SINGLEFS_LAYER0_THREADS，
+                                 --start-over 时设 SINGLEFS_LAYER0_START_OVER=1、不然清掉它，再起 cargo test --release …）；线程数：调用方设了
+                                 SINGLEFS_LAYER0_THREADS 用它（explicit），没设取 nproc（default）。交不出退 2，原因打在 stdout
   crash-case-marker-check <根> <键> <指纹> <清单文件>
                                  这批输入那一格全绿标记在不在、作不作数：退 0 作数（stdout 第一行「ok <标记路径> <跑完的时刻>」，
                                  其后是标记里的计数行）；退 1 不作数（stdout 是原因，没有那一格时再比最近写的一格与这一次的清单）
   crash-case-judge <根> <键> <日志> <记录行文件> --machine-cores <核数> --threads <线程数> --threads-origin explicit|default
                                  判 --full 跑那一条的日志：退 0 判绿（记进标记的行写进记录行文件，stdout 是线程那一句）；退 1 判红（stdout 逐条原因）
-  crash-case-record <根> <键> <指纹> <清单文件> <记录行文件> --files <数> --excluded <数> --started <时刻> --judged-root <路径> --threads-text <一句>
-                                 判绿之后写那一格全绿标记（同目录临时文件写完再改名换上）；stdout 打标记路径
+  crash-case-record <根> <键> <指纹> <清单文件> <记录行文件> --files <数> --excluded <数> --started <时刻> --judged-root <路径>
+                    --machine-cores <核数> --threads <线程数> --threads-origin explicit|default
+                                 判绿之后写那一格全绿标记（同目录临时文件写完再改名换上）；stdout 打标记路径。线程分两格记：
+                                 configured_worker_threads= 是配的（线程数、显式设的还是取本机核数、本机几核），started_worker_threads= 是
+                                 crash-case-judge 记下的起了几个（登记了 threads= 的取 LAYER0_PARALLEL_FINISHED，没登记的写读不到）
   crash-case-marker-path <根> <键> <指纹>
                                  那一格全绿标记的路径（判红时阶段删它）
   gate-preconditions <阶段所在的仓根> <阶段文件名>
@@ -88,20 +102,33 @@
 raise-in-gate-reuse（门禁复用判定里抛异常，stage-must-run.sh 必须按要跑处理）、skip-gate-preconditions（门禁的前提不判）、
 ignore-gate-environment（门禁复用判定不比环境）、skip-build-environment（构建环境不进指纹）、ignore-linked-directories（指向目录的
 符号链接照旧滤掉）、exclude-mentioned-test-targets（别的测试目标被代码点了名也减掉）、threads-by-worker-count（照旧只看
-worker_threads=1 判「只用了一个线程」，不看这一趟跑了几片）。
+worker_threads=1 判「只用了一个线程」，不看这一趟跑了几片）、first-definition-only（用例函数只看第一处定义）、
+ignore-attribute-without-space（`# [ignore]` 不认）、concat-include-only（只认不带路径前缀的 include!(concat!(…))）、
+computed-include-subtracts-non-test-files（有算出来的 include 时照样减 mutations.tsv 与 src/bin/）、runner-program-only（runner / wrapper
+只按首词的程序进指纹）、whole-module-in-judging-digest（判法摘要换回整份准入模块）、judging-digest-without-dispatch（分派表那几项不进摘要）、
+single-worker-threads-field（标记里线程只记一格 worker_threads=）。
 
 管不到的：跑的是不是按今天的源码编出来的二进制（指纹按源码算，跑的是 target/ 里的旧二进制时两边对不上，
 replay.sh 与 cargo run 开跑前都会重编）；登记的路径少写了一条（那条输入变了不会放行，要靠强制开关，登记行本身进指纹，
 补登记之后自然放行）；产物头的指纹行是不是真由那一趟写的（由装置入口调本模块写，拷来的产物照样带着）。
 崩溃枚举用例减去的文件认不出的：用拼出来的名字在运行期读别的测试文件（名字不以整词出现在任何代码里）、构建脚本（build.rs 与 package.build
-指的那一份）之外的构建期代码按目录读 tests/、编译期拼出来的名字（include!(concat!(…)) 这一类）指到别的包的测试文件；这几种会让那份文件
-被减掉而它其实被读了。认得出形状、认不出读的是哪一份的两种按宽处理：包里有 include! / include_str! / include_bytes! 套 concat! 的，
-这个包的测试文件一份都不减；任何一个包的构建脚本（注释去掉之后）出现整词 tests 的，哪个包的测试文件都不减。
+指的那一份）之外的构建期代码按目录读 tests/（构建脚本 `mod` 进来的文件也算在这一类）、编译期算出来的名字（include!(concat!(…)) 这一类）
+指到别的包的测试文件；这几种会让那份文件被减掉而它其实被读了。认得出形状、认不出读的是哪一份的两种按宽处理：包里有 include! / include_str! /
+include_bytes! 的参数不以字符串字面量开头（套 concat!、env!、option_env! 或别的宏，宏名前带不带 ::core:: / std:: 都算）的，这个包的测试文件
+一份都不减，任何一份 .rs 里有这种 include 的，crates/mutations.tsv 与 src/bin/ 下的也一份都不减；任何一个包的构建脚本（注释去掉之后）
+出现整词 tests 的，哪个包的测试文件都不减。
+runner / wrapper 指的程序与参数里的脚本按内容进指纹，它们再 source、再读的文件不进（不跑它就不知道它读什么）：runner 脚本 source 的那一份改了，
+指纹不变、旧标记照样作数。判法摘要看不见的：靠 ast 里的静态引用求闭包，getattr、字符串拼出来的函数名、exec 这一类引到的定义不进；
+Python 自身（解释器、标准库）升级不进。54 号不进指纹：它里面还定着结论的只剩流程的次序（开跑与跑完各算一次指纹、先判日志再写标记），
+改了它旧标记照样作数，由 --selftest 的「54 号」那几格核，快档的范围那一问照样把它算进去。
+用例函数标没标 #[ignore] 按同名的每一处 fn <名>( 判：同一个测试目标里别的模块有同名、合法不标 ignore 的快用例时误判没标（多拒、自查判错，
+方向是多跑一步，接受）。
 减得少的（改了照样让用例重跑）：几个测试目标共用的测试模块（tests/common_*/ 这一类）要顺着模块图才减得准，这里不走模块图；
 包里有构建脚本、[[test]]、autotests 时整包不减。登记的四条里第二条流全量约 2.3 天（推的），这几种改动都会让它重跑。
 减掉之后仍可能让用例红的：src/bin/ 下的文件编不过时 cargo test 也编不过（集成测试要先编本包的 bin），这一种交给构建与 clippy 那几道，
 不靠崩溃枚举用例的标记。linker（配置文件里的 target.*.linker、环境变量 CARGO_TARGET_*_LINKER）指的程序不按内容进。
 """
+import ast
 import glob
 import hashlib
 import os
@@ -149,13 +176,32 @@
 LAYER0_PARALLEL_FINISHED_PREFIX = "LAYER0_PARALLEL_FINISHED "
 PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\. 1 passed; 0 failed; ")
 MARKER_DIFFERENCES_LISTED_AT_MOST = 20
-# 用例函数要标的属性：#[ignore] 或 #[ignore = "…"]（cfg_attr 里的条件 ignore 不算：条件不成立时不带 --ignored 照样跑它）
-IGNORE_ATTRIBUTE_FORM = re.compile(r"^#\[\s*ignore\s*(?:\]|=)")
+POSITIVE_INTEGER_FORM = re.compile(r"[1-9][0-9]*")
+# --full 起用例时设的环境变量与续跑的进度目录（crash_case_launch）
+LAYER0_THREADS_VARIABLE = "SINGLEFS_LAYER0_THREADS"
+LAYER0_START_OVER_VARIABLE = "SINGLEFS_LAYER0_START_OVER"
+LAYER0_PROGRESS_DIRECTORY_NAME = "singlefs-layer0-progress"
+# 判法摘要（crash_case_judging_digest_text）：崩溃枚举用例「日志判不判绿、标记作不作数、登记行怎么读、起用例的命令与环境」从这几个入口起求闭包
+CRASH_CASE_JUDGING_ENTRIES = ("judge_crash_case_log", "judge_worker_threads", "crash_case_marker_problems", "read_crash_case_marker",
+                              "check_crash_case_marker", "write_crash_case_marker", "crash_case_marker_path", "parse_crash_case",
+                              "crash_case_of_key", "crash_cases_of", "crash_case_launch", "started_worker_threads_text",
+                              "configured_worker_threads_text")
+# 分派表 COMMANDS 里这几项子命令：「子命令 → 函数」进摘要，函数也当入口（有人把 crash-case-judge 指到别的函数，摘要跟着变）
+CRASH_CASE_JUDGING_SUBCOMMANDS = ("crash-case-command", "crash-case-judge", "crash-case-record", "crash-case-marker-check", "crash-case-marker-path")
+# 原文进摘要、不顺着它的引用往下走的定义：main 引自证，顺下去自证就进来了
+CRASH_CASE_JUDGING_LEAVES = ("main",)
+CRASH_CASE_JUDGING_DIGEST_NAME = "<判法摘要：admission.py 里崩溃枚举用例的判法与起用例的命令>"
+# 用例函数要标的属性：#[ignore] 或 #[ignore = "…"]，# 与 [ 之间许空白（rustc 认 `# [ignore]`）；cfg_attr 里的条件 ignore 不算：条件不成立时不带 --ignored 照样跑它
+IGNORE_ATTRIBUTE_FORM = re.compile(r"^#\s*\[\s*ignore\s*(?:\]|=)")
+IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE = re.compile(r"^#\[\s*ignore\s*(?:\]|=)")  # 弄坏开关 ignore-attribute-without-space 换回的旧写法
 ATTRIBUTE_START = re.compile(r"#!?\s*\[")
 # 属性与 fn 之间可以夹的限定词（一次认一个，从 fn 往前剥）
 FUNCTION_QUALIFIER_AT_END = re.compile(r'(?:\bpub(?:\s*\([^()]*\))?|\basync|\bunsafe|\bconst|\bextern(?:\s*"[^"]*")?)$')
-# 编译期拼出来的文件名：include! / include_str! / include_bytes! 套 concat!（拼出来的是哪一份认不出，整包不减）
-COMPILE_TIME_CONCATENATED_INCLUDE = re.compile(r"\binclude(?:_str|_bytes)?!\s*[(\[{]\s*concat!")
+# 编译期算出来的文件名：include! / include_str! / include_bytes!（宏名前带不带 ::core:: / std:: 这类路径都算）的参数不以字符串字面量开头——
+# 套 concat!、env!、option_env! 或别的宏，读的是哪一份认不出，按宽处理（这个包的测试文件、mutations.tsv 与 src/bin/ 都不减）
+COMPILE_TIME_COMPUTED_INCLUDE = re.compile(r'\binclude(?:_str|_bytes)?\s*!\s*[(\[{]\s*(?!b?r#*"|b?")\S')
+# 弄坏开关 concat-include-only 换回的旧写法：只认紧跟着不带路径的 concat!
+COMPILE_TIME_CONCATENATED_INCLUDE_ONLY = re.compile(r"\binclude(?:_str|_bytes)?!\s*[(\[{]\s*concat!")
 # 崩溃枚举用例的输入里减去的、不是测试文件的那几份：仓根起的路径 → 为什么用例读不到它（有代码按文件名点名它时照留）
 CRASH_CASE_FILES_NOT_READ = {"crates/mutations.tsv": "crates 变异表：59 号按它改源码再跑点名的测试，用例编译期与运行期都不读它"}
 # 集成测试拿本包 bin 的路径靠这个前缀的环境变量；没有代码读它时，src/bin/ 下的文件编不进也读不到用例
@@ -497,12 +543,19 @@
 
 
 def program_contents(words, base_directory, environment):
-    """首词是程序、其后是参数的一个值：交回那个程序的内容（进指纹）；找不到的交「找不到」一句（值本身另有一行进指纹）。"""
+    """首词是程序、其后是参数的一个值：交回那个程序的内容（进指纹），找不到的交「找不到」一句（值本身另有一行进指纹）；
+    首词之后指到现存文件的参数（绝对路径，或从 base_directory 起的相对路径）接着按内容进：`bash tools/r.sh` 这一类，真正定行为的脚本在参数里。
+    那个脚本再 source、再读的文件不进（文件头「管不到的」）。"""
     program = words[0] if words else ""
     path = resolved_program(program, base_directory, environment)
-    if path and os.path.isfile(path):
-        return read_configuration_file(path)
-    return f"找不到：{program}".encode("utf-8", "surrogateescape")
+    contents = read_configuration_file(path) if path and os.path.isfile(path) else f"找不到：{program}".encode("utf-8", "surrogateescape")
+    if break_is_set("runner-program-only"):
+        return contents
+    for argument in words[1:]:
+        candidate = argument if os.path.isabs(argument) else os.path.join(base_directory, argument)
+        if os.path.isfile(candidate):
+            contents += b"\n<" + argument.encode("utf-8", "surrogateescape") + b">\n" + read_configuration_file(candidate)
+    return contents
 
 
 def compiler_version(program, root, environment, described_as):
@@ -1040,15 +1093,18 @@
 
 # ── 崩溃枚举用例（门禁 54 号：逐条按复用判定跑、逐条记全绿标记）────────────────
 #
-# 一条用例的输入 = 登记路径（整个 crates/ 与 Cargo 清单、锁）下 git 列得出的文件，减去用例读不到的文件，再加判它的 54 号、
-# 准入模块（这一份）、工具链、构建环境与这条用例的登记行（54 号按 --extra-file、--toolchain、--build-environment 给）。用例读不到的文件有三种：
+# 一条用例的输入 = 登记路径（整个 crates/ 与 Cargo 清单、锁）下 git 列得出的文件，减去用例读不到的文件，再加准入模块（这一份）里
+# 崩溃枚举用例的判法摘要、工具链、构建环境与这条用例的登记行（54 号按 --judging-digest、--toolchain、--build-environment 给；
+# 54 号自己不进：起用例的命令与环境由这一份的 crash-case-command 交出，在判法摘要里）。用例读不到的文件有三种：
 #   ① 别的测试目标 X 独占的文件：它所在的包没有构建脚本（build.rs 或 package.build）、没写 [[test]] 与 package.autotests，包里没有
-#      include! / include_str! / include_bytes! 套 concat! 的代码，任何一个包的构建脚本（注释去掉之后）都不出现整词 tests；
+#      参数是算出来的 include! / include_str! / include_bytes!（COMPILE_TIME_COMPUTED_INCLUDE：套 concat!、env!、option_env! 或别的宏，
+#      宏名前带不带 ::core:: / std:: 都算），任何一个包的构建脚本（注释去掉之后）都不出现整词 tests；
 #      X 是 tests/X.rs（或 tests/X/main.rs 那种目录目标、连同那个目录），而 X 的名字不以整词出现在别处任何 .rs（注释去掉之后）与
 #      Cargo.toml 里——`mod X;`、`#[path = "…X.rs"]`、`include_str!("…X.rs")`、运行期按字面路径读它，都让它留在输入里；
 #   ② CRASH_CASE_FILES_NOT_READ 里的（crates/mutations.tsv），没有代码与 Cargo.toml 按文件名点名它；
 #   ③ 没有任何代码读 CARGO_BIN_EXE_ 时，各包 src/bin/ 下的文件（包里有构建脚本、[[test]]、autotests 的不减；
 #      src/bin/ 之外的代码按 `bin/<它在 src/bin/ 下的路径>` 点名它的不减）。
+#   ② ③ 两类在任何一份 .rs 里有参数是算出来的 include 时一份都不减（拼出来的可能就是它们）。
 # 登记只写整个 crates/，不按用例手列它读哪些文件：新加的共用文件（tests/common.rs、build.rs、新拆出的 crate）默认就在输入里。
 
 class CrashCase:
@@ -1158,16 +1214,16 @@
         single_file = os.path.join(package_directory, "tests", case.target + ".rs")
         directory_target = os.path.join(package_directory, "tests", case.target)
         raise RegistrationError(f"崩溃枚举用例 {case.key} 的测试目标 {case.target} 不在（没有 {single_file}，也没有 {directory_target}/main.rs）")
-    for relative in files:
-        with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
-            attributes = attributes_before_function(rust_code_without_comments(handle.read()), case.function)
-        if attributes is None:
-            continue
-        if not attributes_mark_ignored(attributes):
-            raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 没标 #[ignore]（{relative}）：不带 --ignored 的 "
-                                    "cargo test 也会跑到全量；给它补上 #[ignore = \"<为什么平时不跑>\"]")
-        return files
-    raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 在 {'、'.join(files)} 里找不到（改了名？）")
+    marks = definitions_marked_ignored(root, files, case.function)
+    if not marks:
+        raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 在 {'、'.join(files)} 里找不到字面的 fn {case.function}(："
+                                "改了名？宏生成的用例判不出标没标 #[ignore]，要写成字面的函数")
+    unmarked = [relative for relative, marked in marks if not marked]
+    if unmarked:
+        raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 有 {len(unmarked)} 处定义没标 #[ignore]（{'、'.join(unmarked)}；"
+                                f"共 {len(marks)} 处，cfg 二选一的、子模块里同名的都算）：不带 --ignored 的 cargo test 也会跑到全量；"
+                                "给每一处补上 #[ignore = \"<为什么平时不跑>\"]")
+    return files
 
 
 def rust_code_without_comments(text):
@@ -1264,45 +1320,62 @@
     return spans
 
 
-def attributes_before_function(code, function):
-    """去掉注释之后的代码里 `fn <function>(` 前面紧挨着的那串属性（原文，一条一个，按出现的次序）；没有这个函数交 None。
-    属性与 fn 之间可以夹 pub、pub(…)、async、unsafe、const、extern "…"。"""
-    found = re.search(r"\bfn\s+" + re.escape(function) + r"\s*\(", code)
-    if not found:
-        return None
-    head = code[:found.start()].rstrip()
-    while True:
-        qualifier = FUNCTION_QUALIFIER_AT_END.search(head)
-        if not qualifier:
+def attributes_before_each_definition(code, function):
+    """去掉注释之后的代码里每一处 `fn <function>(`（cfg 二选一的、子模块里同名的都算，按出现的次序）前面紧挨着的那串属性：
+    [[属性原文，一条一个，按出现的次序], …]；一处都没有交空表。属性与 fn 之间可以夹 pub、pub(…)、async、unsafe、const、extern "…"。
+    弄坏开关 first-definition-only 下只交第一处。"""
+    attribute_lists = []
+    for found in re.finditer(r"\bfn\s+" + re.escape(function) + r"\s*\(", code):
+        head = code[:found.start()].rstrip()
+        while True:
+            qualifier = FUNCTION_QUALIFIER_AT_END.search(head)
+            if not qualifier:
+                break
+            head = head[:qualifier.start()].rstrip()
+        start_of_attribute_ending_at = {end: start for start, end in attribute_spans(head)}
+        attributes = []
+        cursor = len(head)
+        while cursor in start_of_attribute_ending_at:
+            start = start_of_attribute_ending_at[cursor]
+            attributes.append(head[start:cursor])
+            cursor = len(head[:start].rstrip())
+        attribute_lists.append(list(reversed(attributes)))
+        if break_is_set("first-definition-only"):
             break
-        head = head[:qualifier.start()].rstrip()
-    start_of_attribute_ending_at = {end: start for start, end in attribute_spans(head)}
-    attributes = []
-    cursor = len(head)
-    while cursor in start_of_attribute_ending_at:
-        start = start_of_attribute_ending_at[cursor]
-        attributes.append(head[start:cursor])
-        cursor = len(head[:start].rstrip())
-    return list(reversed(attributes))
+    return attribute_lists
 
 
 def attributes_mark_ignored(attributes):
-    """那串属性里有没有 #[ignore] 或 #[ignore = "…"]。"""
-    return any(IGNORE_ATTRIBUTE_FORM.match(attribute) for attribute in attributes)
+    """那串属性里有没有 #[ignore] 或 #[ignore = "…"]（# 与 [ 之间许空白）。"""
+    form = IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE if break_is_set("ignore-attribute-without-space") else IGNORE_ATTRIBUTE_FORM
+    return any(form.match(attribute) for attribute in attributes)
+
+
+def definitions_marked_ignored(root, relative_files, function):
+    """测试目标的源文件（relative_files，相对 root）里每一处 `fn <function>(` 标没标 #[ignore]：[(文件, 标了?)]，按文件与出现的次序；
+    一处都没有交空表，读不了抛 OSError。弄坏开关 first-definition-only 下只看头一份有定义的文件里的第一处。"""
+    marks = []
+    for relative in relative_files:
+        with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
+            attribute_lists = attributes_before_each_definition(rust_code_without_comments(handle.read()), function)
+        marks += [(relative, attributes_mark_ignored(attributes)) for attributes in attribute_lists]
+        if marks and break_is_set("first-definition-only"):
+            break
+    return marks
 
 
 def test_function_is_marked_ignored(root, package_directory, target, function):
-    """root 底下包目录 package_directory（相对 root）里测试目标 target 的用例函数 function 标没标 #[ignore]：标了 True，没标 False，
-    目标不在、读不了、没有这个函数交 None。.claude/hooks/lib_heavy_tests.py（重型测试闸）按文件路径导入本模块调它，与 crash-cases 自查同一套判法。"""
-    for relative in test_target_source_files(root, package_directory, target):
-        try:
-            with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
-                attributes = attributes_before_function(rust_code_without_comments(handle.read()), function)
-        except OSError:
-            return None
-        if attributes is not None:
-            return attributes_mark_ignored(attributes)
-    return None
+    """root 底下包目录 package_directory（相对 root）里测试目标 target 的用例函数 function 标没标 #[ignore]：同名的每一处定义都标了 True，
+    有一处没标 False，目标不在、读不了、一处定义都找不到（宏生成的用例这一类）交 None。
+    .claude/hooks/lib_heavy_tests.py（重型测试闸）按文件路径导入本模块调它，与 crash-cases 自查同一套判法；闸把 None 按没标算。"""
+    files = test_target_source_files(root, package_directory, target)
+    try:
+        marks = definitions_marked_ignored(root, files, function)
+    except OSError:
+        return None
+    if not marks:
+        return None
+    return all(marked for _relative, marked in marks)
 
 
 def whole_word_form(word):
@@ -1378,12 +1451,12 @@
     tests_word = whole_word_form("tests")
     if any(tests_word.search(code_texts.get(name, "")) for name in build_script_names(root, package_directories, listed_set)):
         return set()
-    concatenating = [name for name, text in code_texts.items() if name.endswith(".rs") and COMPILE_TIME_CONCATENATED_INCLUDE.search(text)]
+    computing = files_with_computed_include(code_texts)
     candidates = {}
     for package_directory in package_directories:
         if package_keeps_every_test_file(root, package_directory, listed_set):
             continue
-        if any(name.startswith(package_prefix(package_directory)) for name in concatenating):
+        if any(name.startswith(package_prefix(package_directory)) for name in computing):
             continue
         for target, files in test_targets_of_package(package_directory, listed_names).items():
             if (os.path.normpath(package_directory), target) != (os.path.normpath(own_package_directory), own_target):
@@ -1398,9 +1471,18 @@
     return exclusive
 
 
+def files_with_computed_include(code_texts):
+    """code_texts 里有 include! / include_str! / include_bytes! 的参数是算出来的（不以字符串字面量开头）的 .rs：路径的清单。"""
+    form = COMPILE_TIME_CONCATENATED_INCLUDE_ONLY if break_is_set("concat-include-only") else COMPILE_TIME_COMPUTED_INCLUDE
+    return [name for name, text in code_texts.items() if name.endswith(".rs") and form.search(text)]
+
+
 def files_crash_cases_do_not_read(root, listed_names, code_texts):
-    """listed_names（仓根起的 str）里不是测试文件、用例也读不到的（str 的集合）：判法见本节开头的 ② ③。"""
+    """listed_names（仓根起的 str）里不是测试文件、用例也读不到的（str 的集合）：判法见本节开头的 ② ③。
+    任何一份 .rs 里有算出来的 include（files_with_computed_include）时一份都不减：它拼出来的可能就是 mutations.tsv 或 src/bin/ 下的文件。"""
     listed_set = set(listed_names)
+    if files_with_computed_include(code_texts) and not break_is_set("computed-include-subtracts-non-test-files"):
+        return set()
     left_out = {name for name in CRASH_CASE_FILES_NOT_READ
                 if name in listed_set and not any(os.path.basename(name) in text for text in code_texts.values())}
     if any(BIN_EXECUTABLE_VARIABLE_PREFIX in text for text in code_texts.values()):
@@ -1524,9 +1606,28 @@
         else:
             thread_notes.append(note)
     recorded_lines += ["parallel_finished=" + line for line in log_lines if line.startswith(LAYER0_PARALLEL_FINISHED_PREFIX)]
+    if not break_is_set("single-worker-threads-field"):
+        recorded_lines.append("started_worker_threads=" + started_worker_threads_text(case, thread_notes))
     return problems, recorded_lines, thread_notes
 
 
+def started_worker_threads_text(case, thread_notes):
+    """全绿标记里「实际起的工作线程」那一格：登记了 threads= 的取判线程那几句（LAYER0_PARALLEL_FINISHED 里记的 worker_threads），
+    没登记的写「读不到」——配的是几个另记在 configured_worker_threads=，那不是起了几个。"""
+    if thread_notes:
+        return "；".join(thread_notes)
+    if case.thread_lines:
+        return "读不到：日志没判绿，线程那一格没判"
+    return ("读不到：这条用例没登记 threads=，门禁不读它起了几个工作线程（配的线程数在 configured_worker_threads=，不是起了几个；"
+            "日志里有 LAYER0_PARALLEL_FINISHED 的另原样记在 parallel_finished=）")
+
+
+def configured_worker_threads_text(machine_cores, threads, threads_origin):
+    """全绿标记里「配的线程数」那一格：传给用例的 SINGLEFS_LAYER0_THREADS、它是显式设的还是取的本机核数、本机几核。"""
+    origin = "显式设的" if threads_origin == "explicit" else "没设，取本机核数"
+    return f"SINGLEFS_LAYER0_THREADS={threads}（{origin}），本机 {machine_cores} 核"
+
+
 def read_crash_case_marker(path):
     """全绿标记的 key=value 行、原样的计数行与 input_file 行；读不了返回 None。"""
     try:
@@ -1635,6 +1736,109 @@
     return marker_path
 
 
+def crash_case_worker_threads(environment):
+    """--full 跑崩溃枚举用例的线程数：(本机核数, 线程数, "explicit" 或 "default")。SINGLEFS_LAYER0_THREADS 设了（非空）就用它，
+    没设取本机核数；本机核数取 nproc（在 environment 的 PATH 里找，它认 OMP_NUM_THREADS 与 CPU 亲和）。
+    nproc 起不来、打的不是正整数，SINGLEFS_LAYER0_THREADS 不是正整数，都抛 InputManifestError。"""
+    nproc = shutil.which("nproc", path=environment.get("PATH")) or "nproc"
+    try:
+        completed = subprocess.run([nproc], capture_output=True, text=True, errors="replace", env=dict(environment))
+    except OSError as error:
+        raise InputManifestError(f"nproc 起不来：{error}") from error
+    cores_text = completed.stdout.strip()
+    if completed.returncode != 0 or not POSITIVE_INTEGER_FORM.fullmatch(cores_text):
+        raise InputManifestError(f"nproc 退 {completed.returncode}、打了「{cores_text}」，不是正整数")
+    configured = environment.get(LAYER0_THREADS_VARIABLE, "")
+    if not configured:
+        return int(cores_text), int(cores_text), "default"
+    if not POSITIVE_INTEGER_FORM.fullmatch(configured):
+        raise InputManifestError(f"{LAYER0_THREADS_VARIABLE}={configured} 不是正整数")
+    return int(cores_text), int(configured), "explicit"
+
+
+def crash_case_launch(root, case, fingerprint, start_over, environment):
+    """--full 跑一条崩溃枚举用例的命令与环境（门禁 54 号原样执行；54 号不进指纹，这一段在判法摘要里）：
+    返回 (本机核数, 线程数, "explicit" 或 "default", 续跑的进度目录, 整条命令的词)。
+    命令是 env 设好续跑的三个变量与线程数再起 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数> --nocapture：
+    进度目录 <git common-dir>/singlefs-layer0-progress/<这批输入的指纹>、SINGLEFS_LAYER0_INPUT_FINGERPRINT=<指纹>；start_over 时
+    SINGLEFS_LAYER0_START_OVER=1，不然从调用方的环境里清掉它。取不到 common-dir、线程数取不到抛 InputManifestError。"""
+    common_directory = git_common_directory(root)
+    if common_directory is None:
+        raise InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），续跑的进度文件没处放")
+    machine_cores, threads, threads_origin = crash_case_worker_threads(environment)
+    progress_directory = os.path.join(common_directory, LAYER0_PROGRESS_DIRECTORY_NAME, fingerprint)
+    start_over_setting = [f"{LAYER0_START_OVER_VARIABLE}=1"] if start_over else ["-u", LAYER0_START_OVER_VARIABLE]
+    command = ["env", *start_over_setting, f"SINGLEFS_LAYER0_PROGRESS_DIRECTORY={progress_directory}",
+               f"SINGLEFS_LAYER0_INPUT_FINGERPRINT={fingerprint}", f"{LAYER0_THREADS_VARIABLE}={threads}",
+               "cargo", "test", "--release", "-p", case.package, "--test", case.target,
+               "--", "--include-ignored", "--exact", case.function, "--nocapture"]
+    return machine_cores, threads, threads_origin, progress_directory, command
+
+
+def top_level_definitions(tree):
+    """模块级的 def、class 与赋值：名字 → [节点]（同名的几处都留着）。"""
+    definitions = {}
+    for node in tree.body:
+        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
+            definitions.setdefault(node.name, []).append(node)
+        elif isinstance(node, (ast.Assign, ast.AnnAssign)):
+            for target in (node.targets if isinstance(node, ast.Assign) else [node.target]):
+                if isinstance(target, ast.Name):
+                    definitions.setdefault(target.id, []).append(node)
+    return definitions
+
+
+def source_of_node(lines, node):
+    """一个模块级节点的原文（连同它的装饰器）。"""
+    first = min([node.lineno] + [decorator.lineno for decorator in getattr(node, "decorator_list", [])])
+    return "\n".join(lines[first - 1:node.end_lineno])
+
+
+def crash_case_judging_digest_text(source):
+    """准入模块 source 里崩溃枚举用例的判法摘要原文：返回 (原文的 bytes, 闭包里定义的个数)。
+    从 CRASH_CASE_JUDGING_ENTRIES 与分派表 COMMANDS 里 CRASH_CASE_JUDGING_SUBCOMMANDS 那几项指的函数起，顺着 ast 里引用的模块级名字
+    （def、class、赋值）求闭包；闭包里每个定义的原文按名字排好，再加分派表那几项（子命令 → 函数）、main 与 `if __name__` 那一段的原文
+    （这两样不往下顺：main 引自证，顺下去自证就进来了）。闭包之外的部分（实验准入、门禁复用、排除法、构建环境、自证）改了摘要不变。
+    分派表里少了那几项的，抛 InputManifestError。弄坏开关 whole-module-in-judging-digest 下原文是整份模块，judging-digest-without-dispatch 下不加分派那几项。"""
+    if break_is_set("whole-module-in-judging-digest"):
+        return source.encode("utf-8", "surrogateescape"), 0
+    tree = ast.parse(source)
+    lines = source.split("\n")
+    definitions = top_level_definitions(tree)
+    dispatch = {}
+    for node in definitions.get("COMMANDS", []):
+        if isinstance(node, ast.Assign) and isinstance(node.value, ast.Dict):
+            for key, value in zip(node.value.keys, node.value.values):
+                if isinstance(key, ast.Constant) and key.value in CRASH_CASE_JUDGING_SUBCOMMANDS:
+                    dispatch[key.value] = ast.unparse(value)
+    missing = [name for name in CRASH_CASE_JUDGING_SUBCOMMANDS if name not in dispatch]
+    if missing:
+        raise InputManifestError(f"准入模块的分派表 COMMANDS 里没有 {'、'.join(missing)}：判法摘要算不全")
+    wanted = set()
+    queue = [name for name in CRASH_CASE_JUDGING_ENTRIES + tuple(dispatch.values()) if name in definitions]
+    while queue:
+        name = queue.pop()
+        if name in wanted or name in CRASH_CASE_JUDGING_LEAVES:
+            continue
+        wanted.add(name)
+        for node in definitions[name]:
+            queue += [inner.id for inner in ast.walk(node) if isinstance(inner, ast.Name) and inner.id in definitions and inner.id not in wanted]
+    pieces = [f"## {name}\n" + "\n".join(source_of_node(lines, node) for node in definitions[name]) for name in sorted(wanted)]
+    if not break_is_set("judging-digest-without-dispatch"):
+        pieces.append("## 分派\n" + "".join(f"{name} → {dispatch[name]}\n" for name in CRASH_CASE_JUDGING_SUBCOMMANDS))
+    pieces += [f"## {name}\n" + "\n".join(source_of_node(lines, node) for node in definitions.get(name, [])) for name in CRASH_CASE_JUDGING_LEAVES]
+    pieces += ["## 入口\n" + source_of_node(lines, node) for node in tree.body
+               if isinstance(node, ast.If) and "__name__" in ast.unparse(node.test)]
+    return "\n".join(pieces).encode("utf-8", "surrogateescape"), len(wanted)
+
+
+def crash_case_judging_digest_line():
+    """清单末尾那一行「判法摘要」：(名字, 原文)。原文取正在跑的这一份准入模块（__file__）。"""
+    with open(os.path.abspath(__file__), encoding="utf-8", errors="surrogateescape") as handle:
+        text, _count = crash_case_judging_digest_text(handle.read())
+    return (CRASH_CASE_JUDGING_DIGEST_NAME, text)
+
+
 # ── 命令行 ────────────────────────────────────────────────────────────────────
 
 def command_gate_reuse(arguments):
@@ -1709,6 +1913,8 @@
             named_lines.append(toolchain_line(root))
         elif kind == "--build-environment":
             named_lines.extend(build_environment_lines(root))
+        elif kind == "--judging-digest":
+            named_lines.append(crash_case_judging_digest_line())
         else:
             named_lines.append(registration_line)
     return named_lines
@@ -1813,16 +2019,18 @@
 
 
 def command_crash_case_manifest(arguments):
-    requested = named_line_requests(arguments[3:], ("--toolchain", "--build-environment")) if len(arguments) >= 3 else None
+    requested = (named_line_requests(arguments[3:], ("--toolchain", "--build-environment", "--judging-digest")) if len(arguments) >= 3
+                 else None)
     if requested is None:
-        print("  ✗ 用法：admission.py crash-case-manifest <项目根> <键> <清单文件> [--extra-file <名字> <文件>]... [--toolchain] [--build-environment]")
+        print("  ✗ 用法：admission.py crash-case-manifest <项目根> <键> <清单文件> [--extra-file <名字> <文件>]... [--judging-digest] [--toolchain] "
+              "[--build-environment]")
         print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写；末尾那几行按参数次序加")
         return EXIT_REGISTRATION_ERROR
     root, key, manifest_file = arguments[0], arguments[1], arguments[2]
     try:
         case = crash_case_of_key(read_registration_rows(root), key)
         manifest, fingerprint, file_count, excluded = crash_case_manifest(root, case, named_lines_of(root, requested))
-    except (RegistrationError, InputManifestError, OSError) as error:
+    except (RegistrationError, InputManifestError, OSError, SyntaxError) as error:
         print(f"写不出 {key} 的输入清单：{error}")
         return EXIT_REGISTRATION_ERROR
     with open(manifest_file, "wb") as handle:
@@ -1891,13 +2099,35 @@
     return 0
 
 
+def command_crash_case_command(arguments):
+    """--full 跑一条崩溃枚举用例的命令与环境（crash_case_launch），stdout 以 NUL 分隔：本机核数、线程数、explicit 或 default、续跑的进度目录，
+    其后是整条命令的词。有错退 2，原因打在 stdout（不带 ✗，54 号自己打 ✗ 与出路）。"""
+    start_over = arguments[3:] == ["--start-over"]
+    if len(arguments) not in (3, 4) or (len(arguments) == 4 and not start_over) or not re.fullmatch(r"[0-9a-f]{64}", arguments[2] if len(arguments) > 2 else ""):
+        print("  ✗ 用法：admission.py crash-case-command <项目根> <键> <输入指纹> [--start-over]")
+        print("     → 怎么办：指纹取开跑时 crash-case-manifest 那一趟的（64 位十六进制）；要丢掉进度文件、从头跑才带 --start-over")
+        return EXIT_REGISTRATION_ERROR
+    root, key, fingerprint = arguments[:3]
+    try:
+        case = crash_case_of_key(read_registration_rows(root), key)
+        machine_cores, threads, threads_origin, progress_directory, command = crash_case_launch(root, case, fingerprint, start_over, os.environ)
+    except (RegistrationError, InputManifestError) as error:
+        print(f"交不出 {key} 的命令：{error}")
+        return EXIT_REGISTRATION_ERROR
+    words = [str(machine_cores), str(threads), threads_origin, progress_directory, *command]
+    sys.stdout.write("".join(word + "\0" for word in words))
+    return 0
+
+
 def command_crash_case_record(arguments):
-    names = ("--files", "--excluded", "--started", "--judged-root", "--threads-text")
+    names = ("--files", "--excluded", "--started", "--judged-root", "--machine-cores", "--threads", "--threads-origin")
     values = option_values(arguments[5:], names) if len(arguments) >= 5 else None
-    if values is None or set(values) != set(names):
+    if (values is None or set(values) != set(names) or not values["--machine-cores"].isdigit() or not values["--threads"].isdigit()
+            or values["--threads-origin"] not in ("explicit", "default")):
         print("  ✗ 用法：admission.py crash-case-record <项目根> <键> <指纹> <清单文件> <记录行文件> --files <数> --excluded <数> "
-              "--started <时刻> --judged-root <路径> --threads-text <一句>")
-        print("     → 怎么办：指纹、清单文件、文件数与减去的文件数取开跑时 crash-case-manifest 那一趟的，记录行文件取 crash-case-judge 写的")
+              "--started <时刻> --judged-root <路径> --machine-cores <核数> --threads <线程数> --threads-origin explicit|default")
+        print("     → 怎么办：指纹、清单文件、文件数与减去的文件数取开跑时 crash-case-manifest 那一趟的，记录行文件取 crash-case-judge 写的，"
+              "核数、线程数与 explicit|default 取 crash-case-command 交的前三个")
         return EXIT_REGISTRATION_ERROR
     root, key, fingerprint, manifest_file, recorded_file = arguments[:5]
     try:
@@ -1906,9 +2136,11 @@
             manifest_text = handle.read()
         with open(recorded_file, encoding="utf-8", errors="surrogateescape") as handle:
             recorded_lines = [line for line in handle.read().split("\n") if line]
+        configured = configured_worker_threads_text(values["--machine-cores"], values["--threads"], values["--threads-origin"])
+        thread_field = ("worker_threads", configured) if break_is_set("single-worker-threads-field") else ("configured_worker_threads", configured)
         details = [("input_file_count", values["--files"]), ("excluded_file_count", values["--excluded"]),
                    ("started_utc", values["--started"]), ("finished_utc", time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())),
-                   ("judged_root", values["--judged-root"]), ("worker_threads", values["--threads-text"])]
+                   ("judged_root", values["--judged-root"]), thread_field]
         marker_path = write_crash_case_marker(root, case, fingerprint, manifest_text, recorded_lines, details)
     except (RegistrationError, InputManifestError, OSError) as error:
         print(f"{key} 的全绿标记没写成：{error}")
@@ -2147,6 +2379,7 @@
         run_rust_comment_cells(selftest)
         run_crash_case_input_cells(selftest, module)
         run_crash_case_log_cells(selftest)
+        run_judging_digest_cells(selftest, module)
         # ⑯ 门禁 54 号这一份的流程：拷进临时仓（不在 .claude/gate.d/ 下）、拿打合成日志的假 cargo 跑
         run_layer0_stage_cells(selftest, module)
 
@@ -2162,7 +2395,9 @@
         print("     → 怎么办：照上面每一格的说明改被测的那一段；判据与为什么这么定见文件头。弄坏开关那几格红不了，说明那一格分不出差别")
         return 1
     print(f"  ✓ admission.py 自证通过：{selftest.checked} 格都对（含弄坏开关 skip-unchanged、skip-preconditions、skip-gate-preconditions、"
-          "ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count "
+          "ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count、"
+          "first-definition-only、ignore-attribute-without-space、concat-include-only、computed-include-subtracts-non-test-files、runner-program-only、"
+          "whole-module-in-judging-digest、judging-digest-without-dispatch、single-worker-threads-field "
           "下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）")
     return 0
 
@@ -2568,13 +2803,21 @@
         for label, changes in [("环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER（值带参数）",
                                 {"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER": program + " --flag"}),
                                ("环境变量 RUSTC_WRAPPER", {"RUSTC_WRAPPER": program}),
-                               ("环境变量 CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER（不带斜杠、在 PATH 里找）", {"CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER": "selftest-runner"})]:
+                               ("环境变量 CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER（不带斜杠、在 PATH 里找）", {"CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER": "selftest-runner"}),
+                               ("环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=bash <脚本>（首词是解释器，脚本在参数里）",
+                                {"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER": "bash " + program})]:
             before, after = program_edit_fingerprints(changes)
             selftest.expect(f"构建环境：{label}指的程序换了内容、值不变，指纹变", before != after and not before.startswith("退"),
                             f"换之前 {before[:16]}，换之后 {after[:16]}")
+        broken_before, broken_after = program_edit_fingerprints({"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER": "bash " + program,
+                                                                 BREAK_VARIABLE: "runner-program-only"})
+        selftest.expect("弄坏开关 runner-program-only 下「runner = bash <脚本>，脚本换了内容」那一格红（指纹不变）", broken_before == broken_after,
+                        f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
         repository_configuration = os.path.join(work, ".cargo/config.toml")
         for label, configuration_text in [
                 ("仓根 .cargo/config.toml 里 target.<三元组>.runner", f'[target.x86_64-unknown-linux-gnu]\nrunner = ["{program}", "--flag"]\n'),
+                ("仓根 .cargo/config.toml 里 target.<三元组>.runner = [\"bash\", <从 .cargo 那一层起的相对路径>]",
+                 f'[target.x86_64-unknown-linux-gnu]\nrunner = ["bash", "{os.path.relpath(program, work)}"]\n'),
                 ("仓根 .cargo/config.toml 里 build.rustc-wrapper（从 .cargo 所在的那一层起的相对路径）",
                  f'[build]\nrustc-wrapper = "{os.path.relpath(program, work)}"\n')]:
             write_text(repository_configuration, configuration_text)
@@ -2740,6 +2983,9 @@
         with open(own_case_path, encoding="utf-8") as handle:
             own_case_text = handle.read()
         modules = "mod common;\nmod helper_target;\n"
+        cfg_split_case = "#[cfg(debug_assertions)]\n#[test]\n#[ignore]\nfn the_case() {}\n#[cfg(not(debug_assertions))]\n#[test]\nfn the_case() {}\n"
+        submodule_first_case = "mod slow {\n    #[test]\n    #[ignore]\n    fn the_case() {}\n}\n#[test]\nfn the_case() {}\n"
+        spaced_ignore_case = "#[test]\n# [ignore]\nfn the_case() {}\n"
         ignore_cells = [
             ("用例函数没标 #[ignore]", modules + "#[test]\nfn the_case() {}\n", False),
             ("只有 cfg_attr 里的条件 ignore", modules + "#[test]\n#[cfg_attr(miri, ignore)]\nfn the_case() {}\n", False),
@@ -2747,6 +2993,10 @@
             ("#[ignore] 只写在注释里", modules + "#[test]\n// #[ignore]\nfn the_case() {}\n", False),
             ("#[ignore] 在 #[test] 前面、fn 前面有 pub", modules + "#[ignore]\n#[test]\npub fn the_case() {}\n", True),
             ("ignore 的理由里带方括号、引号与 //", modules + '#[test]\n#[ignore = "样本 [x] \\"y\\" // z"]\nfn the_case() {}\n', True),
+            ("同名函数 cfg 二选一：debug 那一份标 ignore、release 那一份不标", modules + cfg_split_case, False),
+            ("子模块里同名标 ignore 的写在前面、顶层那一份（--exact the_case 跑的）不标", modules + submodule_first_case, False),
+            ("# [ignore]（# 与 [ 之间有空格，rustc 认）", modules + spaced_ignore_case, True),
+            ("同名的两处都标了 ignore（cfg 二选一）", modules + cfg_split_case.replace("#[test]\nfn the_case", "#[test]\n#[ignore]\nfn the_case"), True),
         ]
         for label, source_text, marked in ignore_cells:
             write_text(own_case_path, source_text)
@@ -2754,6 +3004,14 @@
             green = exit_code == 0 if marked else (exit_code == EXIT_REGISTRATION_ERROR and "没标 #[ignore]" in output)
             selftest.expect(f"crash-cases 自查：{label} ⇒ {'退 0' if marked else '退 2 并说没标 #[ignore]'}", green,
                             f"退 {exit_code}，stdout「{output.strip()}」")
+        for switch, label, source_text, broken_exit_code in (
+                ("first-definition-only", "同名函数 cfg 二选一、一份不标", modules + cfg_split_case, 0),
+                ("first-definition-only", "子模块里同名标 ignore 的写在前面、顶层那一份不标", modules + submodule_first_case, 0),
+                ("ignore-attribute-without-space", "# [ignore]", modules + spaced_ignore_case, EXIT_REGISTRATION_ERROR)):
+            write_text(own_case_path, source_text)
+            exit_code, output, _messages = admission("crash-cases", work, changes={BREAK_VARIABLE: switch})
+            selftest.expect(f"弄坏开关 {switch} 下「{label}」那一格红（crash-cases 退 {broken_exit_code}）", exit_code == broken_exit_code,
+                            f"弄坏之后退 {exit_code}：这一格分不出差别；stdout「{output.strip()}」")
         write_text(own_case_path, own_case_text)
 
         # 认得出形状、认不出读的是哪一份的按宽处理：包里 include!(concat!(…))、任何一个包的构建脚本出现整词 tests ⇒ 改 other_target.rs 指纹变
@@ -2764,9 +3022,16 @@
         two_members = '[workspace]\nmembers = ["crates/pkg", "crates/gen"]\n'
         gen_package = '[package]\nname = "gen"\nversion = "0.0.0"\n'
         reads_tests = 'fn main() { let _ = std::fs::read_dir("../pkg/tests"); }\n'
+        qualified_include = ('const PIECE: &str = ::core::include_str!(::core::concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oth", '
+                             '"er_target.rs"));\n')
+        environment_include = 'const PIECE: &str = include_str!(env!("SAMPLE_FIXTURE"));\n'
         wide_cells = [
             ("用例里 include!(concat!(\"other_\", \"target.rs\"))", {"crates/pkg/tests/own_case.rs": own_case_text + 'include!(concat!("other_", "target.rs"));\n'},
              True),
+            ("用例里 ::core::include_str!(::core::concat!(…))（宏名带路径）", {"crates/pkg/tests/own_case.rs": own_case_text + qualified_include}, True),
+            ("用例里 include_str!(env!(\"…\"))（路径在构建环境里）", {"crates/pkg/tests/own_case.rs": own_case_text + environment_include}, True),
+            ("对照：用例里 include_str!(\"helper_target.rs\")（字面量，读的是哪一份认得出）",
+             {"crates/pkg/tests/own_case.rs": own_case_text + 'const PIECE: &str = include_str!("helper_target.rs");\n'}, False),
             ("别的包的 build.rs 按目录读 ../pkg/tests", {"Cargo.toml": two_members, "crates/gen/Cargo.toml": gen_package, "crates/gen/src/lib.rs": "",
                                                     "crates/gen/build.rs": reads_tests}, True),
             ("别的包 package.build 指的构建脚本按目录读 tests", {"Cargo.toml": two_members, "crates/gen/Cargo.toml": gen_package + 'build = "generate.rs"\n',
@@ -2786,6 +3051,14 @@
             shutil.rmtree(os.path.join(work, "crates/gen"), ignore_errors=True)
             selftest.expect(f"崩溃枚举用例的输入：{label}，改 other_target.rs 指纹{'变' if should_move else '不变'}",
                             (before != after) == should_move and not before.startswith("退"), f"改之前 {before[:16]}，改之后 {after[:16]}")
+        write_text(own_case_path, own_case_text + qualified_include)
+        broken_before, _count = fingerprint({BREAK_VARIABLE: "concat-include-only"})
+        write_text(other_target_path, "#[test]\nfn other() { assert!(true); }\n")
+        broken_after, _count = fingerprint({BREAK_VARIABLE: "concat-include-only"})
+        write_text(other_target_path, "#[test]\nfn other() {}\n")
+        write_text(own_case_path, own_case_text)
+        selftest.expect("弄坏开关 concat-include-only 下「::core::include_str!(::core::concat!(…))」那一格红（改 other_target.rs 指纹不变）",
+                        broken_before == broken_after, f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
 
         # 用例读不到的非测试文件：crates/mutations.tsv 与（没有代码读 CARGO_BIN_EXE_ 时）src/bin/ 下的，改了指纹不变；有代码点名它们时照留
         lib_path = os.path.join(work, "crates/pkg/src/lib.rs")
@@ -2793,6 +3066,7 @@
             lib_text = handle.read()
         table_versions = ("# 样本变异表\n", "# 样本变异表\n多一行\n")
         tool_versions = ("fn main() {}\n", "fn main() { let changed = 1; }\n")
+        concatenated_table_include = 'const TABLE_TEXT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../mutations", ".tsv"));\n'
         not_read_cells = [
             ("crates/mutations.tsv 加一行（没有代码点名它）", {}, "crates/mutations.tsv", table_versions, False),
             ("lib.rs 的字符串里点名 mutations.tsv 时，crates/mutations.tsv 加一行", {"crates/pkg/src/lib.rs": lib_text + 'pub const TABLE: &str = "../mutations.tsv";\n'},
@@ -2802,6 +3076,11 @@
              "crates/pkg/src/bin/tool.rs", tool_versions, True),
             ("lib.rs 用 #[path = \"bin/tool.rs\"] 把它编进来时，src/bin/tool.rs 改了", {"crates/pkg/src/lib.rs": lib_text + '#[path = "bin/tool.rs"]\nmod tool;\n'},
              "crates/pkg/src/bin/tool.rs", tool_versions, True),
+            ("用例里 include_str!(concat!(…)) 拼出 ../mutations.tsv 时，crates/mutations.tsv 加一行",
+             {"crates/pkg/tests/own_case.rs": own_case_text + concatenated_table_include}, "crates/mutations.tsv", table_versions, True),
+            ("用例里 include!(concat!(…)) 拼出 src/bin/tool.rs 时，src/bin/tool.rs 改了",
+             {"crates/pkg/tests/own_case.rs": own_case_text + 'mod tool_code { include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bin/", "tool.rs")); }\n'},
+             "crates/pkg/src/bin/tool.rs", tool_versions, True),
         ]
         for label, setup, relative, (first_text, second_text), should_move in not_read_cells:
             for setup_relative, content in setup.items():
@@ -2815,6 +3094,15 @@
             write_text(lib_path, lib_text)
             selftest.expect(f"崩溃枚举用例的输入：{label}，指纹{'变' if should_move else '不变'}", (before != after) == should_move and not before.startswith("退"),
                             f"改之前 {before[:16]}，改之后 {after[:16]}")
+        write_text(own_case_path, own_case_text + concatenated_table_include)
+        write_text(os.path.join(work, "crates/mutations.tsv"), table_versions[0])
+        broken_before, _count = fingerprint({BREAK_VARIABLE: "computed-include-subtracts-non-test-files"})
+        write_text(os.path.join(work, "crates/mutations.tsv"), table_versions[1])
+        broken_after, _count = fingerprint({BREAK_VARIABLE: "computed-include-subtracts-non-test-files"})
+        os.remove(os.path.join(work, "crates/mutations.tsv"))
+        write_text(own_case_path, own_case_text)
+        selftest.expect("弄坏开关 computed-include-subtracts-non-test-files 下「include_str!(concat!(…)) 拼出 ../mutations.tsv」那一格红（指纹不变）",
+                        broken_before == broken_after, f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
         restored_after_wide_cells, _count = fingerprint()
         selftest.expect("崩溃枚举用例的输入：按宽处理与读不到的那几格改回之后，指纹回到原样", restored_after_wide_cells == baseline,
                         f"原样 {baseline[:16]}，改回之后 {restored_after_wide_cells[:16]}")
@@ -2824,10 +3112,24 @@
         write_text(judged_file, "test_result=test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.00s\n"
                                 "LAYER0 states=5 closed_form=5 exhaustive=true\nparallel_finished=LAYER0_PARALLEL_FINISHED states=5 slices=1\n")
         fingerprint_now, _count = fingerprint()
-        exit_code, marker_path, _messages = admission("crash-case-record", work, "crash-case:own", fingerprint_now, manifest_file, judged_file,
-                                                      "--files", "6", "--excluded", "1", "--started", "2026-09-26T00:00:00Z",
-                                                      "--judged-root", work, "--threads-text", "样本")
+        record_arguments = ("crash-case-record", work, "crash-case:own", fingerprint_now, manifest_file, judged_file, "--files", "6", "--excluded", "1",
+                            "--started", "2026-09-26T00:00:00Z", "--judged-root", work, "--machine-cores", "32", "--threads", "7",
+                            "--threads-origin", "explicit")
+        exit_code, marker_path, _messages = admission(*record_arguments, changes={BREAK_VARIABLE: "single-worker-threads-field"})
+        with open(marker_path.strip(), encoding="utf-8") as handle:
+            broken_marker_lines = handle.read().split("\n")
+        selftest.expect("弄坏开关 single-worker-threads-field 下「配的与起的线程分两格记」那一格红（只剩一格 worker_threads=）",
+                        exit_code == 0 and any(line.startswith("worker_threads=") for line in broken_marker_lines)
+                        and not any(line.startswith("configured_worker_threads=") for line in broken_marker_lines),
+                        f"写退 {exit_code}，标记里 {[line for line in broken_marker_lines if 'threads' in line]}")
+        exit_code, marker_path, _messages = admission(*record_arguments)
         marker_path = marker_path.strip()
+        with open(marker_path, encoding="utf-8") as handle:
+            marker_lines = handle.read().split("\n")
+        selftest.expect("crash-case-record 把配的线程数记成 configured_worker_threads=（线程数、显式设的、本机几核），不记旧的 worker_threads=",
+                        exit_code == 0 and "configured_worker_threads=SINGLEFS_LAYER0_THREADS=7（显式设的），本机 32 核" in marker_lines
+                        and not any(line.startswith("worker_threads=") for line in marker_lines),
+                        f"写退 {exit_code}，标记里 {[line for line in marker_lines if 'threads' in line]}")
         exit_code_check, output, _messages = admission("crash-case-marker-check", work, "crash-case:own", fingerprint_now, manifest_file)
         selftest.expect("crash-case-record 写的那一格，crash-case-marker-check 判作数并带出计数行",
                         exit_code == 0 and os.path.isfile(marker_path) and exit_code_check == 0 and output.startswith(f"ok {marker_path} ")
@@ -2906,6 +3208,84 @@
             case, synthetic_layer0_log(worker_threads=1, resumed_slices=63, freshly_run_slices=1), 32, False)
     selftest.expect("弄坏开关 threads-by-worker-count 下「只剩 1 片要跑、起 1 个线程」那一格红（照旧只看 worker_threads=1）", bool(problems),
                     "弄坏之后仍判绿：这一格分不出「这一趟跑了几片」")
+    # 标记里「实际起的工作线程」那一格（started_worker_threads=）：登记了 threads= 的取 LAYER0_PARALLEL_FINISHED 里起了几个，没登记的写读不到
+    problems, recorded_lines, _notes = judge_crash_case_log(case, synthetic_layer0_log(worker_threads=24), 32, False)
+    started = [line for line in recorded_lines if line.startswith("started_worker_threads=")]
+    selftest.expect("判绿时记 started_worker_threads=：登记了 threads= 的记 LAYER0_PARALLEL_FINISHED 里起了几个（24 个，不是配的 32）",
+                    not problems and started == ["started_worker_threads=LAYER0：24 个工作线程跑了 64 片、从进度文件读回 0 片（共 64 片）"],
+                    f"判出 {problems}，记下 {started}")
+    unthreaded = parse_crash_case(RegistrationRow("crash-case:plain", ["crates/"], ["test=pkg:target:function", "count-line=PLAIN"], 1, ""))
+    problems, recorded_lines, _notes = judge_crash_case_log(unthreaded, "PLAIN states=4\n" + PASSED_ONE_LINE + "\n", 32, False)
+    started = [line for line in recorded_lines if line.startswith("started_worker_threads=")]
+    selftest.expect("判绿时记 started_worker_threads=：没登记 threads= 的记「读不到」（门禁不读它起了几个）",
+                    not problems and len(started) == 1 and started[0].startswith("started_worker_threads=读不到：这条用例没登记 threads="),
+                    f"判出 {problems}，记下 {started}")
+
+
+def edit_inside_definition(source, name, old, new):
+    """只在模块级定义 name（def、class 或赋值）的原文里把 old 换成 new，old 在那里要恰好一处；返回换过的整份源码。
+    自证拿它改准入模块自己的一份拷贝：只认那个定义里的那一处，自证里写着的同一串字不算。命中不是一处抛 ValueError。"""
+    nodes = top_level_definitions(ast.parse(source)).get(name, [])
+    if len(nodes) != 1:
+        raise ValueError(f"模块级定义 {name} 有 {len(nodes)} 处，要恰好一处")
+    lines = source.split("\n")
+    first = min([nodes[0].lineno] + [decorator.lineno for decorator in getattr(nodes[0], "decorator_list", [])])
+    head, body, tail = "\n".join(lines[:first - 1]), "\n".join(lines[first - 1:nodes[0].end_lineno]), "\n".join(lines[nodes[0].end_lineno:])
+    if body.count(old) != 1:
+        raise ValueError(f"{name} 里「{old}」有 {body.count(old)} 处，要恰好一处")
+    return head + "\n" + body.replace(old, new) + "\n" + tail
+
+
+# (说明, 改哪个模块级定义, 旧串, 新串, 判法摘要该不该变)：前四种是判法之外的改动（K4 那一轮量过它们让四条用例全重跑），后几种是判法本身
+JUDGING_DIGEST_VARIANTS = [
+    ("实验准入：admit_experiment 里加一行注释", "admit_experiment", "def admit_experiment(root, key):\n", "def admit_experiment(root, key):\n    # 样本\n",
+     False),
+    ("门禁复用：gate_reuse 的一句说明改字", "gate_reuse", "强制跑一趟", "强制跑一次", False),
+    ("自证：run_selftest 里加一行注释", "run_selftest", "def run_selftest():\n", "def run_selftest():\n    # 样本\n", False),
+    ("排除规则：files_crash_cases_do_not_read 里加一行注释（它的结果变了，清单自己会变）", "files_crash_cases_do_not_read",
+     "    listed_set = set(listed_names)\n", "    # 样本\n    listed_set = set(listed_names)\n", False),
+    ("判法：judge_worker_threads 把「至少两片」改成「至少三片」", "judge_worker_threads", "freshly_run_slices >= 2 and", "freshly_run_slices >= 3 and", True),
+    ("判法调的小函数：fields_of_line 改写法", "fields_of_line", "line.split()", 'line.split(" ")', True),
+    ("判法用的常量：PASSED_ONE_TEST_FORM 放宽", "PASSED_ONE_TEST_FORM", "1 passed", "[0-9]+ passed", True),
+    ("登记行怎么读：parse_crash_case 不再要求 exhaustive= / threads= 先登记成 count-line=", "parse_crash_case",
+     "if prefix not in count_lines]", "if False]", True),
+    ("起用例的命令：crash_case_launch 把 --include-ignored 换成 --ignored", "crash_case_launch", '"--include-ignored"', '"--ignored"', True),
+    ("标记里配的线程那一格：configured_worker_threads_text 改字", "configured_worker_threads_text", "本机 {machine_cores} 核", "{machine_cores} 核", True),
+    ("main：分派的写法改了", "main", "return COMMANDS[arguments[0]](arguments[1:])", "return COMMANDS[arguments[0]](arguments[2:])", True),
+]
+
+
+def run_judging_digest_cells(selftest, module):
+    """判法摘要（D2）：判法之外的改动摘要不变、判法（连同它调的小函数、常量、起用例的命令、main 与分派表那几项）改了摘要变；
+    两个弄坏开关下各自那一格转红。拿本模块自己的源码按 edit_inside_definition 改一份算，不动文件。"""
+    with open(module, encoding="utf-8") as handle:
+        source = handle.read()
+    baseline, definition_count = crash_case_judging_digest_text(source)
+    selftest.expect("判法摘要算得出来，闭包里有判日志、读写标记、起用例的命令与 crash-case-command 那几个定义",
+                    definition_count > 0 and all(f"## {name}\n".encode() in baseline for name in
+                                                 ("judge_crash_case_log", "crash_case_launch", "command_crash_case_command", "crash_case_marker_problems")),
+                    f"闭包 {definition_count} 个定义")
+    for label, name, old, new, should_change in JUDGING_DIGEST_VARIANTS:
+        try:
+            changed = crash_case_judging_digest_text(edit_inside_definition(source, name, old, new))[0]
+        except ValueError as error:
+            selftest.expect(f"判法摘要：{label} 这一格的改动做得出来", False, str(error))
+            continue
+        selftest.expect(f"判法摘要：{label} ⇒ 摘要{'变' if should_change else '不变'}", (changed != baseline) == should_change,
+                        f"摘要{'变了' if changed != baseline else '没变'}")
+    swapped = edit_inside_definition(source, "COMMANDS", '"crash-case-judge": command_crash_case_judge,\n    "crash-case-record": command_crash_case_record,',
+                                     '"crash-case-judge": command_crash_case_record,\n    "crash-case-record": command_crash_case_judge,')
+    selftest.expect("判法摘要：分派表里 crash-case-judge 与 crash-case-record 指的函数对调（闭包里的定义一个没变） ⇒ 摘要变",
+                    crash_case_judging_digest_text(swapped)[0] != baseline, "摘要没变：分派表那几项没进摘要")
+    with BreakSwitch("judging-digest-without-dispatch"):
+        broken_baseline, broken_swapped = crash_case_judging_digest_text(source)[0], crash_case_judging_digest_text(swapped)[0]
+    selftest.expect("弄坏开关 judging-digest-without-dispatch 下「分派表两项对调」那一格红（摘要没变）", broken_baseline == broken_swapped,
+                    "弄坏之后摘要照样变了：这一格分不出分派表进没进摘要")
+    commented = edit_inside_definition(source, "admit_experiment", "def admit_experiment(root, key):\n", "def admit_experiment(root, key):\n    # 样本\n")
+    with BreakSwitch("whole-module-in-judging-digest"):
+        broken_baseline, broken_commented = crash_case_judging_digest_text(source)[0], crash_case_judging_digest_text(commented)[0]
+    selftest.expect("弄坏开关 whole-module-in-judging-digest 下「实验准入加一行注释」那一格红（摘要变了）", broken_baseline != broken_commented,
+                    "弄坏之后摘要照样没变：这一格分不出摘要是不是整份模块")
 
 
 # 假 cargo：-V 打版本；test 带 --include-ignored 的记下目标、过滤、续跑的三个环境变量，照控制目录里那条目标的日志与退出码打；
@@ -2922,8 +3302,9 @@
   elif [[ "$word" == --test ]]; then target="${arguments[position + 1]}"
   elif [[ "$word" == --release ]]; then release=1; fi
 done
-printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$target" "$filter" "$include_ignored" "${SINGLEFS_LAYER0_PROGRESS_DIRECTORY-unset}" \
-  "${SINGLEFS_LAYER0_INPUT_FINGERPRINT-unset}" "${SINGLEFS_LAYER0_START_OVER-unset}" "$exact" "$release" >> "$FAKE_CARGO_CONTROL/invocations"
+printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$target" "$filter" "$include_ignored" "${SINGLEFS_LAYER0_PROGRESS_DIRECTORY-unset}" \
+  "${SINGLEFS_LAYER0_INPUT_FINGERPRINT-unset}" "${SINGLEFS_LAYER0_START_OVER-unset}" "$exact" "$release" "${SINGLEFS_LAYER0_THREADS-unset}" \
+  >> "$FAKE_CARGO_CONTROL/invocations"
 if (( ! include_ignored )); then echo "test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s"; exit 0; fi
 if [[ -f "$FAKE_CARGO_CONTROL/touch-during-run" ]]; then
   echo "// 跑的过程中改的" >> "$(cat "$FAKE_CARGO_CONTROL/touch-during-run")"
@@ -2948,9 +3329,10 @@
 
 def run_layer0_stage_cells(selftest, module):
     """门禁 54 号这一份的流程：拷进临时仓的 .claude/stage-under-test/（不在 .claude/gate.d/ 下，名字照旧是 54-layer0-replay.sh），
-    假 cargo、rustc、nproc 在 PATH 最前面。核：逐条跑、续跑的三个环境变量、判绿写标记、快档核标记、第二趟全复用、只重跑输入变了的那一条、
-    --start-over、只剩 1 片要跑不误红、1 个线程跑了 64 片判红、0 passed 判红、cargo 退非 0 判红而别的照跑、跑的过程中输入变了不写标记、
-    快档不带 SINGLEFS_GATE_FULL=1 时只改登记表、只改 54 号、只改准入模块照样核标记（不退 77）。"""
+    假 cargo、rustc、nproc 在 PATH 最前面。核：逐条跑、续跑的三个环境变量与线程数（由 crash-case-command 交出）、判绿写标记（线程分记配的与起的）、
+    快档核标记、第二趟全复用、只重跑输入变了的那一条、--start-over、显式设线程数、只剩 1 片要跑不误红、1 个线程跑了 64 片判红、0 passed 判红、
+    cargo 退非 0 判红而别的照跑、跑的过程中输入变了不写标记、快档不带 SINGLEFS_GATE_FULL=1 时只改登记表判红、只改 54 号与只改准入模块判法之外的部分
+    照样核标记而判绿（54 号不进指纹）、改准入模块的判法判红（都不退 77）。"""
     repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
     real_stage = os.path.join(repository_root, ".claude/gate.d/54-layer0-replay.sh")
     work = tempfile.mkdtemp(prefix="admission-selftest-stage54-")
@@ -2967,7 +3349,7 @@
         write_text(os.path.join(work, REGISTRATION_TABLE), "".join(row + "\n" for row in rows))
         os.makedirs(os.path.join(work, "research/scripts"))
         shutil.copy(module, os.path.join(work, "research/scripts/admission.py"))
-        for helper in ("stage-must-run.sh", "change-touches-crates.sh"):
+        for helper in ("stage-must-run.sh", "change-touches-crates.sh", "stage-run-or-skip.sh"):
             shutil.copy(os.path.join(SELFTEST_HERE, helper), os.path.join(work, "research/scripts", helper))
         stage_copy = os.path.join(work, ".claude/stage-under-test/54-layer0-replay.sh")
         os.makedirs(os.path.dirname(stage_copy))
@@ -3012,9 +3394,7 @@
         def expected_fingerprint(key):
             _exit_code, output, _messages = run_in_environment(
                 [sys.executable, os.path.join(work, "research/scripts/admission.py"), "crash-case-manifest", work, key,
-                 os.path.join(control, "expected-manifest"), "--extra-file", "<判它的 54 号：54-layer0-replay.sh>", stage_copy,
-                 "--extra-file", "<判它的准入模块：admission.py>", os.path.join(work, "research/scripts/admission.py"),
-                 "--toolchain", "--build-environment"], environment)
+                 os.path.join(control, "expected-manifest"), "--judging-digest", "--toolchain", "--build-environment"], environment)
             return output.split()[0] if output.split() else ""
 
         def markers():
@@ -3037,6 +3417,23 @@
         selftest.expect("54 号 --full 设续跑的环境变量：进度目录 <common-dir>/singlefs-layer0-progress/<这条用例的指纹>、输入指纹是这条用例的；"
                         "调用方环境里的 SINGLEFS_LAYER0_START_OVER=1 在不带 --start-over 时被清掉",
                         progress_settings_ok and len(set(fingerprints.values())) == 3, f"跑的时候看到 {runs}，这几条用例的指纹 {fingerprints}")
+        selftest.expect("54 号 --full 的线程数由 crash-case-command 交出：调用方没设 SINGLEFS_LAYER0_THREADS，用例看到的是 nproc 的 32",
+                        len(runs) == 3 and all(call[8] == "32" for call in runs), f"跑的时候看到 {runs}")
+
+        def marker_text(case_name):
+            names = [name for name in markers() if f".{case_name}." in name]
+            if len(names) != 1:
+                return ""
+            with open(os.path.join(common_directory, names[0]), encoding="utf-8") as handle:
+                return handle.read()
+        stream_a_marker, case_c_marker = marker_text("stream-a"), marker_text("case-c")
+        selftest.expect("54 号 --full 写的标记里线程分两格：configured_worker_threads= 记配的（没设，取本机核数 32），"
+                        "started_worker_threads= 记登记了 threads= 的用例起了几个（32 个工作线程），没登记的记读不到；没有旧的 worker_threads= 那一格",
+                        "\nconfigured_worker_threads=SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核\n" in stream_a_marker
+                        and "\nstarted_worker_threads=LAYER0：32 个工作线程跑了 64 片" in stream_a_marker
+                        and "\nstarted_worker_threads=读不到：这条用例没登记 threads=" in case_c_marker
+                        and not any(line.startswith("worker_threads=") for line in (stream_a_marker + case_c_marker).split("\n")),
+                        f"stream-a 那一格：{stream_a_marker[:600]}；case-c 那一格：{case_c_marker[:400]}")
         exit_code, output, calls = stage()
         selftest.expect("54 号快档：两条流跑不标 ignored 的用例，三条用例的标记都作数，判绿",
                         exit_code == 0 and all(key in output for key, _t, _f, _c in LAYER0_STAGE_CASES) and not full_runs(calls)
@@ -3061,6 +3458,15 @@
         selftest.expect("54 号 --full --start-over：设 SINGLEFS_LAYER0_START_OVER=1；只剩 1 片要跑、起 1 个线程不误红，写回那一格",
                         exit_code == 0 and [(call[0], call[5]) for call in runs] == [("second_transaction_step_zero_layer0", "1")]
                         and any(".stream-b." in name for name in markers()), f"退 {exit_code}，跑了 {runs}，输出尾部：{output.strip()[-600:]}")
+        for name in [name for name in markers() if ".stream-b." in name]:
+            os.remove(os.path.join(common_directory, name))
+        set_logs()
+        exit_code, output, messages = run_in_environment(["bash", stage_copy, "--full", work], dict(environment, SINGLEFS_LAYER0_THREADS="7"))
+        runs = full_runs(invocations())
+        selftest.expect("54 号 --full 显式设 SINGLEFS_LAYER0_THREADS=7：用例看到 7，标记里 configured_worker_threads= 记「显式设的」",
+                        exit_code == 0 and [(call[0], call[8]) for call in runs] == [("second_transaction_step_zero_layer0", "7")]
+                        and "configured_worker_threads=SINGLEFS_LAYER0_THREADS=7（显式设的），本机 32 核" in marker_text("stream-b"),
+                        f"退 {exit_code}，跑了 {runs}，输出尾部：{(output + messages).strip()[-600:]}")
         exit_code, output, _calls = run_in_environment(["bash", stage_copy, "--start-over", work], environment)
         selftest.expect("54 号 --start-over 不带 --full：退 2，说只跟 --full 一起用", exit_code == 2 and "只跟 --full 一起用" in output,
                         f"退 {exit_code}，输出「{output.strip()}」")
@@ -3125,21 +3531,35 @@
         selftest.expect("54 号快档（不带 SINGLEFS_GATE_FULL=1）：提交之后一个改动都没有，范围那一问答「没碰」，退 77", exit_code == 77,
                         f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
         every_case = [key for key, _target, _function, _conditions in LAYER0_STAGE_CASES]
+        copied_module = os.path.join(work, "research/scripts/admission.py")
+        # (说明, 改哪份, 怎么改, 该判红点名的用例；空表是标记照样作数、判绿)
         scope_cells = [
             ("只改登记表（stream-b 那一行第三列去掉 threads=LAYER0B）", os.path.join(work, REGISTRATION_TABLE),
              lambda text: text.replace(" threads=LAYER0B", "", 1), ["crash-case:stream-b"]),
-            ("只改 54 号（加一行注释）", stage_copy, lambda text: text + "# 样本：只改 54 号\n", every_case),
-            ("只改准入模块（加一行注释）", os.path.join(work, "research/scripts/admission.py"), lambda text: text + "# 样本：只改准入模块\n", every_case),
+            ("只改 54 号（加一行注释；54 号不进指纹）", stage_copy, lambda text: text + "# 样本：只改 54 号\n", []),
+            ("只改准入模块判法之外的部分（文件尾加一行注释）", copied_module, lambda text: text + "# 样本：只改准入模块\n", []),
+            ("只改准入模块的判法（judge_worker_threads 把「至少两片」改成「至少三片」）", copied_module,
+             lambda text: edit_inside_definition(text, "judge_worker_threads", "freshly_run_slices >= 2 and", "freshly_run_slices >= 3 and"), every_case),
         ]
         for label, path, change, named_cases in scope_cells:
             with open(path, encoding="utf-8") as handle:
                 original_text = handle.read()
-            write_text(path, change(original_text))
+            try:
+                changed_text = change(original_text)
+            except ValueError as error:
+                selftest.expect(f"54 号快档：{label} 这一格的改动做得出来", False, str(error))
+                continue
+            write_text(path, changed_text)
             exit_code, output = quick_tier_without_forcing()
             write_text(path, original_text)
-            selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记，判红并点名 {'、'.join(named_cases)}，不退 77",
-                            exit_code == 1 and "没有作数的全绿标记" in output and all(key in output for key in named_cases),
-                            f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
+            if named_cases:
+                selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记，判红并点名 {'、'.join(named_cases)}，不退 77",
+                                exit_code == 1 and "没有作数的全绿标记" in output and all(key in output for key in named_cases),
+                                f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
+            else:
+                selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记（不退 77），三条标记都作数，判绿",
+                                exit_code == 0 and all(key in output for key in every_case) and "没有作数的全绿标记" not in output,
+                                f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
     except (OSError, shutil.Error) as error:
         selftest.expect("54 号这一份拷得进临时仓、跑得起来", False, f"{error}（真仓的 54 号在 {real_stage}）")
     finally:
@@ -3173,6 +3593,7 @@
     "crash-cases": command_crash_cases,
     "crash-case-manifest": command_crash_case_manifest,
     "crash-case-marker-check": command_crash_case_marker_check,
+    "crash-case-command": command_crash_case_command,
     "crash-case-judge": command_crash_case_judge,
     "crash-case-record": command_crash_case_record,
     "crash-case-marker-path": command_crash_case_marker_path,
@@ -3184,7 +3605,7 @@
         return run_selftest()
     if not arguments or arguments[0] not in COMMANDS:
         say("  ✗ 用法：admission.py {gate-reuse|gate-preconditions|gate-record-environment|experiment|paths|manifest|keys|crash-cases|"
-            "crash-case-manifest|crash-case-marker-check|crash-case-judge|crash-case-record|crash-case-marker-path} … 或 --selftest")
+            "crash-case-manifest|crash-case-marker-check|crash-case-command|crash-case-judge|crash-case-record|crash-case-marker-path} … 或 --selftest")
         say("     → 怎么办：各子命令的参数见文件头")
         return EXIT_REGISTRATION_ERROR
     return COMMANDS[arguments[0]](arguments[1:])
--- a/.claude/hooks/lib_heavy_tests.py
+++ b/.claude/hooks/lib_heavy_tests.py
@@ -11,6 +11,8 @@
   command_under_launcher(words) -> (list[str], str | None) | None
       words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序（/usr/bin/time、flock、rustup run、chrt、prlimit、systemd-run、strace、perf）时，
       剥掉它交回它起的那条命令与那条命令的当前目录（None 是不变）；调用方把交回的命令再交给 lib_shell_words 切一遍、逐条判。
+      短选项合写（`-fo <文件>`、`-xw 10`、`-qu <名>`）逐个字母查那张表：头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；
+      systemd-run 的 `-E NAME=VALUE` / `--setenv=NAME=VALUE` 设的变量写在交回那条命令的最前面（再切一遍时当它的环境变量，认 runner 要用）。
       这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
   classify_process(argv, cwd) -> list[HeavyTest]
       一个在跑的进程：argv 是 /proc/<pid>/cmdline 按 NUL 切开的那一串，cwd 是 /proc/<pid>/cwd 指向的目录。
@@ -27,20 +29,28 @@
 
 认的输入：cargo 命令行（test / t、run / r，以及起测试的 nextest run、miri、llvm-cov、hack、mutants）、按名字认的脚本与门禁阶段、
 qemu-system-*、herd7，以及直接执行的测试二进制 `<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制哈希>`：按 <名字> 判，
-名字含 layer0 的算层 0。libtest 参数带 --list 的（只列用例、一条都不跑）不算重型。
+名字含 layer0 的算层 0。libtest 参数里 --list 当选项出现的（只列用例、一条都不跑）不算重型；跟在带一个值的 libtest 选项（--skip、--logfile、
+--test-threads、--format、--color、-Z、--shuffle-seed：LIBTEST_OPTIONS_WITH_VALUE）后面的 --list 是那个选项的值，照样全跑，不算只列。
 崩溃枚举用例（门禁 54 号逐条跑的那几条，登记在 .claude/gate.d/stage-inputs.tsv 键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>）：
 跑到它们的测试目标（cargo test 点名它、通配命中它、或不挑目标而包里有它；直接执行它的测试二进制）而下面任一条成立的算重型：
   libtest 参数带 --ignored 或 --include-ignored（nextest 是 --run-ignored 的值不是 default）；
-  可能带上它而看不见：cargo 全局选项 --config 里定了别名（alias.<名>，且子命令就是那个别名）或 runner（.runner），
-  命令看得到的环境变量里有 CARGO_TARGET_*_RUNNER，或 CARGO_ALIAS_<名> 定的别名就是子命令；
-  登记的用例函数没标 #[ignore]（读那个包里测试目标的源码，判法与 research/scripts/admission.py 的 crash-cases 自查同一份：本模块按文件路径导入它）。
+  可能带上它而看不见：cargo 全局选项 --config 里定了别名（alias.<名>，且子命令就是那个别名）或 runner（按正则认 .runner =，
+  另把值按 TOML 读，定了 target.<任何>.runner 也算：带引号的键 "runner" 这一类），
+  命令看得到的环境变量里有 CARGO_TARGET_*_RUNNER（systemd-run -E / --setenv 设给里面那条命令的也算），或 CARGO_ALIAS_<名> 定的别名就是子命令；
+  登记的用例函数有一处定义没标 #[ignore]，或判不出标没标（找不到那个目标、那个函数，宏生成的用例，读不了源码，导入不了 admission.py：都按没标算）。
+  读那个包里测试目标的源码，判法与 research/scripts/admission.py 的 crash-cases 自查同一份：本模块按文件路径导入它，同名的每一处 fn <名>( 都判。
 另有一条按参数认：任何命令（按文本处理参数的 grep、git、sed 这一类除外）参数里有 --ignored 或 --include-ignored、又有登记的用例函数名，
 按跑崩溃枚举用例算（拷走改名的测试二进制、find -exec 起的这类，名字认不出，靠点名的用例函数认）。
 登记表取两份的并：从命令的当前目录（有 --manifest-path 时取它所在的目录）往上找到的第一份，与这份文件所在仓的那一份。
-接受的误拒：`--include-ignored --exact <同一目标里的快用例>` 照拒（过滤之后剩下哪几条，执行前判不出）；别名展开之后不是 test 的照拒。
+接受的误拒：`--include-ignored --exact <同一目标里的快用例>` 照拒（过滤之后剩下哪几条，执行前判不出）；别名展开之后不是 test 的照拒；
+同一目标里别的模块有与登记的用例函数同名、合法不标 #[ignore] 的快用例时照拒；admission.py 导入不了时，点名登记目标的 cargo test 一律拒。
 看不见的：.cargo/config.toml 与 `--config <文件>` 里定的别名与 runner；拷走改名、又不点名用例函数的测试二进制；
 cargo mutants -d 指到别处的树。别名与 runner 那两种由看门狗（research/scripts/agent-watch.py）在进程这一层兜：cargo 最后照样以原名
 带 --ignored 起那个测试二进制（推的，没量）；拷走改名、又不点名用例函数的，看门狗同样认不出。
+弄坏开关（只给自证用，证明那几格会红）：LIB_HEAVY_TESTS_BREAK 设成下面一个或几个（逗号分隔），--selftest 与 heavy-test-guard.sh --selftest 都必须判红：
+  launcher-whole-word-options（短选项合写不拆，照旧按整词查表）、launcher-drops-setenv（systemd-run -E / --setenv 设的变量不带进里面那条命令）、
+  list-by-presence（见到 --list 这个词就算只列）、undecided-ignore-allowed（判不出标没标时放行）、
+  runner-configuration-by-regex-only（--config 只按正则认 runner）。
 """
 import fnmatch, functools, glob, importlib.util, os, re, shlex, shutil, sys, tempfile, tomllib
 from typing import NamedTuple
@@ -88,6 +98,23 @@
     return HeavyTest(kind, KIND_CATEGORY[kind], detail)
 
 
+def break_is_set(switch_name):
+    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")
+
+
+def libtest_lists_only(libtest_arguments):
+    """libtest 的参数里 --list 当选项出现（只列用例、一条都不跑）：跳过带一个值的选项的那个值再找，`--skip --list` 里的 --list 不算。"""
+    if break_is_set("list-by-presence"):
+        return LIST_ONLY_TEST_ARGUMENT in libtest_arguments
+    position = 0
+    while position < len(libtest_arguments):
+        argument = libtest_arguments[position]
+        if argument == LIST_ONLY_TEST_ARGUMENT:
+            return True
+        position += 2 if argument in LIBTEST_OPTIONS_WITH_VALUE else 1
+    return False
+
+
 # 崩溃枚举用例的登记表（与 research/scripts/admission.py 读的是同一份）：键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>
 CRASH_CASE_REGISTRY = os.path.join(".claude", "gate.d", "stage-inputs.tsv")
 CRASH_CASE_TEST_CONDITION = re.compile(r"^test=(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
@@ -95,6 +122,10 @@
 IGNORED_TEST_ARGUMENTS = {"--ignored", "--include-ignored"}
 # libtest 只列用例、一条都不跑的参数
 LIST_ONLY_TEST_ARGUMENT = "--list"
+# libtest 带一个值的选项：`--skip --list` 里的 --list 是 --skip 的值（照样全跑），不是「只列」
+LIBTEST_OPTIONS_WITH_VALUE = {"--logfile", "--skip", "--test-threads", "--format", "--color", "-Z", "--shuffle-seed"}
+# 弄坏开关的环境变量（写法见文件头）
+BREAK_VARIABLE = "LIB_HEAVY_TESTS_BREAK"
 # 判用例函数标没标 #[ignore] 的那一份（与门禁 54 号的 crash-cases 自查同一套判法）
 ADMISSION_MODULE_PATH = os.path.join(HOOK_REPOSITORY, "research", "scripts", "admission.py")
 # 按参数认崩溃枚举用例时不看的命令：它们把参数当文本搜、打印、比对，不执行测试二进制
@@ -177,12 +208,16 @@
 
 
 def crash_case_function_runs_without_ignored(case, package_directory):
-    """登记的用例函数在 package_directory 这个包里没标 #[ignore]（不带 --ignored 的 cargo test 也会跑它）：没标 True；
-    标了、找不到那个目标或那个函数、导入不了 admission.py 都交 False（判不出的不在这里拒，提交时 54 号的 crash-cases 自查判红）。"""
+    """登记的用例函数在 package_directory 这个包里可能不带 --ignored 也被跑到：同名的每一处定义都标了 #[ignore] 才交 False；
+    有一处没标，或判不出（找不到包、那个目标、那个函数，宏生成的用例，读不了源码，导入不了 admission.py）都交 True——
+    判不出时放行，这一条在提交时 54 号的 crash-cases 自查判红之前就已经跑完了。"""
     module = admission_module()
     if module is None or package_directory is None:
-        return False
-    return module.test_function_is_marked_ignored(package_directory, ".", case.target, case.function) is False
+        return not break_is_set("undecided-ignore-allowed")
+    marked = module.test_function_is_marked_ignored(package_directory, ".", case.target, case.function)
+    if break_is_set("undecided-ignore-allowed"):
+        return marked is False
+    return marked is not True
 
 
 def manifest_sections(path):
@@ -354,9 +389,21 @@
     return None
 
 
+def configuration_value_names_runner(value):
+    """--config 的一个值按 TOML 读：定了 target.<任何>.runner（带不带引号的键、点号两边带不带空白都算）交 True；
+    读不成 TOML 的（多半是配置文件的路径，文件头「看不见的」）交 False，交给正则那一道。"""
+    try:
+        settings = tomllib.loads(value)
+    except ValueError:  # TOMLDecodeError 是 ValueError
+        return False
+    targets = settings.get("target") if isinstance(settings.get("target"), dict) else {}
+    return any(isinstance(table, dict) and "runner" in table for table in targets.values())
+
+
 def runner_of_test_binaries(command, environment):
     """--config 里定了 …runner、或环境变量里有 CARGO_TARGET_*_RUNNER 时交一句（runner 能替测试二进制加 --ignored，命令行上看不见）；没有交 None。"""
-    if any(RUNNER_CONFIGURATION.search(value) for value in command.configuration_values):
+    by_table = not break_is_set("runner-configuration-by-regex-only")
+    if any(RUNNER_CONFIGURATION.search(value) or (by_table and configuration_value_names_runner(value)) for value in command.configuration_values):
         return "--config 里定的 runner（它能替测试二进制加 --ignored）"
     runners = sorted(name for name in environment if RUNNER_ENVIRONMENT_VARIABLE.match(name))
     if runners:
@@ -419,7 +466,7 @@
         return None
     described = f"cargo {found.subcommand}"
     libtest_arguments = rest[rest.index("--") + 1:] if "--" in rest else []
-    if LIST_ONLY_TEST_ARGUMENT in libtest_arguments:
+    if libtest_lists_only(libtest_arguments):
         return None
     if whole_workspace:
         return heavy_test("full-cargo", f"{described} 带 --workspace / --all")
@@ -553,17 +600,37 @@
 }
 # 选项之后、那条命令之前还有几个位置参数：flock 的锁文件、chrt 的优先级、rustup run 的工具链
 LAUNCHER_POSITIONAL_COUNT = {"flock": 1, "chrt": 1, "rustup": 1}
+# 给里面那条命令设环境变量的选项（值是 NAME=VALUE）：设的变量写在交回那条命令的最前面
+LAUNCHER_ENVIRONMENT_OPTIONS = {"systemd-run": {"-E", "--setenv"}}
+# 交回的命令以 NAME=VALUE 打头时，那是设的变量、不是程序（值里的路径 basename 碰巧叫 strace 也不当包装剥）
+ENVIRONMENT_ASSIGNMENT_WORD = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
 # perf 起一条命令的子命令；后面还要再跟一个 record 的那几组
 PERF_SUBCOMMANDS_RUNNING_A_COMMAND = {"stat", "record", "trace"}
 PERF_GROUPS_WITH_RECORD = {"mem", "c2c", "sched", "lock", "kmem", "kwork"}
 
 
+def launcher_option(word, next_word, options_with_value):
+    """包装程序的一个选项词：交回 (选项名, 它的值, 这个选项占几个词)。长选项 `--名=值` 与 `--名 值` 按整词查表；
+    短选项合写（`-fo 文件`、`-xw 10`、`-o文件`）逐个字母查表，头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；一个要值的字母都没有的只占自己。"""
+    if word.startswith("--") or break_is_set("launcher-whole-word-options"):
+        option, has_attached_value, attached_value = word.partition("=")
+        if has_attached_value:
+            return option, attached_value, 1
+        return (option, next_word, 2) if option in options_with_value else (option, "", 1)
+    for index, letter in enumerate(word[1:], start=1):
+        if "-" + letter in options_with_value:
+            attached_value = word[index + 1:]
+            return ("-" + letter, attached_value, 1) if attached_value else ("-" + letter, next_word, 2)
+    return word, "", 1
+
+
 def command_under_launcher(words):
     """words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序时，剥掉它（连同它的选项与位置参数），交回 (它起的那条命令的词, 那条命令的当前目录：
     None 是不变，systemd-run --working-directory 给的是相对当前目录的写法)；不是这几样、或后面没有要起的命令（strace -p、flock 只给 fd）交 None。
+    选项词怎么切见 launcher_option；systemd-run 的 -E / --setenv 设的 NAME=VALUE 写在交回那条命令的最前面。
     flock 的 -c <字符串> 交回成 sh -c <字符串>。rustup 只认 rustup run，perf 只认 stat / record / trace 与 <组> record。"""
     name = os.path.basename(words[0])
-    if name not in LAUNCHER_OPTIONS_WITH_VALUE:
+    if name not in LAUNCHER_OPTIONS_WITH_VALUE or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
         return None
     position = 1
     if name == "rustup":
@@ -577,7 +644,8 @@
             position = 3
         else:
             return None
-    options_with_value, working_directory = LAUNCHER_OPTIONS_WITH_VALUE[name], None
+    options_with_value, working_directory, assignments = LAUNCHER_OPTIONS_WITH_VALUE[name], None, []
+    environment_options = LAUNCHER_ENVIRONMENT_OPTIONS.get(name, set())
     while position < len(words):
         word = words[position]
         if word == "--":
@@ -585,15 +653,16 @@
             break
         if not word.startswith("-") or word == "-":
             break
-        option, has_attached_value, attached_value = word.partition("=")
-        value = attached_value if has_attached_value else (words[position + 1] if position + 1 < len(words) else "")
+        option, value, word_count = launcher_option(word, words[position + 1] if position + 1 < len(words) else "", options_with_value)
         if name == "systemd-run" and option == "--working-directory":
             working_directory = value
-        position += 2 if option in options_with_value and not has_attached_value else 1
+        if option in environment_options and "=" in value and not break_is_set("launcher-drops-setenv"):
+            assignments.append(value)
+        position += word_count
     if name == "flock" and position + 1 < len(words) and words[position + 1] in ("-c", "--command"):
         return (["sh", "-c", words[position + 2]], working_directory) if position + 2 < len(words) else None
     position += LAUNCHER_POSITIONAL_COUNT.get(name, 0)
-    return (words[position:], working_directory) if position < len(words) else None
+    return (assignments + words[position:], working_directory) if position < len(words) else None
 
 
 def classify(words, directory, environment=None):
@@ -604,7 +673,7 @@
         return heavy_test(STAGE_KIND[stage], f"门禁 {stage} 号（{name}）") if stage in STAGE_KIND else None
     binary = test_binary_name(words[0])
     if binary is not None:
-        if LIST_ONLY_TEST_ARGUMENT in arguments:
+        if libtest_lists_only(arguments):
             return None
         if "layer0" in binary:
             return heavy_test("layer0-binary", f"直接执行名字含 layer0 的测试二进制（{binary}）")
@@ -691,9 +760,11 @@
 
 
 def build_sample_workspace(work):
-    """仓根与 research/ 两个虚工作区，harness 里有一个名字含 layer0 的测试目标；登记表登记三条崩溃枚举用例：
+    """仓根与 research/ 两个虚工作区，harness 里有一个名字含 layer0 的测试目标；登记表登记七条崩溃枚举用例：
     harness 里的 sample_crash_enumeration，与只有它一个测试目标的包 singlefs-checker 里的 checker_crash_enumeration（用例函数都标了 #[ignore]），
-    harness 里用例函数没标 #[ignore] 的 unmarked_crash_enumeration。"""
+    harness 里用例函数没标 #[ignore] 的 unmarked_crash_enumeration；harness 里另四种写法：同名函数 cfg 二选一、一份不标（cfg_split），
+    子模块里同名标 ignore 的写在前面、顶层那一份不标（submodule_first），宏生成的用例（macro，没有字面的 fn <名>(），
+    `# [ignore]`（# 与 [ 之间有空格，spaced_ignore，算标了）。"""
     def write(relative, text):
         path = os.path.join(work, relative)
         os.makedirs(os.path.dirname(path), exist_ok=True)
@@ -709,12 +780,24 @@
     marked_case = '#[test]\nfn quick_case() {}\n#[test]\n#[ignore = "崩溃枚举（样本）：平时不跑"]\nfn the_full_case() {}\n'
     write("crates/singlefs-harness/tests/sample_crash_enumeration.rs", marked_case)
     write("crates/singlefs-harness/tests/unmarked_crash_enumeration.rs", "#[test]\nfn the_unmarked_case() {}\n")
+    write("crates/singlefs-harness/tests/cfg_split_crash_enumeration.rs",
+          "#[cfg(debug_assertions)]\n#[test]\n#[ignore]\nfn the_split_case() {}\n#[cfg(not(debug_assertions))]\n#[test]\nfn the_split_case() {}\n")
+    write("crates/singlefs-harness/tests/submodule_first_crash_enumeration.rs",
+          "mod slow {\n    #[test]\n    #[ignore]\n    fn the_shadowed_case() {}\n}\n#[test]\nfn the_shadowed_case() {}\n")
+    write("crates/singlefs-harness/tests/macro_crash_enumeration.rs",
+          "macro_rules! crash_case {\n    ($name:ident) => {\n        #[test]\n        fn $name() {}\n    };\n}\ncrash_case!(the_generated_case);\n")
+    write("crates/singlefs-harness/tests/spaced_ignore_crash_enumeration.rs", "#[test]\n# [ignore]\nfn the_spaced_case() {}\n")
     write("crates/singlefs-checker/Cargo.toml", '[package]\nname = "singlefs-checker"\nversion = "0.0.0"\n')
     write("crates/singlefs-checker/tests/checker_crash_enumeration.rs", marked_case)
     write(CRASH_CASE_REGISTRY, "# 样本登记表\n54-layer0-replay.sh\tcrates/\t# 样本\n"
                                "crash-case:sample\tcrates/ Cargo.toml\ttest=singlefs-harness:sample_crash_enumeration:the_full_case\t# 样本\n"
                                "crash-case:checker\tcrates/ Cargo.toml\ttest=singlefs-checker:checker_crash_enumeration:the_full_case\t# 样本\n"
-                               "crash-case:unmarked\tcrates/ Cargo.toml\ttest=singlefs-harness:unmarked_crash_enumeration:the_unmarked_case\t# 样本\n")
+                               "crash-case:unmarked\tcrates/ Cargo.toml\ttest=singlefs-harness:unmarked_crash_enumeration:the_unmarked_case\t# 样本\n"
+                               "crash-case:cfg-split\tcrates/ Cargo.toml\ttest=singlefs-harness:cfg_split_crash_enumeration:the_split_case\t# 样本\n"
+                               "crash-case:submodule-first\tcrates/ Cargo.toml\t"
+                               "test=singlefs-harness:submodule_first_crash_enumeration:the_shadowed_case\t# 样本\n"
+                               "crash-case:macro\tcrates/ Cargo.toml\ttest=singlefs-harness:macro_crash_enumeration:the_generated_case\t# 样本\n"
+                               "crash-case:spaced-ignore\tcrates/ Cargo.toml\ttest=singlefs-harness:spaced_ignore_crash_enumeration:the_spaced_case\t# 样本\n")
     write("research/Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["e7-index-bench"]\n')
     write("research/e7-index-bench/Cargo.toml", '[package]\nname = "e7-index-bench"\nversion = "0.0.0"\n')
 
@@ -796,6 +879,35 @@
              work, None),
             ("环境变量里有 runner、点名不是崩溃枚举用例的目标", ["env", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/x", "cargo", "test", "-p", "singlefs-core",
                                                   "--test", "core_contract"], work, None),
+            # --config 的值按 TOML 读：带引号的键、点号两边带空白的 runner 正则认不出（开关 runner-configuration-by-regex-only 下这两格红）
+            ("--config 里带引号的键 target.<三元组>.\"runner\"、点名崩溃枚举用例不带 --ignored",
+             ["cargo", "--config", 'target.x86_64-unknown-linux-gnu."runner"="/tmp/add-ignored.sh"', "test", "-p", "singlefs-harness", "--test",
+              "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("--config 里点号两边带空白的 target . <三元组> . runner",
+             ["cargo", "--config", 'target . x86_64-unknown-linux-gnu . runner = "/tmp/add-ignored.sh"', "test", "-p", "singlefs-harness", "--test",
+              "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("--config 定的是 build.jobs、不是 runner", ["cargo", "--config", "build.jobs=4", "test", "-p", "singlefs-harness", "--test",
+                                                    "sample_crash_enumeration"], work, None),
+            # --list 是 --skip、--logfile 这类带值选项的值时照样全跑（开关 list-by-presence 下前两格红）
+            ("点名崩溃枚举用例、-- --include-ignored --skip --list（--list 是 --skip 的值）",
+             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--include-ignored", "--skip", "--list"], work, "crash-case-cargo"),
+            ("点名崩溃枚举用例、-- --include-ignored --exact the_full_case --logfile --list（--list 是日志文件名）",
+             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--include-ignored", "--exact", "the_full_case",
+              "--logfile", "--list"], work, "crash-case-cargo"),
+            ("点名崩溃枚举用例、-- --ignored --test-threads 4 --list（只列）",
+             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored", "--test-threads", "4", "--list"], work, None),
+            ("点名崩溃枚举用例、-- --ignored --skip=--list（一个词，没有 --list 选项）",
+             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored", "--skip=--list"], work, "crash-case-cargo"),
+            # 用例函数的每一处定义都要标 #[ignore]；判不出标没标的按没标算（开关：admission.py 的 first-definition-only、ignore-attribute-without-space，
+            # 这里的 undecided-ignore-allowed）
+            ("点名同名函数 cfg 二选一、一份不标的登记目标、不带 --ignored", ["cargo", "test", "--release", "-p", "singlefs-harness", "--test",
+                                                          "cfg_split_crash_enumeration"], work, "crash-case-cargo"),
+            ("点名子模块里同名标 ignore 写在前面、顶层那一份不标的登记目标、不带 --ignored",
+             ["cargo", "test", "-p", "singlefs-harness", "--test", "submodule_first_crash_enumeration"], work, "crash-case-cargo"),
+            ("点名用例是宏生成的（找不到字面的 fn <名>(）登记目标、不带 --ignored：判不出按没标算",
+             ["cargo", "test", "-p", "singlefs-harness", "--test", "macro_crash_enumeration"], work, "crash-case-cargo"),
+            ("点名用例函数标的是 # [ignore]（# 与 [ 之间有空格）的登记目标、不带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test",
+                                                                      "spaced_ignore_crash_enumeration"], work, None),
         ]
         # lib_shell_words 前缀表之外、包在命令外面照样起那条命令的程序（command_under_launcher）
         launcher_cases = [
@@ -823,6 +935,25 @@
             ("rustup show 不是 rustup run", ["rustup", "show"], work, None),
             ("perf report 不起命令", ["perf", "report", "-i", "perf.data"], work, None),
             ("/usr/bin/time 包一个不重型的测试目标", ["/usr/bin/time", "-v", "cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
+            # 短选项合写：逐个字母查表，头一个要值的字母之后剩下的是值、没剩下取下一个词（开关 launcher-whole-word-options 下前四格红）
+            ("strace -fo 文件（短选项合写）", ["strace", "-fo", "/tmp/s", "cargo", "test", "--all"], work, "full-cargo"),
+            ("flock -xw 10 锁文件（短选项合写）", ["flock", "-xw", "10", "/tmp/l", "cargo", "test", "--all"], work, "full-cargo"),
+            ("systemd-run --user --scope -qu 名字（短选项合写）", ["systemd-run", "--user", "--scope", "-qu", "k1", "cargo", "test", "--all"], work, "full-cargo"),
+            ("/usr/bin/time -ao 文件（短选项合写）", ["/usr/bin/time", "-ao", "/tmp/t", "cargo", "test", "--all"], work, "full-cargo"),
+            ("strace -o文件（值贴着写）", ["strace", "-o/tmp/s", "cargo", "test", "--all"], work, "full-cargo"),
+            # systemd-run -E / --setenv 设的变量带进里面那条命令（开关 launcher-drops-setenv 下前三格红）
+            ("systemd-run -E 设 runner、点名崩溃枚举用例不带 --ignored",
+             ["systemd-run", "--user", "--scope", "-E", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p",
+              "singlefs-harness", "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("systemd-run --setenv= 设 runner、点名崩溃枚举用例不带 --ignored",
+             ["systemd-run", "--user", "--scope", "--setenv=CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p",
+              "singlefs-harness", "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("systemd-run -qE 合写设 runner、点名崩溃枚举用例不带 --ignored",
+             ["systemd-run", "--user", "--scope", "-qE", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p",
+              "singlefs-harness", "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("systemd-run -E 设的不是 runner（值的末段叫 strace 也不当包装剥）、点名崩溃枚举用例不带 --ignored",
+             ["systemd-run", "--user", "--scope", "-E", "TRACER=/usr/bin/strace", "cargo", "test", "-p", "singlefs-harness", "--test",
+              "sample_crash_enumeration"], work, None),
         ]
         binary_cases = [
             ("绝对路径的层 0 测试二进制", [os.path.join(work, "target", "release", "deps", layer0_binary), "--test-threads", "4"], work, "layer0-binary"),
@@ -842,6 +973,11 @@
             ("崩溃枚举用例的测试二进制 --ignored --list", [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef"),
                                                   "--ignored", "--list"], work, None),
             ("层 0 测试二进制 --list", [os.path.join(work, "target", "release", "deps", layer0_binary), "--list"], work, None),
+            ("崩溃枚举用例的测试二进制 --include-ignored --skip --list（--list 是 --skip 的值）",
+             [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef"), "--include-ignored", "--skip", "--list"],
+             work, "crash-case-binary"),
+            ("层 0 测试二进制 --logfile --list（--list 是日志文件名）", [os.path.join(work, "target", "release", "deps", layer0_binary), "--logfile", "--list"],
+             work, "layer0-binary"),
             # 名字认不出（拷走改名、find -exec）时按参数认：带 --ignored 又点名登记的用例函数
             ("拷走改名的测试二进制带 --ignored --exact 登记的用例函数", ["/tmp/elsewhere/rc", "--ignored", "--exact", "the_full_case"], work, "crash-case-binary"),
             ("find -exec 执行测试二进制、点名登记的用例函数",
@@ -862,6 +998,17 @@
         for label, argv, cwd, want in cargo_cases + launcher_cases + binary_cases:
             found = classify_process(argv, cwd)
             results.append((label, want, found[0].kind if found else None))
+        # 导入不了 admission.py 时判不出标没标：点名登记目标、不带 --ignored 的 cargo test 按没标算（开关 undecided-ignore-allowed 下这一格红）
+        module_globals, saved_module_path = globals(), ADMISSION_MODULE_PATH
+        module_globals["ADMISSION_MODULE_PATH"] = os.path.join(work, "no-such-admission.py")
+        admission_module.cache_clear()
+        try:
+            found = classify_process(["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration"], work)
+        finally:
+            module_globals["ADMISSION_MODULE_PATH"] = saved_module_path
+            admission_module.cache_clear()
+        results.append(("导入不了 admission.py：点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 按没标算", "crash-case-cargo",
+                        found[0].kind if found else None))
         name_cases = [
             ("门禁阶段按名字判", ".claude/gate.d/12-no-prime-marks.sh", work, True),
             ("仓内位置上的 gate-staged.sh 按名字判", "research/scripts/gate-staged.sh", work, True),
@@ -908,13 +1055,16 @@
     for label, want, got in failures:
         print(f"  ✗ lib_heavy_tests 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
     if failures:
-        print("    → 看 classify()、cargo_subcommand()、cargo_use()、test_invocation()、crash_case_run()、command_under_launcher()、test_binary_name()、"
+        print("    → 看 classify()、cargo_subcommand()、cargo_use()、test_invocation()、crash_case_run()、command_under_launcher()、launcher_option()、"
+              "libtest_lists_only()、runner_of_test_binaries()、crash_case_function_runs_without_ignored()、test_binary_name()、"
               "runs_compiled_code()、classify_process()、judged_by_name() 与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / "
-              "KNOWN_SCRIPT_LOCATIONS / LAUNCHER_OPTIONS_WITH_VALUE 几张表；判用例函数标没标 #[ignore] 的那一段在 research/scripts/admission.py")
+              "KNOWN_SCRIPT_LOCATIONS / LAUNCHER_OPTIONS_WITH_VALUE / LIBTEST_OPTIONS_WITH_VALUE 几张表；判用例函数标没标 #[ignore] 的那一段在 "
+              f"research/scripts/admission.py；{BREAK_VARIABLE} 或 ADMISSION_BREAK 设着的话这里本来就该红")
         return 1
     print(f"  ✓ lib_heavy_tests 自检通过（查了 {len(results)} 种：cargo 与包装过的命令行 {len(cargo_cases)} 种、"
           f"/usr/bin/time、flock、systemd-run 这一类包在外面的 {len(launcher_cases)} 种、"
-          f"直接执行的测试二进制 {len(binary_cases)} 种、按名字判的脚本 {len(name_cases)} 种、跑不跑编译出来的代码 {len(compiled_cases)} 种）")
+          f"直接执行的测试二进制 {len(binary_cases)} 种、按名字判的脚本 {len(name_cases)} 种、跑不跑编译出来的代码 {len(compiled_cases)} 种、"
+          "导入不了 admission.py 1 种）")
     return 0
 
 
--- a/.claude/hooks/heavy-test-guard.sh
+++ b/.claude/hooks/heavy-test-guard.sh
@@ -21,7 +21,8 @@
 # 一条命令 100 秒没判完、拖慢每个 agent 的每条命令（同一张表第 18 行）：所以直接执行的只读 shell 脚本，同一份脚本一次判定里只读一遍、只判一遍。
 #
 # 重型测试，按类（只认命令位置；「cargo test」在下面各类里一样指 cargo t、cargo nextest run、cargo miri test、cargo llvm-cov、cargo hack test、
-# cargo mutants 与 --config / CARGO_ALIAS_ 定的别名；libtest 参数带 --list 的只列用例、一条都不跑，哪一类都不算）：
+# cargo mutants 与 --config / CARGO_ALIAS_ 定的别名；libtest 参数里 --list 当选项出现的只列用例、一条都不跑，哪一类都不算，
+# 跟在 --skip、--logfile 这类带值的选项后面的 --list 是那个选项的值，照算）：
 #   层 0          cargo test 会跑到名字含 layer0 的测试二进制：--test 的名字含 layer0、--test 的通配命中它、
 #                 或不带目标选择（带 --tests / --all-targets 也算）而包里有这种二进制；.claude/gate.d/54-*；
 #                 直接执行名字含 layer0 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）
@@ -36,8 +37,9 @@
 #   E152 装置     e152-file-system-benchmark（直接起、或 cargo run 它）、research/scripts/e152-run.sh
 #   崩溃枚举用例  .claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的那几条用例（门禁 54 号逐条跑）的测试目标——cargo test 点名它、--test 的通配命中它、
 #                 或不挑目标而包里有它；直接执行它的测试二进制——而下面任一条成立：libtest 参数带 --ignored 或 --include-ignored（nextest 是
-#                 --run-ignored only / all）；--config 或环境变量里定了测试二进制的 runner、或子命令是别名（带没带 --ignored 看不见）；
-#                 登记的用例函数没标 #[ignore]（读源码判，不带 --ignored 也跑到全量）。另按参数认：任何命令（grep、git 这类按文本处理参数的除外）
+#                 --run-ignored only / all）；--config 或环境变量里定了测试二进制的 runner（--config 的值另按 TOML 读；systemd-run -E / --setenv
+#                 设给里面那条命令的也算）、或子命令是别名（带没带 --ignored 看不见）；登记的用例函数有一处定义没标 #[ignore]，或判不出标没标
+#                 （读源码判，同名的每一处都算；找不到函数、宏生成的、导入不了 admission.py 按没标算）。另按参数认：任何命令（grep、git 这类按文本处理参数的除外）
 #                 带 --ignored / --include-ignored 又点名登记的用例函数（拷走改名的测试二进制、find -exec 起的）。判定与登记表取法在 lib_heavy_tests.py
 #   .claude/gate.d/ 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑、不用带前缀。
 # 谁、带什么才放行（都要带环境变量 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，别的值或没带一律拒）：
@@ -80,7 +82,8 @@
 #   命令词不带斜杠、从 PATH 里找到的脚本（直接执行的 `x.sh`）；同一条命令里用 WrittenScript 认不出的写法先写出再执行的脚本——
 #   执行时文件还不在的按「不存在」放行并记检出，已经在的读到的是旧内容；source 进来的文件里 export 的变量对外层后面命令的影响；
 #   .cargo/config.toml 与 `--config <文件>` 里定的别名与 runner；拷走改名、又不点名登记的用例函数的测试二进制（看门狗在进程这一层同样认不出）。
-# 接受的误拒：`--include-ignored --exact <同一目标里的快用例>` 照拒（过滤之后剩下哪几条，执行前判不出）；别名展开之后不是 test 的照拒。
+# 接受的误拒：`--include-ignored --exact <同一目标里的快用例>` 照拒（过滤之后剩下哪几条，执行前判不出）；别名展开之后不是 test 的照拒；
+#   同一目标里别的模块有与登记的用例函数同名、合法不标 #[ignore] 的快用例时照拒；admission.py 导入不了时点名登记目标的 cargo test 一律拒。
 # 另一道，与重型不重型无关：子 agent 跑编译出来的代码——cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算）、
 #   直接执行 cargo 编出来的二进制（lib_heavy_tests.runs_compiled_code）——要经 research/scripts/run-with-memory-cap.sh 跑，不经它的拒
 #   （它先判整机放不放得下、放不下排队，撞了上限只杀这一条；records/2026-09-16-subagent拆分提案.md 第四十节第 30 行）。放在这里而不另起一个 hook：
@@ -96,7 +99,8 @@
 #
 #   heavy-test-guard.sh             # 从 stdin 读 hook 的 JSON；拒绝退出 2，放行退出 0
 #   heavy-test-guard.sh --selftest  # 在临时工作区里走一遍拒绝与放行（连同 lib_heavy_tests.py 的自检）；HEAVY_TEST_GUARD_DISABLE_CHECK=1、
-#                                   # HEAVY_TEST_GUARD_IGNORE_WRITTEN_SCRIPTS=1（不认同一条命令里写出的脚本）、HEAVY_TEST_GUARD_IGNORE_MEMORY_CAP=1（不判经没经内存包装）时自检必须判红
+#                                   # HEAVY_TEST_GUARD_IGNORE_WRITTEN_SCRIPTS=1（不认同一条命令里写出的脚本）、HEAVY_TEST_GUARD_IGNORE_MEMORY_CAP=1（不判经没经内存包装）时自检必须判红；
+#                                   # 判定那一半的弄坏开关（lib_heavy_tests.py 的 LIB_HEAVY_TESTS_BREAK、admission.py 的 ADMISSION_BREAK，写法在两份的文件头）设了哪一个也必须判红
 set -uo pipefail
 HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
 # python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 write-guard.sh 同一个坑）。
@@ -767,6 +771,30 @@
              commit + "/usr/bin/time -v bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-harness --test sample_crash_enumeration "
              "-- --include-ignored --exact the_full_case", 0),
             ("主 agent 不带前缀 flock 包一层跑 gate.sh", None, "flock /tmp/gate.lock bash .claude/scripts/gate.sh --staged", 2),
+            # 短选项合写、systemd-run -E 设的 runner、--list 是别的选项的值、--config 里带引号的 runner 键、用例函数有一处没标或判不出：
+            # lib_heavy_tests.py 的 LIB_HEAVY_TESTS_BREAK 与 admission.py 的 ADMISSION_BREAK 那几个开关下这几格红（开关写在两份的文件头）
+            ("实现员内存包装里 strace -fo 包一层跑崩溃枚举用例", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G strace -fo /tmp/s cargo test -p singlefs-harness --test sample_crash_enumeration -- --ignored", 2),
+            ("实现员不经内存包装 strace -fo 包一层跑崩溃枚举用例", writer,
+             "strace -fo /tmp/s cargo test -p singlefs-harness --test sample_crash_enumeration -- --ignored", 2),
+            ("实现员内存包装里 flock -xw 10 包一层跑崩溃枚举用例", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G flock -xw 10 /tmp/l cargo test -p singlefs-harness --test sample_crash_enumeration -- --ignored", 2),
+            ("实现员内存包装里 systemd-run -E 设 runner、不带 --ignored", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G systemd-run --user --scope -E CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh "
+             "cargo test -p singlefs-harness --test sample_crash_enumeration", 2),
+            ("实现员内存包装里 --config 带引号的 runner 键、不带 --ignored", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo --config 'target.x86_64-unknown-linux-gnu.\"runner\"=\"/tmp/add-ignored.sh\"' test "
+             "-p singlefs-harness --test sample_crash_enumeration", 2),
+            ("实现员内存包装里 -- --include-ignored --skip --list（--list 是 --skip 的值）", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test sample_crash_enumeration -- --include-ignored --skip --list", 2),
+            ("实现员内存包装里直接执行崩溃枚举用例的测试二进制 --include-ignored --skip --list", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G ./target/release/deps/sample_crash_enumeration-0123456789abcdef --include-ignored --skip --list", 2),
+            ("实现员内存包装里跑同名函数 cfg 二选一、一份不标的登记目标（不带 --ignored）", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo test --release -p singlefs-harness --test cfg_split_crash_enumeration", 2),
+            ("实现员内存包装里跑用例是宏生成的登记目标（判不出，按没标算）", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test macro_crash_enumeration", 2),
+            ("实现员内存包装里跑用例函数标 # [ignore] 的登记目标（算标了，不带 --ignored 放行）", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test spaced_ignore_crash_enumeration", 0),
             # 起测试的外部子命令、别名与 runner、只列不跑、没标 #[ignore] 的登记用例
             ("实现员经内存包装 cargo nextest run --run-ignored all 跑崩溃枚举用例", writer,
              "bash research/scripts/run-with-memory-cap.sh 4G cargo nextest run -p singlefs-harness --test sample_crash_enumeration --run-ignored all", 2),
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -55,9 +55,9 @@
 - 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
 - 全部实验复跑：87 号。
 - E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
-- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标（`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制），而下面任一条成立的：libtest 参数带 `--ignored` 或 `--include-ignored`（`cargo nextest run` 是 `--run-ignored only` / `all`）；`--config` 或环境变量里定了测试二进制的 runner，或子命令是 `--config` / `CARGO_ALIAS_` 定的别名；登记的用例函数没标 `#[ignore]`。另外任何命令带 `--ignored` / `--include-ignored` 又点名登记的用例函数的也算（`grep`、`git` 这类按文本处理参数的除外）。不带这两个参数、用例函数标了 `#[ignore]`、只跑那个测试目标里快用例的不算。
+- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标（`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制），而下面任一条成立的：libtest 参数带 `--ignored` 或 `--include-ignored`（`cargo nextest run` 是 `--run-ignored only` / `all`）；`--config` 或环境变量里定了测试二进制的 runner（`--config` 的值按 TOML 读，带引号的键也算；`systemd-run -E` / `--setenv` 设给里面那条命令的环境变量也算），或子命令是 `--config` / `CARGO_ALIAS_` 定的别名；登记的用例函数有一处定义没标 `#[ignore]`（同名的每一处 `fn <名>(` 都算），或判不出标没标（找不到那个函数、宏生成的用例、导入不了 `research/scripts/admission.py`，按没标算）。另外任何命令带 `--ignored` / `--include-ignored` 又点名登记的用例函数的也算（`grep`、`git` 这类按文本处理参数的除外）。不带这两个参数、用例函数标了 `#[ignore]`、只跑那个测试目标里快用例的不算。
 
-只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算，`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf stat|record|trace` 包在外面的剥掉照算。重型测试清单各类里的 `cargo test` 同样指 `cargo nextest run`、`cargo miri test`、`cargo llvm-cov`、`cargo hack test`、`cargo mutants` 与别名；libtest 参数带 `--list`（只列用例、一条都不跑）的哪一类都不算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
+只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算，`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf stat|record|trace` 包在外面的剥掉照算（短选项合写的 `strace -fo <文件>`、`flock -xw 10` 同样剥）。重型测试清单各类里的 `cargo test` 同样指 `cargo nextest run`、`cargo miri test`、`cargo llvm-cov`、`cargo hack test`、`cargo mutants` 与别名；libtest 参数里 `--list` 当选项出现（只列用例、一条都不跑）的哪一类都不算，跟在 `--skip`、`--logfile` 这类带值的选项后面的 `--list` 是那个选项的值，照算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
 
 | 场合 | 跑不跑 |
 |---|---|
```

## 二、G2 diff（`research/prompts/defs-gatebatch-m2-g2-changes.diff`，1498 行，原样）

```diff
--- a/research/scripts/admission.py
+++ b/research/scripts/admission.py
@@ -18,14 +18,18 @@
                                           （例 herd7 的版本）。不在 git 树里的东西靠它进复用判定，见 ①
       正则、值、路径与参数里不许有空白（条件之间按空白切）。实验行写了门禁的种类、门禁行写了实验的种类，自查都判错。
   崩溃枚举用例行（键是 crash-case:<名>，门禁 54 号逐条按复用判定跑、逐条记全绿标记）：路径写整个 crates/ 与 Cargo 清单、锁，
-    不按用例手列；算输入时自动减去用例读不到的文件（见「崩溃枚举用例」一节）。第三列认四种：
+    不按用例手列；算输入时自动减去用例读不到的文件（见「崩溃枚举用例」一节）。第三列认五种：
       test=<包>:<测试目标>:<用例函数>     恰好一条：跑的是 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数>；
                                          用例函数的每一处定义都要标 #[ignore]（不带 --ignored 的 cargo test 不跑它，重型测试闸按这一条认它）：
                                          同名的每一处 fn <用例函数>( 都算（cfg 二选一的、子模块里同名的也算），有一处没标就算没标；
                                          # 与 [ 之间许空白；找不到字面的定义（宏生成的用例）判错
       count-line=<前缀>                  日志里以「<前缀> 」开头的行恰好一行，原样记进全绿标记
       exhaustive=<前缀>                  那一行带 exhaustive=true（前缀要先登记成 count-line=）
-      threads=<前缀>                     按那一行的 states= 找 LAYER0_PARALLEL_FINISHED 行判工作线程（前缀要先登记成 count-line=）
+      threads=<前缀>                     按那一行的 states= 找 LAYER0_PARALLEL_FINISHED 行判工作线程（前缀要先登记成 count-line=）；
+                                         那一行带 shards= 的（双机分片 merge 那一趟）逐片判，见 judge_threads_of_each_shard
+      shard=across-machines              至多一条：这条用例的枚举认双机分片开关（SINGLEFS_LAYER0_SHARD，crates/singlefs-harness/src/layer0_progress.rs），
+                                         门禁 54 号 --full 在本地分片配置可用时把它交给 research/scripts/layer0-shard-run.sh 两台各跑一片再 merge；
+                                         登记了它的用例，驱动脚本与它 eval 的配置判法（SHARD_DRIVER_FILES）按内容进这条用例的输入清单
 
 输入指纹里的构建环境（build_environment_lines；实验的指纹与 manifest / crash-case-manifest 带 --build-environment 时进清单）：
   仓根与它往上每一层目录的 .cargo/config、.cargo/config.toml、rust-toolchain、rust-toolchain.toml，CARGO_HOME（没设取 ~/.cargo）下的
@@ -66,6 +70,7 @@
                                  「<键><制表符><包><制表符><测试目标><制表符><用例函数>」；登记有错退 2，原因打在 stdout（不带 ✗，阶段自己打 ✗ 与出路）
   crash-case-manifest <根> <键> <清单文件> [--extra-file <名字> <文件>]... [--judging-digest] [--toolchain] [--build-environment]
                                  这条用例的输入清单：登记路径下的文件减去别的测试目标独占的测试文件，末尾按参数次序加名字行，
+                                 登记了 shard=across-machines 的再按内容加双机分片的驱动脚本与配置判法两行（SHARD_DRIVER_FILES，参数不管），
                                  最后一行是这条用例的登记行（键、路径与第三列）；stdout 打「<指纹> <文件数> <减去的文件数>」。
                                  --judging-digest 加一行判法摘要：这一份准入模块按 ast 从判法入口（CRASH_CASE_JUDGING_ENTRIES，连同分派表 COMMANDS 里
                                  CRASH_CASE_JUDGING_SUBCOMMANDS 那几项指的函数）顺着引用的模块级名字求闭包，闭包里每个定义的原文按名字排好，
@@ -73,7 +78,8 @@
   crash-case-command <根> <键> <输入指纹> [--start-over]
                                  --full 跑这一条的命令与环境（54 号原样执行）：stdout 以 NUL 分隔，本机核数、线程数、explicit|default、续跑的进度目录，
                                  其后是整条命令（env 设 SINGLEFS_LAYER0_PROGRESS_DIRECTORY、SINGLEFS_LAYER0_INPUT_FINGERPRINT、SINGLEFS_LAYER0_THREADS，
-                                 --start-over 时设 SINGLEFS_LAYER0_START_OVER=1、不然清掉它，再起 cargo test --release …）；线程数：调用方设了
+                                 --start-over 时设 SINGLEFS_LAYER0_START_OVER=1、不然清掉它，分片开关 SINGLEFS_LAYER0_SHARD 一律清掉，
+                                 再起 cargo test --release …）；线程数：调用方设了
                                  SINGLEFS_LAYER0_THREADS 用它（explicit），没设取 nproc（default）。交不出退 2，原因打在 stdout
   crash-case-marker-check <根> <键> <指纹> <清单文件>
                                  这批输入那一格全绿标记在不在、作不作数：退 0 作数（stdout 第一行「ok <标记路径> <跑完的时刻>」，
@@ -87,6 +93,7 @@
                                  crash-case-judge 记下的起了几个（登记了 threads= 的取 LAYER0_PARALLEL_FINISHED，没登记的写读不到）
   crash-case-marker-path <根> <键> <指纹>
                                  那一格全绿标记的路径（判红时阶段删它）
+  crash-case-shardable <根> <键>  这条用例登记了 shard=across-machines 没有：退 0 登记了、退 1 没登记（stdout 各一句）、退 2 登记有错
   gate-preconditions <阶段所在的仓根> <阶段文件名>
                                  门禁开跑前判第三列的前提（command=、readwrite=、probe=）：退 0 齐了（stdout 一个字不打）；
                                  退 1 没齐、退 2 登记有错，两种都在 stdout 打 ✗、明细与出路，阶段照判红。
@@ -106,7 +113,9 @@
 ignore-attribute-without-space（`# [ignore]` 不认）、concat-include-only（只认不带路径前缀的 include!(concat!(…))）、
 computed-include-subtracts-non-test-files（有算出来的 include 时照样减 mutations.tsv 与 src/bin/）、runner-program-only（runner / wrapper
 只按首词的程序进指纹）、whole-module-in-judging-digest（判法摘要换回整份准入模块）、judging-digest-without-dispatch（分派表那几项不进摘要）、
-single-worker-threads-field（标记里线程只记一格 worker_threads=）。
+single-worker-threads-field（标记里线程只记一格 worker_threads=）、threads-ignore-shards（merge 那一行带 shards= 也不逐片判，照旧看 n 片之和）、
+shardable-outside-judging-digest（分派表里 crash-case-shardable 那一项不进判法摘要）、shard-driver-outside-manifest（登记了 shard=across-machines 的
+用例，驱动脚本与配置判法不进输入清单）、keep-caller-shard-switch（crash-case-command 起用例时不清调用方环境里的 SINGLEFS_LAYER0_SHARD）。
 
 管不到的：跑的是不是按今天的源码编出来的二进制（指纹按源码算，跑的是 target/ 里的旧二进制时两边对不上，
 replay.sh 与 cargo run 开跑前都会重编）；登记的路径少写了一条（那条输入变了不会放行，要靠强制开关，登记行本身进指纹，
@@ -120,7 +129,8 @@
 runner / wrapper 指的程序与参数里的脚本按内容进指纹，它们再 source、再读的文件不进（不跑它就不知道它读什么）：runner 脚本 source 的那一份改了，
 指纹不变、旧标记照样作数。判法摘要看不见的：靠 ast 里的静态引用求闭包，getattr、字符串拼出来的函数名、exec 这一类引到的定义不进；
 Python 自身（解释器、标准库）升级不进。54 号不进指纹：它里面还定着结论的只剩流程的次序（开跑与跑完各算一次指纹、先判日志再写标记），
-改了它旧标记照样作数，由 --selftest 的「54 号」那几格核，快档的范围那一问照样把它算进去。
+改了它旧标记照样作数，由 --selftest 的「54 号」那几格核，快档的范围那一问照样把它算进去。双机分片的驱动脚本按内容进登记了 shard= 的用例，
+它 source 的 preflight.sh 这一类不进；分片关着（照单机跑）时驱动脚本照样在指纹里，改了它这两条照样重跑（多跑，接受）。
 用例函数标没标 #[ignore] 按同名的每一处 fn <名>( 判：同一个测试目标里别的模块有同名、合法不标 ignore 的快用例时误判没标（多拒、自查判错，
 方向是多跑一步，接受）。
 减得少的（改了照样让用例重跑）：几个测试目标共用的测试模块（tests/common_*/ 这一类）要顺着模块图才减得准，这里不走模块图；
@@ -169,7 +179,14 @@
 # 崩溃枚举用例（门禁 54 号逐条跑、逐条记全绿标记）
 CRASH_CASE_KEY_PREFIX = "crash-case:"
 CRASH_CASE_KEY_FORM = re.compile(r"^crash-case:[a-z0-9][a-z0-9-]*$")
-CRASH_CASE_CONDITION_FORM = re.compile(r"^(?P<kind>test|count-line|exhaustive|threads)=(?P<value>\S+)$")
+CRASH_CASE_CONDITION_FORM = re.compile(r"^(?P<kind>test|count-line|exhaustive|threads|shard)=(?P<value>\S+)$")
+# shard= 只认这一个值：两台机器各跑一片再 merge（双机分片，里程碑三第六项）
+CRASH_CASE_SHARD_ACROSS_MACHINES = "across-machines"
+# 登记了 shard=across-machines 的用例，54 号 --full 在分片开着时交给驱动脚本跑：它自己起两片与 merge 的 cargo、判两片与账本，不经 crash-case-command，
+# 起法不在判法摘要里；它 eval 配置判法打出的赋值。这两份按内容进这条用例的输入清单（路径从被判的仓根起；不在的进「找不到」一句）
+SHARD_DRIVER_FILES = ("research/scripts/layer0-shard-run.sh", "research/scripts/layer0-shard-configuration-check.sh")
+# crash-case-command 起用例时从调用方环境里清掉的分片开关：调用方设着它，单机跑的那一趟会只跑一片或去 merge
+LAYER0_SHARD_VARIABLE = "SINGLEFS_LAYER0_SHARD"
 CRASH_CASE_TEST_FORM = re.compile(r"^(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
 COUNT_LINE_PREFIX_FORM = re.compile(r"^[A-Z][A-Z0-9_]*$")
 CRASH_CASE_MARKER_PREFIX = "singlefs-crash-case-green."
@@ -186,8 +203,10 @@
                               "check_crash_case_marker", "write_crash_case_marker", "crash_case_marker_path", "parse_crash_case",
                               "crash_case_of_key", "crash_cases_of", "crash_case_launch", "started_worker_threads_text",
                               "configured_worker_threads_text")
-# 分派表 COMMANDS 里这几项子命令：「子命令 → 函数」进摘要，函数也当入口（有人把 crash-case-judge 指到别的函数，摘要跟着变）
-CRASH_CASE_JUDGING_SUBCOMMANDS = ("crash-case-command", "crash-case-judge", "crash-case-record", "crash-case-marker-check", "crash-case-marker-path")
+# 分派表 COMMANDS 里这几项子命令：「子命令 → 函数」进摘要，函数也当入口（有人把 crash-case-judge 指到别的函数，摘要跟着变）；
+# crash-case-shardable 定 54 号 --full 这一条单机跑还是交给双机分片的驱动脚本
+CRASH_CASE_JUDGING_SUBCOMMANDS = ("crash-case-command", "crash-case-judge", "crash-case-record", "crash-case-marker-check", "crash-case-marker-path",
+                                  "crash-case-shardable")
 # 原文进摘要、不顺着它的引用往下走的定义：main 引自证，顺下去自证就进来了
 CRASH_CASE_JUDGING_LEAVES = ("main",)
 CRASH_CASE_JUDGING_DIGEST_NAME = "<判法摘要：admission.py 里崩溃枚举用例的判法与起用例的命令>"
@@ -1095,7 +1114,8 @@
 #
 # 一条用例的输入 = 登记路径（整个 crates/ 与 Cargo 清单、锁）下 git 列得出的文件，减去用例读不到的文件，再加准入模块（这一份）里
 # 崩溃枚举用例的判法摘要、工具链、构建环境与这条用例的登记行（54 号按 --judging-digest、--toolchain、--build-environment 给；
-# 54 号自己不进：起用例的命令与环境由这一份的 crash-case-command 交出，在判法摘要里）。用例读不到的文件有三种：
+# 54 号自己不进：起用例的命令与环境由这一份的 crash-case-command 交出，在判法摘要里）；登记了 shard=across-machines 的另加双机分片的驱动脚本与
+# 配置判法（SHARD_DRIVER_FILES，按内容：分片开着时起两片与 merge 的是驱动脚本，不经 crash-case-command）。用例读不到的文件有三种：
 #   ① 别的测试目标 X 独占的文件：它所在的包没有构建脚本（build.rs 或 package.build）、没写 [[test]] 与 package.autotests，包里没有
 #      参数是算出来的 include! / include_str! / include_bytes!（COMPILE_TIME_COMPUTED_INCLUDE：套 concat!、env!、option_env! 或别的宏，
 #      宏名前带不带 ::core:: / std:: 都算），任何一个包的构建脚本（注释去掉之后）都不出现整词 tests；
@@ -1108,8 +1128,9 @@
 # 登记只写整个 crates/，不按用例手列它读哪些文件：新加的共用文件（tests/common.rs、build.rs、新拆出的 crate）默认就在输入里。
 
 class CrashCase:
-    def __init__(self, row, package, target, function, count_lines, exhaustive_lines, thread_lines):
+    def __init__(self, row, package, target, function, count_lines, exhaustive_lines, thread_lines, is_shardable_across_machines):
         self.row = row
+        self.is_shardable_across_machines = is_shardable_across_machines
         self.key = row.key
         self.name = row.key[len(CRASH_CASE_KEY_PREFIX):]
         self.package = package
@@ -1124,13 +1145,14 @@
 
 
 def parse_crash_case(row):
-    """崩溃枚举用例行的第三列：写法认不出、test= 不是恰好一条、exhaustive= / threads= 点名的前缀没登记成 count-line=，都抛 RegistrationError。"""
-    tests, count_lines, exhaustive_lines, thread_lines = [], [], [], []
+    """崩溃枚举用例行的第三列：写法认不出、test= 不是恰好一条、exhaustive= / threads= 点名的前缀没登记成 count-line=、
+    shard= 不是 across-machines 或多于一条，都抛 RegistrationError。"""
+    tests, count_lines, exhaustive_lines, thread_lines, shard_values = [], [], [], [], []
     for token in row.conditions:
         match = CRASH_CASE_CONDITION_FORM.match(token)
         if not match:
             raise RegistrationError(f"崩溃枚举用例 {row.key} 的第三列认不出 {token}（只认 test=<包>:<测试目标>:<用例函数>、"
-                                    "count-line=<前缀>、exhaustive=<前缀>、threads=<前缀>）")
+                                    f"count-line=<前缀>、exhaustive=<前缀>、threads=<前缀>、shard={CRASH_CASE_SHARD_ACROSS_MACHINES}）")
         kind, value = match.group("kind"), match.group("value")
         if kind == "test":
             test = CRASH_CASE_TEST_FORM.match(value)
@@ -1138,11 +1160,18 @@
                 raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token} 要写成 test=<包>:<测试目标>:<用例函数>（字母、数字、_，包名另许 -）")
             tests.append(test)
             continue
+        if kind == "shard":
+            if value != CRASH_CASE_SHARD_ACROSS_MACHINES:
+                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：shard= 只认 {CRASH_CASE_SHARD_ACROSS_MACHINES}")
+            shard_values.append(value)
+            continue
         if not COUNT_LINE_PREFIX_FORM.match(value):
             raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：前缀只许大写字母、数字、_，以字母开头")
         {"count-line": count_lines, "exhaustive": exhaustive_lines, "threads": thread_lines}[kind].append(value)
     if len(tests) != 1:
         raise RegistrationError(f"崩溃枚举用例 {row.key} 要恰好一条 test=，实际 {len(tests)} 条")
+    if len(shard_values) > 1:
+        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 shard= 至多一条，实际 {len(shard_values)} 条")
     if len(set(count_lines)) != len(count_lines):
         raise RegistrationError(f"崩溃枚举用例 {row.key} 的 count-line= 有重复的前缀")
     unregistered = [prefix for prefix in exhaustive_lines + thread_lines if prefix not in count_lines]
@@ -1150,7 +1179,7 @@
         raise RegistrationError(f"崩溃枚举用例 {row.key} 的 exhaustive= / threads= 点名的 {' '.join(unregistered)} 没有登记成 count-line=")
     test = tests[0]
     return CrashCase(row, test.group("package"), test.group("target"), test.group("function"),
-                     count_lines, exhaustive_lines, thread_lines)
+                     count_lines, exhaustive_lines, thread_lines, bool(shard_values))
 
 
 def crash_cases_of(rows):
@@ -1515,10 +1544,26 @@
     excluded = {os.fsencode(name) for name in excluded_names}
     kept = [name for name in listed_files if name not in excluded]
     registration = (f"<登记行：{case.key}>", case.row.text_with_conditions().encode("utf-8", "surrogateescape"))
-    manifest, fingerprint, file_count = manifest_of_files(root, kept, list(named_lines) + [registration])
+    manifest, fingerprint, file_count = manifest_of_files(root, kept, list(named_lines) + shard_driver_lines(root, case) + [registration])
     return manifest, fingerprint, file_count, sorted(excluded)
 
 
+def shard_driver_lines(root, case):
+    """登记了 shard=across-machines 的用例：SHARD_DRIVER_FILES 每一份一行 (名字, 内容)，排在登记行之前；没登记的交空表。
+    不在的那一份内容记「找不到 <路径>」（驱动脚本不在时 54 号照单机跑，指纹照样算得出）。弄坏开关 shard-driver-outside-manifest 下交空表。"""
+    if not case.is_shardable_across_machines or break_is_set("shard-driver-outside-manifest"):
+        return []
+    lines = []
+    for relative_path in SHARD_DRIVER_FILES:
+        try:
+            with open(os.path.join(root, relative_path), "rb") as handle:
+                content = handle.read()
+        except FileNotFoundError:
+            content = f"找不到 {relative_path}".encode("utf-8")
+        lines.append((f"<双机分片：{relative_path}>", content))
+    return lines
+
+
 def git_common_directory(root):
     return_code, output = git_output(root, "rev-parse", "--path-format=absolute", "--git-common-dir")
     common_directory = output.strip() if return_code == 0 else ""
@@ -1560,6 +1605,8 @@
                 f"{finished[0]}"), ""
     if freshly_run_slices > 0 and worker_threads == 0:
         return f"这一趟跑了 {freshly_run_slices} 片，却报起了 0 个工作线程：{finished[0]}", ""
+    if "shards" in fields and not break_is_set("threads-ignore-shards"):
+        return judge_threads_of_each_shard(prefix, finished[0], fields)
     if break_is_set("threads-by-worker-count"):
         judged_on_one_thread = worker_threads == 1
     else:
@@ -1574,6 +1621,42 @@
     return None, note
 
 
+# merge 那一行逐片报的数（crates/singlefs-harness/src/crash.rs 的 merge_the_shard_ledgers，逗号分隔、按第几片排）
+SHARD_FIELD_NAMES = ("worker_threads", "configured_worker_threads", "worker_threads_sources", "available_parallelism",
+                     "resumed_slices", "freshly_run_slices")
+
+
+def judge_threads_of_each_shard(prefix, finished_line, fields):
+    """双机分片 merge 那一趟的 LAYER0_PARALLEL_FINISHED（带 shards= 与逐片的 shard_…=）：每一片照单机的判法判。返回 (原因或 None, 一句说明)。
+    判红：逐片字段缺、片数不一、不是整数；某一片跑了片却报 0 个线程；某一片这一趟跑了至少两片、只起了 1 个工作线程、那台机器多于 1 核、
+    那一片的线程数不是显式设成 1（来源是环境变量、配的是 1）。机器核数与「显式设成 1」都取那一片账本里记的，不取 merge 这台的。"""
+    try:
+        shard_count = int(fields["shards"])
+        columns = {name: fields[f"shard_{name}"].split(",") for name in SHARD_FIELD_NAMES}
+        if shard_count < 1 or any(len(values) != shard_count for values in columns.values()):
+            raise ValueError("片数不一")
+        numbers = {name: [int(value) for value in values] for name, values in columns.items() if name != "worker_threads_sources"}
+    except (KeyError, ValueError):
+        return (f"{prefix} 对得上的 LAYER0_PARALLEL_FINISHED 带 shards=，逐片的 shard_…= 字段缺、片数与 shards= 不一或不是整数：{finished_line}"), ""
+    notes = []
+    for shard_index in range(shard_count):
+        spawned = numbers["worker_threads"][shard_index]
+        configured = numbers["configured_worker_threads"][shard_index]
+        source = columns["worker_threads_sources"][shard_index]
+        cores = numbers["available_parallelism"][shard_index]
+        resumed = numbers["resumed_slices"][shard_index]
+        freshly_run = numbers["freshly_run_slices"][shard_index]
+        shard_text = f"第 {shard_index}/{shard_count} 片"
+        if freshly_run > 0 and spawned == 0:
+            return f"{shard_text}跑了 {freshly_run} 片，却报起了 0 个工作线程：{finished_line}", ""
+        explicitly_one = source == "environment_variable" and configured == 1
+        if freshly_run >= 2 and spawned == 1 and cores > 1 and not explicitly_one:
+            return (f"{shard_text}：那台机器 {cores} 核、线程数没显式设成 1，跑了 {freshly_run} 片却只起了 1 个工作线程"
+                    f"（多半是线程数没传进去）：{finished_line}"), ""
+        notes.append(f"{shard_text} {spawned} 个线程跑了 {freshly_run} 片、读回 {resumed} 片（那台 {cores} 核）")
+    return None, f"{prefix}：双机分片 {shard_count} 片再 merge，{'；'.join(notes)}"
+
+
 def judge_crash_case_log(case, log_text, machine_cores, threads_explicitly_one):
     """判 --full 跑一条用例的日志：返回 (原因的清单, 要记进全绿标记的行, 线程那几句)。原因的清单为空才算判绿。"""
     log_lines = log_text.split("\n")
@@ -1761,14 +1844,17 @@
     返回 (本机核数, 线程数, "explicit" 或 "default", 续跑的进度目录, 整条命令的词)。
     命令是 env 设好续跑的三个变量与线程数再起 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数> --nocapture：
     进度目录 <git common-dir>/singlefs-layer0-progress/<这批输入的指纹>、SINGLEFS_LAYER0_INPUT_FINGERPRINT=<指纹>；start_over 时
-    SINGLEFS_LAYER0_START_OVER=1，不然从调用方的环境里清掉它。取不到 common-dir、线程数取不到抛 InputManifestError。"""
+    SINGLEFS_LAYER0_START_OVER=1，不然从调用方的环境里清掉它；调用方环境里的分片开关 SINGLEFS_LAYER0_SHARD 一律清掉（这一趟单机跑整条流，
+    分片只由 research/scripts/layer0-shard-run.sh 自己设）。取不到 common-dir、线程数取不到抛 InputManifestError。"""
     common_directory = git_common_directory(root)
     if common_directory is None:
         raise InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），续跑的进度文件没处放")
     machine_cores, threads, threads_origin = crash_case_worker_threads(environment)
     progress_directory = os.path.join(common_directory, LAYER0_PROGRESS_DIRECTORY_NAME, fingerprint)
     start_over_setting = [f"{LAYER0_START_OVER_VARIABLE}=1"] if start_over else ["-u", LAYER0_START_OVER_VARIABLE]
-    command = ["env", *start_over_setting, f"SINGLEFS_LAYER0_PROGRESS_DIRECTORY={progress_directory}",
+    # env 的 -u 要写在第一个 NAME=VALUE 之前（之后的 -u 被当成要起的命令）
+    shard_setting = [] if break_is_set("keep-caller-shard-switch") else ["-u", LAYER0_SHARD_VARIABLE]
+    command = ["env", *shard_setting, *start_over_setting, f"SINGLEFS_LAYER0_PROGRESS_DIRECTORY={progress_directory}",
                f"SINGLEFS_LAYER0_INPUT_FINGERPRINT={fingerprint}", f"{LAYER0_THREADS_VARIABLE}={threads}",
                "cargo", "test", "--release", "-p", case.package, "--test", case.target,
                "--", "--include-ignored", "--exact", case.function, "--nocapture"]
@@ -1805,13 +1891,15 @@
     tree = ast.parse(source)
     lines = source.split("\n")
     definitions = top_level_definitions(tree)
+    subcommands = tuple(name for name in CRASH_CASE_JUDGING_SUBCOMMANDS
+                        if not (name == "crash-case-shardable" and break_is_set("shardable-outside-judging-digest")))
     dispatch = {}
     for node in definitions.get("COMMANDS", []):
         if isinstance(node, ast.Assign) and isinstance(node.value, ast.Dict):
             for key, value in zip(node.value.keys, node.value.values):
-                if isinstance(key, ast.Constant) and key.value in CRASH_CASE_JUDGING_SUBCOMMANDS:
+                if isinstance(key, ast.Constant) and key.value in subcommands:
                     dispatch[key.value] = ast.unparse(value)
-    missing = [name for name in CRASH_CASE_JUDGING_SUBCOMMANDS if name not in dispatch]
+    missing = [name for name in subcommands if name not in dispatch]
     if missing:
         raise InputManifestError(f"准入模块的分派表 COMMANDS 里没有 {'、'.join(missing)}：判法摘要算不全")
     wanted = set()
@@ -1825,7 +1913,7 @@
             queue += [inner.id for inner in ast.walk(node) if isinstance(inner, ast.Name) and inner.id in definitions and inner.id not in wanted]
     pieces = [f"## {name}\n" + "\n".join(source_of_node(lines, node) for node in definitions[name]) for name in sorted(wanted)]
     if not break_is_set("judging-digest-without-dispatch"):
-        pieces.append("## 分派\n" + "".join(f"{name} → {dispatch[name]}\n" for name in CRASH_CASE_JUDGING_SUBCOMMANDS))
+        pieces.append("## 分派\n" + "".join(f"{name} → {dispatch[name]}\n" for name in subcommands))
     pieces += [f"## {name}\n" + "\n".join(source_of_node(lines, node) for node in definitions.get(name, [])) for name in CRASH_CASE_JUDGING_LEAVES]
     pieces += ["## 入口\n" + source_of_node(lines, node) for node in tree.body
                if isinstance(node, ast.If) and "__name__" in ast.unparse(node.test)]
@@ -2149,6 +2237,24 @@
     return 0
 
 
+def command_crash_case_shardable(arguments):
+    if len(arguments) != 2:
+        print("  ✗ 用法：admission.py crash-case-shardable <项目根> <键>")
+        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写")
+        return EXIT_REGISTRATION_ERROR
+    root, key = arguments
+    try:
+        case = crash_case_of_key(read_registration_rows(root), key)
+    except RegistrationError as error:
+        print(f"找不到 {key}：{error}")
+        return EXIT_REGISTRATION_ERROR
+    if case.is_shardable_across_machines:
+        print(f"{key} 登记了 shard={CRASH_CASE_SHARD_ACROSS_MACHINES}")
+        return 0
+    print(f"{key} 没登记 shard={CRASH_CASE_SHARD_ACROSS_MACHINES}：单机跑")
+    return 1
+
+
 def command_crash_case_marker_path(arguments):
     if len(arguments) != 3:
         print("  ✗ 用法：admission.py crash-case-marker-path <项目根> <键> <指纹>")
@@ -2387,6 +2493,7 @@
         repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
         real_problems = lint_registration_rows(read_registration_rows(repository_root), repository_root)
         selftest.expect(f"真仓的登记表 {REGISTRATION_TABLE} 自查没有问题", not real_problems, "；".join(real_problems))
+        run_real_crash_case_row_cells(selftest, read_registration_rows(repository_root))
     finally:
         shutil.rmtree(work, ignore_errors=True)
 
@@ -2397,7 +2504,8 @@
     print(f"  ✓ admission.py 自证通过：{selftest.checked} 格都对（含弄坏开关 skip-unchanged、skip-preconditions、skip-gate-preconditions、"
           "ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count、"
           "first-definition-only、ignore-attribute-without-space、concat-include-only、computed-include-subtracts-non-test-files、runner-program-only、"
-          "whole-module-in-judging-digest、judging-digest-without-dispatch、single-worker-threads-field "
+          "whole-module-in-judging-digest、judging-digest-without-dispatch、single-worker-threads-field、threads-ignore-shards、"
+          "shardable-outside-judging-digest、shard-driver-outside-manifest、keep-caller-shard-switch "
           "下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）")
     return 0
 
@@ -2594,6 +2702,22 @@
 REPLAY_STRIP_HEADER_COMMAND = "sed '/^E7INPUT /d'"
 
 
+# 临时仓里跑真仓脚本的副本时要从真仓链进来的两样：规范副本与项目的 preflight 垫片（.claude/scripts/preflight.sh）所在的目录
+LINKED_INTO_SELFTEST_REPOSITORIES = (".claude/singlefs-ai-sop", ".claude/scripts")
+
+
+def link_sop_copy(work):
+    """临时仓里跑真仓脚本的副本（replay.sh、54 号与它们调的 stage-must-run.sh 这一类）时，副本开头按相对路径 source .claude/scripts/preflight.sh，
+    垫片再找规范副本里的 preflight.sh：LINKED_INTO_SELFTEST_REPOSITORIES 那两样在临时仓里建成指向真仓那一份的符号链接（缺了 source 落空、
+    preflight 一步没判，随后的 set -- 把参数清空），并写进临时仓的 .gitignore（不算进改动范围）。"""
+    os.makedirs(os.path.join(work, ".claude"), exist_ok=True)
+    real_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
+    with open(os.path.join(work, ".gitignore"), "a", encoding="utf-8") as handle:
+        for relative_path in LINKED_INTO_SELFTEST_REPOSITORIES:
+            os.symlink(os.path.join(real_root, relative_path), os.path.join(work, relative_path))
+            handle.write(relative_path + "\n")
+
+
 def replay_copy_text(table_rows, break_header_strip):
     """真仓 replay.sh 的副本：只换掉复跑表、在表后面拼上桩驱动；break_header_strip 时把比对前删产物头那一步换成 cat。
     replay.sh 的形状变了（表的起止、删头那一句不是恰好一处）抛 RegistrationError，那一格判错。"""
@@ -2629,6 +2753,7 @@
             write_text(os.path.join(work, f"research/src/e{number}.txt"), f"输入 {number}\n")
             write_text(os.path.join(work, RESULTS_DIRECTORY, f"e{number}-2026-09-01.out"), stored_body)
         write_questions(work, "开着")
+        link_sop_copy(work)
         run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
         run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
         table_rows = [f"E{number}|@driver_e{number}||e{number}-2026-09-01.out|exact" for number in ("900", "901", "902")]
@@ -3220,6 +3345,57 @@
     selftest.expect("判绿时记 started_worker_threads=：没登记 threads= 的记「读不到」（门禁不读它起了几个）",
                     not problems and len(started) == 1 and started[0].startswith("started_worker_threads=读不到：这条用例没登记 threads="),
                     f"判出 {problems}，记下 {started}")
+    run_shard_merge_log_cells(selftest, case)
+
+
+def synthetic_merge_log(shard_worker_threads=(16, 16), shard_configured=(16, 16), shard_sources=("environment_variable", "environment_variable"),
+                        shard_cores=(32, 32), shard_resumed=(0, 0), shard_freshly_run=(32, 32), shards=None):
+    """合成的双机分片 merge 那一趟日志：LAYER0_PARALLEL_FINISHED 的合计照 crates/singlefs-harness/src/crash.rs 的 merge_the_shard_ledgers
+    （线程与片数是各片之和、来源写 shard_ledgers），另带 shards= 与逐片的 shard_…=。"""
+    joined = lambda values: ",".join(str(value) for value in values)
+    finished_fields = (f"worker_threads={sum(shard_worker_threads)} configured_worker_threads={sum(shard_configured)} worker_threads_source=shard_ledgers "
+                       f"resumed_slices={sum(shard_resumed)} freshly_run_slices={sum(shard_freshly_run)} "
+                       f"shards={shards if shards is not None else len(shard_worker_threads)} shard_worker_threads={joined(shard_worker_threads)} "
+                       f"shard_configured_worker_threads={joined(shard_configured)} shard_worker_threads_sources={joined(shard_sources)} "
+                       f"shard_available_parallelism={joined(shard_cores)} shard_resumed_slices={joined(shard_resumed)} "
+                       f"shard_freshly_run_slices={joined(shard_freshly_run)} shard_elapsed_milliseconds=1000,1000")
+    return synthetic_layer0_log(finished_fields=finished_fields)
+
+
+def run_shard_merge_log_cells(selftest, case):
+    """双机分片 merge 那一趟的日志：带 shards= 的 LAYER0_PARALLEL_FINISHED 逐片判工作线程（机器核数与「显式设成 1」取那一片自己的）；
+    登记行的 shard= 只认 across-machines、至多一条。"""
+    cells = [
+        ("merge：两片各 16 个线程各跑 32 片", synthetic_merge_log(), True),
+        ("merge：第 1 片那台 32 核、线程数没显式设，跑了 32 片却只起 1 个线程",
+         synthetic_merge_log(shard_worker_threads=(16, 1), shard_configured=(16, 32), shard_sources=("environment_variable", "available_parallelism")), False),
+        ("merge：第 1 片只起 1 个线程，那一片的线程数显式设成 1",
+         synthetic_merge_log(shard_worker_threads=(16, 1), shard_configured=(16, 1)), True),
+        ("merge：第 1 片只起 1 个线程，那台只有 1 核", synthetic_merge_log(shard_worker_threads=(16, 1), shard_configured=(16, 1),
+                                                          shard_sources=("environment_variable", "available_parallelism"), shard_cores=(32, 1)), True),
+        ("merge：第 1 片跑了 32 片却报起了 0 个线程", synthetic_merge_log(shard_worker_threads=(16, 0)), False),
+        ("merge：shards=3 而逐片字段只有两片", synthetic_merge_log(shards=3), False),
+    ]
+    for label, log_text, expected_green in cells:
+        problems, _recorded, notes = judge_crash_case_log(case, log_text, 32, False)
+        green = not problems
+        selftest.expect(f"判 --full 日志：{label} ⇒ {'绿' if expected_green else '红'}",
+                        green == expected_green and (not green or any("双机分片 2 片再 merge" in note for note in notes)),
+                        f"判成{'绿' if green else '红'}：{problems}；线程那一句 {notes}")
+    with BreakSwitch("threads-ignore-shards"):
+        problems, _recorded, _notes = judge_crash_case_log(case, cells[1][1], 32, False)
+    selftest.expect("弄坏开关 threads-ignore-shards 下「第 1 片只起 1 个线程跑了 32 片」那一格判绿（只看各片之和 17 个线程，分不出是哪一片只起了 1 个）",
+                    not problems, f"弄坏之后仍判红：{problems}")
+    for label, conditions, expected in (
+            ("登记了 shard=across-machines", ["test=pkg:target:function", "shard=across-machines"], True),
+            ("没登记 shard=", ["test=pkg:target:function"], False),
+            ("shard= 写成别的值", ["test=pkg:target:function", "shard=yes"], "error"),
+            ("shard= 写了两条", ["test=pkg:target:function", "shard=across-machines", "shard=across-machines"], "error")):
+        try:
+            outcome = parse_crash_case(RegistrationRow("crash-case:demo", ["crates/"], conditions, 1, "")).is_shardable_across_machines
+        except RegistrationError:
+            outcome = "error"
+        selftest.expect(f"崩溃枚举用例登记行：{label} ⇒ {expected}", outcome == expected, f"读成 {outcome}")
 
 
 def edit_inside_definition(source, name, old, new):
@@ -3236,6 +3412,8 @@
     return head + "\n" + body.replace(old, new) + "\n" + tail
 
 
+# command_crash_case_shardable 里「没登记」那一支的末两行（自证把 return 1 改成 return 0 造一个判法改动）
+SHARDABLE_ANSWER_UNREGISTERED = '：单机跑")\n    return 1'
 # (说明, 改哪个模块级定义, 旧串, 新串, 判法摘要该不该变)：前四种是判法之外的改动（K4 那一轮量过它们让四条用例全重跑），后几种是判法本身
 JUDGING_DIGEST_VARIANTS = [
     ("实验准入：admit_experiment 里加一行注释", "admit_experiment", "def admit_experiment(root, key):\n", "def admit_experiment(root, key):\n    # 样本\n",
@@ -3252,6 +3430,8 @@
     ("起用例的命令：crash_case_launch 把 --include-ignored 换成 --ignored", "crash_case_launch", '"--include-ignored"', '"--ignored"', True),
     ("标记里配的线程那一格：configured_worker_threads_text 改字", "configured_worker_threads_text", "本机 {machine_cores} 核", "{machine_cores} 核", True),
     ("main：分派的写法改了", "main", "return COMMANDS[arguments[0]](arguments[1:])", "return COMMANDS[arguments[0]](arguments[2:])", True),
+    ("单机跑还是双机分片：command_crash_case_shardable 把「没登记 shard=」也答成登记了", "command_crash_case_shardable",
+     SHARDABLE_ANSWER_UNREGISTERED, SHARDABLE_ANSWER_UNREGISTERED.replace("return 1", "return 0"), True),
 ]
 
 
@@ -3286,6 +3466,12 @@
         broken_baseline, broken_commented = crash_case_judging_digest_text(source)[0], crash_case_judging_digest_text(commented)[0]
     selftest.expect("弄坏开关 whole-module-in-judging-digest 下「实验准入加一行注释」那一格红（摘要变了）", broken_baseline != broken_commented,
                     "弄坏之后摘要照样没变：这一格分不出摘要是不是整份模块")
+    shardable_answer = edit_inside_definition(source, "command_crash_case_shardable", SHARDABLE_ANSWER_UNREGISTERED,
+                                              SHARDABLE_ANSWER_UNREGISTERED.replace("return 1", "return 0"))
+    with BreakSwitch("shardable-outside-judging-digest"):
+        broken_baseline, broken_answer = crash_case_judging_digest_text(source)[0], crash_case_judging_digest_text(shardable_answer)[0]
+    selftest.expect("弄坏开关 shardable-outside-judging-digest 下「crash-case-shardable 把没登记答成登记了」那一格红（摘要没变）",
+                    broken_baseline == broken_answer, "弄坏之后摘要照样变了：这一格分不出 crash-case-shardable 进没进摘要")
 
 
 # 假 cargo：-V 打版本；test 带 --include-ignored 的记下目标、过滤、续跑的三个环境变量，照控制目录里那条目标的日志与退出码打；
@@ -3314,7 +3500,7 @@
 exit "$(cat "$FAKE_CARGO_CONTROL/exit.$target" 2>/dev/null || echo 0)"
 '''
 LAYER0_STAGE_CASES = [
-    ("crash-case:stream-a", "first_transaction_step_seven_layer0", "stream_a_full", "count-line=LAYER0 exhaustive=LAYER0 threads=LAYER0"),
+    ("crash-case:stream-a", "first_transaction_step_seven_layer0", "stream_a_full", "count-line=LAYER0 exhaustive=LAYER0 threads=LAYER0 shard=across-machines"),
     ("crash-case:stream-b", "second_transaction_step_zero_layer0", "stream_b_full", "count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B"),
     ("crash-case:case-c", "case_c", "case_c_full", ""),
 ]
@@ -3355,6 +3541,7 @@
         os.makedirs(os.path.dirname(stage_copy))
         shutil.copy(real_stage, stage_copy)
         write_text(os.path.join(work, ".gitignore"), ".control/\n.tools/\n")
+        link_sop_copy(work)
         tools = os.path.join(work, ".tools")
         control = os.path.join(work, ".control")
         os.makedirs(control)
@@ -3406,6 +3593,8 @@
         set_logs()
         exit_code, output, calls = stage("--full")
         runs = full_runs(calls)
+        selftest.expect("54 号 --full：没有本地分片配置（临时仓里没有 layer0-shard-configuration-check.sh 判得过的配置）⇒ 打一行「双机分片：关」、逐条单机跑",
+                        "双机分片：关" in output, f"输出尾部：{output.strip()[-600:]}")
         fingerprints = {key: expected_fingerprint(key) for key, _target, _function, _conditions in LAYER0_STAGE_CASES}
         progress_settings_ok = all(
             call[3] == os.path.join(common_directory, "singlefs-layer0-progress", fingerprints[key]) and call[4] == fingerprints[key] and call[5] == "unset"
@@ -3516,6 +3705,9 @@
                         exit_code == 1 and "跑的过程中它的输入变了" in output and "crates/singlefs-harness/src/lib.rs" in output
                         and not any(".stream-a." in name for name in markers()), f"退 {exit_code}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")
 
+        run_layer0_stage_shard_cells(selftest, work, control, stage, full_runs, markers, set_logs, expected_fingerprint, common_directory)
+        run_crash_case_command_shard_switch_cells(selftest, work)
+
         # 范围那一问：只改登记表、只改 54 号、只改准入模块的改动，快档不带 SINGLEFS_GATE_FULL=1 照样核标记（不退 77）
         set_logs()
         stage("--full")
@@ -3566,6 +3758,128 @@
         shutil.rmtree(work, ignore_errors=True)
 
 
+# 假的分片配置检查：退 0（能分片）；假的驱动脚本：--merged-log 时记下参数与续跑开关，把控制目录里 stream-a 那一份日志写进日志文件，退出码照控制目录
+FAKE_SHARD_CONFIGURATION_CHECK = "#!/usr/bin/env bash\necho \"样本配置，第二台是样本\"\n"
+FAKE_SHARD_DRIVER = r'''#!/usr/bin/env bash
+printf '%s\t%s\n' "$*" "${SINGLEFS_LAYER0_START_OVER-unset}" >> "$FAKE_CARGO_CONTROL/driver-invocations"
+[[ "$1" == --merged-log ]] || exit 9
+cat "$FAKE_CARGO_CONTROL/log.first_transaction_step_seven_layer0" > "$5"
+exit "$(cat "$FAKE_CARGO_CONTROL/driver-exit" 2>/dev/null || echo 0)"
+'''
+
+
+C561_SIGMA_STATES = 262144
+
+
+def c561_sigma_log(exhaustive="true", worker_threads=32):
+    """合成的 crash-case:c561-sigma-full 一趟 --full 日志：用例打的计数行与同形的线程行（照 crates/singlefs-harness/tests/
+    record_checker_judges_absence_by_the_persisted_set.rs 的 count_line 与线程行）。"""
+    return (f"LAYER0_PARALLEL_FINISHED states={C561_SIGMA_STATES} slices=64 worker_threads={worker_threads} configured_worker_threads=32 "
+            f"worker_threads_source=available_parallelism resumed_slices=0 freshly_run_slices=64 progress_file_after_completion=none elapsed_seconds=1.0\n"
+            f"C561_SIGMA_FULL states={C561_SIGMA_STATES} closed_form_states={C561_SIGMA_STATES} exhaustive={exhaustive} "
+            f"record_claimed_state_missing_unit=0\n{PASSED_ONE_LINE}\n")
+
+
+def run_real_crash_case_row_cells(selftest, rows):
+    """真仓登记表里两条崩溃枚举用例的第三列拿合成日志判：c561-sigma-full 的计数行不带 exhaustive=true、1 个线程跑了 64 片都判红，两样都对判绿
+    （只登记 count-line= 时这两种都判绿）；crash-injection-fast-tier 登记着，计数行 CRASH_INJECTION_FINISHED 恰好一行判绿、两行判红。"""
+    fast_tier_line = "CRASH_INJECTION_FINISHED seeds=[1,25) slices=24 worker_threads=4 elapsed_seconds=1.0\n"
+    cells = [
+        ("crash-case:c561-sigma-full", "两行都对", c561_sigma_log(), True),
+        ("crash-case:c561-sigma-full", "计数行 exhaustive=false（状态数不等于闭式）", c561_sigma_log(exhaustive="false"), False),
+        ("crash-case:c561-sigma-full", "本机 32 核、线程数没显式设，1 个线程跑了 64 片", c561_sigma_log(worker_threads=1), False),
+        ("crash-case:crash-injection-fast-tier", "计数行恰好一行", fast_tier_line + PASSED_ONE_LINE + "\n", True),
+        ("crash-case:crash-injection-fast-tier", "计数行打了两行", fast_tier_line * 2 + PASSED_ONE_LINE + "\n", False),
+    ]
+    for key, label, log_text, expected_green in cells:
+        try:
+            problems = judge_crash_case_log(crash_case_of_key(rows, key), log_text, 32, False)[0]
+        except RegistrationError as error:
+            problems = [f"真仓登记表里读不出这一条：{error}"]
+        green = not problems
+        selftest.expect(f"真仓登记的 {key}：{label} ⇒ {'绿' if expected_green else '红'}", green == expected_green,
+                        f"判成{'绿' if green else '红'}：{problems}")
+
+
+def shard_switch_seen_by_the_case(work, caller_shard_switch):
+    """照 crash_case_launch 交的命令起一趟（cargo 起往后换成打 SINGLEFS_LAYER0_SHARD 的 sh），调用方环境里设着 caller_shard_switch：交回那一趟看到的值。"""
+    case = crash_case_of_key(read_registration_rows(work), "crash-case:stream-b")
+    environment = dict(os.environ, SINGLEFS_LAYER0_THREADS="2", SINGLEFS_LAYER0_SHARD=caller_shard_switch)
+    command = crash_case_launch(work, case, "0" * 64, False, environment)[4]
+    launched = command[:command.index("cargo")] + ["sh", "-c", 'printf "%s" "${SINGLEFS_LAYER0_SHARD-unset}"']
+    return subprocess.run(launched, env=environment, capture_output=True, text=True, check=False).stdout
+
+
+def run_crash_case_command_shard_switch_cells(selftest, work):
+    """crash-case-command 交的命令起用例时清掉调用方环境里的 SINGLEFS_LAYER0_SHARD（单机跑的那一趟不许只跑一片或去 merge）；
+    弄坏开关 keep-caller-shard-switch 下那一格转红。"""
+    seen = shard_switch_seen_by_the_case(work, "0/2")
+    selftest.expect("crash-case-command：调用方环境里设着 SINGLEFS_LAYER0_SHARD=0/2 ⇒ 起的用例看不到它", seen == "unset", f"用例看到的是「{seen}」")
+    with BreakSwitch("keep-caller-shard-switch"):
+        seen = shard_switch_seen_by_the_case(work, "0/2")
+    selftest.expect("弄坏开关 keep-caller-shard-switch 下「调用方设着分片开关」那一格红（用例看到 0/2）", seen == "0/2",
+                    f"弄坏之后用例看到的是「{seen}」：这一格分不出命令清没清它")
+
+
+def run_layer0_stage_shard_cells(selftest, work, control, stage, full_runs, markers, set_logs, expected_fingerprint, common_directory):
+    """54 号 --full 在分片开着时：登记了 shard=across-machines 的 stream-a 交给驱动脚本 --merged-log（参数是键、树根、这批输入的指纹、日志文件），
+    另两条照旧单机跑；驱动脚本交回的日志照单机的判法判、写同一格标记；驱动脚本退非 0 判红、删那一格。假的检查与驱动脚本放进临时仓，跑完删掉。"""
+    check_path = os.path.join(work, "research/scripts/layer0-shard-configuration-check.sh")
+    driver_path = os.path.join(work, "research/scripts/layer0-shard-run.sh")
+    invocations_path = os.path.join(control, "driver-invocations")
+    rows = read_registration_rows(work)
+    sharded_case, unsharded_case = crash_case_of_key(rows, "crash-case:stream-a"), crash_case_of_key(rows, "crash-case:stream-b")
+
+    def fingerprints_of_both():
+        return crash_case_manifest(work, sharded_case, [])[1], crash_case_manifest(work, unsharded_case, [])[1]
+
+    try:
+        before = fingerprints_of_both()
+        with BreakSwitch("shard-driver-outside-manifest"):
+            broken_before = fingerprints_of_both()
+        write_executable(check_path, FAKE_SHARD_CONFIGURATION_CHECK)
+        write_executable(driver_path, FAKE_SHARD_DRIVER)
+        after = fingerprints_of_both()
+        with BreakSwitch("shard-driver-outside-manifest"):
+            broken_after = fingerprints_of_both()
+        selftest.expect("输入清单：放进双机分片的驱动脚本与配置判法 ⇒ 登记了 shard=across-machines 的 stream-a 指纹变、没登记的 stream-b 不变",
+                        before[0] != after[0] and before[1] == after[1], f"stream-a {before[0][:16]} → {after[0][:16]}，stream-b {before[1][:16]} → {after[1][:16]}")
+        selftest.expect("弄坏开关 shard-driver-outside-manifest 下「放进驱动脚本」那一格红（stream-a 指纹没变）", broken_before[0] == broken_after[0],
+                        "弄坏之后 stream-a 的指纹照样变了：这一格分不出驱动脚本进没进清单")
+        for name in markers():
+            os.remove(os.path.join(common_directory, name))
+        set_logs()
+        exit_code, output, calls = stage("--full")
+        driver_calls = []
+        if os.path.exists(invocations_path):
+            with open(invocations_path, encoding="utf-8") as handle:
+                driver_calls = [line.split("\t") for line in handle.read().split("\n") if line]
+            os.remove(invocations_path)
+        expected_arguments = f"--merged-log crash-case:stream-a {work} {expected_fingerprint('crash-case:stream-a')} "
+        selftest.expect("54 号 --full 分片开着：打一行「双机分片：开」；stream-a（登记了 shard=across-machines）交给驱动脚本 --merged-log，"
+                        "另两条单机跑；三条都判绿、各写一格",
+                        exit_code == 0 and "双机分片：开" in output and len(driver_calls) == 1 and driver_calls[0][0].startswith(expected_arguments)
+                        and driver_calls[0][1] == "unset"
+                        and [call[0] for call in full_runs(calls)] == ["second_transaction_step_zero_layer0", "case_c"] and len(markers()) == 3,
+                        f"退 {exit_code}，驱动脚本被调 {driver_calls}，cargo 跑了 {full_runs(calls)}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")
+        for name in [name for name in markers() if ".stream-a." in name]:
+            os.remove(os.path.join(common_directory, name))
+        write_text(os.path.join(control, "driver-exit"), "1\n")
+        exit_code, output, calls = stage("--full")
+        os.remove(os.path.join(control, "driver-exit"))
+        if os.path.exists(invocations_path):
+            os.remove(invocations_path)
+        selftest.expect("54 号 --full 分片开着：驱动脚本退非 0 ⇒ stream-a 判红、不写标记，另两条复用",
+                        exit_code == 1 and "stream-a" in output and not any(".stream-a." in name for name in markers()) and not full_runs(calls),
+                        f"退 {exit_code}，cargo 跑了 {full_runs(calls)}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")
+    finally:
+        for path in (check_path, driver_path):
+            if os.path.exists(path):
+                os.remove(path)
+    set_logs()
+    stage("--full")
+
+
 # manifest 子命令（不带 --build-environment）同一套步骤的另一份实现，只给自证当对照：它不 import 本模块，用的是 sha256sum 与 sort。
 INDEPENDENT_MANIFEST_SCRIPT = r'''
 set -uo pipefail
@@ -3597,6 +3911,7 @@
     "crash-case-judge": command_crash_case_judge,
     "crash-case-record": command_crash_case_record,
     "crash-case-marker-path": command_crash_case_marker_path,
+    "crash-case-shardable": command_crash_case_shardable,
 }
 
 
@@ -3605,7 +3920,8 @@
         return run_selftest()
     if not arguments or arguments[0] not in COMMANDS:
         say("  ✗ 用法：admission.py {gate-reuse|gate-preconditions|gate-record-environment|experiment|paths|manifest|keys|crash-cases|"
-            "crash-case-manifest|crash-case-marker-check|crash-case-command|crash-case-judge|crash-case-record|crash-case-marker-path} … 或 --selftest")
+            "crash-case-manifest|crash-case-marker-check|crash-case-command|crash-case-judge|crash-case-record|crash-case-marker-path|"
+            "crash-case-shardable} … 或 --selftest")
         say("     → 怎么办：各子命令的参数见文件头")
         return EXIT_REGISTRATION_ERROR
     return COMMANDS[arguments[0]](arguments[1:])
--- a/.claude/gate.d/stage-inputs.tsv
+++ b/.claude/gate.d/stage-inputs.tsv
@@ -13,9 +13,11 @@
 #   崩溃枚举用例（键是 crash-case:<名>）：门禁 54 号逐条按复用判定跑、逐条记全绿标记（在 git common-dir，按用例与它的输入指纹分格），
 #     输入没变复用、变了才重跑。路径用排除法写：只写整个 crates/ 与 Cargo 清单、锁，不按用例手列它读哪些文件——admission.py 算输入时
 #     自动减去用例读不到的文件（别的测试目标独占的测试文件、crates/mutations.tsv、没有代码读 CARGO_BIN_EXE_ 时的 src/bin/；判法见它的
-#     「崩溃枚举用例」一节），再加判它的 54 号、准入模块 admission.py、工具链、构建环境与这一行本身。
-#     第三列：test=<包>:<测试目标>:<用例函数> 恰好一条（用例函数要标 #[ignore]），count-line= / exhaustive= / threads= 定日志怎么判
-#     （写法见 admission.py 文件头）：用例跑起来打的行里有的才登记，打不出的那一项不登记、在注释里写明缺哪一项。
+#     「崩溃枚举用例」一节），再加准入模块 admission.py 里崩溃枚举用例的判法摘要、工具链、构建环境与这一行本身；登记了 shard=across-machines 的
+#     另按内容加双机分片的驱动脚本 research/scripts/layer0-shard-run.sh 与配置判法 layer0-shard-configuration-check.sh。54 号不进。
+#     第三列：test=<包>:<测试目标>:<用例函数> 恰好一条（用例函数要标 #[ignore]），count-line= / exhaustive= / threads= 定日志怎么判，
+#     shard=across-machines 定 54 号 --full 在分片配置可用时能不能交给驱动脚本两台各跑一片（写法见 admission.py 文件头）：
+#     用例跑起来打的行里有的才登记，打不出的那一项不登记、在注释里写明缺哪一项。
 #
 # ⚠️ 这份清单**自己也进门禁的比对**（stage-must-run.sh 无条件把它加进路径列表）。少写一条输入，那条输入就永远
 # 不会让这道阶段重跑——而清单本身变了必定重跑，改清单的代价因此是「下一趟全跑一次」，不是零；加一个实验行也一样。
@@ -23,7 +25,7 @@
 #
 # 宁宽勿窄：多写一条路径只会多跑几趟，少写一条会让一次真的改动被跳过。拿不准就写上。
 # 只写从仓库根起的路径；目录写成带斜杠的前缀（git diff 与 git ls-files 的路径限定都按前缀匹配）。
-54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/admission.py	command=cargo command=rustc	# 跑 crates 的崩溃点重放；research/scripts/admission.py 是它判日志、算每条崩溃枚举用例的输入指纹、读写全绿标记的准入模块（它也进每条用例的指纹），改了它这一道不许复用上一次的判定；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）。这一行管整道阶段的复用判定与改动范围；它逐条跑的崩溃枚举用例各自的输入登记在本表 crash-case: 开头的那几行。前提：cargo 与 rustc（两档都要 cargo -V && rustc -V 算输入指纹，快档与 --full 都起 cargo test）
+54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/admission.py research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-configuration-check.sh	command=cargo command=rustc	# 跑 crates 的崩溃点重放；research/scripts/admission.py 是它判日志、算每条崩溃枚举用例的输入指纹、读写全绿标记、交起用例命令的准入模块（进每条用例指纹的是它里面崩溃枚举用例的判法摘要，不是整份），改了它这一道不许复用上一次的判定；layer0-shard-run.sh 与 layer0-shard-configuration-check.sh 是 --full 在分片配置可用时跑登记了 shard=across-machines 的用例的驱动脚本与配置判法（按内容进那几条用例的指纹），改了它们快档同样要核标记；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）。这一行管整道阶段的复用判定与改动范围；它逐条跑的崩溃枚举用例各自的输入登记在本表 crash-case: 开头的那几行。前提：cargo 与 rustc（两档都要 cargo -V && rustc -V 算输入指纹，快档与 --full 都起 cargo test）
 55-qemu-first-transaction.sh	crates/ Cargo.toml Cargo.lock research/scripts/ research/results/	command=qemu-system-x86_64 readwrite=/dev/kvm probe=research/scripts/vm-kernel.sh:--check	# 起虚机跑 crates 的装置，虚机与复跑脚本都在 research/scripts/；这几条路径同时是改动范围（change-touches-crates.sh）的前缀。前提是 .claude/kb/vm-harness.md「三个前置」：缺 QEMU 装 qemu-system-x86；/dev/kvm 不可读写查 kvm 组成员身份，不许用 setfacl 补；找不到可读的内核镜像就跑 bash research/scripts/vm-kernel.sh，按它打印的路径设 SINGLEFS_KERNEL
 57-lkmm.sh	litmus/ .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/	probe=.claude/scripts/lkmm.sh:--herd7-version environment=.claude/scripts/lkmm.sh:--herd7-version	# 判 litmus/ 下每条 Never 的对照组、代码绑定与 herd7 判定。lkmm.sh 是判法，它 source 的 lib.sh、找不到内核树时调的 fetch-deps.sh 一并登记；crates/ 整个取：litmus 的 singlefs-models 锚点今天指 crates/singlefs-core/src/transaction.rs 与 recovery.rs，读 litmus 文件名的测试在 crates/singlefs-harness/tests/publish_order_matches_litmus.rs，而 lkmm.sh 按文件名在全部 crates/**/*.rs 里找测试、新加一条 litmus 的锚点可以指到 crates/ 任何一处，只登记这三份会让下一条新锚点的改动被跳过（宁宽勿窄）。herd7 的版本不在 git 树里，经第三列 environment= 进复用判定（lkmm.sh --herd7-version 打的那一行，57 号判绿之后记进 git common-dir）；前提是找得到 herd7（probe= 同一个入口，PATH 里没有就试 opam 的环境；缺了 opam install herdtools7）。内核树的 tools/memory-model 不进判定，靠 stage-must-run.sh 的 24 小时复用上限兜
 59-crates-mutation-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh	command=cargo	# 按 crates/mutations.tsv 逐条改坏 crates/ 里的源码再跑点名的测试；每条经 run-with-memory-cap.sh 带内存上限跑，它判不判得出撞顶也是这一道的判据。前提：cargo（装 Rust 工具链，bash .claude/scripts/env.sh 会报缺什么）
@@ -31,7 +33,8 @@
 87-replay.sh	crates/ Cargo.toml Cargo.lock research/e7-index-bench/ research/scripts/ research/results/	# 复跑入库的实验产物，装置在 research/e7-index-bench/，登记表与脚本在 research/scripts/
 E142	research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs research/e7-index-bench/src/lib.rs research/e7-index-bench/Cargo.toml research/Cargo.toml research/Cargo.lock research/mutations/e142_first_transaction_dry_run.tsv research/results/e142-first-txn-dry-run-2026-09-25-r16-arm-n15.out .claude/kb/decisions/08-核心索引结构.md .claude/kb/decisions/13-验证路线.md .claude/kb/decisions/16-发布语义.md .claude/kb/decisions/18-块里携带什么信息.md .claude/kb/decisions/23-journal的角色与格式.md .claude/kb/layout/01-first-txn.md .claude/kb/decisions/22-单元原子性怎么合成.md .claude/kb/decisions/15-格式冻结政策.md .claude/kb/feature-bits.md research/e7-index-bench/src/bin/e142_region_diff_independent.rs research/mutations/e142_region_diff_independent.tsv crates/ Cargo.toml Cargo.lock	# E142（第一个事务的干跑）默认调用（replay.sh 的 driver_e142 那一种）。装置源码、它 use 的 e7_index_bench 库、research 的 Cargo 清单与锁、变异表；driver_e142 传给装置的臂 N15 参照产物；replay.sh 不进（跑完把登记行指到新产物就是改它，进了指纹，新产物一存进来就对不上自己；驱动换了参数而这几条路径没变时要靠强制开关）；跑前登记 research/prompts/e142-r17-prereg.md 第二节被测条款所在的 D8（核心索引结构）、D13（验证路线）、D16（发布语义）、D18（块里携带什么信息）、D23（journal 的角色与格式）五份决策正文，加上宽度对账逐格抄的 layout/01（跑前登记第二节没列它，宁宽勿窄加上）；第十八次跑的跑前登记 research/prompts/e142-r18-prereg.md 第二节另加被测条款 D22（单元原子性怎么合成）、D15（格式冻结政策）与 feature-bits.md，那一次用独立比对 bin e142_region_diff_independent 判改前改后，它与它的变异表一并登记（2026-09-26 主 agent 按设计员报的漏列补上）；crates/ 整个取：driver_e142 编的 e142_first_transaction_write_dump 在 singlefs-harness，它依赖 checker、core、format 另外三个 crate，四个就是整个 crates/，多出来的只有 crates/mutations.tsv；按文件精确取要跟着模块图走，漏一个模块就是该跑的不跑
 E142/layer0	@E142	question-row=research/prompts/m2-keyspace-rerun-questions.md#6:够判[：:][^（(|]*对照(本身)?是好的 product-field=E142:verdict:control_violations_ok=true product-field=E142:verdict:positive_control_main_geometry_ok=true	# E142 第二段（设了 E142_LAYER0_MAIN 的调用：主臂层 0 整轮），输入与默认调用相同。前提照跑前登记 research/prompts/e142-r17-prereg.md 6.2、6.3：问题单第 6 行判成「对照是好的」之后才跑，且 Q142.26 caught = 12（最新产物判决行 positive_control_main_geometry_ok）；control_violations_ok 是第 6 行够判那一格的计数判定
-crash-case:layer0-first-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:first_transaction_step_seven_layer0:layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0	# 第一条流（mkfs → 取号 → 暖机 → A）的层 0 全量，带断点续跑；54 号 --full 原来整批跑的两条之一。LAYER0 是计数行、要 exhaustive=true，CHECKER 是逐条不变量行
-crash-case:layer0-second-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_step_zero_layer0:full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B	# 第二条流（里程碑「第二个事务」固定脚本到 E 再正常卸载）的层 0 全量，带断点续跑；54 号 --full 原来整批跑的两条之二
+crash-case:layer0-first-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:first_transaction_step_seven_layer0:layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0 shard=across-machines	# 第一条流（mkfs → 取号 → 暖机 → A）的层 0 全量，带断点续跑、可双机分片（shard=across-machines：枚举经 Layer0Resume::from_environment 认 SINGLEFS_LAYER0_SHARD）；54 号 --full 原来整批跑的两条之一。LAYER0 是计数行、要 exhaustive=true，CHECKER 是逐条不变量行
+crash-case:layer0-second-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_step_zero_layer0:full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B shard=across-machines	# 第二条流（里程碑「第二个事务」固定脚本到 E 再正常卸载）的层 0 全量，带断点续跑、可双机分片（同第一条流）；54 号 --full 原来整批跑的两条之二
 crash-case:floor-raise-pushed-by-the-session	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_crash_inside_the_floor_raise_pushed_by_the_session:crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED	# 崩在会话推的那一串抬 F 中间：按层 0 的枚举域枚举那一段、恢复之后与再挂载之后各判池级 checker（标了 ignore，release 跑）；不留进度文件，计数由用例自己断言。用例自己不打计数行，count-line= 登记的是它经 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0_in_state_slices 打的那一行 LAYER0_PARALLEL_FINISHED（只调一次，恰好一行），threads= 按它判枚举那一段的工作线程（挂载那一遍自己起线程，不在判里）；没登记 exhaustive=：那一行不带 exhaustive=true
-crash-case:c561-sigma-full	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:record_checker_judges_absence_by_the_persisted_set:every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present count-line=C561_SIGMA_FULL	# C561（记录核对器的复用豁免在复用只落一半时假红） 的 σ 全量：σ 那一段 2^18 个状态记录核对器一个都不判缺席（标了 ignore，release 跑）；没登记 exhaustive= 与 threads=：计数行 C561_SIGMA_FULL 不带 exhaustive=true（状态数由用例断言等于 262144），用例自己起线程、不经 enumerate_layer0_in_state_slices，不打 LAYER0_PARALLEL_FINISHED
+crash-case:c561-sigma-full	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:record_checker_judges_absence_by_the_persisted_set:every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present count-line=C561_SIGMA_FULL exhaustive=C561_SIGMA_FULL threads=C561_SIGMA_FULL	# C561（记录核对器的复用豁免在复用只落一半时假红） 的 σ 全量：σ 那一段 2^18 个状态记录核对器一个都不判缺席（标了 ignore，release 跑）；计数行 C561_SIGMA_FULL 在评过的状态数等于闭式 2^18（crash::closed_form_state_count 只对 σ 算）时带 exhaustive=true；用例按掩码区间切片多线程跑、自己打一行与 LAYER0_PARALLEL_FINISHED 同形的线程行（不留进度文件：resumed_slices=0），threads= 按它判；不经 enumerate_layer0_in_state_slices（那一路每个状态多跑一遍不看 journal 的恢复与池级 checker、状态集合差一个）
+crash-case:crash-injection-fast-tier	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_supplement_three_crash_injection:crash_injection_fast_tier_recovers_only_into_versions_the_model_committed count-line=CRASH_INJECTION_FINISHED	# 随机崩溃注入快档（种子基起 24 段、每段 24 步、每段 4 个崩溃状态）：每个崩溃状态上只读恢复与判定、再起可写挂载发一次布跑 checker、挂载途中取号/写行/暖机各一个二次崩溃（代码审阅第 2 条）；debug 下单跑一百来秒，标了 ignore，release 跑；普通 cargo test 由同文件的小快档（头 4 段）守。计数行 CRASH_INJECTION_FINISHED 只调一次、恰好一行；不打 LAYER0_PARALLEL_FINISHED，没登记 threads=；抽样，没登记 exhaustive=
--- a/.claude/hooks/lib_heavy_tests.py
+++ b/.claude/hooks/lib_heavy_tests.py
@@ -12,7 +12,8 @@
       words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序（/usr/bin/time、flock、rustup run、chrt、prlimit、systemd-run、strace、perf）时，
       剥掉它交回它起的那条命令与那条命令的当前目录（None 是不变）；调用方把交回的命令再交给 lib_shell_words 切一遍、逐条判。
       短选项合写（`-fo <文件>`、`-xw 10`、`-qu <名>`）逐个字母查那张表：头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；
-      systemd-run 的 `-E NAME=VALUE` / `--setenv=NAME=VALUE` 设的变量写在交回那条命令的最前面（再切一遍时当它的环境变量，认 runner 要用）。
+      systemd-run 的 `-E NAME=VALUE` / `--setenv=NAME=VALUE`、strace 的 `-E NAME=VALUE` / `--env=NAME=VALUE` 设的变量写在交回那条命令的最前面
+      （再切一遍时当它的环境变量，认 runner 要用；strace 的 `-E NAME` 只清变量，不带）。
       这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
   classify_process(argv, cwd) -> list[HeavyTest]
       一个在跑的进程：argv 是 /proc/<pid>/cmdline 按 NUL 切开的那一串，cwd 是 /proc/<pid>/cwd 指向的目录。
@@ -36,7 +37,7 @@
   libtest 参数带 --ignored 或 --include-ignored（nextest 是 --run-ignored 的值不是 default）；
   可能带上它而看不见：cargo 全局选项 --config 里定了别名（alias.<名>，且子命令就是那个别名）或 runner（按正则认 .runner =，
   另把值按 TOML 读，定了 target.<任何>.runner 也算：带引号的键 "runner" 这一类），
-  命令看得到的环境变量里有 CARGO_TARGET_*_RUNNER（systemd-run -E / --setenv 设给里面那条命令的也算），或 CARGO_ALIAS_<名> 定的别名就是子命令；
+  命令看得到的环境变量里有 CARGO_TARGET_*_RUNNER（systemd-run -E / --setenv、strace -E / --env 设给里面那条命令的也算），或 CARGO_ALIAS_<名> 定的别名就是子命令；
   登记的用例函数有一处定义没标 #[ignore]，或判不出标没标（找不到那个目标、那个函数，宏生成的用例，读不了源码，导入不了 admission.py：都按没标算）。
   读那个包里测试目标的源码，判法与 research/scripts/admission.py 的 crash-cases 自查同一份：本模块按文件路径导入它，同名的每一处 fn <名>( 都判。
 另有一条按参数认：任何命令（按文本处理参数的 grep、git、sed 这一类除外）参数里有 --ignored 或 --include-ignored、又有登记的用例函数名，
@@ -50,7 +51,7 @@
 弄坏开关（只给自证用，证明那几格会红）：LIB_HEAVY_TESTS_BREAK 设成下面一个或几个（逗号分隔），--selftest 与 heavy-test-guard.sh --selftest 都必须判红：
   launcher-whole-word-options（短选项合写不拆，照旧按整词查表）、launcher-drops-setenv（systemd-run -E / --setenv 设的变量不带进里面那条命令）、
   list-by-presence（见到 --list 这个词就算只列）、undecided-ignore-allowed（判不出标没标时放行）、
-  runner-configuration-by-regex-only（--config 只按正则认 runner）。
+  runner-configuration-by-regex-only（--config 只按正则认 runner）、strace-drops-env（strace -E / --env 设的变量不带进里面那条命令）。
 """
 import fnmatch, functools, glob, importlib.util, os, re, shlex, shutil, sys, tempfile, tomllib
 from typing import NamedTuple
@@ -91,6 +92,7 @@
     "gate-staged.sh": "research/scripts/gate-staged.sh", "mutate.sh": "research/scripts/mutate.sh",
     "vm-bench.sh": "research/scripts/vm-bench.sh", "e152-run.sh": "research/scripts/e152-run.sh",
     "capped.sh": "research/scripts/capped.sh", "run-with-memory-cap.sh": "research/scripts/run-with-memory-cap.sh",
+    "layer0-shard-run.sh": "research/scripts/layer0-shard-run.sh",
 }
 
 
@@ -592,7 +594,7 @@
                     "--uid", "--gid", "--nice", "--working-directory", "--on-active", "--on-boot", "--on-startup", "--on-unit-active",
                     "--on-unit-inactive", "--on-calendar", "--path-property", "--socket-property", "--timer-property", "--service-type",
                     "-C", "--capsule"},
-    "strace": {"-a", "-b", "-e", "-E", "-I", "-o", "-O", "-p", "-P", "-s", "-S", "-u", "-U", "-X"},
+    "strace": {"-a", "-b", "-e", "-E", "--env", "-I", "-o", "-O", "-p", "-P", "-s", "-S", "-u", "-U", "-X"},
     "perf": {"-e", "--event", "-o", "--output", "-p", "--pid", "-t", "--tid", "-C", "--cpu", "-r", "--repeat", "-I", "--interval-print",
              "-G", "--cgroup", "-x", "--field-separator", "-F", "--freq", "-c", "--count", "-m", "--mmap-pages", "-u", "--uid", "-j",
              "--branch-filter", "--call-graph", "-D", "--delay", "-M", "--metrics"},
@@ -601,7 +603,7 @@
 # 选项之后、那条命令之前还有几个位置参数：flock 的锁文件、chrt 的优先级、rustup run 的工具链
 LAUNCHER_POSITIONAL_COUNT = {"flock": 1, "chrt": 1, "rustup": 1}
 # 给里面那条命令设环境变量的选项（值是 NAME=VALUE）：设的变量写在交回那条命令的最前面
-LAUNCHER_ENVIRONMENT_OPTIONS = {"systemd-run": {"-E", "--setenv"}}
+LAUNCHER_ENVIRONMENT_OPTIONS = {"systemd-run": {"-E", "--setenv"}, "strace": {"-E", "--env"}}
 # 交回的命令以 NAME=VALUE 打头时，那是设的变量、不是程序（值里的路径 basename 碰巧叫 strace 也不当包装剥）
 ENVIRONMENT_ASSIGNMENT_WORD = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
 # perf 起一条命令的子命令；后面还要再跟一个 record 的那几组
@@ -627,7 +629,8 @@
 def command_under_launcher(words):
     """words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序时，剥掉它（连同它的选项与位置参数），交回 (它起的那条命令的词, 那条命令的当前目录：
     None 是不变，systemd-run --working-directory 给的是相对当前目录的写法)；不是这几样、或后面没有要起的命令（strace -p、flock 只给 fd）交 None。
-    选项词怎么切见 launcher_option；systemd-run 的 -E / --setenv 设的 NAME=VALUE 写在交回那条命令的最前面。
+    选项词怎么切见 launcher_option；LAUNCHER_ENVIRONMENT_OPTIONS 里的选项（systemd-run 的 -E / --setenv、strace 的 -E / --env）设的 NAME=VALUE
+    写在交回那条命令的最前面。
     flock 的 -c <字符串> 交回成 sh -c <字符串>。rustup 只认 rustup run，perf 只认 stat / record / trace 与 <组> record。"""
     name = os.path.basename(words[0])
     if name not in LAUNCHER_OPTIONS_WITH_VALUE or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
@@ -645,7 +648,7 @@
         else:
             return None
     options_with_value, working_directory, assignments = LAUNCHER_OPTIONS_WITH_VALUE[name], None, []
-    environment_options = LAUNCHER_ENVIRONMENT_OPTIONS.get(name, set())
+    environment_options = set() if name == "strace" and break_is_set("strace-drops-env") else LAUNCHER_ENVIRONMENT_OPTIONS.get(name, set())
     while position < len(words):
         word = words[position]
         if word == "--":
@@ -700,6 +703,9 @@
         return heavy_test("gate-sh", " ".join(["gate.sh", *arguments]))
     if name == "gate-staged.sh":
         return None if "--selftest" in arguments else heavy_test("gate-staged", "research/scripts/gate-staged.sh（跑 gate.sh --staged）")
+    if name == "layer0-shard-run.sh":
+        return None if "--selftest" in arguments else heavy_test(
+            "crash-case-cargo", "research/scripts/layer0-shard-run.sh（两台各跑一片登记的崩溃枚举用例、带 --include-ignored，再本机 merge）")
     if name in ("e152-file-system-benchmark", "e152-run.sh"):
         return heavy_test("e152", name)
     if name == "cargo":
@@ -820,6 +826,10 @@
             ("经 nice 包一层", ["/usr/bin/nice", "-n", "19", "cargo", "test", "--workspace"], work, "full-cargo"),
             ("bash -c 里", ["bash", "-c", "cd crates/singlefs-harness && cargo test"], work, "layer0-cargo"),
             ("bash 起 54 号", ["bash", ".claude/gate.d/54-layer0-replay.sh", "--full", "/tmp/wt"], work, "layer0-stage"),
+            ("bash 起双机分片的驱动脚本", ["bash", "research/scripts/layer0-shard-run.sh", "crash-case:sample", "/tmp/wt"], work, "crash-case-cargo"),
+            ("双机分片的驱动脚本 --merged-log", ["bash", "research/scripts/layer0-shard-run.sh", "--merged-log", "crash-case:sample", "/tmp/wt", "0" * 64, "/tmp/log"],
+             work, "crash-case-cargo"),
+            ("双机分片的驱动脚本 --selftest 不重型", ["bash", "research/scripts/layer0-shard-run.sh", "--selftest"], work, None),
             ("cargo run E152", ["cargo", "run", "--release", "--bin", "e152-file-system-benchmark"], os.path.join(work, "research"), "e152"),
             ("一个不是层 0 的测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
             ("clippy --all-targets", ["cargo", "clippy", "--all-targets"], work, None),
@@ -954,6 +964,22 @@
             ("systemd-run -E 设的不是 runner（值的末段叫 strace 也不当包装剥）、点名崩溃枚举用例不带 --ignored",
              ["systemd-run", "--user", "--scope", "-E", "TRACER=/usr/bin/strace", "cargo", "test", "-p", "singlefs-harness", "--test",
               "sample_crash_enumeration"], work, None),
+            # strace -E / --env 设的变量同样带进里面那条命令（开关 strace-drops-env 下前四格红）
+            ("strace -E 设 runner、点名崩溃枚举用例不带 --ignored",
+             ["strace", "-f", "-E", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness",
+              "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("strace --env= 设 runner、点名崩溃枚举用例不带 --ignored",
+             ["strace", "--env=CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness",
+              "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("strace --env 值另起一个词、设 runner、点名崩溃枚举用例不带 --ignored",
+             ["strace", "--env", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness",
+              "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("strace -fE 合写设 runner、点名崩溃枚举用例不带 --ignored",
+             ["strace", "-fE", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness",
+              "--test", "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("strace -E 只清变量（不带 =）、点名崩溃枚举用例不带 --ignored",
+             ["strace", "-E", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER", "cargo", "test", "-p", "singlefs-harness", "--test",
+              "sample_crash_enumeration"], work, None),
         ]
         binary_cases = [
             ("绝对路径的层 0 测试二进制", [os.path.join(work, "target", "release", "deps", layer0_binary), "--test-threads", "4"], work, "layer0-binary"),
@@ -1012,6 +1038,7 @@
         name_cases = [
             ("门禁阶段按名字判", ".claude/gate.d/12-no-prime-marks.sh", work, True),
             ("仓内位置上的 gate-staged.sh 按名字判", "research/scripts/gate-staged.sh", work, True),
+            ("仓内位置上的 layer0-shard-run.sh 按名字判", "research/scripts/layer0-shard-run.sh", work, True),
             ("从 research 里相对着写的 mutate.sh", "scripts/mutate.sh", os.path.join(work, "research"), True),
             ("仓外的同名 mutate.sh 不按名字判", "/tmp/claude-1000/somewhere/mutate.sh", work, False),
             ("没登记的脚本", "run-chain.sh", work, False),
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -55,7 +55,7 @@
 - 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
 - 全部实验复跑：87 号。
 - E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
-- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标（`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制），而下面任一条成立的：libtest 参数带 `--ignored` 或 `--include-ignored`（`cargo nextest run` 是 `--run-ignored only` / `all`）；`--config` 或环境变量里定了测试二进制的 runner（`--config` 的值按 TOML 读，带引号的键也算；`systemd-run -E` / `--setenv` 设给里面那条命令的环境变量也算），或子命令是 `--config` / `CARGO_ALIAS_` 定的别名；登记的用例函数有一处定义没标 `#[ignore]`（同名的每一处 `fn <名>(` 都算），或判不出标没标（找不到那个函数、宏生成的用例、导入不了 `research/scripts/admission.py`，按没标算）。另外任何命令带 `--ignored` / `--include-ignored` 又点名登记的用例函数的也算（`grep`、`git` 这类按文本处理参数的除外）。不带这两个参数、用例函数标了 `#[ignore]`、只跑那个测试目标里快用例的不算。
+- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标（`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制），而下面任一条成立的：libtest 参数带 `--ignored` 或 `--include-ignored`（`cargo nextest run` 是 `--run-ignored only` / `all`）；`--config` 或环境变量里定了测试二进制的 runner（`--config` 的值按 TOML 读，带引号的键也算；`systemd-run -E` / `--setenv`、`strace -E` / `--env` 设给里面那条命令的环境变量也算），或子命令是 `--config` / `CARGO_ALIAS_` 定的别名；登记的用例函数有一处定义没标 `#[ignore]`（同名的每一处 `fn <名>(` 都算），或判不出标没标（找不到那个函数、宏生成的用例、导入不了 `research/scripts/admission.py`，按没标算）。另外任何命令带 `--ignored` / `--include-ignored` 又点名登记的用例函数的也算（`grep`、`git` 这类按文本处理参数的除外）。不带这两个参数、用例函数标了 `#[ignore]`、只跑那个测试目标里快用例的不算。双机分片的驱动脚本 `research/scripts/layer0-shard-run.sh` 带什么参数都算（`--merged-log` 也算），参数里有 `--selftest` 的不算。
 
 只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算，`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf stat|record|trace` 包在外面的剥掉照算（短选项合写的 `strace -fo <文件>`、`flock -xw 10` 同样剥）。重型测试清单各类里的 `cargo test` 同样指 `cargo nextest run`、`cargo miri test`、`cargo llvm-cov`、`cargo hack test`、`cargo mutants` 与别名；libtest 参数里 `--list` 当选项出现（只列用例、一条都不跑）的哪一类都不算，跟在 `--skip`、`--logfile` 这类带值的选项后面的 `--list` 是那个选项的值，照算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
 
--- a/.claude/gate.d/54-layer0-replay.sh
+++ b/.claude/gate.d/54-layer0-replay.sh
@@ -22,6 +22,11 @@
 #     断点续跑（crash-case-command 设）：跑用例时设 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=<common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>
 #     （不随 worktree 删掉）、SINGLEFS_LAYER0_INPUT_FINGERPRINT=<这条用例的输入指纹>；--start-over 设 SINGLEFS_LAYER0_START_OVER=1（丢掉进度文件、从头跑），
 #     不带它时从调用方的环境里清掉这个变量。续跑的判法（片方案、校验和、观察者计数、判红删进度文件）在 crates/singlefs-harness/src/layer0_progress.rs。
+#     双机分片（里程碑三第六项，用户 2026-09-27 定默认不分片）：本地配置（${SINGLEFS_LAYER0_SHARD_CONFIG:-<主工作树的根>/layer0-shard.env}，
+#     模板是仓根 layer0-shard.env.example）在、research/scripts/layer0-shard-configuration-check.sh 判得过（键齐、第二台 ssh 连得上、它上面有 cargo）
+#     才开；开着时登记了 shard=across-machines 的用例（admission.py crash-case-shardable）交给 research/scripts/layer0-shard-run.sh --merged-log
+#     （本机 0/2、第二台 1/2、本机 merge/2；驱动脚本与配置判法按内容进这几条用例的指纹），它交回的 merge 那一趟日志照单机的判法判、写同一格标记；
+#     别的用例、配置不在或判不过时，照 crash-case-command 单机跑。开没开、为什么，开跑时打一行。
 #     这一份 54 号不进指纹：改它（出路句、快档、次序）不废旧标记。它里面还定着结论的只剩流程的次序（开跑与跑完各算一次指纹、先判日志再写标记、
 #     判红删那一格），由 admission.py --selftest 的「54 号」那几格核；改它时快档照样核标记（范围那一问不摘掉它）。
 #   bash .claude/gate.d/54-layer0-replay.sh [项目根]           整轮门禁的默认（gate.sh 只传项目根）
@@ -55,7 +60,7 @@
 layer0_tier="quick"
 layer0_start_over=0
 root_argument=""
-source "$(dirname "${BASH_SOURCE[0]}")/../singlefs-ai-sop/scripts/preflight.sh"
+source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
 preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
 for stage_argument in "$@"; do
   case "$stage_argument" in
@@ -229,6 +234,15 @@
   return "${PIPESTATUS[0]}"
 }
 
+# run_crash_case_in_two_shards <键> <日志> <输入指纹>：双机分片跑一条（layer0-shard-run.sh --merged-log），merge 那一趟的整段输出进日志；
+# 两片各自的 LAYER0_PROGRESS 由驱动脚本边跑边转出来。--start-over 照样交给两片（merge 那一趟驱动脚本自己不带）。
+run_crash_case_in_two_shards() {
+  local -a start_over_setting=(-u SINGLEFS_LAYER0_START_OVER)
+  if [[ "$layer0_start_over" == 1 ]]; then start_over_setting=(SINGLEFS_LAYER0_START_OVER=1); fi
+  env "${start_over_setting[@]}" bash "$layer0_shard_driver" --merged-log "$1" "$ROOT" "$3" "$2" 2>&1 | sed -u 's/^/    /'
+  return "${PIPESTATUS[0]}"
+}
+
 # report_manifest_differences <前一份清单> <后一份清单> <前一份的叫法> <后一份的叫法>：逐个列出两份清单里不同的文件，最多 20 个，另报总数。
 report_manifest_differences() {
   local difference_lines difference_count
@@ -333,6 +347,15 @@
 start_over_note="不带 --start-over：有进度文件就接着跑"
 if [[ "$layer0_start_over" == 1 ]]; then start_over_note="带 --start-over：进度文件整份丢掉、从头跑"; fi
 echo "  · --full 开跑（${full_started_utc}）：${#crash_case_rows[@]} 条崩溃枚举用例逐条照复用判定跑（这批输入那一格全绿标记在就复用）；续跑的进度文件在 ${layer0_progress_root}/<输入指纹>/，${start_over_note}"
+# 双机分片开不开：本地配置在、判得过才开（判法在 layer0-shard-configuration-check.sh，与驱动脚本的运行条件同一份）
+layer0_shard_driver="$ROOT/research/scripts/layer0-shard-run.sh"
+if layer0_shard_configuration_note="$(bash "$ROOT/research/scripts/layer0-shard-configuration-check.sh" "$ROOT" 2>&1)"; then
+  layer0_sharded=1
+  echo "  · 双机分片：开（${layer0_shard_configuration_note//$'\n'/；}）；登记了 shard=across-machines 的用例两台各跑一片、本机 merge，别的单机跑"
+else
+  layer0_sharded=0
+  echo "  · 双机分片：关（${layer0_shard_configuration_note//$'\n'/；}）；每条用例单机跑"
+fi
 red_cases=()
 green_cases=()
 reused_cases=()
@@ -368,11 +391,30 @@
   case_threads_origin_text="没设，取本机核数"
   if [[ "$case_threads_origin" == explicit ]]; then case_threads_origin_text="显式设的"; fi
   case_threads_note="SINGLEFS_LAYER0_THREADS=${case_threads}（${case_threads_origin_text}），本机 ${case_machine_cores} 核"
-  echo "  · $case_key 开跑（${case_started_utc}；${case_command[*]}；${case_threads_note}；这批输入的指纹 ${fingerprint_at_start:0:16}…，${file_count_at_start} 个文件，减去用例读不到的 ${excluded_count_at_start} 个）："
+  case_sharded=0
+  if [[ "$layer0_sharded" == 1 ]] && python3 "$layer0_admission_module" crash-case-shardable "$ROOT" "$case_key" >/dev/null; then case_sharded=1; fi
+  case_way="单机跑：${case_command[*]}"
+  if [[ "$case_sharded" == 1 ]]; then
+    case_way="双机分片跑（本机 0/2、第二台 1/2，本机 merge/2；bash research/scripts/layer0-shard-run.sh --merged-log $case_key <树根> <指纹> <日志>）"
+    case_threads_note="本机那一片 ${case_threads_note}，第二台那一片取第二台的核数（逐片的数在 merge 那一行 LAYER0_PARALLEL_FINISHED 的 shard_…= 里）"
+  fi
+  echo "  · $case_key 开跑（${case_started_utc}；${case_way}；${case_threads_note}；这批输入的指纹 ${fingerprint_at_start:0:16}…，${file_count_at_start} 个文件，减去用例读不到的 ${excluded_count_at_start} 个）："
   sed 's/^/      /' <<< "$marker_check_output"
-  if ! run_crash_case "$case_log" "${case_command[@]}"; then
-    tail -40 "$case_log"
+  if [[ "$case_sharded" == 1 ]]; then
+    run_crash_case_in_two_shards "$case_key" "$case_log" "$fingerprint_at_start"
+  else
+    run_crash_case "$case_log" "${case_command[@]}"
+  fi
+  case_run_exit=$?
+  if [[ "$case_run_exit" != 0 ]]; then
+    [[ -f "$case_log" ]] && tail -40 "$case_log"
     delete_crash_case_marker "$case_key" "$fingerprint_at_start"
+    if [[ "$case_sharded" == 1 ]]; then
+      echo "  ✗ $case_key 判红：双机分片那一趟退 $case_run_exit（上面是驱动脚本的输出与 merge 那一趟日志的尾部）"
+      echo "     → 怎么办：驱动脚本输出里判红的那一句说清卡在哪一步（工具链、指纹、某一片、账本、merge）；要单机复核，挪开本地配置 layer0-shard.env 再跑 --full"
+      red_cases+=("$case_key")
+      continue
+    fi
     echo "  ✗ $case_key 判红：cargo test 退非 0（上面是它的尾部）"
     echo "     → 怎么办：单跑看细节：cargo test --release -p $case_package --test $case_target -- --include-ignored --exact $case_function --nocapture"
     echo "                断言消息里是第一处对不上的计数或违例。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
--- /dev/null
+++ b/research/scripts/layer0-shard-run.sh
@@ -0,0 +1,280 @@
+#!/usr/bin/env bash
+# 层 0 崩溃重放按双机分片跑一条崩溃枚举用例（里程碑三第六项，.claude/kb/milestone/03-third-txn.md 第六节）：本机跑第 0/2 片、
+# 第二台跑第 1/2 片（SINGLEFS_LAYER0_SHARD，切法与账本在 crates/singlefs-harness/src/layer0_progress.rs），第二台的账本拷回本机，
+# 本机 SINGLEFS_LAYER0_SHARD=merge/2 再跑同一条用例（只读两份账本、核齐、按切片序号并，不枚举）。
+#
+#   layer0-shard-run.sh <崩溃枚举用例的键> <树根>
+#       两台各跑一片、merge，merge 那一趟的日志照门禁 54 号 --full 判（admission.py crash-case-judge），判绿写 54 号那一格全绿标记
+#       （同一组标记、同一个输入指纹，crash-case-record）；判红删这批输入那一格。树根是 HEAD + 暂存区那棵树（建法同 54 号 --full）。
+#   layer0-shard-run.sh --merged-log <崩溃枚举用例的键> <树根> <输入指纹> <日志文件>
+#       门禁 54 号 --full 在分片开着时调：只做两片与 merge，merge 那一趟的整段输出写进日志文件，退出码是那一趟的；判与写标记归 54 号。
+#   layer0-shard-run.sh --selftest
+#       转给 research/scripts/layer0-shard-run-selftest.sh：不碰第二台，本机两个进程各跑一片走同一条路（含 merge、判与标记）。
+#
+# 步骤：① 两台各 rustc -Vv、cargo -V、nproc，rustc -Vv 前三行与 host 行、cargo -V 不同就拒；② 树 rsync 到第二台
+# <PEER_REPOSITORY_DIRECTORY>/runs/<这一趟>/（新建的空目录，不带 --delete），在那里 git init（算输入指纹要 git 列文件）；
+# ③ 两台各算这条用例的输入指纹（admission.py crash-case-manifest，参数与 54 号相同），不同就拒；清场（配置里写了的话，回读验证，
+# 复原挂在 EXIT 上，跑红了也复原）；④ 两片同时跑，各记 pid、各 wait 取退出码、各写自己的日志，LAYER0_PROGRESS 边跑边转出来；
+# ⑤ 第二台的账本拷回本机进度目录；⑥ 本机 merge；⑦ 删掉第二台上这一趟的树与编译目录（两片的进度目录留着：被杀之后下一趟接着跑）。
+# 进度目录：本机 <git common-dir>/singlefs-layer0-progress/<输入指纹>（与 54 号相同），第二台 <PEER_REPOSITORY_DIRECTORY>/progress/<输入指纹>。
+# 线程：本机那一片取 SINGLEFS_LAYER0_THREADS（没设取本机 nproc），第二台那一片取第二台的 nproc。SINGLEFS_LAYER0_START_OVER=1 交给两片，
+# merge 那一趟不带。只接登记了 shard=across-machines 的用例（它的枚举经 Layer0Resume::from_environment 认分片开关）。
+# 登记的崩溃枚举用例都标了 #[ignore]，三趟 cargo test 都带 --include-ignored --exact <用例函数>；只供测试的开关
+# SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0 时不带 --include-ignored（--selftest 那条小流用例不标 ignore，不必把别的 ignore 用例放进过滤范围）。
+#
+# 配置（本地、git 忽略）：${SINGLEFS_LAYER0_SHARD_CONFIG:-<主工作树的根>/layer0-shard.env}，键与写法见仓根 layer0-shard.env.example；
+# 在不在、齐不齐、第二台连不连得上由 research/scripts/layer0-shard-configuration-check.sh 判（门禁 54 号拿同一份判走不走分片）。
+# 这一份与配置判法按内容进登记了 shard=across-machines 的用例的输入指纹（research/scripts/admission.py 的 SHARD_DRIVER_FILES）：两片与 merge 的
+# cargo 由这里起、不经 admission.py crash-case-command，改了这里那几条用例的旧全绿标记不再作数。
+# 重型：跑的是标了 ignore 的崩溃枚举用例（--include-ignored），只在提交时（SINGLEFS_HEAVY_TESTS=commit）或用户要求时跑；--selftest 不算。
+#
+# 配置在不在、判不判得过不写成运行条件（--selftest 不要配置）：开跑之后第一件事调配置判法，判不过照它的原因与出路退 1。
+#
+# admission: always 每次调都跑这一刻这棵树上的一条崩溃枚举用例；复用判定在门禁 54 号的全绿标记里，不在这里
+# run-condition: command cargo rustc python3 rsync ssh nproc git
+set -uo pipefail
+source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
+preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
+layer0_shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
+[[ "${1:-}" != --selftest ]] || exec bash "$layer0_shard_script_directory/layer0-shard-run-selftest.sh"
+
+layer0_shard_mode=standalone
+if [[ "${1:-}" == --merged-log ]]; then layer0_shard_mode=merged-log; shift; fi
+case_key="${1:-}"
+tree_root="${2:-}"
+given_fingerprint="${3:-}"
+merged_log_file="${4:-}"
+usage_ok=1
+[[ "$case_key" == crash-case:* && -n "$tree_root" && -d "$tree_root" ]] || usage_ok=0
+if [[ "$layer0_shard_mode" == merged-log && ( ! "$given_fingerprint" =~ ^[0-9a-f]{64}$ || -z "$merged_log_file" ) ]]; then usage_ok=0; fi
+if (( ! usage_ok )); then
+  echo "  ✗ 用法：layer0-shard-run.sh <crash-case:键> <树根>，或 --merged-log <crash-case:键> <树根> <输入指纹> <日志文件>，或 --selftest"
+  echo "     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 那一行写；树根是 HEAD + 暂存区那棵树（建法见门禁 54 号 --full 的出路句）"
+  exit 2
+fi
+tree_root="$(cd "$tree_root" && pwd)"
+admission_module="$tree_root/research/scripts/admission.py"
+
+fail() { # fail <原因> <出路>
+  echo "  ✗ $case_key 双机分片：$1"
+  echo "     → 怎么办：$2"
+  exit 1
+}
+fail_after_the_run() { # fail_after_the_run <原因> <出路>：跑过之后判红——单独跑时删这批输入那一格全绿标记（先绿后红，前一趟那一格不再作数），再 fail
+  local marker_path
+  if [[ "$layer0_shard_mode" == standalone ]] \
+      && marker_path="$(python3 "$admission_module" crash-case-marker-path "$tree_root" "$case_key" "$local_fingerprint")" && [[ -n "$marker_path" ]]; then
+    rm -f -- "${marker_path:?}"
+  fi
+  fail "$1" "$2"
+}
+
+if ! configuration_assignments="$(bash "$layer0_shard_script_directory/layer0-shard-configuration-check.sh" --emit-assignments "$tree_root")"; then
+  printf '%s\n' "$configuration_assignments"
+  exit 1
+fi
+eval "$configuration_assignments"
+peer_is_this_machine="${SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE:-0}"
+
+# 这条用例登记的包、测试目标、用例函数（第三列的 test=）；要登记了 shard=across-machines
+registration_row="$(awk -F'\t' -v key="$case_key" '$1 == key' "$tree_root/.claude/gate.d/stage-inputs.tsv")"
+test_token="$(tr ' ' '\n' <<< "$(cut -f3 <<< "$registration_row")" | grep '^test=' | head -1)"
+IFS=: read -r case_package case_target case_function <<< "${test_token#test=}"
+[[ -n "$registration_row" && -n "$case_package" && -n "$case_target" && -n "$case_function" ]] \
+  || fail "stage-inputs.tsv 里找不到这一行或它的 test=<包>:<测试目标>:<用例函数>" "照 research/scripts/admission.py 文件头「崩溃枚举用例行」登记"
+python3 "$admission_module" crash-case-shardable "$tree_root" "$case_key" >/dev/null \
+  || fail "这条用例没登记 shard=across-machines（它的枚举不认分片开关，分了片也是每台各跑全量）" "单机跑它：bash .claude/gate.d/54-layer0-replay.sh --full <树根>；要分片先让用例经 Layer0Resume::from_environment 取续跑设置，再在登记行第三列加 shard=across-machines"
+
+if ! git_common_directory="$(git -C "$tree_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
+  fail "$tree_root 不是 git 工作树（取不到 git common-dir）" "在 HEAD + 暂存区的 worktree 里跑，建法见门禁 54 号 --full 的出路句"
+fi
+run_label="${case_key#crash-case:}-$(date -u +%Y%m%dT%H%M%SZ)-$$"
+peer_run_directory="$PEER_REPOSITORY_DIRECTORY/runs/$run_label"
+scratch_directory="$(mktemp -d)"
+quiesced=0
+restore_on_exit() {
+  if (( quiesced )); then
+    echo "  · 复原：$QUIESCE_START_COMMAND"
+    bash -c "$QUIESCE_START_COMMAND"
+    if bash -c "$QUIESCE_STARTED_CHECK_COMMAND"; then
+      echo "  · 复原之后回读过了（$QUIESCE_STARTED_CHECK_COMMAND 退 0）"
+    else
+      echo "  ✗ 复原之后回读没过：$QUIESCE_STARTED_CHECK_COMMAND 退非 0"
+      echo "     → 怎么办：手动复原（$QUIESCE_START_COMMAND），回读到它退 0 为止"
+    fi
+  fi
+  rm -rf -- "${scratch_directory:?}"
+}
+trap restore_on_exit EXIT
+
+# run_on_peer <目录> <命令文本>：在第二台的这个目录里 bash -c 跑（PATH 前面加 PEER_CARGO_BIN_DIRECTORY）；第二台是本机（只供测试）时本机跑
+run_on_peer() {
+  if [[ "$peer_is_this_machine" == 1 ]]; then
+    (cd "$1" && PATH="$PEER_CARGO_BIN_DIRECTORY:$PATH" bash -c "$2")
+  else
+    ssh -o BatchMode=yes "$PEER_SSH_HOST" "cd $(printf '%q' "$1") && PATH=$(printf '%q' "$PEER_CARGO_BIN_DIRECTORY"):\$PATH bash -c $(printf '%q' "$2")" </dev/null
+  fi
+}
+# copy_to_peer / copy_from_peer：rsync -a，不带 --delete
+copy_to_peer() {
+  if [[ "$peer_is_this_machine" == 1 ]]; then rsync -a "$@"; else rsync -a -e "ssh -o BatchMode=yes" "$@"; fi
+}
+peer_path() { # peer_path <第二台上的路径>：rsync 的参数写法
+  if [[ "$peer_is_this_machine" == 1 ]]; then printf '%s' "$1"; else printf '%s:%s' "$PEER_SSH_HOST" "$1"; fi
+}
+if [[ "$peer_is_this_machine" == 1 ]]; then
+  echo "  ! 第二台是本机上的 $PEER_REPOSITORY_DIRECTORY（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1）"
+fi
+
+# ① 工具链：rustc -Vv 前三行与 host 行、cargo -V 两台相同；线程数各取各的核数
+toolchain_description_command='rustc -Vv | head -3 && rustc -Vv | grep "^host: " && cargo -V'
+local_toolchain="$(bash -c "$toolchain_description_command")" || fail "本机 rustc / cargo 报不出版本" "看 PATH 里有没有 rustc 与 cargo"
+peer_toolchain="$(run_on_peer / "$toolchain_description_command")" || fail "第二台 rustc / cargo 报不出版本" "看 $PEER_CARGO_BIN_DIRECTORY 下有没有 rustc 与 cargo"
+[[ "$local_toolchain" == "$peer_toolchain" ]] \
+  || fail "两台的工具链不同：本机「${local_toolchain//$'\n'/；}」，第二台「${peer_toolchain//$'\n'/；}」" "两台装同一个 rustup 工具链（rustc -Vv 的 commit 相同）再跑"
+local_cores="$(nproc)"
+peer_cores="$(run_on_peer / nproc)" || fail "第二台 nproc 跑不起来" "看 ssh $PEER_SSH_HOST 能不能跑命令"
+if [[ -n "${SINGLEFS_LAYER0_THREADS:-}" ]]; then
+  local_threads="$SINGLEFS_LAYER0_THREADS"; local_threads_origin=explicit; local_threads_text="SINGLEFS_LAYER0_THREADS=${local_threads}（显式设的）"
+else
+  local_threads="$local_cores"; local_threads_origin=default; local_threads_text="SINGLEFS_LAYER0_THREADS=${local_threads}（没设，取本机核数）"
+fi
+echo "  · ① 工具链两台相同（${local_toolchain%%$'\n'*}）；本机 ${local_cores} 核跑 0/2（${local_threads_text}），第二台 ${peer_cores} 核跑 1/2（取第二台核数）"
+
+# ② 树拷到第二台这一趟的专用目录（新建的空目录，不带 --delete），在那里 git init
+run_on_peer / "mkdir -p $(printf '%q' "$peer_run_directory") $(printf '%q' "$PEER_REPOSITORY_DIRECTORY/progress")" || fail "第二台建不了 $peer_run_directory" "看 PEER_REPOSITORY_DIRECTORY 可不可写"
+copy_to_peer --exclude target --exclude .git "$tree_root/" "$(peer_path "$peer_run_directory")/" || fail "树 rsync 到第二台失败" "看第二台盘满没满、$PEER_REPOSITORY_DIRECTORY 可不可写"
+run_on_peer "$peer_run_directory" "git init -q ." || fail "第二台上 git init 失败" "看第二台装没装 git"
+echo "  · ② 树拷到第二台 $peer_run_directory"
+
+# ③ 两台各算这条用例的输入指纹（参数与门禁 54 号相同：准入模块的判法摘要、工具链与构建环境；登记了 shard=across-machines 的，
+# 这一份驱动脚本与配置判法由准入模块按内容加进清单）
+local_manifest="$scratch_directory/manifest.local"
+local_summary="$(python3 "$admission_module" crash-case-manifest "$tree_root" "$case_key" "$local_manifest" \
+  --judging-digest --toolchain --build-environment)" || fail "本机算不出输入指纹：$local_summary" "单跑 python3 research/scripts/admission.py crash-case-manifest <树根> $case_key <清单文件> --judging-digest --toolchain --build-environment 看它报什么"
+read -r local_fingerprint local_file_count local_excluded_count <<< "$local_summary"
+peer_summary="$(run_on_peer "$peer_run_directory" "python3 research/scripts/admission.py crash-case-manifest . $(printf '%q' "$case_key") .layer0-shard-manifest \
+  --judging-digest --toolchain --build-environment")" || fail "第二台算不出输入指纹：$peer_summary" "在第二台 $peer_run_directory 里单跑同一条 crash-case-manifest 看它报什么"
+peer_fingerprint="${peer_summary%% *}"
+[[ "$local_fingerprint" == "$peer_fingerprint" ]] \
+  || fail "两台算出的输入指纹不同（本机 ${local_fingerprint:0:16}…，第二台 ${peer_fingerprint:0:16}…）：构建环境（~/.cargo/config、RUSTFLAGS 这一类）或树不一样" "对一对两台的 ~/.cargo/config*、环境变量 RUSTFLAGS / CARGO_*，改成一样再跑"
+if [[ "$layer0_shard_mode" == merged-log && "$local_fingerprint" != "$given_fingerprint" ]]; then
+  fail "本机算出的输入指纹 ${local_fingerprint:0:16}… 与 54 号给的 ${given_fingerprint:0:16}… 不同" "别在有人改这些路径的树里跑；在 HEAD + 暂存区的 worktree 里重跑 54 号 --full"
+fi
+echo "  · ③ 两台的输入指纹相同：${local_fingerprint:0:16}…（${local_file_count} 个文件，减去用例读不到的 ${local_excluded_count} 个）"
+
+local_progress_directory="$git_common_directory/singlefs-layer0-progress/$local_fingerprint"
+peer_progress_directory="$PEER_REPOSITORY_DIRECTORY/progress/$local_fingerprint"
+mkdir -p "$local_progress_directory"
+run_on_peer / "mkdir -p $(printf '%q' "$peer_progress_directory")" || fail "第二台建不了进度目录 $peer_progress_directory" "看 PEER_REPOSITORY_DIRECTORY 可不可写"
+
+# 清场（配置里写了才做）：跑完回读，复原挂在 EXIT 上
+if [[ -n "$QUIESCE_STOP_COMMAND" ]]; then
+  echo "  · 清场：$QUIESCE_STOP_COMMAND"
+  quiesced=1
+  bash -c "$QUIESCE_STOP_COMMAND"
+  bash -c "$QUIESCE_STOPPED_CHECK_COMMAND" || fail "清场之后回读没过：$QUIESCE_STOPPED_CHECK_COMMAND 退非 0" "手动清场，回读到它退 0 为止；复原命令在退出时照跑"
+  echo "  · 清场之后回读过了（$QUIESCE_STOPPED_CHECK_COMMAND 退 0）"
+fi
+
+# ④ 两片同时跑
+start_over_settings=(-u SINGLEFS_LAYER0_START_OVER)
+start_over_text=""
+if [[ "${SINGLEFS_LAYER0_START_OVER:-0}" == 1 ]]; then start_over_settings=(SINGLEFS_LAYER0_START_OVER=1); start_over_text="SINGLEFS_LAYER0_START_OVER=1 "; fi
+libtest_arguments=(--exact "$case_function" --nocapture)
+case "${SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED:-1}" in
+  1) libtest_arguments=(--include-ignored "${libtest_arguments[@]}") ;;
+  0) echo "  ! 不带 --include-ignored（只供测试的开关 SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0；标了 ignore 的用例这样跑一条都跑不到，会判红）" ;;
+  *) fail "SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED 只许 0 或 1，读到「${SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED}」" "不设它（默认 1）；只有 --selftest 设成 0" ;;
+esac
+cargo_test_text="cargo test --release -p $case_package --test $case_target -- ${libtest_arguments[*]}"
+local_log="$scratch_directory/shard-0-of-2.log"
+peer_log="$scratch_directory/shard-1-of-2.log"
+rm -f -- "$scratch_directory"/exit.*
+{
+  ( cd "$tree_root" && env "${start_over_settings[@]}" SINGLEFS_LAYER0_SHARD=0/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY="$local_progress_directory" \
+    SINGLEFS_LAYER0_INPUT_FINGERPRINT="$local_fingerprint" SINGLEFS_LAYER0_THREADS="$local_threads" \
+    cargo test --release -p "$case_package" --test "$case_target" -- "${libtest_arguments[@]}" ) 2>&1 \
+    | tee "$local_log" | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } | sed -u 's/^/    [本机 0\/2] /'
+  echo "${PIPESTATUS[0]}" > "$scratch_directory/exit.local"
+} &
+local_shard_process=$!
+{
+  run_on_peer "$peer_run_directory" "env SINGLEFS_HEAVY_TESTS=${SINGLEFS_HEAVY_TESTS:-} ${start_over_text}SINGLEFS_LAYER0_SHARD=1/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=$(printf '%q' "$peer_progress_directory") SINGLEFS_LAYER0_INPUT_FINGERPRINT=$local_fingerprint SINGLEFS_LAYER0_THREADS=$peer_cores $(printf '%q ' cargo test --release -p "$case_package" --test "$case_target" -- "${libtest_arguments[@]}") 2>&1" \
+    | tee "$peer_log" | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } | sed -u 's/^/    [第二台 1\/2] /'
+  echo "${PIPESTATUS[0]}" > "$scratch_directory/exit.peer"
+} &
+peer_shard_process=$!
+echo "  · ④ 两片开跑：本机 ${start_over_text}SINGLEFS_LAYER0_SHARD=0/2 $cargo_test_text（pid $local_shard_process），第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条（pid $peer_shard_process）"
+wait "$local_shard_process"
+wait "$peer_shard_process"
+local_exit="$(cat "$scratch_directory/exit.local" 2>/dev/null || echo missing)"
+peer_exit="$(cat "$scratch_directory/exit.peer" 2>/dev/null || echo missing)"
+shard_problems=()
+[[ "$local_exit" == 0 ]] || shard_problems+=("本机那一片 cargo test 退 $local_exit")
+[[ "$peer_exit" == 0 ]] || shard_problems+=("第二台那一片 cargo test 退 $peer_exit")
+grep -q 'LAYER0_SHARD mode=run shard=0/2 ' "$local_log" || shard_problems+=("本机的日志里没有 LAYER0_SHARD mode=run shard=0/2 那一行（这条用例没按分片跑）")
+grep -q 'LAYER0_SHARD mode=run shard=1/2 ' "$peer_log" || shard_problems+=("第二台的日志里没有 LAYER0_SHARD mode=run shard=1/2 那一行（这条用例没按分片跑）")
+remove_peer_run_directory() { # ⑦ 删掉第二台上这一趟的树与编译目录
+  local peer_run_size
+  peer_run_size="$(run_on_peer / "du -sh $(printf '%q' "$peer_run_directory") | cut -f1")"
+  if run_on_peer / "rm -rf -- $(printf '%q' "${peer_run_directory:?}")"; then
+    echo "  · ⑦ 删掉第二台上这一趟的树与编译目录 $peer_run_directory（${peer_run_size:-大小没读到}）；两片的进度目录留着"
+  else
+    echo "  ! 第二台上的 $peer_run_directory 没删掉：下一趟另起一个目录，不碍事；手动删它"
+  fi
+}
+if (( ${#shard_problems[@]} > 0 )); then
+  echo "    本机那一片的日志尾部："; tail -20 "$local_log" | sed 's/^/      /'
+  echo "    第二台那一片的日志尾部："; tail -20 "$peer_log" | sed 's/^/      /'
+  remove_peer_run_directory
+  fail_after_the_run "$(IFS='；'; echo "${shard_problems[*]}")" "单跑那一片看细节（本机：SINGLEFS_LAYER0_SHARD=0/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=… SINGLEFS_LAYER0_INPUT_FINGERPRINT=… $cargo_test_text）；断言消息里是第一处对不上的计数或违例"
+fi
+echo "  · ④ 两片都跑完"
+
+# ⑤ 第二台的账本拷回本机进度目录
+copy_to_peer --include 'layer0-shard-*-shard-1-of-2.tally' --exclude '*' "$(peer_path "$peer_progress_directory")/" "$local_progress_directory/" \
+  || fail_after_the_run "第二台的账本拷不回来" "看 $peer_progress_directory 里有没有 layer0-shard-*-shard-1-of-2.tally"
+remove_peer_run_directory
+fetched_ledgers=("$local_progress_directory"/layer0-shard-*-shard-1-of-2.tally)
+own_ledgers=("$local_progress_directory"/layer0-shard-*-shard-0-of-2.tally)
+[[ -f "${fetched_ledgers[0]}" && ${#fetched_ledgers[@]} == 1 && -f "${own_ledgers[0]}" && ${#own_ledgers[@]} == 1 ]] \
+  || fail_after_the_run "本机进度目录里两片的账本不是各恰好一份（第 0 片 ${#own_ledgers[@]} 份、第 1 片 ${#fetched_ledgers[@]} 份，在 $local_progress_directory）" "看两片的日志里 LAYER0_SHARD mode=run 那一行的 ledger= 在哪；删掉多出来的旧账本再跑"
+echo "  · ⑤ 第二台的账本拷回 ${fetched_ledgers[0]}"
+
+# ⑥ 本机 merge：只读两份账本、核齐、按切片序号并，照旧走用例钉死的计数断言
+merge_log="$scratch_directory/merge.log"
+merge_exit=0
+( cd "$tree_root" && env -u SINGLEFS_LAYER0_START_OVER SINGLEFS_LAYER0_SHARD=merge/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY="$local_progress_directory" \
+  SINGLEFS_LAYER0_INPUT_FINGERPRINT="$local_fingerprint" SINGLEFS_LAYER0_THREADS="$local_threads" \
+  cargo test --release -p "$case_package" --test "$case_target" -- "${libtest_arguments[@]}" ) > "$merge_log" 2>&1 || merge_exit=$?
+if [[ "$merge_exit" == 0 ]] && ! grep -q 'LAYER0_SHARD mode=merge shards=2 ' "$merge_log"; then merge_exit=merge-line-missing; fi
+if [[ "$layer0_shard_mode" == merged-log ]]; then
+  cp -- "$merge_log" "$merged_log_file"
+  echo "  · ⑥ merge 那一趟退 $merge_exit，整段输出交给 54 号判（$merged_log_file）"
+  [[ "$merge_exit" == 0 ]] && exit 0
+  exit 1
+fi
+if [[ "$merge_exit" != 0 ]]; then
+  tail -40 "$merge_log" | sed 's/^/      /'
+  fail_after_the_run "merge 那一趟退 $merge_exit（上面是它的尾部；merge-line-missing 是日志里没有 LAYER0_SHARD mode=merge shards=2 那一行）" "账本核不齐时 panic 那一句说清缺哪一片、哪一处不同；计数断言红了与单机全量红了是同一回事，照 54 号 --full 的出路查"
+fi
+judged_lines_file="$scratch_directory/judged"
+case_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
+if ! judge_output="$(python3 "$admission_module" crash-case-judge "$tree_root" "$case_key" "$merge_log" "$judged_lines_file" \
+    --machine-cores "$local_cores" --threads "$local_threads" --threads-origin "$local_threads_origin")"; then
+  printf '%s\n' "$judge_output" | sed 's/^/       /'
+  fail_after_the_run "merge 那一趟跑过了，日志却判不绿（上面逐条列出）" "计数行、test result、exhaustive=true、每一片的工作线程，照 54 号 --full 同一句出路查"
+fi
+finish_manifest="$scratch_directory/manifest.finish"
+finish_summary="$(python3 "$admission_module" crash-case-manifest "$tree_root" "$case_key" "$finish_manifest" \
+  --judging-digest --toolchain --build-environment)" || fail "跑完之后算不出输入指纹：$finish_summary；不写全绿标记" "等改动停下，在 HEAD + 暂存区的 worktree 里重跑"
+[[ "${finish_summary%% *}" == "$local_fingerprint" ]] \
+  || fail "跑的过程中这条用例的输入变了（开跑 ${local_fingerprint:0:16}…，跑完 ${finish_summary:0:16}…）：不写全绿标记" "别在有人改这些路径的树里跑；在 HEAD + 暂存区的 worktree 里重跑"
+threads_text="${judge_output//$'\n'/；}${judge_output:+；}双机分片：本机 ${local_threads_text}、本机 ${local_cores} 核，第二台 ${peer_cores} 核"
+record_output="$(python3 "$admission_module" crash-case-record "$tree_root" "$case_key" "$local_fingerprint" "$local_manifest" "$judged_lines_file" \
+  --files "$local_file_count" --excluded "$local_excluded_count" --started "$case_started_utc" --judged-root "$tree_root" \
+  --machine-cores "$local_cores" --threads "$local_threads" --threads-origin "$local_threads_origin")" \
+  || fail "判绿，全绿标记却没写成：$record_output" "看 $git_common_directory 可不可写、盘满没满"
+echo "  ✓ $case_key 双机分片判绿（${threads_text}）：全绿标记写进 ${record_output}，记下的行原样："
+sed 's/^/      /' "$judged_lines_file"
--- /dev/null
+++ b/research/scripts/layer0-shard-run-selftest.sh
@@ -0,0 +1,207 @@
+#!/usr/bin/env bash
+# research/scripts/layer0-shard-run.sh --selftest 的本体：不碰第二台。拷一份仓进临时目录、git init，登记一条 shard=across-machines 的用例，
+# 「第二台」是本机上的另一个目录（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1），本机两个进程各跑一片，走驱动脚本同一条路：
+# 工具链与输入指纹两边比、清场与复原（回读）、两片、拷账本、merge、判、写 54 号那一格全绿标记。
+# 默认用假 cargo（不编译、不跑用例）：登记的是临时仓里新写的一条标了 #[ignore] 的替身用例，三趟都不带 --include-ignored，不算重型；
+# 假 cargo 分片跑时往进度目录写账本、merge 时核两份账本在不在、记的输入指纹对不对，打与层 0 同形的计数行与带 shards= 的线程行。
+# 带 SINGLEFS_HEAVY_TESTS（commit 或 user-request）时换真 cargo：登记 crates/singlefs-harness/tests/crash_enumeration_sharded_across_processes.rs
+# 里不标 ignore 的小流用例（第一条流按甲二展开 29 个状态），release 下真编真跑；不带时成功行写明真 cargo 那一趟本次未跑。逐格核：
+#   ① 不分片跑那条用例打的计数行，与双机分片 merge 之后记进全绿标记的那一行逐字相同；标记作数；清场复原了；第二台这一趟的目录删了
+#   ② --merged-log（门禁 54 号调的那一条）：merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数
+#   ③ 「第二台」的 rustc -Vv 第一行不同 ⇒ 拒（工具链不同），标记不动
+#   ④ 「第二台」那一片没写账本 ⇒ 判红，删这批输入那一格标记
+#   ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒（退 1），出路指到 layer0-shard.env.example
+#   ⑥ 没登记 shard=across-machines 的用例 ⇒ 拒
+# 成功行报核了几格（现算）。
+#
+# admission: always 自证判的是这一刻的驱动脚本与仓，每次调都要现跑
+# run-condition: command cargo rustc python3 rsync git nproc
+set -uo pipefail
+source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
+preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
+selftest_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
+
+repository_root="$(cd "$selftest_script_directory/../.." && pwd)"
+work="$(mktemp -d)"
+remove_work() {
+  echo "  · 自证的临时目录 $work（$(du -sh "$work" 2>/dev/null | cut -f1)）删掉"
+  rm -rf -- "${work:?}"
+}
+trap remove_work EXIT
+checked=0
+failures=0
+expect() { # expect <名> <判据的退出码：0 为过> <不过时的细节>
+  checked=$((checked + 1))
+  if [[ "$2" == 0 ]]; then
+    echo "  ✓ $1"
+  else
+    echo "  ✗ $1：$3"  # gate-lint:detail
+    failures=$((failures + 1))
+  fi
+}
+
+selftest_case_key="crash-case:sharded-selftest"
+copy="$work/repository"
+rsync -a --exclude target --exclude .git --exclude layer0-shard.env "$repository_root/" "$copy/"
+git -C "$copy" init -q
+real_rustc="$(command -v rustc)"
+if [[ -n "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
+  cargo_mode="真 cargo（SINGLEFS_HEAVY_TESTS=${SINGLEFS_HEAVY_TESTS}）"
+  selftest_function="the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts"
+  selftest_target="crash_enumeration_sharded_across_processes"
+  selftest_cargo="$(command -v cargo)"
+else
+  cargo_mode="假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）"
+  selftest_function="stand_in_for_the_driver_selftest"
+  selftest_target="sharded_driver_selftest_stand_in"
+  printf '#[test]\n#[ignore]\nfn %s() {}\n' "$selftest_function" > "$copy/crates/singlefs-harness/tests/$selftest_target.rs"
+  mkdir -p "$work/stand-in-bin"
+  selftest_cargo="$work/stand-in-bin/cargo"
+  cat > "$selftest_cargo" <<'STAND_IN_CARGO'
+#!/usr/bin/env bash
+# 驱动脚本自证的假 cargo：-V 打版本；test 按 SINGLEFS_LAYER0_SHARD 分三种：不设打计数行与线程行，<i>/2 往进度目录写那一片的账本，
+# merge/2 核两份账本在、记的输入指纹与这一趟的相同、START_OVER 没设，再打计数行与带 shards= 的线程行。别的一律退 101。
+if [[ "${1:-}" == -V ]]; then echo "cargo 0.0.0-layer0-shard-selftest"; exit 0; fi
+[[ "${1:-}" == test ]] || { echo "假 cargo 只认 -V 与 test：$*"; exit 101; }
+count_line="LAYER0_SHARDED states=29 closed_form=29 violations=0 exhaustive=true"
+passed="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s"
+case "${SINGLEFS_LAYER0_SHARD-}" in
+  "")
+    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=2 configured_worker_threads=2 worker_threads_source=environment_variable resumed_slices=0 freshly_run_slices=4 progress_file_after_completion=none elapsed_seconds=0.1"
+    printf '%s\n%s\n' "$count_line" "$passed" ;;
+  0/2|1/2)
+    [[ -n "${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}" && -n "${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "分片跑没设进度目录或输入指纹"; exit 101; }
+    shard_index="${SINGLEFS_LAYER0_SHARD%/2}"
+    ledger="$SINGLEFS_LAYER0_PROGRESS_DIRECTORY/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
+    mkdir -p "$SINGLEFS_LAYER0_PROGRESS_DIRECTORY" && printf 'input_fingerprint=%s\n' "$SINGLEFS_LAYER0_INPUT_FINGERPRINT" > "$ledger" || exit 101
+    printf 'LAYER0_PROGRESS slice=1/2 shard=%s\nLAYER0_SHARD mode=run shard=%s ledger=%s\n%s\n' "$SINGLEFS_LAYER0_SHARD" "$SINGLEFS_LAYER0_SHARD" "$ledger" "$passed" ;;
+  merge/2)
+    [[ -z "${SINGLEFS_LAYER0_START_OVER+set}" ]] || { echo "merge 那一趟带了 SINGLEFS_LAYER0_START_OVER"; exit 101; }
+    for shard_index in 0 1; do
+      ledger="${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
+      [[ "$(cat "$ledger" 2>/dev/null)" == "input_fingerprint=${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "merge：第 $shard_index 片的账本缺或输入指纹不同（$ledger）"; exit 101; }
+    done
+    echo "LAYER0_SHARD mode=merge shards=2 ledgers=${SINGLEFS_LAYER0_PROGRESS_DIRECTORY}"
+    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=4 configured_worker_threads=4 worker_threads_source=shard_ledgers resumed_slices=0 freshly_run_slices=4 shards=2 shard_worker_threads=2,2 shard_configured_worker_threads=2,2 shard_worker_threads_sources=environment_variable,environment_variable shard_available_parallelism=2,2 shard_resumed_slices=0,0 shard_freshly_run_slices=2,2 shard_elapsed_milliseconds=10,10 progress_file_after_completion=kept elapsed_seconds=0.1"
+    printf '%s\n%s\n' "$count_line" "$passed" ;;
+  *) echo "假 cargo 认不出 SINGLEFS_LAYER0_SHARD=${SINGLEFS_LAYER0_SHARD}"; exit 101 ;;
+esac
+STAND_IN_CARGO
+  chmod +x "$selftest_cargo"
+  export PATH="$work/stand-in-bin:$PATH"
+fi
+printf '%s\t%s\t%s\t%s\n' "$selftest_case_key" "crates/ Cargo.toml Cargo.lock" \
+  "test=singlefs-harness:$selftest_target:$selftest_function count-line=LAYER0_SHARDED exhaustive=LAYER0_SHARDED threads=LAYER0_SHARDED shard=across-machines" \
+  "# 驱动脚本自证临时登记的用例" >> "$copy/.claude/gate.d/stage-inputs.tsv"
+selftest_bin_directory="$(dirname "$selftest_cargo")"
+write_configuration() { # write_configuration <文件> <PEER_CARGO_BIN_DIRECTORY>
+  cat > "$1" <<CONFIGURATION
+# 驱动脚本自证用的配置：第二台是本机上的 $work/peer
+PEER_SSH_HOST=selftest-peer-is-this-machine
+PEER_REPOSITORY_DIRECTORY=$work/peer
+PEER_CARGO_BIN_DIRECTORY=$2
+QUIESCE_STOP_COMMAND=touch $work/quiesced
+QUIESCE_STOPPED_CHECK_COMMAND=test -e $work/quiesced
+QUIESCE_START_COMMAND=rm -f $work/quiesced
+QUIESCE_STARTED_CHECK_COMMAND=test ! -e $work/quiesced
+CONFIGURATION
+}
+write_configuration "$work/layer0-shard.env" "$selftest_bin_directory"
+export SINGLEFS_LAYER0_SHARD_CONFIG="$work/layer0-shard.env"
+export SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1
+# 三趟 cargo test 都不带 --include-ignored：真 cargo 那条小流用例不标 ignore（同一个测试目标里标了 ignore 的 golden 子进程用例不进过滤范围）；
+# 假 cargo 的替身用例标了 ignore，不带它就不算重型
+export SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0
+unset SINGLEFS_LAYER0_SHARD SINGLEFS_LAYER0_PROGRESS_DIRECTORY SINGLEFS_LAYER0_INPUT_FINGERPRINT SINGLEFS_LAYER0_START_OVER
+driver="$copy/research/scripts/layer0-shard-run.sh"
+admission_module="$copy/research/scripts/admission.py"
+
+# 不分片跑一次那条用例：计数行是 merge 之后要逐字相同的那一行
+(cd "$copy" && cargo test --release -p singlefs-harness --test "$selftest_target" -- --exact "$selftest_function" --nocapture) > "$work/unsharded.log" 2>&1
+unsharded_exit=$?
+unsharded_count_line="$(grep '^LAYER0_SHARDED ' "$work/unsharded.log")"
+expect "不分片跑登记的那条用例：退 0，恰好一行 LAYER0_SHARDED 计数行" "$([[ $unsharded_exit == 0 && $(grep -c '^LAYER0_SHARDED ' "$work/unsharded.log") == 1 ]]; echo $?)" \
+  "退 $unsharded_exit；日志尾部：$(tail -5 "$work/unsharded.log" | tr '\n' '|')"
+
+# ① 单独跑：两片、merge、判、写标记
+bash "$driver" "$selftest_case_key" "$copy" > "$work/standalone.log" 2>&1
+standalone_exit=$?
+fingerprint="$(python3 "$admission_module" crash-case-manifest "$copy" "$selftest_case_key" "$work/manifest" \
+  --judging-digest --toolchain --build-environment | cut -d' ' -f1)"
+marker_output="$(python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest")"
+marker_exit=$?
+marker_count_line="$(grep '^LAYER0_SHARDED ' <<< "$marker_output")"
+marker_path="$(head -1 <<< "$marker_output" | cut -d' ' -f2)"
+expect "① 单独跑：退 0，两片各跑了（LAYER0_SHARD mode=run 0/2、1/2），这批输入那一格全绿标记作数" \
+  "$([[ $standalone_exit == 0 && $marker_exit == 0 ]] && grep -q 'SINGLEFS_LAYER0_SHARD=0/2' "$work/standalone.log" && grep -q '④ 两片都跑完' "$work/standalone.log"; echo $?)" \
+  "驱动脚本退 $standalone_exit，标记核 $marker_exit（$marker_output）；驱动脚本输出尾部：$(tail -15 "$work/standalone.log" | tr '\n' '|')"
+expect "① 不分片的计数行与双机分片 merge 之后记进标记的那一行逐字相同" "$([[ -n "$unsharded_count_line" && "$marker_count_line" == "$unsharded_count_line" ]]; echo $?)" \
+  "不分片「$unsharded_count_line」，标记里「$marker_count_line」"
+expect "① merge 那一行按片报线程（shards=2、shard_worker_threads=），记进了标记" "$(grep -q '^parallel_finished=LAYER0_PARALLEL_FINISHED .*shards=2 shard_worker_threads=' "$marker_path"; echo $?)" \
+  "标记 $marker_path 里：$(grep 'parallel_finished=' "$marker_path" 2>/dev/null)"
+expect "① 清了场、跑完复原了（回读过），第二台这一趟的目录删了" \
+  "$([[ ! -e "$work/quiesced" ]] && grep -q '清场之后回读过了' "$work/standalone.log" && grep -q '复原之后回读过了' "$work/standalone.log" \
+     && [[ -z "$(ls -A "$work/peer/runs" 2>/dev/null)" ]]; echo $?)" \
+  "清场标记还在：$([[ -e "$work/quiesced" ]] && echo 是 || echo 否)；第二台 runs/ 下：$(ls -A "$work/peer/runs" 2>/dev/null | tr '\n' ' ')"
+
+# ② --merged-log：门禁 54 号调的那一条
+bash "$driver" --merged-log "$selftest_case_key" "$copy" "$fingerprint" "$work/merged.log" > "$work/merged-driver.log" 2>&1
+merged_exit=$?
+expect "② --merged-log：退 0，merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数" \
+  "$([[ $merged_exit == 0 ]] && grep -q 'LAYER0_SHARD mode=merge shards=2 ' "$work/merged.log" && [[ "$(grep '^LAYER0_SHARDED ' "$work/merged.log")" == "$unsharded_count_line" ]]; echo $?)" \
+  "退 $merged_exit；驱动脚本输出尾部：$(tail -10 "$work/merged-driver.log" | tr '\n' '|')"
+
+# ③ 「第二台」的 rustc -Vv 第一行不同
+mkdir -p "$work/other-toolchain"
+printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$selftest_cargo" > "$work/other-toolchain/cargo"
+printf '#!/usr/bin/env bash\nif [[ "${1:-}" == -Vv ]]; then %q -Vv | sed "1s/\$/ (another build)/"; else exec %q "$@"; fi\n' "$real_rustc" "$real_rustc" > "$work/other-toolchain/rustc"
+chmod +x "$work/other-toolchain/cargo" "$work/other-toolchain/rustc"
+write_configuration "$work/other-toolchain.env" "$work/other-toolchain"
+SINGLEFS_LAYER0_SHARD_CONFIG="$work/other-toolchain.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/other-toolchain.log" 2>&1
+toolchain_exit=$?
+python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
+toolchain_marker_exit=$?
+expect "③ 「第二台」rustc -Vv 第一行不同 ⇒ 拒（退 1，说工具链不同），这批输入那一格标记不动" \
+  "$([[ $toolchain_exit == 1 && $toolchain_marker_exit == 0 ]] && grep -q '两台的工具链不同' "$work/other-toolchain.log"; echo $?)" \
+  "退 $toolchain_exit，标记核 $toolchain_marker_exit；输出：$(tail -4 "$work/other-toolchain.log" | tr '\n' '|')"
+
+# ④ 「第二台」那一片退 0 却没写账本
+mkdir -p "$work/no-ledger"
+printf '#!/usr/bin/env bash\nif [[ "${1:-}" == test ]]; then echo "LAYER0_SHARD mode=run shard=1/2 （自证：这一片不写账本）"; exit 0; fi\nexec %q "$@"\n' "$selftest_cargo" > "$work/no-ledger/cargo"
+printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$real_rustc" > "$work/no-ledger/rustc"
+chmod +x "$work/no-ledger/cargo" "$work/no-ledger/rustc"
+write_configuration "$work/no-ledger.env" "$work/no-ledger"
+git_common_directory="$(git -C "$copy" rev-parse --path-format=absolute --git-common-dir)"
+rm -f -- "${git_common_directory:?}/singlefs-layer0-progress/$fingerprint"/layer0-shard-*.tally "${work:?}/peer/progress/$fingerprint"/layer0-shard-*.tally
+SINGLEFS_LAYER0_SHARD_CONFIG="$work/no-ledger.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-ledger.log" 2>&1
+no_ledger_exit=$?
+python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
+no_ledger_marker_exit=$?
+expect "④ 「第二台」那一片没写账本 ⇒ 判红（退 1，说两片的账本不是各恰好一份），删这批输入那一格标记" \
+  "$([[ $no_ledger_exit == 1 && $no_ledger_marker_exit == 1 ]] && grep -q '两片的账本不是各恰好一份' "$work/no-ledger.log"; echo $?)" \
+  "退 $no_ledger_exit，标记核 $no_ledger_marker_exit；输出：$(tail -4 "$work/no-ledger.log" | tr '\n' '|')"
+
+# ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒
+grep -v '^PEER_CARGO_BIN_DIRECTORY=' "$work/layer0-shard.env" > "$work/missing-key.env"
+SINGLEFS_LAYER0_SHARD_CONFIG="$work/missing-key.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/missing-key.log" 2>&1
+missing_key_exit=$?
+SINGLEFS_LAYER0_SHARD_CONFIG="$work/no-such-configuration.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-configuration.log" 2>&1
+no_configuration_exit=$?
+expect "⑤ 配置缺键 PEER_CARGO_BIN_DIRECTORY、配置文件不在 ⇒ 配置判法各拒（退 1，说双机分片不能用），出路指到 layer0-shard.env.example" \
+  "$([[ $missing_key_exit == 1 && $no_configuration_exit == 1 ]] && grep -q '双机分片不能用' "$work/missing-key.log" && grep -q '双机分片不能用' "$work/no-configuration.log" \
+     && grep -q 'layer0-shard.env.example' "$work/missing-key.log" && grep -q 'layer0-shard.env.example' "$work/no-configuration.log"; echo $?)" \
+  "缺键退 $missing_key_exit（$(tail -3 "$work/missing-key.log" | tr '\n' '|')），不在退 $no_configuration_exit（$(tail -3 "$work/no-configuration.log" | tr '\n' '|')）"
+
+# ⑥ 没登记 shard=across-machines 的用例
+bash "$driver" crash-case:c561-sigma-full "$copy" > "$work/not-shardable.log" 2>&1
+not_shardable_exit=$?
+expect "⑥ 没登记 shard=across-machines 的用例 ⇒ 拒（退 1），出路是单机跑" \
+  "$([[ $not_shardable_exit == 1 ]] && grep -q '没登记 shard=across-machines' "$work/not-shardable.log"; echo $?)" \
+  "退 $not_shardable_exit；输出：$(tail -3 "$work/not-shardable.log" | tr '\n' '|')"
+
+if (( failures > 0 )); then
+  echo "  ✗ layer0-shard-run.sh 自证没过：${failures} 格判错（共 ${checked} 格）"
+  echo "     → 怎么办：照上面每一格的说明改驱动脚本（research/scripts/layer0-shard-run.sh）或分片那一段（crates/singlefs-harness/src/layer0_progress.rs、crash.rs）"
+  exit 1
+fi
+echo "  ✓ layer0-shard-run.sh 自证通过：${checked} 格都对（${cargo_mode}；第二台是本机上的另一个目录，没碰真的第二台）"
--- /dev/null
+++ b/research/scripts/layer0-shard-configuration-check.sh
@@ -0,0 +1,98 @@
+#!/usr/bin/env bash
+# 层 0 崩溃重放双机分片（里程碑三第六项，.claude/kb/milestone/03-third-txn.md 第六节）的本地配置：在不在、键齐不齐、写得对不对、
+# 第二台连不连得上、它上面有没有 cargo。门禁 54 号 --full 拿它定走不走分片；research/scripts/layer0-shard-run.sh 拿它当运行条件。
+#
+#   layer0-shard-configuration-check.sh [<仓根>]
+#       退 0 能分片（stdout 一句：配置在哪、第二台是谁）；退 1 不能（stdout 一句原因，下一行是出路）。
+#   layer0-shard-configuration-check.sh --emit-assignments [<仓根>]
+#       判法同上；能分片时 stdout 改打配置里每个键的 bash 赋值（printf %q，调用方 eval），不打那一句。
+#
+# 配置文件：${SINGLEFS_LAYER0_SHARD_CONFIG:-<主工作树的根>/layer0-shard.env}。主工作树的根按 git common-dir 取（在 HEAD + 暂存区的
+# 临时 worktree 里跑也读主工作树那一份）；文件 git 忽略、不进仓，模板是仓根的 layer0-shard.env.example。
+# 写法：一行一个 KEY=值（值不做 shell 展开；两头成对的单引号或双引号去掉一层），# 起头的行与空行不算；认不出的行、不认得的键、
+# 缺键都判不能分片。键与各自的意思见 layer0-shard.env.example。
+# 只供测试的开关：SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1 时不连第二台、不查它的 cargo（layer0-shard-run.sh --selftest 用，
+# 那时「第二台」是本机上的另一个目录）。
+#
+# admission: always 判的是这一刻配置文件与第二台的样子，每次调都要现判
+# run-condition: command git bash
+set -uo pipefail
+source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
+preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
+layer0_shard_check_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
+
+LAYER0_SHARD_CONFIGURATION_KEYS=(PEER_SSH_HOST PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY
+  QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND)
+
+emit_assignments=0
+if [[ "${1:-}" == --emit-assignments ]]; then emit_assignments=1; shift; fi
+repository_root="${1:-$layer0_shard_check_script_directory/../..}"
+
+refuse() { # refuse <原因>：打原因与出路，退 1
+  echo "  ✗ 双机分片不能用：$1"
+  echo "     → 怎么办：照仓根 layer0-shard.env.example 建本地配置 layer0-shard.env（git 忽略），或设 SINGLEFS_LAYER0_SHARD_CONFIG 指到它；第二台连不上先修 ssh 免密登录"
+  exit 1
+}
+
+if [[ -n "${SINGLEFS_LAYER0_SHARD_CONFIG:-}" ]]; then
+  configuration_file="$SINGLEFS_LAYER0_SHARD_CONFIG"
+else
+  if ! common_directory="$(git -C "$repository_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$common_directory" ]]; then
+    refuse "$repository_root 不是 git 工作树，找不到主工作树的根（配置默认在那里）"
+  fi
+  configuration_file="$(dirname "$common_directory")/layer0-shard.env"
+fi
+[[ -f "$configuration_file" ]] || refuse "没有配置文件 $configuration_file"
+
+declare -A configuration=()
+line_number=0
+while IFS= read -r configuration_line || [[ -n "$configuration_line" ]]; do
+  line_number=$((line_number + 1))
+  [[ "$configuration_line" =~ ^[[:space:]]*(#.*)?$ ]] && continue
+  [[ "$configuration_line" =~ ^([A-Z_]+)=(.*)$ ]] || refuse "$configuration_file 第 $line_number 行认不出（要是 KEY=值）"
+  configuration_key="${BASH_REMATCH[1]}"
+  configuration_value="${BASH_REMATCH[2]}"
+  if [[ "$configuration_value" =~ ^\'(.*)\'$ || "$configuration_value" =~ ^\"(.*)\"$ ]]; then configuration_value="${BASH_REMATCH[1]}"; fi
+  known_key=0
+  for expected_key in "${LAYER0_SHARD_CONFIGURATION_KEYS[@]}"; do [[ "$expected_key" == "$configuration_key" ]] && known_key=1; done
+  (( known_key )) || refuse "$configuration_file 第 $line_number 行的键 $configuration_key 不认得（认的是 ${LAYER0_SHARD_CONFIGURATION_KEYS[*]}）"
+  [[ -z "${configuration[$configuration_key]+set}" ]] || refuse "$configuration_file 里 $configuration_key 写了两遍"
+  configuration[$configuration_key]="$configuration_value"
+done < "$configuration_file"
+missing_keys=()
+for expected_key in "${LAYER0_SHARD_CONFIGURATION_KEYS[@]}"; do
+  [[ -n "${configuration[$expected_key]+set}" ]] || missing_keys+=("$expected_key")
+done
+(( ${#missing_keys[@]} == 0 )) || refuse "$configuration_file 缺键 ${missing_keys[*]}（不清场的两对写成空串，键照样要写）"
+[[ -n "${configuration[PEER_SSH_HOST]}" ]] || refuse "PEER_SSH_HOST 是空的"
+for directory_key in PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY; do
+  [[ "${configuration[$directory_key]}" == /* ]] || refuse "$directory_key 要写第二台上的绝对路径，读到「${configuration[$directory_key]}」"
+done
+for command_pair in "QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND" "QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND"; do
+  read -r action_key check_key <<< "$command_pair"
+  if [[ -n "${configuration[$action_key]}" && -z "${configuration[$check_key]}" ]] || [[ -z "${configuration[$action_key]}" && -n "${configuration[$check_key]}" ]]; then
+    refuse "$action_key 与 $check_key 要么都写、要么都空：改了状态的命令跑完要回读一次"
+  fi
+done
+if [[ -n "${configuration[QUIESCE_STOP_COMMAND]}" && -z "${configuration[QUIESCE_START_COMMAND]}" ]]; then
+  refuse "写了 QUIESCE_STOP_COMMAND 就要写 QUIESCE_START_COMMAND：清了场要复原"
+fi
+
+peer_description="第二台 ${configuration[PEER_SSH_HOST]}"
+if [[ "${SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE:-0}" == 1 ]]; then
+  peer_description="第二台是本机（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1）"
+else
+  ssh -o BatchMode=yes -o ConnectTimeout=10 "${configuration[PEER_SSH_HOST]}" true </dev/null >/dev/null 2>&1 \
+    || refuse "ssh -o BatchMode=yes ${configuration[PEER_SSH_HOST]} true 连不上"
+  ssh -o BatchMode=yes -o ConnectTimeout=10 "${configuration[PEER_SSH_HOST]}" "test -x $(printf '%q' "${configuration[PEER_CARGO_BIN_DIRECTORY]}")/cargo" </dev/null >/dev/null 2>&1 \
+    || refuse "第二台上没有 ${configuration[PEER_CARGO_BIN_DIRECTORY]}/cargo（PEER_CARGO_BIN_DIRECTORY 要指到它的 ~/.cargo/bin）"
+fi
+
+if (( emit_assignments )); then
+  for expected_key in "${LAYER0_SHARD_CONFIGURATION_KEYS[@]}"; do
+    printf '%s=%q\n' "$expected_key" "${configuration[$expected_key]}"
+  done
+  printf 'LAYER0_SHARD_CONFIGURATION_FILE=%q\n' "$configuration_file"
+else
+  echo "配置 $configuration_file，$peer_description"
+fi
```

## 三、`git diff --stat ecdf8465 -- <9 paths>`（原样输出；标出哪几处不归这一轮）

主 agent 给的 9 条路径：`.claude/gate.d/54-layer0-replay.sh .claude/gate.d/stage-inputs.tsv research/scripts/admission.py .claude/hooks/lib_heavy_tests.py .claude/hooks/heavy-test-guard.sh .claude/rules/implementation-workflow.md research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-run-selftest.sh research/scripts/layer0-shard-configuration-check.sh`。

```
 .claude/gate.d/54-layer0-replay.sh            | 150 ++++++++++++++++++++++++--
 .claude/gate.d/stage-inputs.tsv               |   2 +-
 .claude/rules/implementation-workflow.md      |   4 +-
 research/scripts/layer0-shard-run-selftest.sh |  34 ++++++
 research/scripts/layer0-shard-run.sh          |  60 ++++++++---
 5 files changed, 227 insertions(+), 23 deletions(-)
```

正文第一节「不归这一轮的」点名：`.claude/gate.d/54-layer0-replay.sh`（这份统计里的 +150 行）与 `research/scripts/layer0-shard-run.sh`（+60 行）里读层 0 发现日志的那几段，以及 `research/scripts/layer0-shard-run-selftest.sh`（+34 行，自证第 ⑦ 格，随发现日志那件一起加），是 2026-09-27 JST 11:4x 工具员加的（报告 `research/prompts/defs-gate54-findings-report.md`；工作区里还没提交），与写发现日志的 `crates/singlefs-harness/src/crash.rs` 一起归代码第二轮三方，不归这一轮。这一趟统计里剩下的 `.claude/gate.d/stage-inputs.tsv`（+2 行）与 `.claude/rules/implementation-workflow.md`（+4 行）：`stage-inputs.tsv` 的改动是正文第二节写明的「第 34 行 E142 那一行加了一个输入 `research/prompts/e142-r19-prereg.md`，与崩溃枚举无关」；`implementation-workflow.md` 的改动是正文第二节写明的「已提交的 2 行（别的会话的措辞改动）」——现跑 `git diff ecdf8465 -- .claude/rules/implementation-workflow.md`，两行改动都是遣词，不涉判据字面（材料员现查，见下）。`research/scripts/admission.py`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`research/scripts/layer0-shard-configuration-check.sh` 四份在这份统计里没有出现，即与 `ecdf8465` 逐字节相同，与正文第二节「逐字节相同」的声明一致。

`git diff ecdf8465 -- .claude/rules/implementation-workflow.md` 原样：

```diff
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index f04de53d..b551f1dd 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -61,10 +61,10 @@
 
 | 场合 | 跑不跑 |
 |---|---|
-| 每次提交代码 | **必须跑**，命令都带 `SINGLEFS_HEAVY_TESTS=commit`：层 0 全量由主 agent 在提交流程里后台起、看门狗盯；QEMU、herd7、crates 变异整表由 `crash-verifier` 跑；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
+| 每次提交代码 | **必须跑**，命令都带 `SINGLEFS_HEAVY_TESTS=commit`：层 0 全量（HEAD + 暂存区的 worktree 里 `--full`，连同登记的崩溃枚举用例）、QEMU、herd7、crates 变异整表由 `crash-verifier` 跑，主 agent 后台派、看门狗盯；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
 | 用户当场要求，或任务确实要跑 | 任务确实要跑时主 agent 先弹窗问用户，用户同意了才跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request` |
 | 其余任何时候 | 不跑 |
-| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：层 0 全量由主 agent 在 HEAD + 暂存区的 worktree 里跑；崩溃验证员跑 55、57、59 号；门禁分诊员跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`）并分诊：54、55、59、74、87 号只在这一趟里能复用上一次整轮全绿的判定（判据在 `research/scripts/stage-must-run.sh` 文件头），要跑时 54 号跑快档并核全绿标记，57 号没有复用、照跑 |
+| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：崩溃验证员在 HEAD + 暂存区的 worktree 里跑 54 号 `--full`（连同登记的崩溃枚举用例），另跑 55、57、59 号；门禁分诊员跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`）并分诊：54、55、59、74、87 号只在这一趟里能复用上一次整轮全绿的判定（判据在 `research/scripts/stage-must-run.sh` 文件头），要跑时 54 号跑快档并核全绿标记，57 号没有复用、照跑 |
 | 其余子 agent | **一律不跑**，只跑自己动到的测试二进制与 fmt / clippy / build |
 
 由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑上面任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带那个环境变量前缀的拒绝；主 agent 不带那个前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。
```

## 四、新文件全文（G2 diff 里 `new file mode` 的三份，取自提交 `ecdf8465`，与正文行数一致：280 / 207 / 98 行）

### `research/scripts/layer0-shard-run.sh`（新文件，280 行）

```bash
#!/usr/bin/env bash
# 层 0 崩溃重放按双机分片跑一条崩溃枚举用例（里程碑三第六项，.claude/kb/milestone/03-third-txn.md 第六节）：本机跑第 0/2 片、
# 第二台跑第 1/2 片（SINGLEFS_LAYER0_SHARD，切法与账本在 crates/singlefs-harness/src/layer0_progress.rs），第二台的账本拷回本机，
# 本机 SINGLEFS_LAYER0_SHARD=merge/2 再跑同一条用例（只读两份账本、核齐、按切片序号并，不枚举）。
#
#   layer0-shard-run.sh <崩溃枚举用例的键> <树根>
#       两台各跑一片、merge，merge 那一趟的日志照门禁 54 号 --full 判（admission.py crash-case-judge），判绿写 54 号那一格全绿标记
#       （同一组标记、同一个输入指纹，crash-case-record）；判红删这批输入那一格。树根是 HEAD + 暂存区那棵树（建法同 54 号 --full）。
#   layer0-shard-run.sh --merged-log <崩溃枚举用例的键> <树根> <输入指纹> <日志文件>
#       门禁 54 号 --full 在分片开着时调：只做两片与 merge，merge 那一趟的整段输出写进日志文件，退出码是那一趟的；判与写标记归 54 号。
#   layer0-shard-run.sh --selftest
#       转给 research/scripts/layer0-shard-run-selftest.sh：不碰第二台，本机两个进程各跑一片走同一条路（含 merge、判与标记）。
#
# 步骤：① 两台各 rustc -Vv、cargo -V、nproc，rustc -Vv 前三行与 host 行、cargo -V 不同就拒；② 树 rsync 到第二台
# <PEER_REPOSITORY_DIRECTORY>/runs/<这一趟>/（新建的空目录，不带 --delete），在那里 git init（算输入指纹要 git 列文件）；
# ③ 两台各算这条用例的输入指纹（admission.py crash-case-manifest，参数与 54 号相同），不同就拒；清场（配置里写了的话，回读验证，
# 复原挂在 EXIT 上，跑红了也复原）；④ 两片同时跑，各记 pid、各 wait 取退出码、各写自己的日志，LAYER0_PROGRESS 边跑边转出来；
# ⑤ 第二台的账本拷回本机进度目录；⑥ 本机 merge；⑦ 删掉第二台上这一趟的树与编译目录（两片的进度目录留着：被杀之后下一趟接着跑）。
# 进度目录：本机 <git common-dir>/singlefs-layer0-progress/<输入指纹>（与 54 号相同），第二台 <PEER_REPOSITORY_DIRECTORY>/progress/<输入指纹>。
# 线程：本机那一片取 SINGLEFS_LAYER0_THREADS（没设取本机 nproc），第二台那一片取第二台的 nproc。SINGLEFS_LAYER0_START_OVER=1 交给两片，
# merge 那一趟不带。只接登记了 shard=across-machines 的用例（它的枚举经 Layer0Resume::from_environment 认分片开关）。
# 登记的崩溃枚举用例都标了 #[ignore]，三趟 cargo test 都带 --include-ignored --exact <用例函数>；只供测试的开关
# SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0 时不带 --include-ignored（--selftest 那条小流用例不标 ignore，不必把别的 ignore 用例放进过滤范围）。
#
# 配置（本地、git 忽略）：${SINGLEFS_LAYER0_SHARD_CONFIG:-<主工作树的根>/layer0-shard.env}，键与写法见仓根 layer0-shard.env.example；
# 在不在、齐不齐、第二台连不连得上由 research/scripts/layer0-shard-configuration-check.sh 判（门禁 54 号拿同一份判走不走分片）。
# 这一份与配置判法按内容进登记了 shard=across-machines 的用例的输入指纹（research/scripts/admission.py 的 SHARD_DRIVER_FILES）：两片与 merge 的
# cargo 由这里起、不经 admission.py crash-case-command，改了这里那几条用例的旧全绿标记不再作数。
# 重型：跑的是标了 ignore 的崩溃枚举用例（--include-ignored），只在提交时（SINGLEFS_HEAVY_TESTS=commit）或用户要求时跑；--selftest 不算。
#
# 配置在不在、判不判得过不写成运行条件（--selftest 不要配置）：开跑之后第一件事调配置判法，判不过照它的原因与出路退 1。
#
# admission: always 每次调都跑这一刻这棵树上的一条崩溃枚举用例；复用判定在门禁 54 号的全绿标记里，不在这里
# run-condition: command cargo rustc python3 rsync ssh nproc git
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
layer0_shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[[ "${1:-}" != --selftest ]] || exec bash "$layer0_shard_script_directory/layer0-shard-run-selftest.sh"

layer0_shard_mode=standalone
if [[ "${1:-}" == --merged-log ]]; then layer0_shard_mode=merged-log; shift; fi
case_key="${1:-}"
tree_root="${2:-}"
given_fingerprint="${3:-}"
merged_log_file="${4:-}"
usage_ok=1
[[ "$case_key" == crash-case:* && -n "$tree_root" && -d "$tree_root" ]] || usage_ok=0
if [[ "$layer0_shard_mode" == merged-log && ( ! "$given_fingerprint" =~ ^[0-9a-f]{64}$ || -z "$merged_log_file" ) ]]; then usage_ok=0; fi
if (( ! usage_ok )); then
  echo "  ✗ 用法：layer0-shard-run.sh <crash-case:键> <树根>，或 --merged-log <crash-case:键> <树根> <输入指纹> <日志文件>，或 --selftest"
  echo "     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 那一行写；树根是 HEAD + 暂存区那棵树（建法见门禁 54 号 --full 的出路句）"
  exit 2
fi
tree_root="$(cd "$tree_root" && pwd)"
admission_module="$tree_root/research/scripts/admission.py"

fail() { # fail <原因> <出路>
  echo "  ✗ $case_key 双机分片：$1"
  echo "     → 怎么办：$2"
  exit 1
}
fail_after_the_run() { # fail_after_the_run <原因> <出路>：跑过之后判红——单独跑时删这批输入那一格全绿标记（先绿后红，前一趟那一格不再作数），再 fail
  local marker_path
  if [[ "$layer0_shard_mode" == standalone ]] \
      && marker_path="$(python3 "$admission_module" crash-case-marker-path "$tree_root" "$case_key" "$local_fingerprint")" && [[ -n "$marker_path" ]]; then
    rm -f -- "${marker_path:?}"
  fi
  fail "$1" "$2"
}

if ! configuration_assignments="$(bash "$layer0_shard_script_directory/layer0-shard-configuration-check.sh" --emit-assignments "$tree_root")"; then
  printf '%s\n' "$configuration_assignments"
  exit 1
fi
eval "$configuration_assignments"
peer_is_this_machine="${SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE:-0}"

# 这条用例登记的包、测试目标、用例函数（第三列的 test=）；要登记了 shard=across-machines
registration_row="$(awk -F'\t' -v key="$case_key" '$1 == key' "$tree_root/.claude/gate.d/stage-inputs.tsv")"
test_token="$(tr ' ' '\n' <<< "$(cut -f3 <<< "$registration_row")" | grep '^test=' | head -1)"
IFS=: read -r case_package case_target case_function <<< "${test_token#test=}"
[[ -n "$registration_row" && -n "$case_package" && -n "$case_target" && -n "$case_function" ]] \
  || fail "stage-inputs.tsv 里找不到这一行或它的 test=<包>:<测试目标>:<用例函数>" "照 research/scripts/admission.py 文件头「崩溃枚举用例行」登记"
python3 "$admission_module" crash-case-shardable "$tree_root" "$case_key" >/dev/null \
  || fail "这条用例没登记 shard=across-machines（它的枚举不认分片开关，分了片也是每台各跑全量）" "单机跑它：bash .claude/gate.d/54-layer0-replay.sh --full <树根>；要分片先让用例经 Layer0Resume::from_environment 取续跑设置，再在登记行第三列加 shard=across-machines"

if ! git_common_directory="$(git -C "$tree_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
  fail "$tree_root 不是 git 工作树（取不到 git common-dir）" "在 HEAD + 暂存区的 worktree 里跑，建法见门禁 54 号 --full 的出路句"
fi
run_label="${case_key#crash-case:}-$(date -u +%Y%m%dT%H%M%SZ)-$$"
peer_run_directory="$PEER_REPOSITORY_DIRECTORY/runs/$run_label"
scratch_directory="$(mktemp -d)"
quiesced=0
restore_on_exit() {
  if (( quiesced )); then
    echo "  · 复原：$QUIESCE_START_COMMAND"
    bash -c "$QUIESCE_START_COMMAND"
    if bash -c "$QUIESCE_STARTED_CHECK_COMMAND"; then
      echo "  · 复原之后回读过了（$QUIESCE_STARTED_CHECK_COMMAND 退 0）"
    else
      echo "  ✗ 复原之后回读没过：$QUIESCE_STARTED_CHECK_COMMAND 退非 0"
      echo "     → 怎么办：手动复原（$QUIESCE_START_COMMAND），回读到它退 0 为止"
    fi
  fi
  rm -rf -- "${scratch_directory:?}"
}
trap restore_on_exit EXIT

# run_on_peer <目录> <命令文本>：在第二台的这个目录里 bash -c 跑（PATH 前面加 PEER_CARGO_BIN_DIRECTORY）；第二台是本机（只供测试）时本机跑
run_on_peer() {
  if [[ "$peer_is_this_machine" == 1 ]]; then
    (cd "$1" && PATH="$PEER_CARGO_BIN_DIRECTORY:$PATH" bash -c "$2")
  else
    ssh -o BatchMode=yes "$PEER_SSH_HOST" "cd $(printf '%q' "$1") && PATH=$(printf '%q' "$PEER_CARGO_BIN_DIRECTORY"):\$PATH bash -c $(printf '%q' "$2")" </dev/null
  fi
}
# copy_to_peer / copy_from_peer：rsync -a，不带 --delete
copy_to_peer() {
  if [[ "$peer_is_this_machine" == 1 ]]; then rsync -a "$@"; else rsync -a -e "ssh -o BatchMode=yes" "$@"; fi
}
peer_path() { # peer_path <第二台上的路径>：rsync 的参数写法
  if [[ "$peer_is_this_machine" == 1 ]]; then printf '%s' "$1"; else printf '%s:%s' "$PEER_SSH_HOST" "$1"; fi
}
if [[ "$peer_is_this_machine" == 1 ]]; then
  echo "  ! 第二台是本机上的 $PEER_REPOSITORY_DIRECTORY（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1）"
fi

# ① 工具链：rustc -Vv 前三行与 host 行、cargo -V 两台相同；线程数各取各的核数
toolchain_description_command='rustc -Vv | head -3 && rustc -Vv | grep "^host: " && cargo -V'
local_toolchain="$(bash -c "$toolchain_description_command")" || fail "本机 rustc / cargo 报不出版本" "看 PATH 里有没有 rustc 与 cargo"
peer_toolchain="$(run_on_peer / "$toolchain_description_command")" || fail "第二台 rustc / cargo 报不出版本" "看 $PEER_CARGO_BIN_DIRECTORY 下有没有 rustc 与 cargo"
[[ "$local_toolchain" == "$peer_toolchain" ]] \
  || fail "两台的工具链不同：本机「${local_toolchain//$'\n'/；}」，第二台「${peer_toolchain//$'\n'/；}」" "两台装同一个 rustup 工具链（rustc -Vv 的 commit 相同）再跑"
local_cores="$(nproc)"
peer_cores="$(run_on_peer / nproc)" || fail "第二台 nproc 跑不起来" "看 ssh $PEER_SSH_HOST 能不能跑命令"
if [[ -n "${SINGLEFS_LAYER0_THREADS:-}" ]]; then
  local_threads="$SINGLEFS_LAYER0_THREADS"; local_threads_origin=explicit; local_threads_text="SINGLEFS_LAYER0_THREADS=${local_threads}（显式设的）"
else
  local_threads="$local_cores"; local_threads_origin=default; local_threads_text="SINGLEFS_LAYER0_THREADS=${local_threads}（没设，取本机核数）"
fi
echo "  · ① 工具链两台相同（${local_toolchain%%$'\n'*}）；本机 ${local_cores} 核跑 0/2（${local_threads_text}），第二台 ${peer_cores} 核跑 1/2（取第二台核数）"

# ② 树拷到第二台这一趟的专用目录（新建的空目录，不带 --delete），在那里 git init
run_on_peer / "mkdir -p $(printf '%q' "$peer_run_directory") $(printf '%q' "$PEER_REPOSITORY_DIRECTORY/progress")" || fail "第二台建不了 $peer_run_directory" "看 PEER_REPOSITORY_DIRECTORY 可不可写"
copy_to_peer --exclude target --exclude .git "$tree_root/" "$(peer_path "$peer_run_directory")/" || fail "树 rsync 到第二台失败" "看第二台盘满没满、$PEER_REPOSITORY_DIRECTORY 可不可写"
run_on_peer "$peer_run_directory" "git init -q ." || fail "第二台上 git init 失败" "看第二台装没装 git"
echo "  · ② 树拷到第二台 $peer_run_directory"

# ③ 两台各算这条用例的输入指纹（参数与门禁 54 号相同：准入模块的判法摘要、工具链与构建环境；登记了 shard=across-machines 的，
# 这一份驱动脚本与配置判法由准入模块按内容加进清单）
local_manifest="$scratch_directory/manifest.local"
local_summary="$(python3 "$admission_module" crash-case-manifest "$tree_root" "$case_key" "$local_manifest" \
  --judging-digest --toolchain --build-environment)" || fail "本机算不出输入指纹：$local_summary" "单跑 python3 research/scripts/admission.py crash-case-manifest <树根> $case_key <清单文件> --judging-digest --toolchain --build-environment 看它报什么"
read -r local_fingerprint local_file_count local_excluded_count <<< "$local_summary"
peer_summary="$(run_on_peer "$peer_run_directory" "python3 research/scripts/admission.py crash-case-manifest . $(printf '%q' "$case_key") .layer0-shard-manifest \
  --judging-digest --toolchain --build-environment")" || fail "第二台算不出输入指纹：$peer_summary" "在第二台 $peer_run_directory 里单跑同一条 crash-case-manifest 看它报什么"
peer_fingerprint="${peer_summary%% *}"
[[ "$local_fingerprint" == "$peer_fingerprint" ]] \
  || fail "两台算出的输入指纹不同（本机 ${local_fingerprint:0:16}…，第二台 ${peer_fingerprint:0:16}…）：构建环境（~/.cargo/config、RUSTFLAGS 这一类）或树不一样" "对一对两台的 ~/.cargo/config*、环境变量 RUSTFLAGS / CARGO_*，改成一样再跑"
if [[ "$layer0_shard_mode" == merged-log && "$local_fingerprint" != "$given_fingerprint" ]]; then
  fail "本机算出的输入指纹 ${local_fingerprint:0:16}… 与 54 号给的 ${given_fingerprint:0:16}… 不同" "别在有人改这些路径的树里跑；在 HEAD + 暂存区的 worktree 里重跑 54 号 --full"
fi
echo "  · ③ 两台的输入指纹相同：${local_fingerprint:0:16}…（${local_file_count} 个文件，减去用例读不到的 ${local_excluded_count} 个）"

local_progress_directory="$git_common_directory/singlefs-layer0-progress/$local_fingerprint"
peer_progress_directory="$PEER_REPOSITORY_DIRECTORY/progress/$local_fingerprint"
mkdir -p "$local_progress_directory"
run_on_peer / "mkdir -p $(printf '%q' "$peer_progress_directory")" || fail "第二台建不了进度目录 $peer_progress_directory" "看 PEER_REPOSITORY_DIRECTORY 可不可写"

# 清场（配置里写了才做）：跑完回读，复原挂在 EXIT 上
if [[ -n "$QUIESCE_STOP_COMMAND" ]]; then
  echo "  · 清场：$QUIESCE_STOP_COMMAND"
  quiesced=1
  bash -c "$QUIESCE_STOP_COMMAND"
  bash -c "$QUIESCE_STOPPED_CHECK_COMMAND" || fail "清场之后回读没过：$QUIESCE_STOPPED_CHECK_COMMAND 退非 0" "手动清场，回读到它退 0 为止；复原命令在退出时照跑"
  echo "  · 清场之后回读过了（$QUIESCE_STOPPED_CHECK_COMMAND 退 0）"
fi

# ④ 两片同时跑
start_over_settings=(-u SINGLEFS_LAYER0_START_OVER)
start_over_text=""
if [[ "${SINGLEFS_LAYER0_START_OVER:-0}" == 1 ]]; then start_over_settings=(SINGLEFS_LAYER0_START_OVER=1); start_over_text="SINGLEFS_LAYER0_START_OVER=1 "; fi
libtest_arguments=(--exact "$case_function" --nocapture)
case "${SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED:-1}" in
  1) libtest_arguments=(--include-ignored "${libtest_arguments[@]}") ;;
  0) echo "  ! 不带 --include-ignored（只供测试的开关 SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0；标了 ignore 的用例这样跑一条都跑不到，会判红）" ;;
  *) fail "SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED 只许 0 或 1，读到「${SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED}」" "不设它（默认 1）；只有 --selftest 设成 0" ;;
esac
cargo_test_text="cargo test --release -p $case_package --test $case_target -- ${libtest_arguments[*]}"
local_log="$scratch_directory/shard-0-of-2.log"
peer_log="$scratch_directory/shard-1-of-2.log"
rm -f -- "$scratch_directory"/exit.*
{
  ( cd "$tree_root" && env "${start_over_settings[@]}" SINGLEFS_LAYER0_SHARD=0/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY="$local_progress_directory" \
    SINGLEFS_LAYER0_INPUT_FINGERPRINT="$local_fingerprint" SINGLEFS_LAYER0_THREADS="$local_threads" \
    cargo test --release -p "$case_package" --test "$case_target" -- "${libtest_arguments[@]}" ) 2>&1 \
    | tee "$local_log" | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } | sed -u 's/^/    [本机 0\/2] /'
  echo "${PIPESTATUS[0]}" > "$scratch_directory/exit.local"
} &
local_shard_process=$!
{
  run_on_peer "$peer_run_directory" "env SINGLEFS_HEAVY_TESTS=${SINGLEFS_HEAVY_TESTS:-} ${start_over_text}SINGLEFS_LAYER0_SHARD=1/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=$(printf '%q' "$peer_progress_directory") SINGLEFS_LAYER0_INPUT_FINGERPRINT=$local_fingerprint SINGLEFS_LAYER0_THREADS=$peer_cores $(printf '%q ' cargo test --release -p "$case_package" --test "$case_target" -- "${libtest_arguments[@]}") 2>&1" \
    | tee "$peer_log" | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } | sed -u 's/^/    [第二台 1\/2] /'
  echo "${PIPESTATUS[0]}" > "$scratch_directory/exit.peer"
} &
peer_shard_process=$!
echo "  · ④ 两片开跑：本机 ${start_over_text}SINGLEFS_LAYER0_SHARD=0/2 $cargo_test_text（pid $local_shard_process），第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条（pid $peer_shard_process）"
wait "$local_shard_process"
wait "$peer_shard_process"
local_exit="$(cat "$scratch_directory/exit.local" 2>/dev/null || echo missing)"
peer_exit="$(cat "$scratch_directory/exit.peer" 2>/dev/null || echo missing)"
shard_problems=()
[[ "$local_exit" == 0 ]] || shard_problems+=("本机那一片 cargo test 退 $local_exit")
[[ "$peer_exit" == 0 ]] || shard_problems+=("第二台那一片 cargo test 退 $peer_exit")
grep -q 'LAYER0_SHARD mode=run shard=0/2 ' "$local_log" || shard_problems+=("本机的日志里没有 LAYER0_SHARD mode=run shard=0/2 那一行（这条用例没按分片跑）")
grep -q 'LAYER0_SHARD mode=run shard=1/2 ' "$peer_log" || shard_problems+=("第二台的日志里没有 LAYER0_SHARD mode=run shard=1/2 那一行（这条用例没按分片跑）")
remove_peer_run_directory() { # ⑦ 删掉第二台上这一趟的树与编译目录
  local peer_run_size
  peer_run_size="$(run_on_peer / "du -sh $(printf '%q' "$peer_run_directory") | cut -f1")"
  if run_on_peer / "rm -rf -- $(printf '%q' "${peer_run_directory:?}")"; then
    echo "  · ⑦ 删掉第二台上这一趟的树与编译目录 $peer_run_directory（${peer_run_size:-大小没读到}）；两片的进度目录留着"
  else
    echo "  ! 第二台上的 $peer_run_directory 没删掉：下一趟另起一个目录，不碍事；手动删它"
  fi
}
if (( ${#shard_problems[@]} > 0 )); then
  echo "    本机那一片的日志尾部："; tail -20 "$local_log" | sed 's/^/      /'
  echo "    第二台那一片的日志尾部："; tail -20 "$peer_log" | sed 's/^/      /'
  remove_peer_run_directory
  fail_after_the_run "$(IFS='；'; echo "${shard_problems[*]}")" "单跑那一片看细节（本机：SINGLEFS_LAYER0_SHARD=0/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=… SINGLEFS_LAYER0_INPUT_FINGERPRINT=… $cargo_test_text）；断言消息里是第一处对不上的计数或违例"
fi
echo "  · ④ 两片都跑完"

# ⑤ 第二台的账本拷回本机进度目录
copy_to_peer --include 'layer0-shard-*-shard-1-of-2.tally' --exclude '*' "$(peer_path "$peer_progress_directory")/" "$local_progress_directory/" \
  || fail_after_the_run "第二台的账本拷不回来" "看 $peer_progress_directory 里有没有 layer0-shard-*-shard-1-of-2.tally"
remove_peer_run_directory
fetched_ledgers=("$local_progress_directory"/layer0-shard-*-shard-1-of-2.tally)
own_ledgers=("$local_progress_directory"/layer0-shard-*-shard-0-of-2.tally)
[[ -f "${fetched_ledgers[0]}" && ${#fetched_ledgers[@]} == 1 && -f "${own_ledgers[0]}" && ${#own_ledgers[@]} == 1 ]] \
  || fail_after_the_run "本机进度目录里两片的账本不是各恰好一份（第 0 片 ${#own_ledgers[@]} 份、第 1 片 ${#fetched_ledgers[@]} 份，在 $local_progress_directory）" "看两片的日志里 LAYER0_SHARD mode=run 那一行的 ledger= 在哪；删掉多出来的旧账本再跑"
echo "  · ⑤ 第二台的账本拷回 ${fetched_ledgers[0]}"

# ⑥ 本机 merge：只读两份账本、核齐、按切片序号并，照旧走用例钉死的计数断言
merge_log="$scratch_directory/merge.log"
merge_exit=0
( cd "$tree_root" && env -u SINGLEFS_LAYER0_START_OVER SINGLEFS_LAYER0_SHARD=merge/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY="$local_progress_directory" \
  SINGLEFS_LAYER0_INPUT_FINGERPRINT="$local_fingerprint" SINGLEFS_LAYER0_THREADS="$local_threads" \
  cargo test --release -p "$case_package" --test "$case_target" -- "${libtest_arguments[@]}" ) > "$merge_log" 2>&1 || merge_exit=$?
if [[ "$merge_exit" == 0 ]] && ! grep -q 'LAYER0_SHARD mode=merge shards=2 ' "$merge_log"; then merge_exit=merge-line-missing; fi
if [[ "$layer0_shard_mode" == merged-log ]]; then
  cp -- "$merge_log" "$merged_log_file"
  echo "  · ⑥ merge 那一趟退 $merge_exit，整段输出交给 54 号判（$merged_log_file）"
  [[ "$merge_exit" == 0 ]] && exit 0
  exit 1
fi
if [[ "$merge_exit" != 0 ]]; then
  tail -40 "$merge_log" | sed 's/^/      /'
  fail_after_the_run "merge 那一趟退 $merge_exit（上面是它的尾部；merge-line-missing 是日志里没有 LAYER0_SHARD mode=merge shards=2 那一行）" "账本核不齐时 panic 那一句说清缺哪一片、哪一处不同；计数断言红了与单机全量红了是同一回事，照 54 号 --full 的出路查"
fi
judged_lines_file="$scratch_directory/judged"
case_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if ! judge_output="$(python3 "$admission_module" crash-case-judge "$tree_root" "$case_key" "$merge_log" "$judged_lines_file" \
    --machine-cores "$local_cores" --threads "$local_threads" --threads-origin "$local_threads_origin")"; then
  printf '%s\n' "$judge_output" | sed 's/^/       /'
  fail_after_the_run "merge 那一趟跑过了，日志却判不绿（上面逐条列出）" "计数行、test result、exhaustive=true、每一片的工作线程，照 54 号 --full 同一句出路查"
fi
finish_manifest="$scratch_directory/manifest.finish"
finish_summary="$(python3 "$admission_module" crash-case-manifest "$tree_root" "$case_key" "$finish_manifest" \
  --judging-digest --toolchain --build-environment)" || fail "跑完之后算不出输入指纹：$finish_summary；不写全绿标记" "等改动停下，在 HEAD + 暂存区的 worktree 里重跑"
[[ "${finish_summary%% *}" == "$local_fingerprint" ]] \
  || fail "跑的过程中这条用例的输入变了（开跑 ${local_fingerprint:0:16}…，跑完 ${finish_summary:0:16}…）：不写全绿标记" "别在有人改这些路径的树里跑；在 HEAD + 暂存区的 worktree 里重跑"
threads_text="${judge_output//$'\n'/；}${judge_output:+；}双机分片：本机 ${local_threads_text}、本机 ${local_cores} 核，第二台 ${peer_cores} 核"
record_output="$(python3 "$admission_module" crash-case-record "$tree_root" "$case_key" "$local_fingerprint" "$local_manifest" "$judged_lines_file" \
  --files "$local_file_count" --excluded "$local_excluded_count" --started "$case_started_utc" --judged-root "$tree_root" \
  --machine-cores "$local_cores" --threads "$local_threads" --threads-origin "$local_threads_origin")" \
  || fail "判绿，全绿标记却没写成：$record_output" "看 $git_common_directory 可不可写、盘满没满"
echo "  ✓ $case_key 双机分片判绿（${threads_text}）：全绿标记写进 ${record_output}，记下的行原样："
sed 's/^/      /' "$judged_lines_file"
```

### `research/scripts/layer0-shard-run-selftest.sh`（新文件，207 行）

```bash
#!/usr/bin/env bash
# research/scripts/layer0-shard-run.sh --selftest 的本体：不碰第二台。拷一份仓进临时目录、git init，登记一条 shard=across-machines 的用例，
# 「第二台」是本机上的另一个目录（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1），本机两个进程各跑一片，走驱动脚本同一条路：
# 工具链与输入指纹两边比、清场与复原（回读）、两片、拷账本、merge、判、写 54 号那一格全绿标记。
# 默认用假 cargo（不编译、不跑用例）：登记的是临时仓里新写的一条标了 #[ignore] 的替身用例，三趟都不带 --include-ignored，不算重型；
# 假 cargo 分片跑时往进度目录写账本、merge 时核两份账本在不在、记的输入指纹对不对，打与层 0 同形的计数行与带 shards= 的线程行。
# 带 SINGLEFS_HEAVY_TESTS（commit 或 user-request）时换真 cargo：登记 crates/singlefs-harness/tests/crash_enumeration_sharded_across_processes.rs
# 里不标 ignore 的小流用例（第一条流按甲二展开 29 个状态），release 下真编真跑；不带时成功行写明真 cargo 那一趟本次未跑。逐格核：
#   ① 不分片跑那条用例打的计数行，与双机分片 merge 之后记进全绿标记的那一行逐字相同；标记作数；清场复原了；第二台这一趟的目录删了
#   ② --merged-log（门禁 54 号调的那一条）：merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数
#   ③ 「第二台」的 rustc -Vv 第一行不同 ⇒ 拒（工具链不同），标记不动
#   ④ 「第二台」那一片没写账本 ⇒ 判红，删这批输入那一格标记
#   ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒（退 1），出路指到 layer0-shard.env.example
#   ⑥ 没登记 shard=across-machines 的用例 ⇒ 拒
# 成功行报核了几格（现算）。
#
# admission: always 自证判的是这一刻的驱动脚本与仓，每次调都要现跑
# run-condition: command cargo rustc python3 rsync git nproc
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
selftest_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

repository_root="$(cd "$selftest_script_directory/../.." && pwd)"
work="$(mktemp -d)"
remove_work() {
  echo "  · 自证的临时目录 $work（$(du -sh "$work" 2>/dev/null | cut -f1)）删掉"
  rm -rf -- "${work:?}"
}
trap remove_work EXIT
checked=0
failures=0
expect() { # expect <名> <判据的退出码：0 为过> <不过时的细节>
  checked=$((checked + 1))
  if [[ "$2" == 0 ]]; then
    echo "  ✓ $1"
  else
    echo "  ✗ $1：$3"  # gate-lint:detail
    failures=$((failures + 1))
  fi
}

selftest_case_key="crash-case:sharded-selftest"
copy="$work/repository"
rsync -a --exclude target --exclude .git --exclude layer0-shard.env "$repository_root/" "$copy/"
git -C "$copy" init -q
real_rustc="$(command -v rustc)"
if [[ -n "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  cargo_mode="真 cargo（SINGLEFS_HEAVY_TESTS=${SINGLEFS_HEAVY_TESTS}）"
  selftest_function="the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts"
  selftest_target="crash_enumeration_sharded_across_processes"
  selftest_cargo="$(command -v cargo)"
else
  cargo_mode="假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）"
  selftest_function="stand_in_for_the_driver_selftest"
  selftest_target="sharded_driver_selftest_stand_in"
  printf '#[test]\n#[ignore]\nfn %s() {}\n' "$selftest_function" > "$copy/crates/singlefs-harness/tests/$selftest_target.rs"
  mkdir -p "$work/stand-in-bin"
  selftest_cargo="$work/stand-in-bin/cargo"
  cat > "$selftest_cargo" <<'STAND_IN_CARGO'
#!/usr/bin/env bash
# 驱动脚本自证的假 cargo：-V 打版本；test 按 SINGLEFS_LAYER0_SHARD 分三种：不设打计数行与线程行，<i>/2 往进度目录写那一片的账本，
# merge/2 核两份账本在、记的输入指纹与这一趟的相同、START_OVER 没设，再打计数行与带 shards= 的线程行。别的一律退 101。
if [[ "${1:-}" == -V ]]; then echo "cargo 0.0.0-layer0-shard-selftest"; exit 0; fi
[[ "${1:-}" == test ]] || { echo "假 cargo 只认 -V 与 test：$*"; exit 101; }
count_line="LAYER0_SHARDED states=29 closed_form=29 violations=0 exhaustive=true"
passed="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s"
case "${SINGLEFS_LAYER0_SHARD-}" in
  "")
    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=2 configured_worker_threads=2 worker_threads_source=environment_variable resumed_slices=0 freshly_run_slices=4 progress_file_after_completion=none elapsed_seconds=0.1"
    printf '%s\n%s\n' "$count_line" "$passed" ;;
  0/2|1/2)
    [[ -n "${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}" && -n "${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "分片跑没设进度目录或输入指纹"; exit 101; }
    shard_index="${SINGLEFS_LAYER0_SHARD%/2}"
    ledger="$SINGLEFS_LAYER0_PROGRESS_DIRECTORY/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
    mkdir -p "$SINGLEFS_LAYER0_PROGRESS_DIRECTORY" && printf 'input_fingerprint=%s\n' "$SINGLEFS_LAYER0_INPUT_FINGERPRINT" > "$ledger" || exit 101
    printf 'LAYER0_PROGRESS slice=1/2 shard=%s\nLAYER0_SHARD mode=run shard=%s ledger=%s\n%s\n' "$SINGLEFS_LAYER0_SHARD" "$SINGLEFS_LAYER0_SHARD" "$ledger" "$passed" ;;
  merge/2)
    [[ -z "${SINGLEFS_LAYER0_START_OVER+set}" ]] || { echo "merge 那一趟带了 SINGLEFS_LAYER0_START_OVER"; exit 101; }
    for shard_index in 0 1; do
      ledger="${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
      [[ "$(cat "$ledger" 2>/dev/null)" == "input_fingerprint=${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "merge：第 $shard_index 片的账本缺或输入指纹不同（$ledger）"; exit 101; }
    done
    echo "LAYER0_SHARD mode=merge shards=2 ledgers=${SINGLEFS_LAYER0_PROGRESS_DIRECTORY}"
    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=4 configured_worker_threads=4 worker_threads_source=shard_ledgers resumed_slices=0 freshly_run_slices=4 shards=2 shard_worker_threads=2,2 shard_configured_worker_threads=2,2 shard_worker_threads_sources=environment_variable,environment_variable shard_available_parallelism=2,2 shard_resumed_slices=0,0 shard_freshly_run_slices=2,2 shard_elapsed_milliseconds=10,10 progress_file_after_completion=kept elapsed_seconds=0.1"
    printf '%s\n%s\n' "$count_line" "$passed" ;;
  *) echo "假 cargo 认不出 SINGLEFS_LAYER0_SHARD=${SINGLEFS_LAYER0_SHARD}"; exit 101 ;;
esac
STAND_IN_CARGO
  chmod +x "$selftest_cargo"
  export PATH="$work/stand-in-bin:$PATH"
fi
printf '%s\t%s\t%s\t%s\n' "$selftest_case_key" "crates/ Cargo.toml Cargo.lock" \
  "test=singlefs-harness:$selftest_target:$selftest_function count-line=LAYER0_SHARDED exhaustive=LAYER0_SHARDED threads=LAYER0_SHARDED shard=across-machines" \
  "# 驱动脚本自证临时登记的用例" >> "$copy/.claude/gate.d/stage-inputs.tsv"
selftest_bin_directory="$(dirname "$selftest_cargo")"
write_configuration() { # write_configuration <文件> <PEER_CARGO_BIN_DIRECTORY>
  cat > "$1" <<CONFIGURATION
# 驱动脚本自证用的配置：第二台是本机上的 $work/peer
PEER_SSH_HOST=selftest-peer-is-this-machine
PEER_REPOSITORY_DIRECTORY=$work/peer
PEER_CARGO_BIN_DIRECTORY=$2
QUIESCE_STOP_COMMAND=touch $work/quiesced
QUIESCE_STOPPED_CHECK_COMMAND=test -e $work/quiesced
QUIESCE_START_COMMAND=rm -f $work/quiesced
QUIESCE_STARTED_CHECK_COMMAND=test ! -e $work/quiesced
CONFIGURATION
}
write_configuration "$work/layer0-shard.env" "$selftest_bin_directory"
export SINGLEFS_LAYER0_SHARD_CONFIG="$work/layer0-shard.env"
export SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1
# 三趟 cargo test 都不带 --include-ignored：真 cargo 那条小流用例不标 ignore（同一个测试目标里标了 ignore 的 golden 子进程用例不进过滤范围）；
# 假 cargo 的替身用例标了 ignore，不带它就不算重型
export SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0
unset SINGLEFS_LAYER0_SHARD SINGLEFS_LAYER0_PROGRESS_DIRECTORY SINGLEFS_LAYER0_INPUT_FINGERPRINT SINGLEFS_LAYER0_START_OVER
driver="$copy/research/scripts/layer0-shard-run.sh"
admission_module="$copy/research/scripts/admission.py"

# 不分片跑一次那条用例：计数行是 merge 之后要逐字相同的那一行
(cd "$copy" && cargo test --release -p singlefs-harness --test "$selftest_target" -- --exact "$selftest_function" --nocapture) > "$work/unsharded.log" 2>&1
unsharded_exit=$?
unsharded_count_line="$(grep '^LAYER0_SHARDED ' "$work/unsharded.log")"
expect "不分片跑登记的那条用例：退 0，恰好一行 LAYER0_SHARDED 计数行" "$([[ $unsharded_exit == 0 && $(grep -c '^LAYER0_SHARDED ' "$work/unsharded.log") == 1 ]]; echo $?)" \
  "退 $unsharded_exit；日志尾部：$(tail -5 "$work/unsharded.log" | tr '\n' '|')"

# ① 单独跑：两片、merge、判、写标记
bash "$driver" "$selftest_case_key" "$copy" > "$work/standalone.log" 2>&1
standalone_exit=$?
fingerprint="$(python3 "$admission_module" crash-case-manifest "$copy" "$selftest_case_key" "$work/manifest" \
  --judging-digest --toolchain --build-environment | cut -d' ' -f1)"
marker_output="$(python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest")"
marker_exit=$?
marker_count_line="$(grep '^LAYER0_SHARDED ' <<< "$marker_output")"
marker_path="$(head -1 <<< "$marker_output" | cut -d' ' -f2)"
expect "① 单独跑：退 0，两片各跑了（LAYER0_SHARD mode=run 0/2、1/2），这批输入那一格全绿标记作数" \
  "$([[ $standalone_exit == 0 && $marker_exit == 0 ]] && grep -q 'SINGLEFS_LAYER0_SHARD=0/2' "$work/standalone.log" && grep -q '④ 两片都跑完' "$work/standalone.log"; echo $?)" \
  "驱动脚本退 $standalone_exit，标记核 $marker_exit（$marker_output）；驱动脚本输出尾部：$(tail -15 "$work/standalone.log" | tr '\n' '|')"
expect "① 不分片的计数行与双机分片 merge 之后记进标记的那一行逐字相同" "$([[ -n "$unsharded_count_line" && "$marker_count_line" == "$unsharded_count_line" ]]; echo $?)" \
  "不分片「$unsharded_count_line」，标记里「$marker_count_line」"
expect "① merge 那一行按片报线程（shards=2、shard_worker_threads=），记进了标记" "$(grep -q '^parallel_finished=LAYER0_PARALLEL_FINISHED .*shards=2 shard_worker_threads=' "$marker_path"; echo $?)" \
  "标记 $marker_path 里：$(grep 'parallel_finished=' "$marker_path" 2>/dev/null)"
expect "① 清了场、跑完复原了（回读过），第二台这一趟的目录删了" \
  "$([[ ! -e "$work/quiesced" ]] && grep -q '清场之后回读过了' "$work/standalone.log" && grep -q '复原之后回读过了' "$work/standalone.log" \
     && [[ -z "$(ls -A "$work/peer/runs" 2>/dev/null)" ]]; echo $?)" \
  "清场标记还在：$([[ -e "$work/quiesced" ]] && echo 是 || echo 否)；第二台 runs/ 下：$(ls -A "$work/peer/runs" 2>/dev/null | tr '\n' ' ')"

# ② --merged-log：门禁 54 号调的那一条
bash "$driver" --merged-log "$selftest_case_key" "$copy" "$fingerprint" "$work/merged.log" > "$work/merged-driver.log" 2>&1
merged_exit=$?
expect "② --merged-log：退 0，merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数" \
  "$([[ $merged_exit == 0 ]] && grep -q 'LAYER0_SHARD mode=merge shards=2 ' "$work/merged.log" && [[ "$(grep '^LAYER0_SHARDED ' "$work/merged.log")" == "$unsharded_count_line" ]]; echo $?)" \
  "退 $merged_exit；驱动脚本输出尾部：$(tail -10 "$work/merged-driver.log" | tr '\n' '|')"

# ③ 「第二台」的 rustc -Vv 第一行不同
mkdir -p "$work/other-toolchain"
printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$selftest_cargo" > "$work/other-toolchain/cargo"
printf '#!/usr/bin/env bash\nif [[ "${1:-}" == -Vv ]]; then %q -Vv | sed "1s/\$/ (another build)/"; else exec %q "$@"; fi\n' "$real_rustc" "$real_rustc" > "$work/other-toolchain/rustc"
chmod +x "$work/other-toolchain/cargo" "$work/other-toolchain/rustc"
write_configuration "$work/other-toolchain.env" "$work/other-toolchain"
SINGLEFS_LAYER0_SHARD_CONFIG="$work/other-toolchain.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/other-toolchain.log" 2>&1
toolchain_exit=$?
python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
toolchain_marker_exit=$?
expect "③ 「第二台」rustc -Vv 第一行不同 ⇒ 拒（退 1，说工具链不同），这批输入那一格标记不动" \
  "$([[ $toolchain_exit == 1 && $toolchain_marker_exit == 0 ]] && grep -q '两台的工具链不同' "$work/other-toolchain.log"; echo $?)" \
  "退 $toolchain_exit，标记核 $toolchain_marker_exit；输出：$(tail -4 "$work/other-toolchain.log" | tr '\n' '|')"

# ④ 「第二台」那一片退 0 却没写账本
mkdir -p "$work/no-ledger"
printf '#!/usr/bin/env bash\nif [[ "${1:-}" == test ]]; then echo "LAYER0_SHARD mode=run shard=1/2 （自证：这一片不写账本）"; exit 0; fi\nexec %q "$@"\n' "$selftest_cargo" > "$work/no-ledger/cargo"
printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$real_rustc" > "$work/no-ledger/rustc"
chmod +x "$work/no-ledger/cargo" "$work/no-ledger/rustc"
write_configuration "$work/no-ledger.env" "$work/no-ledger"
git_common_directory="$(git -C "$copy" rev-parse --path-format=absolute --git-common-dir)"
rm -f -- "${git_common_directory:?}/singlefs-layer0-progress/$fingerprint"/layer0-shard-*.tally "${work:?}/peer/progress/$fingerprint"/layer0-shard-*.tally
SINGLEFS_LAYER0_SHARD_CONFIG="$work/no-ledger.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-ledger.log" 2>&1
no_ledger_exit=$?
python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
no_ledger_marker_exit=$?
expect "④ 「第二台」那一片没写账本 ⇒ 判红（退 1，说两片的账本不是各恰好一份），删这批输入那一格标记" \
  "$([[ $no_ledger_exit == 1 && $no_ledger_marker_exit == 1 ]] && grep -q '两片的账本不是各恰好一份' "$work/no-ledger.log"; echo $?)" \
  "退 $no_ledger_exit，标记核 $no_ledger_marker_exit；输出：$(tail -4 "$work/no-ledger.log" | tr '\n' '|')"

# ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒
grep -v '^PEER_CARGO_BIN_DIRECTORY=' "$work/layer0-shard.env" > "$work/missing-key.env"
SINGLEFS_LAYER0_SHARD_CONFIG="$work/missing-key.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/missing-key.log" 2>&1
missing_key_exit=$?
SINGLEFS_LAYER0_SHARD_CONFIG="$work/no-such-configuration.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-configuration.log" 2>&1
no_configuration_exit=$?
expect "⑤ 配置缺键 PEER_CARGO_BIN_DIRECTORY、配置文件不在 ⇒ 配置判法各拒（退 1，说双机分片不能用），出路指到 layer0-shard.env.example" \
  "$([[ $missing_key_exit == 1 && $no_configuration_exit == 1 ]] && grep -q '双机分片不能用' "$work/missing-key.log" && grep -q '双机分片不能用' "$work/no-configuration.log" \
     && grep -q 'layer0-shard.env.example' "$work/missing-key.log" && grep -q 'layer0-shard.env.example' "$work/no-configuration.log"; echo $?)" \
  "缺键退 $missing_key_exit（$(tail -3 "$work/missing-key.log" | tr '\n' '|')），不在退 $no_configuration_exit（$(tail -3 "$work/no-configuration.log" | tr '\n' '|')）"

# ⑥ 没登记 shard=across-machines 的用例
bash "$driver" crash-case:c561-sigma-full "$copy" > "$work/not-shardable.log" 2>&1
not_shardable_exit=$?
expect "⑥ 没登记 shard=across-machines 的用例 ⇒ 拒（退 1），出路是单机跑" \
  "$([[ $not_shardable_exit == 1 ]] && grep -q '没登记 shard=across-machines' "$work/not-shardable.log"; echo $?)" \
  "退 $not_shardable_exit；输出：$(tail -3 "$work/not-shardable.log" | tr '\n' '|')"

if (( failures > 0 )); then
  echo "  ✗ layer0-shard-run.sh 自证没过：${failures} 格判错（共 ${checked} 格）"
  echo "     → 怎么办：照上面每一格的说明改驱动脚本（research/scripts/layer0-shard-run.sh）或分片那一段（crates/singlefs-harness/src/layer0_progress.rs、crash.rs）"
  exit 1
fi
echo "  ✓ layer0-shard-run.sh 自证通过：${checked} 格都对（${cargo_mode}；第二台是本机上的另一个目录，没碰真的第二台）"
```

### `research/scripts/layer0-shard-configuration-check.sh`（新文件，98 行）

```bash
#!/usr/bin/env bash
# 层 0 崩溃重放双机分片（里程碑三第六项，.claude/kb/milestone/03-third-txn.md 第六节）的本地配置：在不在、键齐不齐、写得对不对、
# 第二台连不连得上、它上面有没有 cargo。门禁 54 号 --full 拿它定走不走分片；research/scripts/layer0-shard-run.sh 拿它当运行条件。
#
#   layer0-shard-configuration-check.sh [<仓根>]
#       退 0 能分片（stdout 一句：配置在哪、第二台是谁）；退 1 不能（stdout 一句原因，下一行是出路）。
#   layer0-shard-configuration-check.sh --emit-assignments [<仓根>]
#       判法同上；能分片时 stdout 改打配置里每个键的 bash 赋值（printf %q，调用方 eval），不打那一句。
#
# 配置文件：${SINGLEFS_LAYER0_SHARD_CONFIG:-<主工作树的根>/layer0-shard.env}。主工作树的根按 git common-dir 取（在 HEAD + 暂存区的
# 临时 worktree 里跑也读主工作树那一份）；文件 git 忽略、不进仓，模板是仓根的 layer0-shard.env.example。
# 写法：一行一个 KEY=值（值不做 shell 展开；两头成对的单引号或双引号去掉一层），# 起头的行与空行不算；认不出的行、不认得的键、
# 缺键都判不能分片。键与各自的意思见 layer0-shard.env.example。
# 只供测试的开关：SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1 时不连第二台、不查它的 cargo（layer0-shard-run.sh --selftest 用，
# 那时「第二台」是本机上的另一个目录）。
#
# admission: always 判的是这一刻配置文件与第二台的样子，每次调都要现判
# run-condition: command git bash
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
layer0_shard_check_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

LAYER0_SHARD_CONFIGURATION_KEYS=(PEER_SSH_HOST PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY
  QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND)

emit_assignments=0
if [[ "${1:-}" == --emit-assignments ]]; then emit_assignments=1; shift; fi
repository_root="${1:-$layer0_shard_check_script_directory/../..}"

refuse() { # refuse <原因>：打原因与出路，退 1
  echo "  ✗ 双机分片不能用：$1"
  echo "     → 怎么办：照仓根 layer0-shard.env.example 建本地配置 layer0-shard.env（git 忽略），或设 SINGLEFS_LAYER0_SHARD_CONFIG 指到它；第二台连不上先修 ssh 免密登录"
  exit 1
}

if [[ -n "${SINGLEFS_LAYER0_SHARD_CONFIG:-}" ]]; then
  configuration_file="$SINGLEFS_LAYER0_SHARD_CONFIG"
else
  if ! common_directory="$(git -C "$repository_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$common_directory" ]]; then
    refuse "$repository_root 不是 git 工作树，找不到主工作树的根（配置默认在那里）"
  fi
  configuration_file="$(dirname "$common_directory")/layer0-shard.env"
fi
[[ -f "$configuration_file" ]] || refuse "没有配置文件 $configuration_file"

declare -A configuration=()
line_number=0
while IFS= read -r configuration_line || [[ -n "$configuration_line" ]]; do
  line_number=$((line_number + 1))
  [[ "$configuration_line" =~ ^[[:space:]]*(#.*)?$ ]] && continue
  [[ "$configuration_line" =~ ^([A-Z_]+)=(.*)$ ]] || refuse "$configuration_file 第 $line_number 行认不出（要是 KEY=值）"
  configuration_key="${BASH_REMATCH[1]}"
  configuration_value="${BASH_REMATCH[2]}"
  if [[ "$configuration_value" =~ ^\'(.*)\'$ || "$configuration_value" =~ ^\"(.*)\"$ ]]; then configuration_value="${BASH_REMATCH[1]}"; fi
  known_key=0
  for expected_key in "${LAYER0_SHARD_CONFIGURATION_KEYS[@]}"; do [[ "$expected_key" == "$configuration_key" ]] && known_key=1; done
  (( known_key )) || refuse "$configuration_file 第 $line_number 行的键 $configuration_key 不认得（认的是 ${LAYER0_SHARD_CONFIGURATION_KEYS[*]}）"
  [[ -z "${configuration[$configuration_key]+set}" ]] || refuse "$configuration_file 里 $configuration_key 写了两遍"
  configuration[$configuration_key]="$configuration_value"
done < "$configuration_file"
missing_keys=()
for expected_key in "${LAYER0_SHARD_CONFIGURATION_KEYS[@]}"; do
  [[ -n "${configuration[$expected_key]+set}" ]] || missing_keys+=("$expected_key")
done
(( ${#missing_keys[@]} == 0 )) || refuse "$configuration_file 缺键 ${missing_keys[*]}（不清场的两对写成空串，键照样要写）"
[[ -n "${configuration[PEER_SSH_HOST]}" ]] || refuse "PEER_SSH_HOST 是空的"
for directory_key in PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY; do
  [[ "${configuration[$directory_key]}" == /* ]] || refuse "$directory_key 要写第二台上的绝对路径，读到「${configuration[$directory_key]}」"
done
for command_pair in "QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND" "QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND"; do
  read -r action_key check_key <<< "$command_pair"
  if [[ -n "${configuration[$action_key]}" && -z "${configuration[$check_key]}" ]] || [[ -z "${configuration[$action_key]}" && -n "${configuration[$check_key]}" ]]; then
    refuse "$action_key 与 $check_key 要么都写、要么都空：改了状态的命令跑完要回读一次"
  fi
done
if [[ -n "${configuration[QUIESCE_STOP_COMMAND]}" && -z "${configuration[QUIESCE_START_COMMAND]}" ]]; then
  refuse "写了 QUIESCE_STOP_COMMAND 就要写 QUIESCE_START_COMMAND：清了场要复原"
fi

peer_description="第二台 ${configuration[PEER_SSH_HOST]}"
if [[ "${SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE:-0}" == 1 ]]; then
  peer_description="第二台是本机（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1）"
else
  ssh -o BatchMode=yes -o ConnectTimeout=10 "${configuration[PEER_SSH_HOST]}" true </dev/null >/dev/null 2>&1 \
    || refuse "ssh -o BatchMode=yes ${configuration[PEER_SSH_HOST]} true 连不上"
  ssh -o BatchMode=yes -o ConnectTimeout=10 "${configuration[PEER_SSH_HOST]}" "test -x $(printf '%q' "${configuration[PEER_CARGO_BIN_DIRECTORY]}")/cargo" </dev/null >/dev/null 2>&1 \
    || refuse "第二台上没有 ${configuration[PEER_CARGO_BIN_DIRECTORY]}/cargo（PEER_CARGO_BIN_DIRECTORY 要指到它的 ~/.cargo/bin）"
fi

if (( emit_assignments )); then
  for expected_key in "${LAYER0_SHARD_CONFIGURATION_KEYS[@]}"; do
    printf '%s=%q\n' "$expected_key" "${configuration[$expected_key]}"
  done
  printf 'LAYER0_SHARD_CONFIGURATION_FILE=%q\n' "$configuration_file"
else
  echo "配置 $configuration_file，$peer_description"
fi
```
