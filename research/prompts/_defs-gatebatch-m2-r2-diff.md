# 附录二：门禁批第二轮被判的 diff（Y1–Y8 改法交回时的 diff，原样）

**生成时刻**：2026-09-26 15:00 UTC（JST 24:0x）。

**基准**：门禁批实现员（做 Y1–Y8）改动前用 `cp -p` 备份的文件快照，备份在
`/tmp/claude-1000/gate-batch-m2-r1-fixes/backup/`（记在同目录 `spec.md`「报告写进交回消息」一段：
「改之前 `cp -p` 备份，比备份」）。**不是**某个 git 提交号：这批改动开工时，仓里的
`.claude/gate.d/54-layer0-replay.sh`、`.claude/gate.d/stage-inputs.tsv`、`.claude/hooks/heavy-test-guard.sh`、
`.claude/hooks/lib_heavy_tests.py`、`.claude/rules/implementation-workflow.md` 已经带着第一轮门禁批
（`/tmp/claude-1000/gate-batch-m2/my-changes.diff`）落下的改动，五个都与 HEAD（`73ba4a4`）不同
（`diff /tmp/claude-1000/gate-batch-m2-r1-fixes/backup/<路径> <(git show 73ba4a4:<路径>)` 逐个判为
`differ`，符合预期：备份取的是第一轮门禁批之后、第二轮 Y1–Y8 之前那一刻）；`research/scripts/admission.py`
仍在 git 里完全未跟踪（`git status --porcelain` 显示 `??`）。材料员核过：把这份 diff 用
`patch -p1` 打在备份副本上（临时目录，跑完已删），六个文件逐字节与今天工作区里的现状相同
（`diff -q` 六个都判 `MATCH`）——这份 diff 原样描述了「备份」到「今天」这一步。

下面「一」原样是 `/tmp/claude-1000/gate-batch-m2-r1-fixes/my-changes.diff` 的内容
（`wc -l` 1707 行、6 个文件，不带 `diff --git` 头，两侧路径相同、没有 `/dev/null`，
不是新建文件，因此不再附「新文件全文」一节）。

## 一、Y1–Y8 交回的 diff（`/tmp/claude-1000/gate-batch-m2-r1-fixes/my-changes.diff` 原样，1707 行，6 个文件）

```diff
--- a/.claude/gate.d/54-layer0-replay.sh
+++ b/.claude/gate.d/54-layer0-replay.sh
@@ -9,11 +9,13 @@
 #   bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full [--start-over] <worktree>
 #     主 agent 暂存之后（提交时由崩溃验证员），在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
 #     逐条崩溃枚举用例（.claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的那几行，照登记表的次序）：先算这条用例这批输入的指纹
-#     （research/scripts/admission.py crash-case-manifest：登记路径下的文件减去别的测试目标独占的测试文件，加判它的 54 号、工具链、构建环境
-#     与这条用例的登记行）；那一格全绿标记在、作数就复用，不跑；不在才跑 `cargo test --release -p <包> --test <测试目标> -- --include-ignored
+#     （research/scripts/admission.py crash-case-manifest：登记路径下的文件减去用例读不到的文件，加判它的 54 号、准入模块 admission.py、
+#     工具链、构建环境与这条用例的登记行）；那一格全绿标记在、作数就复用，不跑；不在才跑 `cargo test --release -p <包> --test <测试目标> -- --include-ignored
 #     --exact <用例函数> --nocapture`，按登记行第三列判日志（crash-case-judge），开跑与跑完各算一次指纹，相同才写那一格（crash-case-record）。
 #     全绿标记在 git common-dir：`singlefs-crash-case-green.<用例名>.<输入指纹>`，不进工作树，各 worktree 读写同一组；别的格不动。
 #     一条判红删它这批输入那一格（先绿后红，前一趟那一格不再作数），接着跑下一条；「跑的过程中输入变了」那一支判红不删：它说不出开跑那一批的好坏。
+#     两趟 --full 同时跑同一条用例、同一个指纹时这里不加锁：后跑完的那一趟判红会删掉先跑完的那一趟刚写的绿标记，只会假红、不会假绿
+#     （崩溃验证员的定义里「另有 --full 在跑时不起」挡着这一种）。
 #     断点续跑：跑用例时设 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=<common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>（不随 worktree 删掉）、
 #     SINGLEFS_LAYER0_INPUT_FINGERPRINT=<这条用例的输入指纹>；--start-over 设 SINGLEFS_LAYER0_START_OVER=1（丢掉进度文件、从头跑），
 #     不带它时从调用方的环境里清掉这个变量。续跑的判法（片方案、校验和、观察者计数、判红删进度文件）在 crates/singlefs-harness/src/layer0_progress.rs。
@@ -90,7 +92,9 @@
 # 一问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
 # 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
 # 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
-# 二问这次改动碰没碰登记给本阶段的那几条路径。这是 C8（范围判定）的粗粒度前身：它只摘得掉「零行输入的改动」，摘不出别的，C8 照旧欠着。
+# 二问这次改动碰没碰登记给本阶段的那几条路径，再加登记表本身、跑的这一份 54 号与准入模块：这三样都进每条崩溃枚举用例的指纹，
+# 只改它们的改动（新登记一条用例、改一条的第三列、改判法）同样要核标记，不能在这一问被摘掉。
+# 这是 C8（范围判定）的粗粒度前身：它只摘得掉「零行输入的改动」，摘不出别的，C8 照旧欠着。
 # 两问之前先核登记的每一条路径 git 至少列得出一个文件：写错的路径两问都拿它答「没碰」，这一道就一直退 77、一次都不跑。
 if [[ "$layer0_tier" == quick ]]; then
   layer0_unlisted_input_paths=()
@@ -111,11 +115,12 @@
     echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。"
     exit 77
   fi
-  scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" "${layer0_registered_input_paths[@]}")"
+  layer0_judge_paths=(.claude/gate.d/stage-inputs.tsv "$(realpath -m --relative-to="$ROOT" "$layer0_stage_script_path")" "$(realpath -m --relative-to="$ROOT" "$layer0_admission_module")")
+  scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" "${layer0_registered_input_paths[@]}" "${layer0_judge_paths[@]}")"
   scope_rc=$?
   if [[ "$scope_rc" != 0 ]]; then
     echo "  ! 本阶段跳过（这次改动没碰它判的东西）：$scope_reason"
-    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${layer0_input_paths_text}，判据见 research/scripts/change-touches-crates.sh。"
+    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${layer0_input_paths_text}，加 ${layer0_judge_paths[*]}，判据见 research/scripts/change-touches-crates.sh。"
     exit 77
   fi
 fi
@@ -165,7 +170,8 @@
 write_crash_case_manifest() {
   local manifest_summary
   if ! manifest_summary="$(python3 "$layer0_admission_module" crash-case-manifest "$ROOT" "$1" "$2" \
-      --extra-file "<判它的 54 号：$layer0_stage_file_name>" "$layer0_stage_script_path" --toolchain --build-environment)"; then
+      --extra-file "<判它的 54 号：$layer0_stage_file_name>" "$layer0_stage_script_path" \
+      --extra-file "<判它的准入模块：admission.py>" "$layer0_admission_module" --toolchain --build-environment)"; then
     case_manifest_problem="$manifest_summary"
     return 1
   fi
@@ -278,13 +284,13 @@
     if marker_check_output="$(python3 "$layer0_admission_module" crash-case-marker-check "$ROOT" "$case_key" "$case_fingerprint" "$case_manifest")"; then
       read -r _ok_word _marker_path marker_finished_utc <<< "$(head -1 <<< "$marker_check_output")"
       {
-        echo "    $case_key：那一格跑完于 ${marker_finished_utc}（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去别的测试目标独占的 ${case_excluded_count} 个）"
+        echo "    $case_key：那一格跑完于 ${marker_finished_utc}（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去用例读不到的 ${case_excluded_count} 个）"
         tail -n +2 <<< "$marker_check_output" | sed 's/^/      /'
       } >> "$present_report"
     else
       missing_cases+=("$case_key")
       {
-        echo "       $case_key（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去别的测试目标独占的 ${case_excluded_count} 个）："
+        echo "       $case_key（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去用例读不到的 ${case_excluded_count} 个）："
         sed 's/^/         /' <<< "$marker_check_output"
       } >> "$missing_report"
     fi
@@ -302,7 +308,7 @@
     exit 1
   fi
   echo "  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量留给 --full）：${quick_tier_report}"
-  echo "  ✓ ${#crash_case_rows[@]} 条崩溃枚举用例的全绿标记都与各自这批输入的指纹相同（登记路径 ${layer0_input_paths_text}，逐条减去别的测试目标独占的测试文件），标记里的计数行原样："
+  echo "  ✓ ${#crash_case_rows[@]} 条崩溃枚举用例的全绿标记都与各自这批输入的指纹相同（登记路径 ${layer0_input_paths_text}，逐条减去用例读不到的文件），标记里的计数行原样："
   cat "$present_report"
   exit 0
 fi
@@ -337,7 +343,7 @@
   fi
   case_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
   case_log="$layer0_scratch_directory/log.$case_label"
-  echo "  · $case_key 开跑（${case_started_utc}；cargo test --release -p $case_package --test $case_target -- --include-ignored --exact $case_function；这批输入的指纹 ${fingerprint_at_start:0:16}…，${file_count_at_start} 个文件，减去别的测试目标独占的 ${excluded_count_at_start} 个）："
+  echo "  · $case_key 开跑（${case_started_utc}；cargo test --release -p $case_package --test $case_target -- --include-ignored --exact $case_function；这批输入的指纹 ${fingerprint_at_start:0:16}…，${file_count_at_start} 个文件，减去用例读不到的 ${excluded_count_at_start} 个）："
   sed 's/^/      /' <<< "$marker_check_output"
   if ! run_crash_case "$case_package" "$case_target" "$case_function" "$case_log" "$fingerprint_at_start"; then
     tail -40 "$case_log"
--- a/.claude/gate.d/stage-inputs.tsv
+++ b/.claude/gate.d/stage-inputs.tsv
@@ -12,8 +12,10 @@
 #     第三列的准入条件没齐 ⇒ 拒绝开跑。路径写成 @<别的键> 就是那个键登记的全部路径。
 #   崩溃枚举用例（键是 crash-case:<名>）：门禁 54 号逐条按复用判定跑、逐条记全绿标记（在 git common-dir，按用例与它的输入指纹分格），
 #     输入没变复用、变了才重跑。路径用排除法写：只写整个 crates/ 与 Cargo 清单、锁，不按用例手列它读哪些文件——admission.py 算输入时
-#     自动减去别的测试目标独占的测试文件（判法见它的「崩溃枚举用例」一节），再加判它的 54 号、工具链、构建环境与这一行本身。
-#     第三列：test=<包>:<测试目标>:<用例函数> 恰好一条，count-line= / exhaustive= / threads= 定日志怎么判（写法见 admission.py 文件头）。
+#     自动减去用例读不到的文件（别的测试目标独占的测试文件、crates/mutations.tsv、没有代码读 CARGO_BIN_EXE_ 时的 src/bin/；判法见它的
+#     「崩溃枚举用例」一节），再加判它的 54 号、准入模块 admission.py、工具链、构建环境与这一行本身。
+#     第三列：test=<包>:<测试目标>:<用例函数> 恰好一条（用例函数要标 #[ignore]），count-line= / exhaustive= / threads= 定日志怎么判
+#     （写法见 admission.py 文件头）：用例跑起来打的行里有的才登记，打不出的那一项不登记、在注释里写明缺哪一项。
 #
 # ⚠️ 这份清单**自己也进门禁的比对**（stage-must-run.sh 无条件把它加进路径列表）。少写一条输入，那条输入就永远
 # 不会让这道阶段重跑——而清单本身变了必定重跑，改清单的代价因此是「下一趟全跑一次」，不是零；加一个实验行也一样。
@@ -21,7 +23,7 @@
 #
 # 宁宽勿窄：多写一条路径只会多跑几趟，少写一条会让一次真的改动被跳过。拿不准就写上。
 # 只写从仓库根起的路径；目录写成带斜杠的前缀（git diff 与 git ls-files 的路径限定都按前缀匹配）。
-54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	command=cargo command=rustc	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）。这一行管整道阶段的复用判定与改动范围；它逐条跑的崩溃枚举用例各自的输入登记在本表 crash-case: 开头的那几行。前提：cargo 与 rustc（两档都要 cargo -V && rustc -V 算输入指纹，快档与 --full 都起 cargo test）
+54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/admission.py	command=cargo command=rustc	# 跑 crates 的崩溃点重放；research/scripts/admission.py 是它判日志、算每条崩溃枚举用例的输入指纹、读写全绿标记的准入模块（它也进每条用例的指纹），改了它这一道不许复用上一次的判定；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）。这一行管整道阶段的复用判定与改动范围；它逐条跑的崩溃枚举用例各自的输入登记在本表 crash-case: 开头的那几行。前提：cargo 与 rustc（两档都要 cargo -V && rustc -V 算输入指纹，快档与 --full 都起 cargo test）
 55-qemu-first-transaction.sh	crates/ Cargo.toml Cargo.lock research/scripts/ research/results/	command=qemu-system-x86_64 readwrite=/dev/kvm probe=research/scripts/vm-kernel.sh:--check	# 起虚机跑 crates 的装置，虚机与复跑脚本都在 research/scripts/；这几条路径同时是改动范围（change-touches-crates.sh）的前缀。前提是 .claude/kb/vm-harness.md「三个前置」：缺 QEMU 装 qemu-system-x86；/dev/kvm 不可读写查 kvm 组成员身份，不许用 setfacl 补；找不到可读的内核镜像就跑 bash research/scripts/vm-kernel.sh，按它打印的路径设 SINGLEFS_KERNEL
 57-lkmm.sh	litmus/ .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/	probe=.claude/scripts/lkmm.sh:--herd7-version environment=.claude/scripts/lkmm.sh:--herd7-version	# 判 litmus/ 下每条 Never 的对照组、代码绑定与 herd7 判定。lkmm.sh 是判法，它 source 的 lib.sh、找不到内核树时调的 fetch-deps.sh 一并登记；crates/ 整个取：litmus 的 singlefs-models 锚点今天指 crates/singlefs-core/src/transaction.rs 与 recovery.rs，读 litmus 文件名的测试在 crates/singlefs-harness/tests/publish_order_matches_litmus.rs，而 lkmm.sh 按文件名在全部 crates/**/*.rs 里找测试、新加一条 litmus 的锚点可以指到 crates/ 任何一处，只登记这三份会让下一条新锚点的改动被跳过（宁宽勿窄）。herd7 的版本不在 git 树里，经第三列 environment= 进复用判定（lkmm.sh --herd7-version 打的那一行，57 号判绿之后记进 git common-dir）；前提是找得到 herd7（probe= 同一个入口，PATH 里没有就试 opam 的环境；缺了 opam install herdtools7）。内核树的 tools/memory-model 不进判定，靠 stage-must-run.sh 的 24 小时复用上限兜
 59-crates-mutation-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh	command=cargo	# 按 crates/mutations.tsv 逐条改坏 crates/ 里的源码再跑点名的测试；每条经 run-with-memory-cap.sh 带内存上限跑，它判不判得出撞顶也是这一道的判据。前提：cargo（装 Rust 工具链，bash .claude/scripts/env.sh 会报缺什么）
@@ -31,5 +33,5 @@
 E142/layer0	@E142	question-row=research/prompts/m2-keyspace-rerun-questions.md#6:够判[：:][^（(|]*对照(本身)?是好的 product-field=E142:verdict:control_violations_ok=true product-field=E142:verdict:positive_control_main_geometry_ok=true	# E142 第二段（设了 E142_LAYER0_MAIN 的调用：主臂层 0 整轮），输入与默认调用相同。前提照跑前登记 research/prompts/e142-r17-prereg.md 6.2、6.3：问题单第 6 行判成「对照是好的」之后才跑，且 Q142.26 caught = 12（最新产物判决行 positive_control_main_geometry_ok）；control_violations_ok 是第 6 行够判那一格的计数判定
 crash-case:layer0-first-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:first_transaction_step_seven_layer0:layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0	# 第一条流（mkfs → 取号 → 暖机 → A）的层 0 全量，带断点续跑；54 号 --full 原来整批跑的两条之一。LAYER0 是计数行、要 exhaustive=true，CHECKER 是逐条不变量行
 crash-case:layer0-second-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_step_zero_layer0:full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B	# 第二条流（里程碑「第二个事务」固定脚本到 E 再正常卸载）的层 0 全量，带断点续跑；54 号 --full 原来整批跑的两条之二
-crash-case:floor-raise-pushed-by-the-session	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_crash_inside_the_floor_raise_pushed_by_the_session:crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green	# 崩在会话推的那一串抬 F 中间：按层 0 的枚举域枚举那一段、恢复之后与再挂载之后各判池级 checker（标了 ignore，release 跑）；不留进度文件，计数由用例自己断言，不打计数行
-crash-case:c561-sigma-full	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:record_checker_judges_absence_by_the_persisted_set:every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present count-line=C561_SIGMA_FULL	# C561（记录核对器的复用豁免在复用只落一半时假红） 的 σ 全量：σ 那一段 2^18 个状态记录核对器一个都不判缺席（标了 ignore，release 跑）
+crash-case:floor-raise-pushed-by-the-session	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_crash_inside_the_floor_raise_pushed_by_the_session:crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED	# 崩在会话推的那一串抬 F 中间：按层 0 的枚举域枚举那一段、恢复之后与再挂载之后各判池级 checker（标了 ignore，release 跑）；不留进度文件，计数由用例自己断言。用例自己不打计数行，count-line= 登记的是它经 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0_in_state_slices 打的那一行 LAYER0_PARALLEL_FINISHED（只调一次，恰好一行），threads= 按它判枚举那一段的工作线程（挂载那一遍自己起线程，不在判里）；没登记 exhaustive=：那一行不带 exhaustive=true
+crash-case:c561-sigma-full	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:record_checker_judges_absence_by_the_persisted_set:every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present count-line=C561_SIGMA_FULL	# C561（记录核对器的复用豁免在复用只落一半时假红） 的 σ 全量：σ 那一段 2^18 个状态记录核对器一个都不判缺席（标了 ignore，release 跑）；没登记 exhaustive= 与 threads=：计数行 C561_SIGMA_FULL 不带 exhaustive=true（状态数由用例断言等于 262144），用例自己起线程、不经 enumerate_layer0_in_state_slices，不打 LAYER0_PARALLEL_FINISHED
--- a/.claude/hooks/heavy-test-guard.sh
+++ b/.claude/hooks/heavy-test-guard.sh
@@ -20,7 +20,8 @@
 # 双引号里没转义的反引号把 `crates/…/walk.rs` 当命令替换执行，这个 `.rs` 被当 shell 脚本读进去、里面像路径的词再当脚本读，一路递归到第 6 层，
 # 一条命令 100 秒没判完、拖慢每个 agent 的每条命令（同一张表第 18 行）：所以直接执行的只读 shell 脚本，同一份脚本一次判定里只读一遍、只判一遍。
 #
-# 重型测试，按类（只认命令位置）：
+# 重型测试，按类（只认命令位置；「cargo test」在下面各类里一样指 cargo t、cargo nextest run、cargo miri test、cargo llvm-cov、cargo hack test、
+# cargo mutants 与 --config / CARGO_ALIAS_ 定的别名；libtest 参数带 --list 的只列用例、一条都不跑，哪一类都不算）：
 #   层 0          cargo test 会跑到名字含 layer0 的测试二进制：--test 的名字含 layer0、--test 的通配命中它、
 #                 或不带目标选择（带 --tests / --all-targets 也算）而包里有这种二进制；.claude/gate.d/54-*；
 #                 直接执行名字含 layer0 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）
@@ -33,8 +34,11 @@
 #   全部实验复跑  .claude/gate.d/87-*
 #   整轮门禁      gate.sh、research/scripts/gate-staged.sh（--selftest 不算）
 #   E152 装置     e152-file-system-benchmark（直接起、或 cargo run 它）、research/scripts/e152-run.sh
-#   崩溃枚举用例  .claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的那几条用例（门禁 54 号逐条跑）的测试目标，libtest 参数带 --ignored 或
-#                 --include-ignored：cargo test 点名它、--test 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制（判定与登记表取法在 lib_heavy_tests.py）
+#   崩溃枚举用例  .claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的那几条用例（门禁 54 号逐条跑）的测试目标——cargo test 点名它、--test 的通配命中它、
+#                 或不挑目标而包里有它；直接执行它的测试二进制——而下面任一条成立：libtest 参数带 --ignored 或 --include-ignored（nextest 是
+#                 --run-ignored only / all）；--config 或环境变量里定了测试二进制的 runner、或子命令是别名（带没带 --ignored 看不见）；
+#                 登记的用例函数没标 #[ignore]（读源码判，不带 --ignored 也跑到全量）。另按参数认：任何命令（grep、git 这类按文本处理参数的除外）
+#                 带 --ignored / --include-ignored 又点名登记的用例函数（拷走改名的测试二进制、find -exec 起的）。判定与登记表取法在 lib_heavy_tests.py
 #   .claude/gate.d/ 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑、不用带前缀。
 # 谁、带什么才放行（都要带环境变量 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，别的值或没带一律拒）：
 #   主 agent（输入里没有 agent_type）：上面每一类；
@@ -43,6 +47,8 @@
 #   其余子 agent：一律拒，带不带前缀都拒。
 # 前缀认三种写法：写在命令前（`SINGLEFS_HEAVY_TESTS=commit bash …`）、写进 `env` 的参数、同一行前面的 `export`；
 # 往 `bash -c '…'`、`capped.sh N …`、`nice`、`timeout` 这类包装里面传。git 的 pre-commit hook 由 git 起，不经这道闸。
+# lib_shell_words 前缀表之外、包在命令外面照样起那条命令的 `/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、
+#   `perf stat|record|trace`（lib_heavy_tests.py 的 LAUNCHER_OPTIONS_WITH_VALUE）剥掉之后，里面那条当一段命令文本接着判：前缀、内存包装、脚本照认。
 # 写进脚本文件再执行的，读脚本正文、逐条照同一张表判：命令位置上是 `bash|sh 文件`（不带 -c）、`./x.sh` 或 `路径/x.sh`、`source 文件` / `. 文件`，
 #   经 capped.sh N、nice、timeout、env、前缀变量包一层的同样认；脚本里再起脚本的递归读，读到第 MAXIMUM_SCRIPT_DEPTH（6）层为止，再往里判「看不全」。
 #   外层命令带的前缀与 export 往脚本里传，脚本里某一行自己写的前缀同样认；谁能跑什么照上面那张表，不因为写在脚本里而变。
@@ -72,7 +78,9 @@
 #   喂给 python 这类非 shell 解释器的程序（`python3 x.py` 里用 subprocess 起的命令；heredoc 正文先剥掉）；
 #   make（Makefile 里起的命令）与 xargs 起的命令；cd 到变量路径之后的裸 cargo test（认不出在不在工作区根）与相对路径的脚本；
 #   命令词不带斜杠、从 PATH 里找到的脚本（直接执行的 `x.sh`）；同一条命令里用 WrittenScript 认不出的写法先写出再执行的脚本——
-#   执行时文件还不在的按「不存在」放行并记检出，已经在的读到的是旧内容；source 进来的文件里 export 的变量对外层后面命令的影响。
+#   执行时文件还不在的按「不存在」放行并记检出，已经在的读到的是旧内容；source 进来的文件里 export 的变量对外层后面命令的影响；
+#   .cargo/config.toml 与 `--config <文件>` 里定的别名与 runner；拷走改名、又不点名登记的用例函数的测试二进制（看门狗在进程这一层同样认不出）。
+# 接受的误拒：`--include-ignored --exact <同一目标里的快用例>` 照拒（过滤之后剩下哪几条，执行前判不出）；别名展开之后不是 test 的照拒。
 # 另一道，与重型不重型无关：子 agent 跑编译出来的代码——cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算）、
 #   直接执行 cargo 编出来的二进制（lib_heavy_tests.runs_compiled_code）——要经 research/scripts/run-with-memory-cap.sh 跑，不经它的拒
 #   （它先判整机放不放得下、放不下排队，撞了上限只杀这一条；records/2026-09-16-subagent拆分提案.md 第四十节第 30 行）。放在这里而不另起一个 hook：
@@ -93,7 +101,7 @@
 HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
 # python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 write-guard.sh 同一个坑）。
 python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
-import importlib.util, json, os, re, shutil, subprocess, sys, tempfile, time
+import importlib.util, json, os, re, shlex, shutil, subprocess, sys, tempfile, time
 from datetime import datetime, timezone
 from typing import NamedTuple
 
@@ -459,10 +467,23 @@
         occurrence = occurrences.get(key, 0)
         occurrences[key] = occurrence + 1
         record_files_written(command, state)
+        # /usr/bin/time、flock、systemd-run 这一类包在外面照样起那条命令的：剥掉它，里面那条按一段命令文本接着判（包装、脚本、内存包装照认）
+        launched = heavy_tests.command_under_launcher(command.words)
+        if launched is not None:
+            inner_words, inner_directory = launched
+            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
+            inner = heavy_uses(shlex.join(inner_words), inner_start, command.environment, frames_of(key, occurrence), None, state, command.memory_capped)
+            uses += inner.uses
+            notices += inner.notices
+            uncapped += inner.uncapped
+            scripts_read += inner.scripts_read
+            hit_depth_limit = hit_depth_limit or inner.hit_depth_limit
+            lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, inner.lowest_cycle_frame)
+            continue
         compiled = heavy_tests.runs_compiled_code(command.words, command.directory)
         if compiled and not command.memory_capped:
             uncapped.append(UncappedRun(compiled, frames_of(key, occurrence)))
-        test = heavy_tests.classify(command.words, command.directory)
+        test = heavy_tests.classify(command.words, command.directory, command.environment)
         if test:
             uses.append(HeavyUse(test, command.environment.get(OCCASION_VARIABLE), frames_of(key, occurrence)))
             continue
@@ -732,6 +753,36 @@
              commit + "bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-harness --test sample_crash_enumeration -- --include-ignored --exact the_full_case", 0),
             ("主 agent 不带前缀直接执行崩溃枚举用例的测试二进制", None, "./target/release/deps/sample_crash_enumeration-0123456789abcdef --ignored", 2),
             ("主 agent 带前缀直接执行崩溃枚举用例的测试二进制", None, commit + "./target/release/deps/sample_crash_enumeration-0123456789abcdef --ignored", 0),
+            # 包在外面照样起那条命令的（/usr/bin/time、flock、systemd-run……）剥掉之后照判；里面经内存包装的算经包装
+            ("实现员 /usr/bin/time -v 包一层跑 --all", writer, "/usr/bin/time -v cargo test --all", 2),
+            ("实现员 systemd-run --scope 自设内存上限跑崩溃枚举用例", writer,
+             "systemd-run --user --scope -q -p MemoryMax=8G cargo test -p singlefs-harness --test sample_crash_enumeration -- --ignored", 2),
+            ("实现员 perf stat 包一层跑 --all", writer, "perf stat -e cycles cargo test --all", 2),
+            ("实现员 /usr/bin/time 包一层跑自己的测试目标、没经内存包装", writer, "/usr/bin/time -v cargo test -p singlefs-core --test core_contract", 2),
+            ("实现员内存包装里 /usr/bin/time 包一层跑自己的测试目标", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G /usr/bin/time -v cargo test -p singlefs-core --test core_contract", 0),
+            ("实现员 /usr/bin/time 包在内存包装外面跑自己的测试目标", writer,
+             "/usr/bin/time -v bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract", 0),
+            ("崩溃验证员带前缀 /usr/bin/time 包一层经内存包装跑崩溃枚举用例", crash,
+             commit + "/usr/bin/time -v bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-harness --test sample_crash_enumeration "
+             "-- --include-ignored --exact the_full_case", 0),
+            ("主 agent 不带前缀 flock 包一层跑 gate.sh", None, "flock /tmp/gate.lock bash .claude/scripts/gate.sh --staged", 2),
+            # 起测试的外部子命令、别名与 runner、只列不跑、没标 #[ignore] 的登记用例
+            ("实现员经内存包装 cargo nextest run --run-ignored all 跑崩溃枚举用例", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo nextest run -p singlefs-harness --test sample_crash_enumeration --run-ignored all", 2),
+            ("实现员 cargo mutants 整包", writer, "cargo mutants -p singlefs-harness", 2),
+            ("实现员经内存包装、前缀里设 runner、点名崩溃枚举用例不带 --ignored", writer,
+             "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness "
+             "--test sample_crash_enumeration", 2),
+            ("实现员经内存包装、--config 定别名再用别名", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo --config 'alias.xt=\"test\"' xt -p singlefs-harness --test sample_crash_enumeration", 2),
+            ("实现员经内存包装列崩溃枚举用例（--ignored --list）", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test sample_crash_enumeration -- --ignored --list", 0),
+            ("实现员经内存包装跑用例函数没标 #[ignore] 的登记目标、不带 --ignored", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test unmarked_crash_enumeration", 2),
+            ("实现员经内存包装执行拷走改名的测试二进制、点名登记的用例函数", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G /tmp/elsewhere/rc --ignored --exact the_full_case", 2),
+            ("实现员 grep 找 --include-ignored 与登记的用例函数名", writer, "grep -rn -e --include-ignored -e the_full_case crates", 0),
             ("崩溃验证员带 =user-request 跑 55 号", crash, request + "bash .claude/gate.d/55-qemu-first-transaction.sh", 0),
             ("崩溃验证员带前缀跑 59 号", crash, "GATE_MUTATION_TARGET_DIR=/tmp/t " + commit + "nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh", 0),
             ("门禁分诊带前缀跑 gate.sh --staged", triage, commit + "nice -n 19 bash .claude/scripts/gate.sh --staged", 0),
--- a/.claude/hooks/lib_heavy_tests.py
+++ b/.claude/hooks/lib_heavy_tests.py
@@ -5,11 +5,17 @@
 谁能跑哪一类、要带什么前缀，是 heavy-test-guard.sh 的事，不在这里。
 
 入口：
-  classify(words, directory) -> HeavyTest | None
-      一条已经剥掉前缀与包装的命令：words[0] 是命令词（照写的样子，没取 basename），其后是参数；directory 是它的当前目录，认不出时 None。
+  classify(words, directory, environment=None) -> HeavyTest | None
+      一条已经剥掉前缀与包装的命令：words[0] 是命令词（照写的样子，没取 basename），其后是参数；directory 是它的当前目录，认不出时 None；
+      environment 是这条命令看得到的、命令文本里设过的变量（lib_shell_words 的 CommandAtPosition.environment），认 runner 与别名要用。
+  command_under_launcher(words) -> (list[str], str | None) | None
+      words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序（/usr/bin/time、flock、rustup run、chrt、prlimit、systemd-run、strace、perf）时，
+      剥掉它交回它起的那条命令与那条命令的当前目录（None 是不变）；调用方把交回的命令再交给 lib_shell_words 切一遍、逐条判。
+      这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
   classify_process(argv, cwd) -> list[HeavyTest]
       一个在跑的进程：argv 是 /proc/<pid>/cmdline 按 NUL 切开的那一串，cwd 是 /proc/<pid>/cwd 指向的目录。
-      先照 lib_shell_words 剥掉 bash / sh 起脚本、bash -c、nice / timeout / env / taskset、capped.sh N 这类包装，再逐条 classify；空列表就是不重型。
+      先照 lib_shell_words 剥掉 bash / sh 起脚本、bash -c、nice / timeout / env / taskset、capped.sh N 这类包装，
+      再照 command_under_launcher 剥掉 /usr/bin/time 这一类，逐条 classify；空列表就是不重型。
   runs_compiled_code(words, directory) -> str | None
       一条已经剥掉前缀与包装的命令会不会跑编译出来的代码：cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算），
       或直接执行 cargo 编出来的二进制（测试二进制，与名字里带 target 的目录底下 debug / release 里的）；会就交一句说明。
@@ -19,13 +25,24 @@
       算不算重型、带什么参数才算，都在名字那一格判完了，要读脚本正文的一方不再读进去。
   python3 lib_heavy_tests.py --selftest
 
-认的输入：cargo 命令行（test 与 run）、按名字认的脚本与门禁阶段、qemu-system-*、herd7，
-以及直接执行的测试二进制 `<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制哈希>`：按 <名字> 判，名字含 layer0 的算层 0。
+认的输入：cargo 命令行（test / t、run / r，以及起测试的 nextest run、miri、llvm-cov、hack、mutants）、按名字认的脚本与门禁阶段、
+qemu-system-*、herd7，以及直接执行的测试二进制 `<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制哈希>`：按 <名字> 判，
+名字含 layer0 的算层 0。libtest 参数带 --list 的（只列用例、一条都不跑）不算重型。
 崩溃枚举用例（门禁 54 号逐条跑的那几条，登记在 .claude/gate.d/stage-inputs.tsv 键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>）：
-跑到它们的测试目标、libtest 参数带 --ignored 或 --include-ignored 的算重型（cargo test 点名它、通配命中它、或不挑目标而包里有它；直接执行它的测试二进制）。
+跑到它们的测试目标（cargo test 点名它、通配命中它、或不挑目标而包里有它；直接执行它的测试二进制）而下面任一条成立的算重型：
+  libtest 参数带 --ignored 或 --include-ignored（nextest 是 --run-ignored 的值不是 default）；
+  可能带上它而看不见：cargo 全局选项 --config 里定了别名（alias.<名>，且子命令就是那个别名）或 runner（.runner），
+  命令看得到的环境变量里有 CARGO_TARGET_*_RUNNER，或 CARGO_ALIAS_<名> 定的别名就是子命令；
+  登记的用例函数没标 #[ignore]（读那个包里测试目标的源码，判法与 research/scripts/admission.py 的 crash-cases 自查同一份：本模块按文件路径导入它）。
+另有一条按参数认：任何命令（按文本处理参数的 grep、git、sed 这一类除外）参数里有 --ignored 或 --include-ignored、又有登记的用例函数名，
+按跑崩溃枚举用例算（拷走改名的测试二进制、find -exec 起的这类，名字认不出，靠点名的用例函数认）。
 登记表取两份的并：从命令的当前目录（有 --manifest-path 时取它所在的目录）往上找到的第一份，与这份文件所在仓的那一份。
+接受的误拒：`--include-ignored --exact <同一目标里的快用例>` 照拒（过滤之后剩下哪几条，执行前判不出）；别名展开之后不是 test 的照拒。
+看不见的：.cargo/config.toml 与 `--config <文件>` 里定的别名与 runner；拷走改名、又不点名用例函数的测试二进制；
+cargo mutants -d 指到别处的树。别名与 runner 那两种由看门狗（research/scripts/agent-watch.py）在进程这一层兜：cargo 最后照样以原名
+带 --ignored 起那个测试二进制（推的，没量）；拷走改名、又不点名用例函数的，看门狗同样认不出。
 """
-import fnmatch, glob, importlib.util, os, re, shlex, shutil, sys, tempfile, tomllib
+import fnmatch, functools, glob, importlib.util, os, re, shlex, shutil, sys, tempfile, tomllib
 from typing import NamedTuple
 
 
@@ -73,9 +90,23 @@
 
 # 崩溃枚举用例的登记表（与 research/scripts/admission.py 读的是同一份）：键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>
 CRASH_CASE_REGISTRY = os.path.join(".claude", "gate.d", "stage-inputs.tsv")
-CRASH_CASE_TEST_CONDITION = re.compile(r"^test=(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):[A-Za-z_][A-Za-z0-9_]*$")
+CRASH_CASE_TEST_CONDITION = re.compile(r"^test=(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
 HOOK_REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
 IGNORED_TEST_ARGUMENTS = {"--ignored", "--include-ignored"}
+# libtest 只列用例、一条都不跑的参数
+LIST_ONLY_TEST_ARGUMENT = "--list"
+# 判用例函数标没标 #[ignore] 的那一份（与门禁 54 号的 crash-cases 自查同一套判法）
+ADMISSION_MODULE_PATH = os.path.join(HOOK_REPOSITORY, "research", "scripts", "admission.py")
+# 按参数认崩溃枚举用例时不看的命令：它们把参数当文本搜、打印、比对，不执行测试二进制
+COMMANDS_TREATING_ARGUMENTS_AS_TEXT = {"grep", "egrep", "fgrep", "rg", "ag", "git", "sed", "awk", "echo", "printf", "cat", "head", "tail",
+                                       "less", "wc", "sort", "uniq", "diff", "tee", "jq", "ls"}
+
+
+class RegisteredCrashCase(NamedTuple):
+    registry_root: str  # 登记它的那份登记表所在的仓根
+    package: str
+    target: str
+    function: str
 
 
 def crash_case_registries(directory):
@@ -96,15 +127,16 @@
     return found
 
 
-def registered_crash_case_targets(directory):
-    """登记的崩溃枚举用例：{(包名, 测试目标)}。读不了的登记表当它没有。"""
-    targets = set()
+def registered_crash_cases(directory):
+    """登记的崩溃枚举用例（RegisteredCrashCase 的集合）。读不了的登记表当它没有。"""
+    cases = set()
     for path in crash_case_registries(directory):
         try:
             with open(path, encoding="utf-8", errors="replace") as handle:
                 lines = handle.read().split("\n")
         except OSError:
             continue
+        registry_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(path))))
         for line in lines:
             columns = line.split("\t")
             if not columns[0].startswith("crash-case:") or len(columns) < 3 or columns[2].lstrip().startswith("#"):
@@ -112,8 +144,45 @@
             for token in columns[2].split():
                 match = CRASH_CASE_TEST_CONDITION.match(token)
                 if match:
-                    targets.add((match.group("package"), match.group("target")))
-    return targets
+                    cases.add(RegisteredCrashCase(registry_root, match.group("package"), match.group("target"), match.group("function")))
+    return cases
+
+
+def registered_crash_case_targets(directory):
+    """登记的崩溃枚举用例：{(包名, 测试目标)}。"""
+    return {(case.package, case.target) for case in registered_crash_cases(directory)}
+
+
+@functools.cache
+def admission_module():
+    """research/scripts/admission.py（按文件路径导入一次）；导入不了交 None，调用方按「判不出」处理。"""
+    try:
+        spec = importlib.util.spec_from_file_location("admission_for_heavy_tests", ADMISSION_MODULE_PATH)
+        module = importlib.util.module_from_spec(spec)
+        spec.loader.exec_module(module)
+    except Exception:  # 文件不在、语法错、导入时抛的都算导入不了
+        return None
+    return module
+
+
+def crash_case_package_directory(case, members):
+    """这条用例的包目录（绝对路径）：命令所在工作区的成员里有它就取那一份（cargo 编的是它），没有就取登记它的那个仓的；都没有交 None。"""
+    if case.package in members:
+        return members[case.package]
+    root_manifest = os.path.join(case.registry_root, "Cargo.toml")
+    sections = manifest_sections(root_manifest)
+    if sections and "workspace" in sections:
+        return workspace_packages(root_manifest, sections).get(case.package)
+    return None
+
+
+def crash_case_function_runs_without_ignored(case, package_directory):
+    """登记的用例函数在 package_directory 这个包里没标 #[ignore]（不带 --ignored 的 cargo test 也会跑它）：没标 True；
+    标了、找不到那个目标或那个函数、导入不了 admission.py 都交 False（判不出的不在这里拒，提交时 54 号的 crash-cases 自查判红）。"""
+    module = admission_module()
+    if module is None or package_directory is None:
+        return False
+    return module.test_function_is_marked_ignored(package_directory, ".", case.target, case.function) is False
 
 
 def manifest_sections(path):
@@ -176,42 +245,137 @@
 
 
 CARGO_GLOBAL_OPTIONS_WITH_VALUE = {"--color", "--config", "-Z"}
+# cargo test 与 cargo nextest run 带一个值的选项（nextest 的 --run-ignored、-E 过滤式这一类也在里面：cargo test 不认它们，放一张表不误读）
 CARGO_TEST_OPTIONS_WITH_VALUE = {"-p", "--package", "--exclude", "--test", "--bin", "--example", "--bench", "-F", "--features",
                                  "--target", "--target-dir", "--manifest-path", "-j", "--jobs", "--profile", "--color",
-                                 "--message-format", "--config", "-Z", "--lockfile-path"}
+                                 "--message-format", "--config", "-Z", "--lockfile-path",
+                                 "--run-ignored", "-E", "--filterset", "-P", "--retries", "--partition", "--test-threads", "--max-fail"}
 NARROWING_SELECTORS = {"--test", "--lib", "--bin", "--bins", "--example", "--examples", "--bench", "--benches", "--doc"}
 GLOB_CHARACTERS = set("*?[")
+# cargo nextest run --run-ignored 的值里会跑 ignored 用例的（default 不跑）
+NEXTEST_RUN_IGNORED_VALUES = {"only", "all", "ignored-only"}
+# cargo llvm-cov 不起测试的子命令；cargo hack 后面接的、起测试或跑程序的子命令
+LLVM_COV_NON_RUNNING_SUBCOMMANDS = {"report", "clean", "show-env"}
+HACK_INNER_SUBCOMMANDS = {"test", "t", "run", "r", "nextest", "miri", "llvm-cov", "mutants"}
+# cargo mutants 的选项里 cargo test 也认、照搬过去的（选包的那几个）
+MUTANTS_PACKAGE_OPTIONS_WITH_VALUE = {"-p", "--package", "--manifest-path"}
+RUNNER_CONFIGURATION = re.compile(r"\.runner\s*=")
+ALIAS_CONFIGURATION = re.compile(r"^\s*alias\.([A-Za-z0-9_-]+)\s*=")
+RUNNER_ENVIRONMENT_VARIABLE = re.compile(r"^CARGO_TARGET_.+_RUNNER$")
+ALIAS_ENVIRONMENT_PREFIX = "CARGO_ALIAS_"
+
+
+class CargoCommand(NamedTuple):
+    subcommand: str
+    rest: list                  # 子命令后面的参数
+    directory: str | None       # 按 -C 换过之后的目录
+    configuration_values: list  # 全局选项 --config 的值（--config=<值> 与 --config <值> 两种写法）
 
 
 def cargo_subcommand(arguments, directory):
-    """cargo 的全局选项（+工具链、--config、-C 目录……）跳过之后的子命令：交 (子命令, 它后面的参数, 按 -C 换过之后的目录)；没有子命令交 None。"""
-    index = 0
+    """cargo 的全局选项（+工具链、--config、-C 目录……）跳过之后的子命令：交 CargoCommand；没有子命令交 None。"""
+    index, configuration_values = 0, []
     while index < len(arguments):
         argument = arguments[index]
         if argument.startswith("+"):
             index += 1
         elif argument in CARGO_GLOBAL_OPTIONS_WITH_VALUE:
+            if argument == "--config" and index + 1 < len(arguments):
+                configuration_values.append(arguments[index + 1])
             index += 2
         elif argument == "-C":
             directory = shell_words.resolve_path(directory, arguments[index + 1]) if index + 1 < len(arguments) else directory
             index += 2
         elif argument.startswith("-"):
+            if argument.startswith("--config="):
+                configuration_values.append(argument[len("--config="):])
             index += 1
         else:
             break
     if index >= len(arguments):
         return None
-    return arguments[index], arguments[index + 1:], directory
+    return CargoCommand(arguments[index], arguments[index + 1:], directory, configuration_values)
+
+
+def mutants_test_arguments(rest):
+    """cargo mutants 的参数 → 它反复起的 cargo test 的参数：选包的选项（-p / --package / --manifest-path / --workspace）照搬，`--` 之后的原样接上。"""
+    before, after = (rest[:rest.index("--")], rest[rest.index("--") + 1:]) if "--" in rest else (rest, [])
+    kept, index = [], 0
+    while index < len(before):
+        word = before[index]
+        if word in MUTANTS_PACKAGE_OPTIONS_WITH_VALUE:
+            kept += before[index:index + 2]
+            index += 2
+            continue
+        if word == "--workspace" or word.startswith(("--package=", "--manifest-path=")) or (word.startswith("-p") and len(word) > 2):
+            kept.append(word)
+        index += 1
+    return kept + after
+
+
+def test_invocation(subcommand, rest):
+    """cargo 的子命令与它后面的参数 → (怎么起的：'test'、'nextest'、'run' 或 None, 照 cargo test 的写法读的参数)。
+    nextest 只认 run（r）；miri、llvm-cov、hack 剥掉自己那一层再看里面那个子命令（llvm-cov 不带子命令时就是跑测试）；mutants 反复起 cargo test。"""
+    if subcommand in ("test", "t"):
+        return "test", rest
+    if subcommand in ("run", "r"):
+        return "run", rest
+    if subcommand == "nextest":
+        return ("nextest", rest[1:]) if rest[:1] in (["run"], ["r"]) else (None, [])
+    if subcommand == "miri":
+        return test_invocation(rest[0], rest[1:]) if rest else (None, [])
+    if subcommand == "llvm-cov":
+        if rest[:1] == ["nextest"]:
+            return "nextest", rest[1:]
+        if rest and rest[0] in LLVM_COV_NON_RUNNING_SUBCOMMANDS:
+            return None, []
+        if rest and not rest[0].startswith("-"):
+            return test_invocation(rest[0], rest[1:])
+        return "test", rest
+    if subcommand == "hack":
+        options = rest[:rest.index("--")] if "--" in rest else rest
+        for index, word in enumerate(options):
+            if word in HACK_INNER_SUBCOMMANDS:
+                return test_invocation(word, rest[:index] + rest[index + 1:])
+        return None, []
+    if subcommand == "mutants":
+        return "test", mutants_test_arguments(rest)
+    return None, []
+
+
+def alias_of_subcommand(command, environment):
+    """子命令是 --config 的 alias.<名> 或环境变量 CARGO_ALIAS_<名> 定的别名时交一句（展开之后是什么、带什么参数都看不见，当 cargo test 判）；不是交 None。"""
+    aliases = {match.group(1) for value in command.configuration_values for match in [ALIAS_CONFIGURATION.match(value)] if match}
+    if command.subcommand in aliases:
+        return f"--config 里定的别名 {command.subcommand}（展开之后带什么参数看不见）"
+    alias_variable = ALIAS_ENVIRONMENT_PREFIX + command.subcommand.upper().replace("-", "_")
+    if alias_variable in environment:
+        return f"环境变量 {alias_variable} 定的别名 {command.subcommand}（展开之后带什么参数看不见）"
+    return None
 
 
-def cargo_use(arguments, directory):
-    """cargo 这一条算不算重型：返回 HeavyTest 或 None。"""
+def runner_of_test_binaries(command, environment):
+    """--config 里定了 …runner、或环境变量里有 CARGO_TARGET_*_RUNNER 时交一句（runner 能替测试二进制加 --ignored，命令行上看不见）；没有交 None。"""
+    if any(RUNNER_CONFIGURATION.search(value) for value in command.configuration_values):
+        return "--config 里定的 runner（它能替测试二进制加 --ignored）"
+    runners = sorted(name for name in environment if RUNNER_ENVIRONMENT_VARIABLE.match(name))
+    if runners:
+        return f"环境变量 {runners[0]} 定的 runner（它能替测试二进制加 --ignored）"
+    return None
+
+
+def cargo_use(arguments, directory, environment=None):
+    """cargo 这一条算不算重型：返回 HeavyTest 或 None。environment 是这条命令看得到的变量（认 runner 与别名）。"""
     found = cargo_subcommand(arguments, directory)
     if found is None:
         return None
-    subcommand, rest, directory = found
+    environment = environment or {}
+    alias_reason = alias_of_subcommand(found, environment)
+    invocation, rest = ("test", found.rest) if alias_reason else test_invocation(found.subcommand, found.rest)
+    unseen_reason = alias_reason or runner_of_test_binaries(found, environment)
+    directory = found.directory
     packages, test_names, selectors, binaries = [], [], set(), []
-    whole_workspace, manifest_argument = False, None
+    whole_workspace, manifest_argument, run_ignored_value = False, None, None
     position = 0
     while position < len(rest):
         argument = rest[position]
@@ -245,35 +409,46 @@
             whole_workspace = True
         elif option in ("--lib", "--bins", "--examples", "--tests", "--benches", "--all-targets", "--doc"):
             selectors.add(option)
-    if subcommand == "run":
+        elif option == "--run-ignored":
+            run_ignored_value = value
+    if invocation == "run":
         if "e152-file-system-benchmark" in binaries:
             return heavy_test("e152", "cargo run --bin e152-file-system-benchmark")
         return None
-    if subcommand not in ("test", "t"):
+    if invocation is None:
+        return None
+    described = f"cargo {found.subcommand}"
+    libtest_arguments = rest[rest.index("--") + 1:] if "--" in rest else []
+    if LIST_ONLY_TEST_ARGUMENT in libtest_arguments:
         return None
     if whole_workspace:
-        return heavy_test("full-cargo", "cargo test 带 --workspace / --all")
+        return heavy_test("full-cargo", f"{described} 带 --workspace / --all")
     named_layer0 = [name for name in test_names if "layer0" in name]
     if named_layer0:
-        return heavy_test("layer0-cargo", f"cargo test --test {named_layer0[0]}")
-    libtest_arguments = rest[rest.index("--") + 1:] if "--" in rest else []
-    runs_ignored = sorted(IGNORED_TEST_ARGUMENTS & set(libtest_arguments))
+        return heavy_test("layer0-cargo", f"{described} --test {named_layer0[0]}")
+    ignored_reasons = sorted(IGNORED_TEST_ARGUMENTS & set(libtest_arguments))
+    if invocation == "nextest" and run_ignored_value in NEXTEST_RUN_IGNORED_VALUES:
+        ignored_reasons.append(f"--run-ignored {run_ignored_value}")
+    if unseen_reason:
+        ignored_reasons.append(unseen_reason)
     manifest_path = shell_words.resolve_path(directory, manifest_argument) if manifest_argument else (nearest_manifest(directory) if directory else None)
-    crash_cases = registered_crash_case_targets(os.path.dirname(manifest_path) if manifest_path else directory) if runs_ignored else set()
-    for pattern in test_names:
-        crash_case_names = sorted({target for _package, target in crash_cases})
-        matched = fnmatch.filter(crash_case_names, pattern) if GLOB_CHARACTERS & set(pattern) else [name for name in crash_case_names if name == pattern]
-        if matched:
-            return heavy_test("crash-case-cargo", f"cargo test --test {pattern} 带 {runs_ignored[0]}，跑到登记的崩溃枚举用例（{'、'.join(matched)}）")
+    crash_cases = registered_crash_cases(os.path.dirname(manifest_path) if manifest_path else directory)
     sections = manifest_sections(manifest_path) if manifest_path else None
+    root_manifest, root_sections = workspace_root_manifest(manifest_path) if sections is not None else (None, None)
+    members = workspace_packages(root_manifest, root_sections) if root_manifest else {}
+    crash_case_targets = sorted({case.target for case in crash_cases})
+    for pattern in test_names:
+        matched = fnmatch.filter(crash_case_targets, pattern) if GLOB_CHARACTERS & set(pattern) else [name for name in crash_case_targets if name == pattern]
+        found_heavy = crash_case_run("crash-case-cargo", f"{described} --test {pattern}", [case for case in crash_cases if case.target in matched],
+                                     ignored_reasons, members) if matched else None
+        if found_heavy:
+            return found_heavy
     if sections is None:
         return None
     is_virtual_workspace_root = "workspace" in sections and "package" not in sections
     narrowed = bool(selectors & NARROWING_SELECTORS)
     if not packages and is_virtual_workspace_root and not narrowed:
-        return heavy_test("full-cargo", f"在工作区根（{os.path.dirname(manifest_path)}）上不带 -p / --test / --lib / --bin 的 cargo test")
-    root_manifest, root_sections = workspace_root_manifest(manifest_path)
-    members = workspace_packages(root_manifest, root_sections) if root_manifest else {}
+        return heavy_test("full-cargo", f"在工作区根（{os.path.dirname(manifest_path)}）上不带 -p / --test / --lib / --bin 的 {described}")
     if packages:
         scope = [members[name] for name in packages if name in members]
     elif is_virtual_workspace_root:
@@ -281,22 +456,36 @@
     else:
         scope = [os.path.normpath(os.path.dirname(manifest_path))]
     if not narrowed and members and set(scope) == set(members.values()):
-        return heavy_test("full-cargo", (f"cargo test 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 "
+        return heavy_test("full-cargo", (f"{described} 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 "
                                          f"{len(members)} 个成员），等于全量"))
     layer0_targets = sorted({name for package_directory in scope for name in layer0_test_targets(package_directory)})
     unselected = not selectors or bool(selectors & {"--tests", "--all-targets"})
-    crash_cases_in_scope = sorted(target for package, target in crash_cases if members.get(package) in scope)
+    crash_cases_in_scope = [case for case in crash_cases if members.get(case.package) in scope]
     if not layer0_targets and unselected and crash_cases_in_scope:
-        return heavy_test("crash-case-cargo", f"cargo test 不挑目标、带 {runs_ignored[0]}，会跑到登记的崩溃枚举用例（{'、'.join(crash_cases_in_scope)}）")
+        found_heavy = crash_case_run("crash-case-cargo", f"{described} 不挑目标", crash_cases_in_scope, ignored_reasons, members)
+        if found_heavy:
+            return found_heavy
     if not layer0_targets:
         return None
     if not selectors or selectors & {"--tests", "--all-targets"}:
-        return heavy_test("layer0-cargo", f"cargo test 不挑目标，会跑到名字含 layer0 的测试二进制（{'、'.join(layer0_targets)}）")
+        return heavy_test("layer0-cargo", f"{described} 不挑目标，会跑到名字含 layer0 的测试二进制（{'、'.join(layer0_targets)}）")
     for pattern in test_names:
         if GLOB_CHARACTERS & set(pattern):
             matched = fnmatch.filter(layer0_targets, pattern)
             if matched:
-                return heavy_test("layer0-cargo", f"cargo test --test {pattern} 命中名字含 layer0 的测试二进制（{'、'.join(matched)}）")
+                return heavy_test("layer0-cargo", f"{described} --test {pattern} 命中名字含 layer0 的测试二进制（{'、'.join(matched)}）")
+    return None
+
+
+def crash_case_run(kind, described, cases, ignored_reasons, members):
+    """点到登记的崩溃枚举用例 cases 的那一条算不算重型：带了（或可能带了）跑 ignored 用例的参数，或有一条的用例函数没标 #[ignore]，
+    交 kind 那一类的 HeavyTest；否则交 None。members 是命令所在工作区的成员（包名 → 包目录），找用例的源码用。"""
+    targets = "、".join(sorted({case.target for case in cases}))
+    if ignored_reasons:
+        return heavy_test(kind, f"{described} 带 {ignored_reasons[0]}，跑到登记的崩溃枚举用例（{targets}）")
+    unmarked = sorted({case.function for case in cases if crash_case_function_runs_without_ignored(case, crash_case_package_directory(case, members))})
+    if unmarked:
+        return heavy_test(kind, f"{described} 不带 --ignored，而登记的用例函数 {unmarked[0]} 没标 #[ignore]，照样跑到全量（{targets}）")
     return None
 
 
@@ -327,35 +516,101 @@
 
 
 def runs_compiled_code(words, directory):
-    """一条剥掉前缀与包装的命令会不会跑编译出来的代码（子 agent 要经 run-with-memory-cap.sh 跑的那一类）：会就交一句说明，不会交 None。"""
+    """一条剥掉前缀与包装的命令会不会跑编译出来的代码（子 agent 要经 run-with-memory-cap.sh 跑的那一类）：会就交一句说明，不会交 None。
+    cargo 的子命令按 test_invocation 认（nextest run、miri、llvm-cov、hack、mutants 同样算），另认 bench 与别名（展开之后看不见，当会跑）。"""
     if os.path.basename(words[0]) == "cargo":
         found = cargo_subcommand(words[1:], directory)
-        if found is None or found[0] not in COMPILED_CODE_SUBCOMMANDS:
+        if found is None:
+            return None
+        invocation, rest = test_invocation(found.subcommand, found.rest)
+        if found.subcommand not in COMPILED_CODE_SUBCOMMANDS and invocation is None and not alias_of_subcommand(found, {}):
             return None
-        subcommand, rest, _directory = found
         options = rest[:rest.index("--")] if "--" in rest else rest
         if "--no-run" in options:
             return None
-        return f"cargo {subcommand}"
+        return f"cargo {found.subcommand}"
     if TEST_BINARY_PATH.search(words[0]) or BUILT_BINARY_PATH.search(words[0]):
         return f"直接执行 cargo 编出来的二进制 {words[0]}"
     return None
 
 
-def classify(words, directory):
-    """一条已经剥掉前缀与包装的命令：返回 HeavyTest 或 None。"""
+# 包在命令外面、之后照样起那条命令的程序（lib_shell_words 那张前缀表之外的）：名字 → 它带一个值的选项。
+# 这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
+LAUNCHER_OPTIONS_WITH_VALUE = {
+    "time": {"-f", "--format", "-o", "--output"},
+    "flock": {"-w", "--wait", "--timeout", "-E", "--conflict-exit-code"},
+    "chrt": {"-T", "--sched-runtime", "-P", "--sched-period", "-D", "--sched-deadline"},
+    "prlimit": {"-o", "--output", "-p", "--pid"},
+    "systemd-run": {"-u", "--unit", "-p", "--property", "--description", "--slice", "-E", "--setenv", "-M", "--machine", "-H", "--host",
+                    "--uid", "--gid", "--nice", "--working-directory", "--on-active", "--on-boot", "--on-startup", "--on-unit-active",
+                    "--on-unit-inactive", "--on-calendar", "--path-property", "--socket-property", "--timer-property", "--service-type",
+                    "-C", "--capsule"},
+    "strace": {"-a", "-b", "-e", "-E", "-I", "-o", "-O", "-p", "-P", "-s", "-S", "-u", "-U", "-X"},
+    "perf": {"-e", "--event", "-o", "--output", "-p", "--pid", "-t", "--tid", "-C", "--cpu", "-r", "--repeat", "-I", "--interval-print",
+             "-G", "--cgroup", "-x", "--field-separator", "-F", "--freq", "-c", "--count", "-m", "--mmap-pages", "-u", "--uid", "-j",
+             "--branch-filter", "--call-graph", "-D", "--delay", "-M", "--metrics"},
+    "rustup": set(),
+}
+# 选项之后、那条命令之前还有几个位置参数：flock 的锁文件、chrt 的优先级、rustup run 的工具链
+LAUNCHER_POSITIONAL_COUNT = {"flock": 1, "chrt": 1, "rustup": 1}
+# perf 起一条命令的子命令；后面还要再跟一个 record 的那几组
+PERF_SUBCOMMANDS_RUNNING_A_COMMAND = {"stat", "record", "trace"}
+PERF_GROUPS_WITH_RECORD = {"mem", "c2c", "sched", "lock", "kmem", "kwork"}
+
+
+def command_under_launcher(words):
+    """words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序时，剥掉它（连同它的选项与位置参数），交回 (它起的那条命令的词, 那条命令的当前目录：
+    None 是不变，systemd-run --working-directory 给的是相对当前目录的写法)；不是这几样、或后面没有要起的命令（strace -p、flock 只给 fd）交 None。
+    flock 的 -c <字符串> 交回成 sh -c <字符串>。rustup 只认 rustup run，perf 只认 stat / record / trace 与 <组> record。"""
+    name = os.path.basename(words[0])
+    if name not in LAUNCHER_OPTIONS_WITH_VALUE:
+        return None
+    position = 1
+    if name == "rustup":
+        if words[1:2] != ["run"]:
+            return None
+        position = 2
+    elif name == "perf":
+        if words[1:2] and words[1] in PERF_SUBCOMMANDS_RUNNING_A_COMMAND:
+            position = 2
+        elif words[1:2] and words[1] in PERF_GROUPS_WITH_RECORD and words[2:3] == ["record"]:
+            position = 3
+        else:
+            return None
+    options_with_value, working_directory = LAUNCHER_OPTIONS_WITH_VALUE[name], None
+    while position < len(words):
+        word = words[position]
+        if word == "--":
+            position += 1
+            break
+        if not word.startswith("-") or word == "-":
+            break
+        option, has_attached_value, attached_value = word.partition("=")
+        value = attached_value if has_attached_value else (words[position + 1] if position + 1 < len(words) else "")
+        if name == "systemd-run" and option == "--working-directory":
+            working_directory = value
+        position += 2 if option in options_with_value and not has_attached_value else 1
+    if name == "flock" and position + 1 < len(words) and words[position + 1] in ("-c", "--command"):
+        return (["sh", "-c", words[position + 2]], working_directory) if position + 2 < len(words) else None
+    position += LAUNCHER_POSITIONAL_COUNT.get(name, 0)
+    return (words[position:], working_directory) if position < len(words) else None
+
+
+def classify(words, directory, environment=None):
+    """一条已经剥掉前缀与包装的命令：返回 HeavyTest 或 None。environment 是这条命令看得到的变量（认 runner 与别名）。"""
     name, arguments = os.path.basename(words[0]), words[1:]
     stage = gate_stage_number(words[0], directory)
     if stage is not None:
         return heavy_test(STAGE_KIND[stage], f"门禁 {stage} 号（{name}）") if stage in STAGE_KIND else None
     binary = test_binary_name(words[0])
     if binary is not None:
+        if LIST_ONLY_TEST_ARGUMENT in arguments:
+            return None
         if "layer0" in binary:
             return heavy_test("layer0-binary", f"直接执行名字含 layer0 的测试二进制（{binary}）")
+        cases = [case for case in registered_crash_cases(directory) if case.target == binary]
         runs_ignored = sorted(IGNORED_TEST_ARGUMENTS & set(arguments))
-        if runs_ignored and binary in {target for _package, target in registered_crash_case_targets(directory)}:
-            return heavy_test("crash-case-binary", f"直接执行登记的崩溃枚举用例的测试二进制（{binary}）、带 {runs_ignored[0]}")
-        return None
+        return crash_case_run("crash-case-binary", f"直接执行登记的崩溃枚举用例的测试二进制（{binary}）", cases, runs_ignored, {}) if cases else None
     if name.startswith("qemu-system"):
         return heavy_test("qemu-system", name)
     if name == "vm-bench.sh":
@@ -379,22 +634,51 @@
     if name in ("e152-file-system-benchmark", "e152-run.sh"):
         return heavy_test("e152", name)
     if name == "cargo":
-        return cargo_use(arguments, directory)
-    return None
+        return cargo_use(arguments, directory, environment)
+    return crash_case_named_in_arguments(name, arguments, directory)
 
 
-def classify_process(argv, cwd):
-    """一个在跑的进程（argv 与 cwd）里认出的重型用法；包装（bash 起脚本、bash -c、nice、timeout、env、capped.sh……）照共用切词剥掉。"""
-    if not argv:
-        return []
+def crash_case_named_in_arguments(name, arguments, directory):
+    """不认得的命令（按文本处理参数的 grep、git 这一类除外）参数里有 --ignored / --include-ignored、又有登记的用例函数名：
+    按跑崩溃枚举用例算（拷走改名的测试二进制、find -exec 起的，名字认不出，靠点名的用例函数认）。"""
+    runs_ignored = sorted(IGNORED_TEST_ARGUMENTS & set(arguments))
+    if name in COMMANDS_TREATING_ARGUMENTS_AS_TEXT or not runs_ignored:
+        return None
+    named = sorted({case.function for case in registered_crash_cases(directory)} & set(arguments))
+    if not named:
+        return None
+    return heavy_test("crash-case-binary", f"{name} 带 {runs_ignored[0]} 与登记的用例函数名 {named[0]}：按跑崩溃枚举用例算")
+
+
+def heavy_tests_in_command_text(text, directory, environment=None):
+    """一段命令文本里命令位置上的每一处重型用法：lib_shell_words 剥掉包装，command_under_launcher 剥掉 /usr/bin/time 这一类（剥完再切一遍），逐条 classify。"""
     found = []
-    for command in shell_words.commands_at_command_position(shlex.join(argv), cwd).commands:
-        test = classify(command.words, command.directory)
+    for command in shell_words.commands_at_command_position(text, directory, environment).commands:
+        launched = command_under_launcher(command.words)
+        if launched is not None:
+            inner_words, inner_directory = launched
+            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
+            found += heavy_tests_in_command_text(shlex.join(inner_words), inner_start, command.environment)
+            continue
+        test = classify(command.words, command.directory, command.environment)
         if test:
             found.append(test)
     return found
 
 
+def classify_process(argv, cwd):
+    """一个在跑的进程（argv 与 cwd）里认出的重型用法；包装（bash 起脚本、bash -c、nice、timeout、env、capped.sh……）照共用切词剥掉，
+    /usr/bin/time、flock、systemd-run 这一类照 command_under_launcher 剥掉。"""
+    if not argv:
+        return []
+    # 进程的 argv[0] 是 time 时它就是那个程序（不是 bash 的关键字 time）：先按 argv 剥，再交给切词
+    launched = command_under_launcher(argv)
+    if launched is not None:
+        inner_words, inner_directory = launched
+        return classify_process(inner_words, shell_words.resolve_path(cwd, inner_directory) if inner_directory else cwd)
+    return heavy_tests_in_command_text(shlex.join(argv), cwd)
+
+
 def judged_by_name(word, directory):
     """按名字判的仓内脚本（门禁阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在仓内位置上）：判定在名字那一格做完，不再读脚本正文。"""
     if gate_stage_number(word, directory) is not None:
@@ -407,8 +691,9 @@
 
 
 def build_sample_workspace(work):
-    """仓根与 research/ 两个虚工作区，harness 里有一个名字含 layer0 的测试目标；登记表登记两条崩溃枚举用例：
-    harness 里的 sample_crash_enumeration，与只有它一个测试目标的包 singlefs-checker 里的 checker_crash_enumeration。"""
+    """仓根与 research/ 两个虚工作区，harness 里有一个名字含 layer0 的测试目标；登记表登记三条崩溃枚举用例：
+    harness 里的 sample_crash_enumeration，与只有它一个测试目标的包 singlefs-checker 里的 checker_crash_enumeration（用例函数都标了 #[ignore]），
+    harness 里用例函数没标 #[ignore] 的 unmarked_crash_enumeration。"""
     def write(relative, text):
         path = os.path.join(work, relative)
         os.makedirs(os.path.dirname(path), exist_ok=True)
@@ -421,12 +706,15 @@
     write("crates/singlefs-harness/Cargo.toml", '[package]\nname = "singlefs-harness"\nversion = "0.0.0"\n')
     write("crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs", "")
     write("crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs", "")
-    write("crates/singlefs-harness/tests/sample_crash_enumeration.rs", "")
+    marked_case = '#[test]\nfn quick_case() {}\n#[test]\n#[ignore = "崩溃枚举（样本）：平时不跑"]\nfn the_full_case() {}\n'
+    write("crates/singlefs-harness/tests/sample_crash_enumeration.rs", marked_case)
+    write("crates/singlefs-harness/tests/unmarked_crash_enumeration.rs", "#[test]\nfn the_unmarked_case() {}\n")
     write("crates/singlefs-checker/Cargo.toml", '[package]\nname = "singlefs-checker"\nversion = "0.0.0"\n')
-    write("crates/singlefs-checker/tests/checker_crash_enumeration.rs", "")
+    write("crates/singlefs-checker/tests/checker_crash_enumeration.rs", marked_case)
     write(CRASH_CASE_REGISTRY, "# 样本登记表\n54-layer0-replay.sh\tcrates/\t# 样本\n"
                                "crash-case:sample\tcrates/ Cargo.toml\ttest=singlefs-harness:sample_crash_enumeration:the_full_case\t# 样本\n"
-                               "crash-case:checker\tcrates/ Cargo.toml\ttest=singlefs-checker:checker_crash_enumeration:the_full_case\t# 样本\n")
+                               "crash-case:checker\tcrates/ Cargo.toml\ttest=singlefs-checker:checker_crash_enumeration:the_full_case\t# 样本\n"
+                               "crash-case:unmarked\tcrates/ Cargo.toml\ttest=singlefs-harness:unmarked_crash_enumeration:the_unmarked_case\t# 样本\n")
     write("research/Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["e7-index-bench"]\n')
     write("research/e7-index-bench/Cargo.toml", '[package]\nname = "e7-index-bench"\nversion = "0.0.0"\n')
 
@@ -467,6 +755,74 @@
             ("只有崩溃枚举用例的包 --lib 带 --ignored", ["cargo", "test", "-p", "singlefs-checker", "--lib", "--", "--ignored"], work, None),
             ("别的测试目标带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "second_transaction_step_one_overwrite", "--", "--ignored"],
              work, None),
+            # 用例函数没标 #[ignore]：不带 --ignored 也跑到全量（读源码判，与 admission.py crash-cases 同一套）
+            ("点名用例函数没标 #[ignore] 的登记目标、不带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "unmarked_crash_enumeration"],
+             work, "crash-case-cargo"),
+            # 只列用例、一条都不跑的不算
+            ("点名崩溃枚举用例、--ignored --list", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored", "--list"],
+             work, None),
+            ("点名层 0 目标、-- --list", ["cargo", "test", "-p", "singlefs-harness", "--test", "first_transaction_step_seven_layer0", "--", "--list"], work, None),
+            # 起测试的外部子命令与包装子命令
+            ("cargo nextest run --run-ignored all 点名崩溃枚举用例",
+             ["cargo", "nextest", "run", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--run-ignored", "all"], work, "crash-case-cargo"),
+            ("cargo nextest run --run-ignored=only -E 过滤式、只有崩溃枚举用例的包",
+             ["cargo", "nextest", "run", "-p", "singlefs-checker", "--run-ignored=only", "-E", "test(=the_full_case)"], work, "crash-case-cargo"),
+            ("cargo nextest run --run-ignored default 只跑快用例",
+             ["cargo", "nextest", "run", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--run-ignored", "default"], work, None),
+            ("cargo nextest run 在工作区根裸跑", ["cargo", "nextest", "run"], work, "full-cargo"),
+            ("cargo nextest list 不起测试", ["cargo", "nextest", "list", "--workspace"], work, None),
+            ("cargo mutants 整包（反复跑 cargo test，包里有层 0 目标）", ["cargo", "mutants", "-p", "singlefs-harness"], work, "layer0-cargo"),
+            ("cargo mutants 尾参交给 cargo test、带 --include-ignored",
+             ["cargo", "mutants", "-p", "singlefs-checker", "--", "--test", "checker_crash_enumeration", "--", "--include-ignored"], work, "crash-case-cargo"),
+            ("cargo miri test 点名崩溃枚举用例、带 --ignored",
+             ["cargo", "miri", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored"], work, "crash-case-cargo"),
+            ("cargo llvm-cov 不带子命令、点名崩溃枚举用例带 --ignored",
+             ["cargo", "llvm-cov", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored"], work, "crash-case-cargo"),
+            ("cargo llvm-cov report 不起测试", ["cargo", "llvm-cov", "report"], work, None),
+            ("cargo hack 在子命令前带自己的选项、test --workspace", ["cargo", "hack", "--each-feature", "test", "--workspace"], work, "full-cargo"),
+            # 别名与 runner：展开之后带什么参数看不见，当可能带 --ignored
+            ("--config 定别名再用别名、点名崩溃枚举用例", ["cargo", "--config", 'alias.xt="test"', "xt", "-p", "singlefs-harness", "--test", "sample_crash_enumeration"],
+             work, "crash-case-cargo"),
+            ("--config= 写法定的 runner、点名崩溃枚举用例不带 --ignored",
+             ["cargo", '--config=target.x86_64-unknown-linux-gnu.runner=["/tmp/add-ignored.sh"]', "test", "-p", "singlefs-harness", "--test",
+              "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("环境变量里的 runner、点名崩溃枚举用例不带 --ignored",
+             ["env", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/add-ignored.sh", "cargo", "test", "-p", "singlefs-harness", "--test",
+              "sample_crash_enumeration"], work, "crash-case-cargo"),
+            ("环境变量 CARGO_ALIAS_XT 定的别名", ["env", "CARGO_ALIAS_XT=test", "cargo", "xt", "-p", "singlefs-harness", "--test", "sample_crash_enumeration"],
+             work, "crash-case-cargo"),
+            ("--config 定了别名、子命令不是它（cargo build）", ["cargo", "--config", 'alias.xt="test"', "build"], work, None),
+            ("环境变量里有 runner、cargo build 不跑测试", ["env", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/x", "cargo", "build", "-p", "singlefs-harness"],
+             work, None),
+            ("环境变量里有 runner、点名不是崩溃枚举用例的目标", ["env", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/tmp/x", "cargo", "test", "-p", "singlefs-core",
+                                                  "--test", "core_contract"], work, None),
+        ]
+        # lib_shell_words 前缀表之外、包在命令外面照样起那条命令的程序（command_under_launcher）
+        launcher_cases = [
+            ("/usr/bin/time -v 包一层、点名崩溃枚举用例带 --ignored",
+             ["/usr/bin/time", "-v", "cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored"], work, "crash-case-cargo"),
+            ("time -f 格式 -o 文件包一层层 0 目标",
+             ["time", "-f", "%M", "-o", "/tmp/t", "cargo", "test", "--release", "-p", "singlefs-harness", "--test", "first_transaction_step_seven_layer0"],
+             work, "layer0-cargo"),
+            ("flock -w 秒数 锁文件 包一层", ["flock", "-w", "5", "/tmp/lock", "cargo", "test", "--workspace"], work, "full-cargo"),
+            ("flock 锁文件 -c 字符串", ["flock", "/tmp/lock", "-c", "cargo test --workspace"], work, "full-cargo"),
+            ("rustup run --install stable 包一层", ["rustup", "run", "--install", "stable", "cargo", "test", "--all"], work, "full-cargo"),
+            ("chrt -i 0 包一层", ["chrt", "-i", "0", "cargo", "test", "--all"], work, "full-cargo"),
+            ("prlimit --as= 包一层", ["prlimit", "--as=8000000000", "cargo", "test", "--all"], work, "full-cargo"),
+            ("systemd-run --user --scope -p MemoryMax=8G 包一层", ["systemd-run", "--user", "--scope", "-q", "-p", "MemoryMax=8G", "cargo", "test", "--all"],
+             work, "full-cargo"),
+            ("systemd-run --working-directory= 进 harness 裸跑",
+             ["systemd-run", "--user", "--scope", "--working-directory=crates/singlefs-harness", "cargo", "test"], work, "layer0-cargo"),
+            ("strace -f -o 文件包一层", ["strace", "-f", "-o", "/tmp/trace", "cargo", "test", "--all"], work, "full-cargo"),
+            ("perf stat -e 事件包一层", ["perf", "stat", "-e", "cycles", "cargo", "test", "--all"], work, "full-cargo"),
+            ("perf record -g -- 之后", ["perf", "record", "-g", "--", "cargo", "test", "--all"], work, "full-cargo"),
+            ("perf mem record 包一层", ["perf", "mem", "record", "cargo", "test", "--all"], work, "full-cargo"),
+            ("nice 里 /usr/bin/time 里 flock 一层套一层", ["nice", "-n", "19", "/usr/bin/time", "-v", "flock", "/tmp/l", "cargo", "test", "--all"], work, "full-cargo"),
+            ("strace -p 进程号：没有要起的命令", ["strace", "-p", "1234"], work, None),
+            ("chrt -p 查优先级：没有要起的命令", ["chrt", "-p", "1234"], work, None),
+            ("rustup show 不是 rustup run", ["rustup", "show"], work, None),
+            ("perf report 不起命令", ["perf", "report", "-i", "perf.data"], work, None),
+            ("/usr/bin/time 包一个不重型的测试目标", ["/usr/bin/time", "-v", "cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
         ]
         binary_cases = [
             ("绝对路径的层 0 测试二进制", [os.path.join(work, "target", "release", "deps", layer0_binary), "--test-threads", "4"], work, "layer0-binary"),
@@ -481,6 +837,18 @@
              [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef"), "--include-ignored", "--exact", "the_full_case"],
              work, "crash-case-binary"),
             ("崩溃枚举用例的测试二进制不带 --ignored", [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef")], work, None),
+            ("用例函数没标 #[ignore] 的登记用例的测试二进制、不带 --ignored",
+             [os.path.join(work, "target", "release", "deps", "unmarked_crash_enumeration-0123456789abcdef")], work, "crash-case-binary"),
+            ("崩溃枚举用例的测试二进制 --ignored --list", [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef"),
+                                                  "--ignored", "--list"], work, None),
+            ("层 0 测试二进制 --list", [os.path.join(work, "target", "release", "deps", layer0_binary), "--list"], work, None),
+            # 名字认不出（拷走改名、find -exec）时按参数认：带 --ignored 又点名登记的用例函数
+            ("拷走改名的测试二进制带 --ignored --exact 登记的用例函数", ["/tmp/elsewhere/rc", "--ignored", "--exact", "the_full_case"], work, "crash-case-binary"),
+            ("find -exec 执行测试二进制、点名登记的用例函数",
+             ["find", "target", "-name", "sample_crash_enumeration-*", "-exec", "{}", "--ignored", "the_full_case", ";"], work, "crash-case-binary"),
+            ("拷走改名的测试二进制带 --ignored、不点名用例函数：认不出（文件头「看不见的」）", ["/tmp/elsewhere/rc", "--ignored"], work, None),
+            ("grep 找 --include-ignored 与用例函数名：按文本处理参数的不算", ["grep", "-rn", "-e", "--include-ignored", "-e", "the_full_case", "crates"],
+             work, None),
         ]
         # 名字不含 layer0 的那几条：含 layer0 的按层 0 那一类先认出来，判不出登记表取没取到
         own_repository_targets = sorted(target for _package, target in registered_crash_case_targets(HOOK_REPOSITORY) if "layer0" not in target)
@@ -491,7 +859,7 @@
         else:
             results.append((f"这份文件所在仓的登记表（{os.path.join(HOOK_REPOSITORY, CRASH_CASE_REGISTRY)}）里有名字不含 layer0 的崩溃枚举用例",
                             True, False))
-        for label, argv, cwd, want in cargo_cases + binary_cases:
+        for label, argv, cwd, want in cargo_cases + launcher_cases + binary_cases:
             found = classify_process(argv, cwd)
             results.append((label, want, found[0].kind if found else None))
         name_cases = [
@@ -506,6 +874,13 @@
         # 跑不跑编译出来的代码（与重型不重型无关）：真 / 假
         compiled_cases = [
             ("cargo test 一个测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], True),
+            ("cargo nextest run", ["cargo", "nextest", "run", "-p", "singlefs-core"], True),
+            ("cargo miri test", ["cargo", "miri", "test", "-p", "singlefs-core"], True),
+            ("cargo llvm-cov 不带子命令", ["cargo", "llvm-cov", "-p", "singlefs-core"], True),
+            ("cargo mutants", ["cargo", "mutants", "-p", "singlefs-core"], True),
+            ("--config 定的别名", ["cargo", "--config", 'alias.xt="test"', "xt"], True),
+            ("cargo nextest list 不跑", ["cargo", "nextest", "list"], False),
+            ("cargo llvm-cov report 不跑", ["cargo", "llvm-cov", "report"], False),
             ("cargo t 简写", ["cargo", "t", "--lib"], True),
             ("cargo run 实验二进制", ["cargo", "run", "--release", "--bin", "e160-random-small-read-share"], True),
             ("cargo r 简写", ["cargo", "r"], True),
@@ -533,10 +908,12 @@
     for label, want, got in failures:
         print(f"  ✗ lib_heavy_tests 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
     if failures:
-        print("    → 看 classify()、cargo_subcommand()、cargo_use()、test_binary_name()、runs_compiled_code()、classify_process()、judged_by_name() "
-              "与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / KNOWN_SCRIPT_LOCATIONS 几张表")
+        print("    → 看 classify()、cargo_subcommand()、cargo_use()、test_invocation()、crash_case_run()、command_under_launcher()、test_binary_name()、"
+              "runs_compiled_code()、classify_process()、judged_by_name() 与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / "
+              "KNOWN_SCRIPT_LOCATIONS / LAUNCHER_OPTIONS_WITH_VALUE 几张表；判用例函数标没标 #[ignore] 的那一段在 research/scripts/admission.py")
         return 1
     print(f"  ✓ lib_heavy_tests 自检通过（查了 {len(results)} 种：cargo 与包装过的命令行 {len(cargo_cases)} 种、"
+          f"/usr/bin/time、flock、systemd-run 这一类包在外面的 {len(launcher_cases)} 种、"
           f"直接执行的测试二进制 {len(binary_cases)} 种、按名字判的脚本 {len(name_cases)} 种、跑不跑编译出来的代码 {len(compiled_cases)} 种）")
     return 0
 
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -55,9 +55,9 @@
 - 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
 - 全部实验复跑：87 号。
 - E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
-- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标，libtest 参数带 `--ignored` 或 `--include-ignored` 的——`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制。不带这两个参数、只跑那个测试目标里快用例的不算。
+- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标（`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制），而下面任一条成立的：libtest 参数带 `--ignored` 或 `--include-ignored`（`cargo nextest run` 是 `--run-ignored only` / `all`）；`--config` 或环境变量里定了测试二进制的 runner，或子命令是 `--config` / `CARGO_ALIAS_` 定的别名；登记的用例函数没标 `#[ignore]`。另外任何命令带 `--ignored` / `--include-ignored` 又点名登记的用例函数的也算（`grep`、`git` 这类按文本处理参数的除外）。不带这两个参数、用例函数标了 `#[ignore]`、只跑那个测试目标里快用例的不算。
 
-只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
+只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算，`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf stat|record|trace` 包在外面的剥掉照算。重型测试清单各类里的 `cargo test` 同样指 `cargo nextest run`、`cargo miri test`、`cargo llvm-cov`、`cargo hack test`、`cargo mutants` 与别名；libtest 参数带 `--list`（只列用例、一条都不跑）的哪一类都不算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
 
 | 场合 | 跑不跑 |
 |---|---|
--- a/research/scripts/admission.py
+++ b/research/scripts/admission.py
@@ -18,8 +18,9 @@
                                           （例 herd7 的版本）。不在 git 树里的东西靠它进复用判定，见 ①
       正则、值、路径与参数里不许有空白（条件之间按空白切）。实验行写了门禁的种类、门禁行写了实验的种类，自查都判错。
   崩溃枚举用例行（键是 crash-case:<名>，门禁 54 号逐条按复用判定跑、逐条记全绿标记）：路径写整个 crates/ 与 Cargo 清单、锁，
-    不按用例手列；算输入时自动减去别的测试目标独占的测试文件（见「崩溃枚举用例」一节）。第三列认四种：
-      test=<包>:<测试目标>:<用例函数>     恰好一条：跑的是 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数>
+    不按用例手列；算输入时自动减去用例读不到的文件（见「崩溃枚举用例」一节）。第三列认四种：
+      test=<包>:<测试目标>:<用例函数>     恰好一条：跑的是 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数>；
+                                         用例函数要标 #[ignore]（不带 --ignored 的 cargo test 不跑它，重型测试闸按这一条认它）
       count-line=<前缀>                  日志里以「<前缀> 」开头的行恰好一行，原样记进全绿标记
       exhaustive=<前缀>                  那一行带 exhaustive=true（前缀要先登记成 count-line=）
       threads=<前缀>                     按那一行的 states= 找 LAYER0_PARALLEL_FINISHED 行判工作线程（前缀要先登记成 count-line=）
@@ -29,7 +30,12 @@
   config、config.toml——在的才进，按内容进，名字里不带绝对路径（同一批内容在主工作区与临时 worktree 里算出同一个数）；
   环境变量 RUSTFLAGS、CARGO_ENCODED_RUSTFLAGS、CARGO_PROFILE_*、CARGO_BUILD_*（CARGO_BUILD_JOBS 除外：只定编译并行度，
   research/scripts/capped.sh 与 mutate.sh 会设它）、CARGO_TARGET_*_RUSTFLAGS、CARGO_TARGET_*_RUNNER、RUSTC_WRAPPER、
-  RUSTC_WORKSPACE_WRAPPER，设了的才进；RUSTC 设了的进它的值与 `$RUSTC -V`。登记路径下指向目录的符号链接按链接指向的目录里的文件计入。
+  RUSTC_WORKSPACE_WRAPPER，设了的才进；RUSTC 设了的进它的值与 `$RUSTC -V`。
+  runner、wrapper 与编译器指的可执行文件另进一行：环境变量 CARGO_TARGET_*_RUNNER、RUSTC_WRAPPER、RUSTC_WORKSPACE_WRAPPER、
+  CARGO_BUILD_RUSTC_WRAPPER、CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER，与上面那几份配置文件里的 target.<三元组或 cfg>.runner、
+  build.rustc-wrapper、build.rustc-workspace-wrapper，指的那个程序按内容进（值的首词是程序；带斜杠的相对路径，环境变量从仓根起、
+  配置文件从 .cargo 所在的那一层起；不带斜杠的在 PATH 里找；找不到的进「找不到」一句）；CARGO_BUILD_RUSTC 与配置文件里的 build.rustc
+  进它的 `-V`（与 RUSTC 同一种）。登记路径下指向目录的符号链接按链接指向的目录里的文件计入。
 
 准入条件分三类，这里管前两类：
   ① 输入没变 ⇒ 不跑，沿用上次结论。门禁：登记的路径在 refs/sop/staged-green 那棵树与这一次的暂存树之间 git 说没变
@@ -53,7 +59,7 @@
                                  把这个键的输入逐文件清单（「sha256  路径」，按路径字节序）写进清单文件，末尾按参数次序
                                  加「sha256  <名字>」行；stdout 打「<整张清单的 sha256> <文件数>」。另算一遍逐字节相同的写法见 --selftest 那一格
   keys <根>                      列出登记表里的实验键，一行一个
-  crash-cases <根>               核登记的崩溃枚举用例（第三列的写法、包与测试目标在不在、用例函数在不在），每条打一行
+  crash-cases <根>               核登记的崩溃枚举用例（第三列的写法、包与测试目标在不在、用例函数在不在、它标没标 #[ignore]），每条打一行
                                  「<键><制表符><包><制表符><测试目标><制表符><用例函数>」；登记有错退 2，原因打在 stdout（不带 ✗，阶段自己打 ✗ 与出路）
   crash-case-manifest <根> <键> <清单文件> [--extra-file <名字> <文件>]... [--toolchain] [--build-environment]
                                  这条用例的输入清单：登记路径下的文件减去别的测试目标独占的测试文件，末尾按参数次序加名字行，
@@ -87,8 +93,14 @@
 管不到的：跑的是不是按今天的源码编出来的二进制（指纹按源码算，跑的是 target/ 里的旧二进制时两边对不上，
 replay.sh 与 cargo run 开跑前都会重编）；登记的路径少写了一条（那条输入变了不会放行，要靠强制开关，登记行本身进指纹，
 补登记之后自然放行）；产物头的指纹行是不是真由那一趟写的（由装置入口调本模块写，拷来的产物照样带着）。
-崩溃枚举用例减去的「别的测试目标独占的文件」认不出的：用拼出来的名字在运行期读别的测试文件（名字不以整词出现在任何代码里）、
-build.rs 之外的构建期代码按目录读 tests/；这两种会让那份文件被减掉而它其实被读了。
+崩溃枚举用例减去的文件认不出的：用拼出来的名字在运行期读别的测试文件（名字不以整词出现在任何代码里）、构建脚本（build.rs 与 package.build
+指的那一份）之外的构建期代码按目录读 tests/、编译期拼出来的名字（include!(concat!(…)) 这一类）指到别的包的测试文件；这几种会让那份文件
+被减掉而它其实被读了。认得出形状、认不出读的是哪一份的两种按宽处理：包里有 include! / include_str! / include_bytes! 套 concat! 的，
+这个包的测试文件一份都不减；任何一个包的构建脚本（注释去掉之后）出现整词 tests 的，哪个包的测试文件都不减。
+减得少的（改了照样让用例重跑）：几个测试目标共用的测试模块（tests/common_*/ 这一类）要顺着模块图才减得准，这里不走模块图；
+包里有构建脚本、[[test]]、autotests 时整包不减。登记的四条里第二条流全量约 2.3 天（推的），这几种改动都会让它重跑。
+减掉之后仍可能让用例红的：src/bin/ 下的文件编不过时 cargo test 也编不过（集成测试要先编本包的 bin），这一种交给构建与 clippy 那几道，
+不靠崩溃枚举用例的标记。linker（配置文件里的 target.*.linker、环境变量 CARGO_TARGET_*_LINKER）指的程序不按内容进。
 """
 import glob
 import hashlib
@@ -137,6 +149,17 @@
 LAYER0_PARALLEL_FINISHED_PREFIX = "LAYER0_PARALLEL_FINISHED "
 PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\. 1 passed; 0 failed; ")
 MARKER_DIFFERENCES_LISTED_AT_MOST = 20
+# 用例函数要标的属性：#[ignore] 或 #[ignore = "…"]（cfg_attr 里的条件 ignore 不算：条件不成立时不带 --ignored 照样跑它）
+IGNORE_ATTRIBUTE_FORM = re.compile(r"^#\[\s*ignore\s*(?:\]|=)")
+ATTRIBUTE_START = re.compile(r"#!?\s*\[")
+# 属性与 fn 之间可以夹的限定词（一次认一个，从 fn 往前剥）
+FUNCTION_QUALIFIER_AT_END = re.compile(r'(?:\bpub(?:\s*\([^()]*\))?|\basync|\bunsafe|\bconst|\bextern(?:\s*"[^"]*")?)$')
+# 编译期拼出来的文件名：include! / include_str! / include_bytes! 套 concat!（拼出来的是哪一份认不出，整包不减）
+COMPILE_TIME_CONCATENATED_INCLUDE = re.compile(r"\binclude(?:_str|_bytes)?!\s*[(\[{]\s*concat!")
+# 崩溃枚举用例的输入里减去的、不是测试文件的那几份：仓根起的路径 → 为什么用例读不到它（有代码按文件名点名它时照留）
+CRASH_CASE_FILES_NOT_READ = {"crates/mutations.tsv": "crates 变异表：59 号按它改源码再跑点名的测试，用例编译期与运行期都不读它"}
+# 集成测试拿本包 bin 的路径靠这个前缀的环境变量；没有代码读它时，src/bin/ 下的文件编不进也读不到用例
+BIN_EXECUTABLE_VARIABLE_PREFIX = "CARGO_BIN_EXE_"
 
 # 构建环境（build_environment_lines）：进指纹的配置文件名与环境变量
 CARGO_CONFIGURATION_FILE_NAMES = ("config", "config.toml")
@@ -146,6 +169,13 @@
                                     re.compile(r"^CARGO_TARGET_.+_RUSTFLAGS$"), re.compile(r"^CARGO_TARGET_.+_RUNNER$"))
 # 名字对得上而不进指纹的：它们不改编出来的东西，而本仓的包装会设它们，进了指纹同一批输入在包装里外算出两个数、全绿标记永远对不上
 BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT = {"CARGO_BUILD_JOBS": "只定编译并行度；research/scripts/capped.sh 与 mutate.sh 会设它"}
+# 指一个程序的环境变量（值的首词是程序）：那个程序按内容另进一行——换了 runner、wrapper 的内容而值不变，编出来或跑起来的就不是同一样
+BUILD_ENVIRONMENT_PROGRAM_VARIABLES = ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER")
+BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS = (re.compile(r"^CARGO_TARGET_.+_RUNNER$"),)
+# 指编译器的环境变量：进它的值与它的 -V
+BUILD_ENVIRONMENT_COMPILER_VARIABLES = ("RUSTC", "CARGO_BUILD_RUSTC")
+# 配置文件 [build] 里指程序、按内容进的键（build.rustc 另取 -V，target.<…>.runner 另按内容进）
+CONFIGURATION_PROGRAM_BUILD_KEYS = ("rustc-wrapper", "rustc-workspace-wrapper")
 
 
 class RegistrationError(Exception):
@@ -425,7 +455,9 @@
         for name in CARGO_CONFIGURATION_FILE_NAMES:
             path = os.path.join(directory, ".cargo", name)
             if os.path.isfile(path) and os.path.realpath(path) not in cargo_home_real_paths:
-                lines.append((f"<构建环境：目录层级里的 .cargo/{name}>", read_configuration_file(path)))
+                content = read_configuration_file(path)
+                lines.append((f"<构建环境：目录层级里的 .cargo/{name}>", content))
+                lines += configured_program_lines(f"目录层级里的 .cargo/{name}", content, directory, root, environment)
         for name in TOOLCHAIN_FILE_NAMES:
             path = os.path.join(directory, name)
             if os.path.isfile(path):
@@ -436,21 +468,77 @@
         directory = parent
     for path in cargo_home_files:
         if os.path.isfile(path):
-            lines.append((f"<构建环境：CARGO_HOME 下的 {os.path.basename(path)}>", read_configuration_file(path)))
+            content = read_configuration_file(path)
+            lines.append((f"<构建环境：CARGO_HOME 下的 {os.path.basename(path)}>", content))
+            lines += configured_program_lines(f"CARGO_HOME 下的 {os.path.basename(path)}", content, os.path.dirname(cargo_home), root, environment)
     for variable in sorted(environment):
         if variable in BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT:
             continue
         if variable in BUILD_ENVIRONMENT_EXACT_VARIABLES or any(form.match(variable) for form in BUILD_ENVIRONMENT_VARIABLE_FORMS):
             lines.append((f"<构建环境：环境变量 {variable}>", environment[variable].encode("utf-8", "surrogateescape")))
-    if "RUSTC" in environment:
-        rustc = environment["RUSTC"]
-        try:
-            completed = subprocess.run([rustc, "-V"], cwd=root, capture_output=True, env=dict(environment))
-        except OSError as error:
-            raise InputManifestError(f"RUSTC={rustc} 起不来：{error}") from error
-        if completed.returncode != 0 or not completed.stdout.strip():
-            raise InputManifestError(f"RUSTC={rustc} -V 退 {completed.returncode}、打了「{completed.stdout.decode('utf-8', 'replace').strip()}」")
-        lines.append(("<构建环境：RUSTC 的值与它的 -V>", rustc.encode("utf-8", "surrogateescape") + b"\n" + completed.stdout))
+    for variable in sorted(environment):
+        if variable in BUILD_ENVIRONMENT_PROGRAM_VARIABLES or any(form.match(variable) for form in BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS):
+            lines.append((f"<构建环境：环境变量 {variable} 指的程序>", program_contents(environment[variable].split(), root, environment)))
+    for variable in BUILD_ENVIRONMENT_COMPILER_VARIABLES:
+        if variable in environment:
+            compiler = environment[variable]
+            lines.append((f"<构建环境：{variable} 的值与它的 -V>", compiler.encode("utf-8", "surrogateescape") + b"\n"
+                          + compiler_version(compiler, root, environment, f"{variable}={compiler}")))
+    return lines
+
+
+def resolved_program(program, base_directory, environment):
+    """cargo 配置或环境变量里写的程序：带斜杠的按 base_directory 解开相对路径，不带斜杠的在 environment 的 PATH 里找；找不到交 None。"""
+    if not program:
+        return None
+    if "/" in program:
+        return program if os.path.isabs(program) else os.path.join(base_directory, program)
+    return shutil.which(program, path=environment.get("PATH"))
+
+
+def program_contents(words, base_directory, environment):
+    """首词是程序、其后是参数的一个值：交回那个程序的内容（进指纹）；找不到的交「找不到」一句（值本身另有一行进指纹）。"""
+    program = words[0] if words else ""
+    path = resolved_program(program, base_directory, environment)
+    if path and os.path.isfile(path):
+        return read_configuration_file(path)
+    return f"找不到：{program}".encode("utf-8", "surrogateescape")
+
+
+def compiler_version(program, root, environment, described_as):
+    """编译器的 `-V`（在仓根跑）；起不来、退非 0、一个字不打都抛 InputManifestError。"""
+    try:
+        completed = subprocess.run([program, "-V"], cwd=root, capture_output=True, env=dict(environment))
+    except OSError as error:
+        raise InputManifestError(f"{described_as} 起不来：{error}") from error
+    if completed.returncode != 0 or not completed.stdout.strip():
+        raise InputManifestError(f"{described_as} -V 退 {completed.returncode}、打了「{completed.stdout.decode('utf-8', 'replace').strip()}」")
+    return completed.stdout
+
+
+def configured_program_lines(label, content, base_directory, root, environment):
+    """一份 cargo 配置文件里指程序的键（build.rustc 取 -V；build.rustc-wrapper、build.rustc-workspace-wrapper、target.<…>.runner 按内容）：
+    [(名字, 内容)]。base_directory 是 .cargo 所在的那一层（相对路径从它起）。读不成 TOML 的不另进（原文已经进了，cargo 也读不了它）。"""
+    try:
+        settings = tomllib.loads(content.decode("utf-8"))
+    except (UnicodeDecodeError, ValueError):
+        return []
+    lines = []
+    build = settings.get("build") if isinstance(settings.get("build"), dict) else {}
+    compiler = build.get("rustc")
+    if isinstance(compiler, str):
+        lines.append((f"<构建环境：{label} 里 build.rustc 的 -V>",
+                      compiler_version(resolved_program(compiler, base_directory, environment) or compiler, root, environment,
+                                       f"{label} 里的 build.rustc = {compiler}")))
+    for key in CONFIGURATION_PROGRAM_BUILD_KEYS:
+        if isinstance(build.get(key), str):
+            lines.append((f"<构建环境：{label} 里 build.{key} 指的程序>", program_contents([build[key]], base_directory, environment)))
+    targets = settings.get("target") if isinstance(settings.get("target"), dict) else {}
+    for target_name in sorted(targets):
+        runner = targets[target_name].get("runner") if isinstance(targets[target_name], dict) else None
+        words = runner.split() if isinstance(runner, str) else [str(word) for word in runner] if isinstance(runner, list) else []
+        if words:
+            lines.append((f"<构建环境：{label} 里 target.{target_name}.runner 指的程序>", program_contents(words, base_directory, environment)))
     return lines
 
 
@@ -952,12 +1040,16 @@
 
 # ── 崩溃枚举用例（门禁 54 号：逐条按复用判定跑、逐条记全绿标记）────────────────
 #
-# 一条用例的输入 = 登记路径（整个 crates/ 与 Cargo 清单、锁）下 git 列得出的文件，减去别的测试目标独占的测试文件，
-# 再加判它的 54 号、工具链、构建环境与这条用例的登记行。别的测试目标 X 独占的文件：它所在的包没有 build.rs、没写 [[test]]、
-# package.autotests 与 package.build，X 是 tests/X.rs（或 tests/X/main.rs 那种目录目标、连同那个目录），而 X 的名字不以整词
-# 出现在别处任何 .rs（注释去掉之后）与 Cargo.toml 里——`mod X;`、`#[path = "…X.rs"]`、`include_str!("…X.rs")`、
-# 运行期按字面路径读它，都让它留在输入里。登记只写整个 crates/，不按用例手列它读哪些文件：新加的共用文件（tests/common.rs、
-# build.rs、新拆出的 crate）默认就在输入里。
+# 一条用例的输入 = 登记路径（整个 crates/ 与 Cargo 清单、锁）下 git 列得出的文件，减去用例读不到的文件，再加判它的 54 号、
+# 准入模块（这一份）、工具链、构建环境与这条用例的登记行（54 号按 --extra-file、--toolchain、--build-environment 给）。用例读不到的文件有三种：
+#   ① 别的测试目标 X 独占的文件：它所在的包没有构建脚本（build.rs 或 package.build）、没写 [[test]] 与 package.autotests，包里没有
+#      include! / include_str! / include_bytes! 套 concat! 的代码，任何一个包的构建脚本（注释去掉之后）都不出现整词 tests；
+#      X 是 tests/X.rs（或 tests/X/main.rs 那种目录目标、连同那个目录），而 X 的名字不以整词出现在别处任何 .rs（注释去掉之后）与
+#      Cargo.toml 里——`mod X;`、`#[path = "…X.rs"]`、`include_str!("…X.rs")`、运行期按字面路径读它，都让它留在输入里；
+#   ② CRASH_CASE_FILES_NOT_READ 里的（crates/mutations.tsv），没有代码与 Cargo.toml 按文件名点名它；
+#   ③ 没有任何代码读 CARGO_BIN_EXE_ 时，各包 src/bin/ 下的文件（包里有构建脚本、[[test]]、autotests 的不减；
+#      src/bin/ 之外的代码按 `bin/<它在 src/bin/ 下的路径>` 点名它的不减）。
+# 登记只写整个 crates/，不按用例手列它读哪些文件：新加的共用文件（tests/common.rs、build.rs、新拆出的 crate）默认就在输入里。
 
 class CrashCase:
     def __init__(self, row, package, target, function, count_lines, exhaustive_lines, thread_lines):
@@ -1042,26 +1134,39 @@
     return packages
 
 
+def test_target_source_files(root, package_directory, target):
+    """包目录（相对 root）下测试目标 target 的源文件（相对 root）：tests/<目标>.rs，或 tests/<目标>/main.rs 那种目录目标里的全部 .rs；
+    目标不在交空表。"""
+    single_file = os.path.join(package_directory, "tests", target + ".rs")
+    directory_target = os.path.join(package_directory, "tests", target)
+    if os.path.isfile(os.path.join(root, single_file)):
+        return [single_file]
+    if os.path.isfile(os.path.join(root, directory_target, "main.rs")):
+        return sorted(os.path.relpath(path, root)
+                      for path in glob.glob(os.path.join(root, directory_target, "**", "*.rs"), recursive=True))
+    return []
+
+
 def crash_case_test_files(root, case):
     """这条用例的测试目标的源文件（仓根起）：tests/<目标>.rs，或 tests/<目标>/main.rs 那种目录目标里的全部 .rs。
-    包不是工作区成员、目标不在、用例函数在目标里找不到，都抛 RegistrationError。"""
+    包不是工作区成员、目标不在、用例函数在目标里找不到、用例函数没标 #[ignore]，都抛 RegistrationError。"""
     package_directory = workspace_package_directories(root).get(case.package)
     if package_directory is None:
         raise RegistrationError(f"崩溃枚举用例 {case.key} 的包 {case.package} 不是仓根 Cargo.toml 的工作区成员")
-    single_file = os.path.join(package_directory, "tests", case.target + ".rs")
-    directory_target = os.path.join(package_directory, "tests", case.target)
-    if os.path.isfile(os.path.join(root, single_file)):
-        files = [single_file]
-    elif os.path.isfile(os.path.join(root, directory_target, "main.rs")):
-        files = sorted(os.path.relpath(path, root)
-                       for path in glob.glob(os.path.join(root, directory_target, "**", "*.rs"), recursive=True))
-    else:
+    files = test_target_source_files(root, package_directory, case.target)
+    if not files:
+        single_file = os.path.join(package_directory, "tests", case.target + ".rs")
+        directory_target = os.path.join(package_directory, "tests", case.target)
         raise RegistrationError(f"崩溃枚举用例 {case.key} 的测试目标 {case.target} 不在（没有 {single_file}，也没有 {directory_target}/main.rs）")
-    function_form = re.compile(r"\bfn\s+" + re.escape(case.function) + r"\s*\(")
     for relative in files:
         with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
-            if function_form.search(rust_code_without_comments(handle.read())):
-                return files
+            attributes = attributes_before_function(rust_code_without_comments(handle.read()), case.function)
+        if attributes is None:
+            continue
+        if not attributes_mark_ignored(attributes):
+            raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 没标 #[ignore]（{relative}）：不带 --ignored 的 "
+                                    "cargo test 也会跑到全量；给它补上 #[ignore = \"<为什么平时不跑>\"]")
+        return files
     raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 在 {'、'.join(files)} 里找不到（改了名？）")
 
 
@@ -1091,36 +1196,12 @@
                     position += 1
             pieces.append(" ")
             continue
-        character = text[position]
-        previous_is_identifier = position > 0 and (text[position - 1].isalnum() or text[position - 1] == "_")
-        raw_string = RAW_STRING_START.match(text, position)
-        if raw_string and not previous_is_identifier:
-            closing = '"' + raw_string.group("hashes")
-            end = text.find(closing, raw_string.end())
-            end = length if end < 0 else end + len(closing)
-            pieces.append(text[position:end])
-            position = end
-            continue
-        if character == '"':
-            end = position + 1
-            while end < length and text[end] != '"':
-                end += 2 if text[end] == "\\" else 1
-            end = min(end + 1, length)
+        end = end_of_literal(text, position)
+        if end is not None:
             pieces.append(text[position:end])
             position = end
             continue
-        if character == "'":
-            if text.startswith("\\", position + 1):
-                closing = text.find("'", position + 3)
-                end = length if closing < 0 else closing + 1
-                pieces.append(text[position:end])
-                position = end
-                continue
-            if position + 2 < length and text[position + 2] == "'":
-                pieces.append(text[position:position + 3])
-                position += 3
-                continue
-        pieces.append(character)
+        pieces.append(text[position])
         position += 1
     return "".join(pieces)
 
@@ -1128,6 +1209,102 @@
 RAW_STRING_START = re.compile(r'b?r(?P<hashes>#*)"')
 
 
+def end_of_literal(code, position):
+    """code[position] 起是字符串、原始字符串或字符字面量时交回它之后那个下标；不是时交 None（生命周期不算字面量）。"""
+    character = code[position]
+    if character in "br":
+        raw_string = RAW_STRING_START.match(code, position)
+        previous_is_identifier = position > 0 and (code[position - 1].isalnum() or code[position - 1] == "_")
+        if raw_string and not previous_is_identifier:
+            closing = '"' + raw_string.group("hashes")
+            end = code.find(closing, raw_string.end())
+            return len(code) if end < 0 else end + len(closing)
+    if character == '"':
+        end = position + 1
+        while end < len(code) and code[end] != '"':
+            end += 2 if code[end] == "\\" else 1
+        return min(end + 1, len(code))
+    if character == "'":
+        if code.startswith("\\", position + 1):
+            closing = code.find("'", position + 3)
+            return len(code) if closing < 0 else closing + 1
+        if position + 2 < len(code) and code[position + 2] == "'":
+            return position + 3
+    return None
+
+
+def attribute_spans(code):
+    """去掉注释之后的代码里每一条属性（`#[…]`、`#![…]`）的 (起, 止)：方括号按层数配对，字符串、原始字符串与字符字面量里的括号不算。"""
+    spans = []
+    position = 0
+    while position < len(code):
+        end = end_of_literal(code, position)
+        if end is not None:
+            position = end
+            continue
+        attribute = ATTRIBUTE_START.match(code, position)
+        if not attribute:
+            position += 1
+            continue
+        depth, cursor = 0, attribute.end() - 1
+        while cursor < len(code):
+            end = end_of_literal(code, cursor)
+            if end is not None:
+                cursor = end
+                continue
+            if code[cursor] == "[":
+                depth += 1
+            elif code[cursor] == "]":
+                depth -= 1
+                if depth == 0:
+                    break
+            cursor += 1
+        spans.append((position, min(cursor + 1, len(code))))
+        position = cursor + 1
+    return spans
+
+
+def attributes_before_function(code, function):
+    """去掉注释之后的代码里 `fn <function>(` 前面紧挨着的那串属性（原文，一条一个，按出现的次序）；没有这个函数交 None。
+    属性与 fn 之间可以夹 pub、pub(…)、async、unsafe、const、extern "…"。"""
+    found = re.search(r"\bfn\s+" + re.escape(function) + r"\s*\(", code)
+    if not found:
+        return None
+    head = code[:found.start()].rstrip()
+    while True:
+        qualifier = FUNCTION_QUALIFIER_AT_END.search(head)
+        if not qualifier:
+            break
+        head = head[:qualifier.start()].rstrip()
+    start_of_attribute_ending_at = {end: start for start, end in attribute_spans(head)}
+    attributes = []
+    cursor = len(head)
+    while cursor in start_of_attribute_ending_at:
+        start = start_of_attribute_ending_at[cursor]
+        attributes.append(head[start:cursor])
+        cursor = len(head[:start].rstrip())
+    return list(reversed(attributes))
+
+
+def attributes_mark_ignored(attributes):
+    """那串属性里有没有 #[ignore] 或 #[ignore = "…"]。"""
+    return any(IGNORE_ATTRIBUTE_FORM.match(attribute) for attribute in attributes)
+
+
+def test_function_is_marked_ignored(root, package_directory, target, function):
+    """root 底下包目录 package_directory（相对 root）里测试目标 target 的用例函数 function 标没标 #[ignore]：标了 True，没标 False，
+    目标不在、读不了、没有这个函数交 None。.claude/hooks/lib_heavy_tests.py（重型测试闸）按文件路径导入本模块调它，与 crash-cases 自查同一套判法。"""
+    for relative in test_target_source_files(root, package_directory, target):
+        try:
+            with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
+                attributes = attributes_before_function(rust_code_without_comments(handle.read()), function)
+        except OSError:
+            return None
+        if attributes is not None:
+            return attributes_mark_ignored(attributes)
+    return None
+
+
 def whole_word_form(word):
     return re.compile(r"(?<![A-Za-z0-9_])" + re.escape(word) + r"(?![A-Za-z0-9_])")
 
@@ -1135,7 +1312,7 @@
 def test_targets_of_package(package_directory, listed_names):
     """包目录下 tests/ 里 cargo 自动认的集成测试目标：目标名 → 它的文件（tests/<名>.rs；tests/<名>/main.rs 那种目标是 tests/<名>/ 下全部文件）。
     package_directory 与 listed_names 都是仓根起的 str 路径。"""
-    tests_prefix = ("" if package_directory == "." else package_directory + "/") + "tests/"
+    tests_prefix = package_prefix(package_directory) + "tests/"
     listed_set = set(listed_names)
     targets = {}
     for name in listed_names:
@@ -1150,29 +1327,38 @@
     return targets
 
 
+def package_prefix(package_directory):
+    return "" if package_directory == "." else package_directory + "/"
+
+
+def package_directories_among(root, listed_names):
+    """listed_names（仓根起）里的包目录：有 [package] 的 Cargo.toml 所在的目录，仓根是 "."。"""
+    return sorted({os.path.dirname(name) or "." for name in listed_names
+                   if os.path.basename(name) == "Cargo.toml" and "package" in (manifest_sections(os.path.join(root, name)) or {})})
+
+
 def package_keeps_every_test_file(root, package_directory, listed_set):
     """包里有 build.rs、写了 [[test]]、package.autotests 或 package.build：测试目标怎么编、读什么由它们另定，这一个包的测试文件一份都不减。"""
-    prefix = "" if package_directory == "." else package_directory + "/"
     sections = manifest_sections(os.path.join(root, package_directory, "Cargo.toml")) or {}
     package = sections.get("package") or {}
-    return (prefix + "build.rs" in listed_set or "test" in sections
+    return (package_prefix(package_directory) + "build.rs" in listed_set or "test" in sections
             or "autotests" in package or "build" in package)
 
 
-def files_exclusive_to_other_test_targets(root, listed_files, own_package_directory, own_target):
-    """listed_files（bytes，仓根起）里别的测试目标独占的文件（bytes 的集合）：判法见本节开头的说明。"""
-    listed_names = [os.fsdecode(name) for name in listed_files]
-    listed_set = set(listed_names)
-    package_directories = sorted({os.path.dirname(name) or "." for name in listed_names
-                                  if os.path.basename(name) == "Cargo.toml"
-                                  and "package" in (manifest_sections(os.path.join(root, name)) or {})})
-    candidates = {}
+def build_script_names(root, package_directories, listed_set):
+    """各包的构建脚本（仓根起、列在 listed_set 里的）：build.rs，或 package.build 指的那一份（package.build = false 的没有）。"""
+    names = []
     for package_directory in package_directories:
-        if package_keeps_every_test_file(root, package_directory, listed_set):
-            continue
-        for target, files in test_targets_of_package(package_directory, listed_names).items():
-            if (os.path.normpath(package_directory), target) != (os.path.normpath(own_package_directory), own_target):
-                candidates[(package_directory, target)] = files
+        build = ((manifest_sections(os.path.join(root, package_directory, "Cargo.toml")) or {}).get("package") or {}).get("build")
+        relative = build if isinstance(build, str) else (None if build is False else "build.rs")
+        name = os.path.normpath(package_prefix(package_directory) + relative) if relative else None
+        if name in listed_set:
+            names.append(name)
+    return names
+
+
+def listed_code_texts(root, listed_names):
+    """listed_names（仓根起）里的 .rs（注释去掉之后）与 Cargo.toml（原文）：路径 → 文字。"""
     code_texts = {}
     for name in listed_names:
         if name.endswith(".rs") or os.path.basename(name) == "Cargo.toml":
@@ -1182,16 +1368,56 @@
             except OSError as error:
                 raise InputManifestError(f"读不了 {name}：{error}") from error
             code_texts[name] = rust_code_without_comments(text) if name.endswith(".rs") else text
+    return code_texts
+
+
+def files_exclusive_to_other_test_targets(root, listed_names, code_texts, own_package_directory, own_target):
+    """listed_names（仓根起的 str）里别的测试目标独占的文件（str 的集合）：判法见本节开头的 ①。"""
+    listed_set = set(listed_names)
+    package_directories = package_directories_among(root, listed_names)
+    tests_word = whole_word_form("tests")
+    if any(tests_word.search(code_texts.get(name, "")) for name in build_script_names(root, package_directories, listed_set)):
+        return set()
+    concatenating = [name for name, text in code_texts.items() if name.endswith(".rs") and COMPILE_TIME_CONCATENATED_INCLUDE.search(text)]
+    candidates = {}
+    for package_directory in package_directories:
+        if package_keeps_every_test_file(root, package_directory, listed_set):
+            continue
+        if any(name.startswith(package_prefix(package_directory)) for name in concatenating):
+            continue
+        for target, files in test_targets_of_package(package_directory, listed_names).items():
+            if (os.path.normpath(package_directory), target) != (os.path.normpath(own_package_directory), own_target):
+                candidates[(package_directory, target)] = files
     exclusive = set()
     for (_package_directory, target), files in candidates.items():
         own_files = set(files)
         named_elsewhere = any(whole_word_form(target).search(text) for name, text in code_texts.items() if name not in own_files)
         if named_elsewhere and not break_is_set("exclude-mentioned-test-targets"):
             continue
-        exclusive.update(os.fsencode(name) for name in files)
+        exclusive.update(files)
     return exclusive
 
 
+def files_crash_cases_do_not_read(root, listed_names, code_texts):
+    """listed_names（仓根起的 str）里不是测试文件、用例也读不到的（str 的集合）：判法见本节开头的 ② ③。"""
+    listed_set = set(listed_names)
+    left_out = {name for name in CRASH_CASE_FILES_NOT_READ
+                if name in listed_set and not any(os.path.basename(name) in text for text in code_texts.values())}
+    if any(BIN_EXECUTABLE_VARIABLE_PREFIX in text for text in code_texts.values()):
+        return left_out
+    for package_directory in package_directories_among(root, listed_names):
+        if package_keeps_every_test_file(root, package_directory, listed_set):
+            continue
+        bin_prefix = package_prefix(package_directory) + "src/bin/"
+        for name in listed_names:
+            if not name.startswith(bin_prefix):
+                continue
+            mention = "bin/" + name[len(bin_prefix):]
+            if not any(mention in text for other, text in code_texts.items() if not other.startswith(bin_prefix)):
+                left_out.add(name)
+    return left_out
+
+
 def crash_case_manifest(root, case, named_lines):
     """这条用例的输入清单：返回 (清单字节, 指纹, 文件数, 减去的文件 [bytes])。末尾在 named_lines 之后加这条用例的登记行（连第三列）。"""
     listed_files = listed_input_files(root, case.row.input_paths)
@@ -1200,7 +1426,11 @@
     package_directory = workspace_package_directories(root).get(case.package)
     if package_directory is None:
         raise RegistrationError(f"崩溃枚举用例 {case.key} 的包 {case.package} 不是仓根 Cargo.toml 的工作区成员")
-    excluded = files_exclusive_to_other_test_targets(root, listed_files, package_directory, case.target)
+    listed_names = [os.fsdecode(name) for name in listed_files]
+    code_texts = listed_code_texts(root, listed_names)
+    excluded_names = (files_exclusive_to_other_test_targets(root, listed_names, code_texts, package_directory, case.target)
+                      | files_crash_cases_do_not_read(root, listed_names, code_texts))
+    excluded = {os.fsencode(name) for name in excluded_names}
     kept = [name for name in listed_files if name not in excluded]
     registration = (f"<登记行：{case.key}>", case.row.text_with_conditions().encode("utf-8", "surrogateescape"))
     manifest, fingerprint, file_count = manifest_of_files(root, kept, list(named_lines) + [registration])
@@ -2325,6 +2555,42 @@
         os.remove(config_path)
         selftest.expect("弄坏开关 skip-build-environment 下「上一层的 .cargo/config.toml」那一格红（指纹不变）", broken_before == broken_after,
                         f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
+        # runner、wrapper 指的程序换了内容而值不变、配置文件里 build.rustc 的 -V 变了而配置原文不变：指纹都要变
+        program = os.path.join(tools, "selftest-runner")
+        write_executable(program, '#!/usr/bin/env bash\nexec "$@"\n')
+
+        def program_edit_fingerprints(changes=None):
+            before = fingerprint(changes)
+            write_executable(program, '#!/usr/bin/env bash\nexec "$@" --skip every_crash_state\n')
+            after = fingerprint(changes)
+            write_executable(program, '#!/usr/bin/env bash\nexec "$@"\n')
+            return before, after
+        for label, changes in [("环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER（值带参数）",
+                                {"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER": program + " --flag"}),
+                               ("环境变量 RUSTC_WRAPPER", {"RUSTC_WRAPPER": program}),
+                               ("环境变量 CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER（不带斜杠、在 PATH 里找）", {"CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER": "selftest-runner"})]:
+            before, after = program_edit_fingerprints(changes)
+            selftest.expect(f"构建环境：{label}指的程序换了内容、值不变，指纹变", before != after and not before.startswith("退"),
+                            f"换之前 {before[:16]}，换之后 {after[:16]}")
+        repository_configuration = os.path.join(work, ".cargo/config.toml")
+        for label, configuration_text in [
+                ("仓根 .cargo/config.toml 里 target.<三元组>.runner", f'[target.x86_64-unknown-linux-gnu]\nrunner = ["{program}", "--flag"]\n'),
+                ("仓根 .cargo/config.toml 里 build.rustc-wrapper（从 .cargo 所在的那一层起的相对路径）",
+                 f'[build]\nrustc-wrapper = "{os.path.relpath(program, work)}"\n')]:
+            write_text(repository_configuration, configuration_text)
+            before, after = program_edit_fingerprints()
+            os.remove(repository_configuration)
+            selftest.expect(f"构建环境：{label}指的程序换了内容、配置原文不变，指纹变", before != after and not before.startswith("退"),
+                            f"换之前 {before[:16]}，换之后 {after[:16]}")
+        write_text(repository_configuration, f'[build]\nrustc = "{switchable_rustc}"\n')
+        write_text(version_file, "rustc 9.9.9-c\n")
+        with_version_c = fingerprint()
+        write_text(version_file, "rustc 9.9.9-d\n")
+        with_version_d = fingerprint()
+        os.remove(repository_configuration)
+        selftest.expect("构建环境：仓根 .cargo/config.toml 里 build.rustc 指的编译器 -V 变了、配置原文不变，指纹变",
+                        with_version_c != with_version_d and not with_version_c.startswith("退"), f"报 c {with_version_c[:16]}，报 d {with_version_d[:16]}")
+        selftest.expect("构建环境：配置文件与环境变量都拿掉之后，指纹回到原样", fingerprint() == baseline, f"原样 {baseline[:16]}")
 
         def experiment_fingerprint_line(changes=None):
             _exit_code, output, _messages = run_in_environment([sys.executable, module, "experiment", work, "E900"],
@@ -2469,6 +2735,90 @@
                             f"退 {exit_code}，stdout「{output.strip()}」")
         write_text(table_path, own_row + "\n")
 
+        # 登记的用例函数要标 #[ignore]：没标的、只有 cfg_attr 里条件 ignore 的、ignore 挂在前一个函数上的、只写在注释里的，crash-cases 退 2
+        own_case_path = os.path.join(work, "crates/pkg/tests/own_case.rs")
+        with open(own_case_path, encoding="utf-8") as handle:
+            own_case_text = handle.read()
+        modules = "mod common;\nmod helper_target;\n"
+        ignore_cells = [
+            ("用例函数没标 #[ignore]", modules + "#[test]\nfn the_case() {}\n", False),
+            ("只有 cfg_attr 里的条件 ignore", modules + "#[test]\n#[cfg_attr(miri, ignore)]\nfn the_case() {}\n", False),
+            ("#[ignore] 挂在前一个函数上", modules + "#[test]\n#[ignore]\nfn other() {}\n#[test]\nfn the_case() {}\n", False),
+            ("#[ignore] 只写在注释里", modules + "#[test]\n// #[ignore]\nfn the_case() {}\n", False),
+            ("#[ignore] 在 #[test] 前面、fn 前面有 pub", modules + "#[ignore]\n#[test]\npub fn the_case() {}\n", True),
+            ("ignore 的理由里带方括号、引号与 //", modules + '#[test]\n#[ignore = "样本 [x] \\"y\\" // z"]\nfn the_case() {}\n', True),
+        ]
+        for label, source_text, marked in ignore_cells:
+            write_text(own_case_path, source_text)
+            exit_code, output, _messages = admission("crash-cases", work)
+            green = exit_code == 0 if marked else (exit_code == EXIT_REGISTRATION_ERROR and "没标 #[ignore]" in output)
+            selftest.expect(f"crash-cases 自查：{label} ⇒ {'退 0' if marked else '退 2 并说没标 #[ignore]'}", green,
+                            f"退 {exit_code}，stdout「{output.strip()}」")
+        write_text(own_case_path, own_case_text)
+
+        # 认得出形状、认不出读的是哪一份的按宽处理：包里 include!(concat!(…))、任何一个包的构建脚本出现整词 tests ⇒ 改 other_target.rs 指纹变
+        other_target_path = os.path.join(work, "crates/pkg/tests/other_target.rs")
+        root_manifest_path = os.path.join(work, "Cargo.toml")
+        with open(root_manifest_path, encoding="utf-8") as handle:
+            root_manifest_text = handle.read()
+        two_members = '[workspace]\nmembers = ["crates/pkg", "crates/gen"]\n'
+        gen_package = '[package]\nname = "gen"\nversion = "0.0.0"\n'
+        reads_tests = 'fn main() { let _ = std::fs::read_dir("../pkg/tests"); }\n'
+        wide_cells = [
+            ("用例里 include!(concat!(\"other_\", \"target.rs\"))", {"crates/pkg/tests/own_case.rs": own_case_text + 'include!(concat!("other_", "target.rs"));\n'},
+             True),
+            ("别的包的 build.rs 按目录读 ../pkg/tests", {"Cargo.toml": two_members, "crates/gen/Cargo.toml": gen_package, "crates/gen/src/lib.rs": "",
+                                                    "crates/gen/build.rs": reads_tests}, True),
+            ("别的包 package.build 指的构建脚本按目录读 tests", {"Cargo.toml": two_members, "crates/gen/Cargo.toml": gen_package + 'build = "generate.rs"\n',
+                                                          "crates/gen/src/lib.rs": "", "crates/gen/generate.rs": reads_tests}, True),
+            ("对照：别的包的 build.rs 不读 tests", {"Cargo.toml": two_members, "crates/gen/Cargo.toml": gen_package, "crates/gen/src/lib.rs": "",
+                                               "crates/gen/build.rs": "fn main() {}\n"}, False),
+        ]
+        for label, files, should_move in wide_cells:
+            for relative, content in files.items():
+                write_text(os.path.join(work, relative), content)
+            before, _count = fingerprint()
+            write_text(other_target_path, "#[test]\nfn other() { assert!(true); }\n")
+            after, _count = fingerprint()
+            write_text(other_target_path, "#[test]\nfn other() {}\n")
+            write_text(own_case_path, own_case_text)
+            write_text(root_manifest_path, root_manifest_text)
+            shutil.rmtree(os.path.join(work, "crates/gen"), ignore_errors=True)
+            selftest.expect(f"崩溃枚举用例的输入：{label}，改 other_target.rs 指纹{'变' if should_move else '不变'}",
+                            (before != after) == should_move and not before.startswith("退"), f"改之前 {before[:16]}，改之后 {after[:16]}")
+
+        # 用例读不到的非测试文件：crates/mutations.tsv 与（没有代码读 CARGO_BIN_EXE_ 时）src/bin/ 下的，改了指纹不变；有代码点名它们时照留
+        lib_path = os.path.join(work, "crates/pkg/src/lib.rs")
+        with open(lib_path, encoding="utf-8") as handle:
+            lib_text = handle.read()
+        table_versions = ("# 样本变异表\n", "# 样本变异表\n多一行\n")
+        tool_versions = ("fn main() {}\n", "fn main() { let changed = 1; }\n")
+        not_read_cells = [
+            ("crates/mutations.tsv 加一行（没有代码点名它）", {}, "crates/mutations.tsv", table_versions, False),
+            ("lib.rs 的字符串里点名 mutations.tsv 时，crates/mutations.tsv 加一行", {"crates/pkg/src/lib.rs": lib_text + 'pub const TABLE: &str = "../mutations.tsv";\n'},
+             "crates/mutations.tsv", table_versions, True),
+            ("src/bin/tool.rs 改了（没有代码读 CARGO_BIN_EXE_）", {}, "crates/pkg/src/bin/tool.rs", tool_versions, False),
+            ("用例读 CARGO_BIN_EXE_tool 时，src/bin/tool.rs 改了", {"crates/pkg/tests/own_case.rs": own_case_text + 'const TOOL: &str = env!("CARGO_BIN_EXE_tool");\n'},
+             "crates/pkg/src/bin/tool.rs", tool_versions, True),
+            ("lib.rs 用 #[path = \"bin/tool.rs\"] 把它编进来时，src/bin/tool.rs 改了", {"crates/pkg/src/lib.rs": lib_text + '#[path = "bin/tool.rs"]\nmod tool;\n'},
+             "crates/pkg/src/bin/tool.rs", tool_versions, True),
+        ]
+        for label, setup, relative, (first_text, second_text), should_move in not_read_cells:
+            for setup_relative, content in setup.items():
+                write_text(os.path.join(work, setup_relative), content)
+            write_text(os.path.join(work, relative), first_text)
+            before, _count = fingerprint()
+            write_text(os.path.join(work, relative), second_text)
+            after, _count = fingerprint()
+            os.remove(os.path.join(work, relative))
+            write_text(own_case_path, own_case_text)
+            write_text(lib_path, lib_text)
+            selftest.expect(f"崩溃枚举用例的输入：{label}，指纹{'变' if should_move else '不变'}", (before != after) == should_move and not before.startswith("退"),
+                            f"改之前 {before[:16]}，改之后 {after[:16]}")
+        restored_after_wide_cells, _count = fingerprint()
+        selftest.expect("崩溃枚举用例的输入：按宽处理与读不到的那几格改回之后，指纹回到原样", restored_after_wide_cells == baseline,
+                        f"原样 {baseline[:16]}，改回之后 {restored_after_wide_cells[:16]}")
+
         # 全绿标记：写了再核作数；换一个指纹不作数且比出不同；标记被改（指纹、exhaustive）不作数
         judged_file = os.path.join(work, "judged.out")
         write_text(judged_file, "test_result=test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.00s\n"
@@ -2599,7 +2949,8 @@
 def run_layer0_stage_cells(selftest, module):
     """门禁 54 号这一份的流程：拷进临时仓的 .claude/stage-under-test/（不在 .claude/gate.d/ 下，名字照旧是 54-layer0-replay.sh），
     假 cargo、rustc、nproc 在 PATH 最前面。核：逐条跑、续跑的三个环境变量、判绿写标记、快档核标记、第二趟全复用、只重跑输入变了的那一条、
-    --start-over、只剩 1 片要跑不误红、1 个线程跑了 64 片判红、0 passed 判红、cargo 退非 0 判红而别的照跑、跑的过程中输入变了不写标记。"""
+    --start-over、只剩 1 片要跑不误红、1 个线程跑了 64 片判红、0 passed 判红、cargo 退非 0 判红而别的照跑、跑的过程中输入变了不写标记、
+    快档不带 SINGLEFS_GATE_FULL=1 时只改登记表、只改 54 号、只改准入模块照样核标记（不退 77）。"""
     repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
     real_stage = os.path.join(repository_root, ".claude/gate.d/54-layer0-replay.sh")
     work = tempfile.mkdtemp(prefix="admission-selftest-stage54-")
@@ -2662,6 +3013,7 @@
             _exit_code, output, _messages = run_in_environment(
                 [sys.executable, os.path.join(work, "research/scripts/admission.py"), "crash-case-manifest", work, key,
                  os.path.join(control, "expected-manifest"), "--extra-file", "<判它的 54 号：54-layer0-replay.sh>", stage_copy,
+                 "--extra-file", "<判它的准入模块：admission.py>", os.path.join(work, "research/scripts/admission.py"),
                  "--toolchain", "--build-environment"], environment)
             return output.split()[0] if output.split() else ""
 
@@ -2757,6 +3109,37 @@
         selftest.expect("54 号 --full：跑的过程中共用的 src/lib.rs 被改了 ⇒ 那一条判红、不写标记，出路里列出变了的文件",
                         exit_code == 1 and "跑的过程中它的输入变了" in output and "crates/singlefs-harness/src/lib.rs" in output
                         and not any(".stream-a." in name for name in markers()), f"退 {exit_code}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")
+
+        # 范围那一问：只改登记表、只改 54 号、只改准入模块的改动，快档不带 SINGLEFS_GATE_FULL=1 照样核标记（不退 77）
+        set_logs()
+        stage("--full")
+        run_quietly(["git", "-C", work, "add", "-A"])
+        run_quietly(["git", "-C", work, "commit", "-q", "-m", "样本"], GIT_IDENTITY)
+        quick_environment = {name: value for name, value in environment.items() if name != "SINGLEFS_GATE_FULL"}
+
+        def quick_tier_without_forcing():
+            exit_code, output, messages = run_in_environment(["bash", stage_copy, work], quick_environment)
+            invocations()
+            return exit_code, output + messages
+        exit_code, output = quick_tier_without_forcing()
+        selftest.expect("54 号快档（不带 SINGLEFS_GATE_FULL=1）：提交之后一个改动都没有，范围那一问答「没碰」，退 77", exit_code == 77,
+                        f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
+        every_case = [key for key, _target, _function, _conditions in LAYER0_STAGE_CASES]
+        scope_cells = [
+            ("只改登记表（stream-b 那一行第三列去掉 threads=LAYER0B）", os.path.join(work, REGISTRATION_TABLE),
+             lambda text: text.replace(" threads=LAYER0B", "", 1), ["crash-case:stream-b"]),
+            ("只改 54 号（加一行注释）", stage_copy, lambda text: text + "# 样本：只改 54 号\n", every_case),
+            ("只改准入模块（加一行注释）", os.path.join(work, "research/scripts/admission.py"), lambda text: text + "# 样本：只改准入模块\n", every_case),
+        ]
+        for label, path, change, named_cases in scope_cells:
+            with open(path, encoding="utf-8") as handle:
+                original_text = handle.read()
+            write_text(path, change(original_text))
+            exit_code, output = quick_tier_without_forcing()
+            write_text(path, original_text)
+            selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记，判红并点名 {'、'.join(named_cases)}，不退 77",
+                            exit_code == 1 and "没有作数的全绿标记" in output and all(key in output for key in named_cases),
+                            f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
     except (OSError, shutil.Error) as error:
         selftest.expect("54 号这一份拷得进临时仓、跑得起来", False, f"{error}（真仓的 54 号在 {real_stage}）")
     finally:
```
