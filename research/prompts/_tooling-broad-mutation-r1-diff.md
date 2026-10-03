# tooling-broad-mutation-r1：被判的改动

## 已入库文件的 diff（`git diff HEAD -- .claude/hooks .claude/gate.d/code-tooling.sh`）

```diff
diff --git a/.claude/gate.d/code-tooling.sh b/.claude/gate.d/code-tooling.sh
index 845f28ff..526d7cba 100755
--- a/.claude/gate.d/code-tooling.sh
+++ b/.claude/gate.d/code-tooling.sh
@@ -251,6 +251,7 @@ SELFTEST_RUNNERS=(
   "python3 research/scripts/migrate-changelog-format.py --selftest"
   "bash research/scripts/crash-amplification.sh --selftest"
   "bash research/scripts/crash-amplification-item.sh --selftest"
+  "python3 research/scripts/broad-mutation.py --selftest"
 )
 SELFTEST_RUNNERS_STARTED_ALONE=("bash research/scripts/layer0-shard-run.sh --selftest" "python3 research/scripts/agent-watch.py --selftest")
 
diff --git a/.claude/hooks/heavy-test-guard.sh b/.claude/hooks/heavy-test-guard.sh
index 82cdfd45..2247ac94 100755
--- a/.claude/hooks/heavy-test-guard.sh
+++ b/.claude/hooks/heavy-test-guard.sh
@@ -39,6 +39,8 @@
 #   全部实验复跑  .claude/gate.d/checker-tier-research-build-and-replay.sh，只跑它的单测那一格（--check research-unit-tests）的除外：那一格落到下一类
 #   整轮门禁      gate.sh、research/scripts/gate-staged.sh
 #   E152 装置     e152-file-system-benchmark（直接起、或 cargo run 它）、research/scripts/e152-run.sh
+#   广谱变异      research/scripts/broad-mutation.py（按名字与参数认，python 脚本不读正文）；带 --sync-tree / --list-items / --summary 的不起测试、不算，
+#                 「--item 点名又带 --skip-stage-two」的每条只起 harness 档四个包的测试、不算；只有主 agent 带前缀能跑
 #   门禁阶段按文件名认，不按编号认；带 --list、--list-items（只打格名表、逐项点名的清单）的哪一道都不算；带 --write（生成）的不算。
 # 提交时才跑的检查（不是重型，放行照同一张表；用户 2026-09-28 定「现在改到提交时候才验证的部分 都删掉」「门禁是靠 脚本和 hook来解决的 … 提示不执行 根本不靠谱」）：
 #   上面几类之外的 .claude/gate.d/ 阶段；.claude/singlefs-ai-sop/scripts/ 下的 doc-lint.sh、gate-lint.sh、shell-lint.sh、preflight-lint.py、gate-overlap.py
diff --git a/.claude/hooks/lib_heavy_tests.py b/.claude/hooks/lib_heavy_tests.py
index 43d24db8..cbc49744 100644
--- a/.claude/hooks/lib_heavy_tests.py
+++ b/.claude/hooks/lib_heavy_tests.py
@@ -69,7 +69,11 @@ cargo mutants -d 指到别处的树。
   EnvironmentFile= 设的变量不带进里面那条命令）、list-past-separator（找 --list 时见到 `--` 不停）、configuration-before-subcommand-only（只收子命令之前的 --config）、mutants-configuration-ignored（cargo mutants 不看 --test-package、--test-workspace 与 .cargo/mutants.toml 的 test_workspace）、no-run-counted-as-running（`--no-run` 只编译也按跑了算）、
   shard-driver-by-name（双机分片的驱动脚本只按名字判、不读树根里的登记行，驱动自检起的那几趟也报）、
   stage-by-number（门禁阶段照旧按两位数编号认，去了编号的重型那几道认不出）、commit-checks-ignored（提交时才跑的检查这一类一样都不认）、
-  replay-cells-ignored（research 构建与复跑那一道不分格，只跑单测那一格也算全部实验复跑）。
+  replay-cells-ignored（research 构建与复跑那一道不分格，只跑单测那一格也算全部实验复跑）、
+  broad-mutation-ignored（广谱变异的逐条驱动不按名字认，整趟跑也放过去）。
+广谱变异的逐条驱动 research/scripts/broad-mutation.py 按名字认（类名「广谱变异」，kind broad-mutation）：它逐条改坏 core / checker 的源码，第二段起 cargo test 与
+checker 档快档，整趟要几天；带 --sync-tree、--list-items、--summary、--help 之一的不起测试、不算；「--item 点名又带 --skip-stage-two」的每条只起 harness 档
+四个包的测试、不算；--selftest 照旧是提交时才跑的检查；其余一律算。它是 python 脚本，闸不读它的正文，只按这里的名字与参数判。
 切词那一份的弄坏开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed（lib_shell_words.py：`python3 -` 后面的词、`bash -n` 的目标当被执行的脚本）下这里的自检同样必须判红。
 """
 import fnmatch, functools, glob, importlib.util, os, re, shlex, shutil, sys, tempfile, tomllib
@@ -102,6 +106,7 @@ KIND_CATEGORY = {
     "full-cargo": "全量测试", "check-sh": "全量测试",
     "gate-sh": "整轮门禁", "gate-staged": "整轮门禁", "replay-all-stage": "全部实验复跑",
     "e152": "E152 装置",
+    "broad-mutation": "广谱变异",
 }
 # 提交时才跑的检查（用户 2026-09-28 定「现在改到提交时候才验证的部分 都删掉」「门禁是靠 脚本和 hook来解决的」）：
 # 扫仓的门禁阶段、上游的几道 lint、样本自检与阶段结构判法、研究脚本与钩子的 --selftest，都归提交前 gate-triage 跑的 research/scripts/gate-staged.sh
@@ -133,7 +138,10 @@ KNOWN_SCRIPT_LOCATIONS = {
     "vm-bench.sh": "research/scripts/vm-bench.sh", "e152-run.sh": "research/scripts/e152-run.sh",
     "capped.sh": "research/scripts/capped.sh", "run-with-memory-cap.sh": "research/scripts/run-with-memory-cap.sh",
     "layer0-shard-run.sh": "research/scripts/layer0-shard-run.sh", "mutation-shard-run.sh": "research/scripts/mutation-shard-run.sh",
+    "broad-mutation.py": "research/scripts/broad-mutation.py",
 }
+# 广谱变异的逐条驱动（research/scripts/broad-mutation.py）：带这几个参数之一的不起测试（导出树、列项、汇总）
+BROAD_MUTATION_QUERY_ARGUMENTS = frozenset({"--sync-tree", "--list-items", "--summary", "--help", "-h"})
 
 
 def heavy_test(kind, detail):
@@ -925,6 +933,13 @@ def classify(words, directory, environment=None):
     if name == "mutation-shard-run.sh":
         return None if "--selftest" in arguments else heavy_test(
             "crates-mutation-stage", "research/scripts/mutation-shard-run.sh（门禁 checker-tier-crates-mutation-replay.sh 的双机分片驱动：两台各跑一趟那一道、判 crates/mutations.tsv 的一部分）")
+    if name == "broad-mutation.py" and located_in(words[0], directory, "research/scripts") and not break_is_set("broad-mutation-ignored"):
+        if BROAD_MUTATION_QUERY_ARGUMENTS & set(arguments):
+            return None
+        if "--item" in arguments and "--skip-stage-two" in arguments:
+            return None   # 点名的几条、只跑第一段：每条只起 harness 档四个包的测试，不碰 checker 档
+        return heavy_test("broad-mutation", "research/scripts/broad-mutation.py（逐条改坏 core / checker 的源码，每条跑 harness 档四个包的测试，"
+                                            "没红的再跑 cargo test 与 checker 档快档；只有「--item 点名又带 --skip-stage-two」的不算）")
     if name in ("e152-file-system-benchmark", "e152-run.sh"):
         return heavy_test("e152", name)
     if name == "cargo":
@@ -1114,6 +1129,17 @@ def selftest():
              "crates-mutation-stage"),
             ("checker-tier-crates-mutation-replay 的双机分片驱动 --selftest 不重型，是提交时才跑的检查", ["bash", "research/scripts/mutation-shard-run.sh", "--selftest"], work, "commit-check-selftest"),
             ("cargo run E152", ["cargo", "run", "--release", "--bin", "e152-file-system-benchmark"], os.path.join(work, "research"), "e152"),
+            # 广谱变异的逐条驱动按名字与参数认（开关 broad-mutation-ignored 下算的那三格红）
+            ("广谱变异整趟跑", ["python3", "research/scripts/broad-mutation.py"], work, "broad-mutation"),
+            ("广谱变异经 nice 与绝对路径、按分片跑", ["nice", "-n", "19", "python3", os.path.join(work, "research", "scripts", "broad-mutation.py"), "--shard", "0/2"], "/",
+             "broad-mutation"),
+            ("广谱变异点名但两段都跑", ["python3", "research/scripts/broad-mutation.py", "--item", "12"], work, "broad-mutation"),
+            ("广谱变异点名、只跑第一段：只起 harness 档的测试", ["python3", "research/scripts/broad-mutation.py", "--item", "12", "--skip-stage-two"], work, None),
+            ("广谱变异 --list-items 只列项", ["python3", "research/scripts/broad-mutation.py", "--list-items"], work, None),
+            ("广谱变异 --summary 只看汇总", ["python3", "research/scripts/broad-mutation.py", "--summary"], work, None),
+            ("广谱变异 --sync-tree 只导出树", ["python3", "research/scripts/broad-mutation.py", "--sync-tree"], work, None),
+            ("广谱变异 --selftest 是提交时才跑的检查", ["python3", "research/scripts/broad-mutation.py", "--selftest"], work, "commit-check-selftest"),
+            ("仓外同名的 broad-mutation.py 不按名字判", ["python3", "/tmp/claude-1000/elsewhere/broad-mutation.py"], work, None),
             # 门禁阶段按文件名认（开关 stage-by-number 下去了编号的那四格红）；不重型的阶段是提交时才跑的检查（开关 commit-checks-ignored 下这一类各格红）
             ("bash 起轻阶段 doc-text：提交时才跑的检查", ["bash", ".claude/gate.d/doc-text.sh"], work, "commit-check-stage"),
             ("去了编号的轻阶段 --check 一格：提交时才跑的检查", ["bash", ".claude/gate.d/code-tooling.sh", "--check", "agent-write-scope"], work, "commit-check-stage"),
```

## 新文件

`research/scripts/broad-mutation.py`（1343 行，未入库）：腿直接读仓里这份文件，sha256 见快照。
