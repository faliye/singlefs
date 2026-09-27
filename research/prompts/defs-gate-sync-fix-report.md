# gate-sync-fix 报告（tooling-writer，2026-09-27，时刻均 UTC）

## 结论

- 第 3 件（heavy-test-guard 自检一格）：现行定案是「层 0 归崩溃验证员」，按这个把自检改好了。`heavy-test-guard.sh --selftest` 从 exit=1 变成 exit=0。63 号、hooks-registered、`lib_heavy_tests.py --selftest` 全绿。
- `lib_heavy_tests.py` 没改：远端根本没动过它，里面也没有「谁能跑什么」的判法（详见下文）。
- 第 4 件（54 号第 270 行）：同一行里另外三处 `$layer0_full_base/tree` 补上 `${…:?}` 守卫。shell-lint 对 54 号转绿；54 号自己的自证（`admission.py --selftest`）绿。
- 两处改动已经被另一个会话提交进 `ecdf8465`（00:32:09 UTC，提交说明里自己写着「heavy-test-guard 自检一格与 54 号 rm -rf 守卫还在改，这里提交的是提交时工作区的样子」）。所以这三份文件对 HEAD 的 `git diff --stat` 现在是空的。
- 推翻条件：如果主 agent 或用户定「层 0 全量由主 agent 跑、不归崩溃验证员」（也就是远端那一侧），那么这一格的期望要改回 2，放行表里还要去掉 layer0-*。

## 第 3 件：判定依据

- 冲突块在 `records/2026-09-27-同步远端冲突块.md` 第 564–680 行（heavy-test-guard.sh 三块，本地留下的一侧是「crash-verifier：层 0、崩溃枚举用例、55 号…」）。另外第 131–180 行是 crash-verifier.md 三块，本地一侧同样是「54 号默认带 --full」。
- 现行各处都说层 0 全量归崩溃验证员：
  - `.claude/agents/crash-verifier.md` 第 3、27–31 行
  - `.claude/gate.d/stage-owners.tsv` 第 37 行（`54-layer0-replay.sh	crash-verifier	…`）
  - `.claude/main-agent.md` 第 57、61 行
  - `.claude/rules/implementation-workflow.md` 第 64、67 行（第 67 行：「崩溃验证员在 HEAD + 暂存区的 worktree 里跑 54 号 `--full`（连同登记的崩溃枚举用例），另跑 55、57、59 号」）
- 提醒：开工时（约 00:2x UTC）读到的这张表第 55 行还是远端那句「层 0 全量由主 agent 在 HEAD + 暂存区的 worktree 里跑；崩溃验证员跑 55、57、59 号」。之后另一个会话在 `faf255e2` 里改成了现在的样子。
- 自检那一格（第 863–864 行）是远端改的，自动合并进来时不在冲突块里，所以还留着「：层 0 不归它」和期望 2。放行表（第 147 行）用的是本地一侧，里面有 layer0-stage，于是实际判 0，这一格就红了。

## 第 3 件：改法

`.claude/hooks/heavy-test-guard.sh` 第 863–865 行改成下面这样：

```
            ("崩溃验证员带前缀不经内存包装跑层 0 测试目标", crash, commit + "cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 2),
            ("崩溃验证员带前缀经内存包装跑 54 号全量", crash,
             commit + "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 0),
```

- 第二、三行：期望改成 0，去掉「：层 0 不归它」。
- 第一行：这一格在 73ba4a4c 里就有（那里的第 767 行），被远端换成了 step_six 那一格，合并时一起丢了。按本地定案它仍然成立（崩溃验证员跑层 0 也要经内存包装），所以补了回来。远端加的 step_six 那一格留着。

## 第 3 件：判红与转绿

- 改之前（`bash .claude/hooks/heavy-test-guard.sh --selftest`，exit=1）：
  `  ✗ 自检：崩溃验证员带前缀经内存包装跑 54 号全量：层 0 不归它 应当是 2，实际 0`
- 改之后（exit=0）：
  `  ✓ 自检通过（查了 661 种，其中该拒 122 种）：…`
- 弄坏的对照：没加新开关。在草稿副本 `/tmp/claude-1000/gate-sync-fix/mut/` 里把放行表换成远端那一版（去掉 layer0-stage、layer0-cargo、layer0-binary），自检 exit=1，红了下面四格（其中最后一格就是新改的那一格）：
  ```
  ✗ 自检：崩溃验证员带前缀跑 54 号全量 应当是 0，实际 2
  ✗ 自检：崩溃验证员带前缀跑层 0 测试目标 应当是 0，实际 2
  ✗ 自检：崩溃验证员带前缀直接执行层 0 测试二进制 应当是 0，实际 2
  ✗ 自检：崩溃验证员带前缀经内存包装跑 54 号全量 应当是 0，实际 2
  ```
- `python3 .claude/hooks/lib_heavy_tests.py --selftest`，exit=0：`  ✓ lib_heavy_tests 自检通过（查了 141 种：…）`

## 远端那 745 行与本地判法的冲突：一处都没有

- `git diff 73ba4a4c ac927812 -- .claude/hooks/lib_heavy_tests.py` 输出为 0 行，`git diff ac927812 260fa60a -- …` 也是空的。也就是说远端在 73ba4a4 到 ac927812 之间没改过这份文件。
- 那 745 行是 HEAD 与工作区之间的差异（`git diff HEAD --stat` 当时是 `745 +++++…`），属于叠回来的本地改动，现在已经进了 ecdf8465。
- 查了一遍：`grep -n 'crash-verifier\|崩溃验证员\|gate-triage\|主 agent\|不归'` 在 lib_heavy_tests.py 里没有命中。「谁能跑什么」只写在 hook 第 147 行附近的放行表里，lib 只判「是不是重型、属于哪一类」。所以 lib 这边不用对齐什么。

## 第 4 件：改法与判红转绿

- 原因：shell-lint 的 S5 是按行判的（`.claude/singlefs-ai-sop/scripts/shell-lint.sh` 第 89–90、166–172 行）。只要一行里有 `rm -r…`，同一行里任何 `$变量/` 都算没守卫。第 270 行的 `rm -rf "${layer0_full_base:?}"` 本身已经带守卫，红的是同一行里另外三处 `"$layer0_full_base/tree…"`（`bash` 的脚本路径、`--full` 的参数、`git worktree remove`）。
- 改法：这三处都写成 `"${layer0_full_base:?}/tree…"`。
- 判红（改之前，`SHELL_LINT_DIR=.claude/gate.d bash …/shell-lint.sh .`）：
  `  ✗ .claude/gate.d/54-layer0-replay.sh:270  rm -rf 作用在变量路径上（$layer0_full_base/），变量为空时会从根目录往下删`
- 对照样本在 `/tmp/claude-1000/gate-sync-fix/lint54/`，同一份脚本只有这三处不同：
  - 把守卫还原掉：`  ✗ 54-layer0-replay.sh:270  rm -rf 作用在变量路径上（$layer0_full_base/）…`，`✗ shell 纪律检查失败：1 处（共检查 1 个脚本）`
  - 带守卫的现版：`  ✓ shell 纪律检查通过（共 1 个脚本）`
- 转绿（`SHELL_LINT_DIR=.claude/gate.d bash .claude/singlefs-ai-sop/scripts/shell-lint.sh`，exit=0）：`  ✓ shell 纪律检查通过（共 76 个脚本）`
- 出路里打印的三行命令：`bash -n` 退 0。空变量时 `git worktree remove --force "${layer0_full_base:?}/tree"` 会报 `layer0_full_base: parameter null or not set` 并中止。
- 54 号自证（文件头第 55–57 行：`admission.py --selftest` 的「54 号」那几格），改前改后都是 exit=0：`  ✓ admission.py 自证通过：232 格都对（…）`

## 改过的文件

- `.claude/hooks/heavy-test-guard.sh`：第 863–865 行
- `.claude/gate.d/54-layer0-replay.sh`：第 270 行

`git diff --stat -- .claude/hooks/heavy-test-guard.sh .claude/hooks/lib_heavy_tests.py .claude/gate.d/54-layer0-replay.sh` 的输出现在为空：两处改动都已经在 ecdf8465 里。确认过的点：
- `git show ecdf8465 -- .claude/hooks/heavy-test-guard.sh` 里有上面那三行 `+`。
- `.claude/gate.d/54-layer0-replay.sh` 里 `grep -cF '${layer0_full_base:?}/tree'` 数到 1 行。

没有新写 `# gate-similar:` / `# hook-events:` 行（没建新门禁或新钩子）。

## 第 7 步收尾（00:3x UTC 串行跑的，日志在 `/tmp/claude-1000/gate-sync-fix/closing/`）

| 项 | 退出码 | 末行 |
|---|---|---|
| 47 号 | 1 | `→ 怎么办：按上面那份自证给的下一步修被测脚本…` |
| 62 号 | 0 | `✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）` |
| 63 号 | 0 | `✓ 写范围闸、Bash 检出 hook、重型测试闸…注册着、自证通过…` |
| 73 号 | 0 | `没扫的目录 0 个（不在）：（没有）` |
| doc-lint | 1 | `✗ 文档铁律检查失败：1 个文件违规、0 处编号引用无定义、11 处编号定义/引用不合规…（检查 518，跳过 0）` |
| rules-lint（项目本地） | 0 | `✓ 规则只写怎么做（扫了 29 份文件 1965 行；…命中 0…）` |
| gate-lint（GATE_LINT_DIR=.claude/gate.d） | 0 | `✓ 门禁自检通过：85 个脚本（.sh 与 .py）、310 条拒绝都带了出路` |
| shell-lint（SHELL_LINT_DIR=.claude/gate.d） | 0 | `✓ shell 纪律检查通过（共 76 个脚本）` |
| preflight-lint | 0 | `✓ 准入与运行条件：判了 138 个脚本…` |
| gate-overlap | 0 | `没判的：装进来的 SOP 副本 37 份只当对照` |
| heavy-test-guard --selftest | 0 | `✓ 自检通过（查了 661 种，其中该拒 122 种）…` |
| lib_heavy_tests --selftest | 0 | `✓ lib_heavy_tests 自检通过（查了 141 种…）` |
| hooks-registered | 0 | `✓ 12 个钩子都注册着，其中 11 个的自检通过` |

阶段归属表里没有登记给 tooling-writer 的阶段（awk 查出 0 行）。

红的几项点名的文件都不在这一轮的改动里，照规矩没修：
- 47 号：点名 `research/scripts/ask-local-selftest.sh`（检测器崩了、找不到时应退 6，实际退 2）和 `research/scripts/check-segment-registry.py --selftest`（段序列登记表与 E142 产物对不上 2 处）。
- doc-lint：点名 `records/2026-09-24-里程碑二收尾调度.md:232`、编号 O1–O3、F23、F24、M32、M34、U11、U13、`.claude/kb/experiments-history.md`、`.claude/kb/experiments/158-择根与修复四岔路.md`。
- `shell-lint.sh .` 这种加了 `.` 扫全仓的写法还剩 59 处红，都在 `.claude/gate.d/fixtures/`（样本本来就该红）和 `research/prompts/*-opus-model/`，54 号不在其中了。

## 另外看到的（不归这一轮）

- 开工时的负载：`ps` 看到另一个会话在跑 `cargo build --release --offline --bin e158_root_choice_repair`（pid 3958266）。没有 qemu、fio、vm-bench。我没编译，也没等锁。
- `gate-overlap.py --list` 是在两处改完之后才跑的，没按顺序在动手前跑。这两处改的都是已有自检格与已有 lint 判据的对象，没有新增判据。
- hooks-registered 报「12 个钩子，其中 11 个的自检通过」，但 exit=0，没有 ✗ 行。第 12 个钩子是哪个、为什么没有自检，没查。

## 没做什么

- 没跑重型测试：54、55、57、59、87 号本身（54 号 `--full` 也没跑）、`gate.sh`、全量 `cargo test`、`check.sh`。
- 没走定义三方；这一轮也不是第二种活，没改任何定义或规则。
- 没提交。两处改动是被别的会话连同它那一批提交进 ecdf8465 的。
- 没改 `lib_heavy_tests.py`：不需要改，理由见上文。
- 没修 47 号、doc-lint 的红，它们不在这一轮的文件里。
- 草稿目录 `/tmp/claude-1000/gate-sync-fix/` 下留着 `mut/`（仓的一部分拷贝，用来做弄坏对照）、`lint54/`、`closing/` 和各份日志，给主 agent 核对用；里面没有编译目录，也没有 worktree。
- 更正上一条：交回前已删掉 `/tmp/claude-1000/gate-sync-fix/mut/`（仓的一部分拷贝），弄坏对照的输出保存在 `/tmp/claude-1000/gate-sync-fix/mut.log`。
