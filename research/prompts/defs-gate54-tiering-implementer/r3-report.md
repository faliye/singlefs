# 门禁 54 号第三轮：换成「开跑不删、没写成标记才删」，外加 README 与规则第 65 行

依据：协调方消息「第 1 条采纳你的变体」。第二轮报告在 `research/prompts/defs-gate54-tiering-implementer/r2-report.md`。

## 结果

真文件 `/home/fy5090/code/singlefs/.claude/gate.d/54-layer0-replay.sh`：397 行，sha256 `232c8c0de11bf6561f9ca1736652745f387d05832ad0a5c99a08bc80c0386bf7`。
- 改前是第二轮那一份 `c3f9360f…`：动手前现核过；备份在 `r3/54-layer0-replay.sh.r2`，本轮 diff 在 `r3/r3.diff`（+9 / −5）。
- 5 处都用 `replace-once.py` 改，每处都是「命中 1 次，已替换并回读确认」；旧/新文本对在 `r3/edits/`。
- 先改草稿副本 `r3/54-layer0-replay.sh`、跑完全部场景，再套到真文件上；套完 `cmp` 报「真文件与 r3 自证用的草稿副本逐字节相同」。

| 编号 | 改了什么 |
|---|---|
| K1 | 文件头「--full」那一段：开跑一格都不删（同一批输入结果确定，前一趟那一格在这一趟跑的过程中照样算数）。这一趟没写成标记就退出（判红、跑的过程中输入变了、被 TERM / INT / HUP 打断）时，退出前删这批输入那一格；被 SIGKILL 杀掉来不及删，前一趟那一格留着。别的格不动 |
| K2 | 文件头自证清单加「同一批输入两趟 --full 撞车不误红、同一批输入先绿后红删掉那一格」 |
| K3 | --full 开跑那一段的小标题 |
| K4 | 删掉开跑那一句 `rm -f -- "${full_green_marker_path:?}"`，换成 `full_marker_written=0` 和一个 EXIT trap：`full_marker_written` 不是 1 时删 `${full_green_marker_path:?}`，并照旧清临时目录。开跑那一句提示改成「不删任何一格，这一趟判红才删这批输入那一格」 |
| K5 | 写成标记之后、报「common-dir 里现有 N 格」之前，置 `full_marker_written=1` |

第 2 条（旧格只增不减）照你说的不改，「common-dir 里现有 N 格」那一句保留。

README 与规则，都用 `replace-once.py`，一处一改，每处命中 1 次。改前副本和 diff 在 `r3/README.md.before`、`r3/README.diff`、`r3/implementation-workflow.md.before`、`r3/implementation-workflow.diff`。
- `README.md` 第 108 行（别的会话已经把原来的第 107 行拆成快档第 107 行与 --full 第 108 行）：改成 `bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>  # 层 0 全量（release）：暂存之后在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（建法见快档判红时的出路句），判绿按输入哈希写一格全绿标记`。
- 第 107 行（快档）现在的说法是对的，没动。
- **另改了第 104 行**：它说整轮 `gate.sh`「含层 0 全量」，现在是假的。改成「含层 0 快档与全绿标记核对、QEMU 真设备，要十几分钟；层 0 全量不在里面」。「十几分钟」没核。
- `.claude/rules/implementation-workflow.md` 第 65 行改成：「崩溃点重放由门禁 54 号判：整轮门禁跑快档，再按这批输入的内容哈希核那一格层 0 全量的全绿标记（两条流都是 `exhaustive=true` 才算）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，没有显式把线程数设成 1……判红」。只写怎么做，没写经过。

## 在 r3 下重跑

装置：`r2/model` 拷成 `r3/model`，`lib.sh` 多一条臂 r3（草稿副本）；合成仓建在 `r3/runs/`。原样输出在 `r3/model/outputs/`，全文在附录。

**S3**（`s3-marker-slot.out`；r2 是第二轮开跑删这批输入那一格，r3 是这一轮）：

| | r2 | r3 |
|---|---|---|
| S3a 验证员跑过 × 门禁起在收尾 --full 的开跑前 / 途中 / 跑完后 | 0 / **1** / 0 | 0 / **0** / 0 |
| S3a 验证员没跑 × 同上 | 1 / 1 / 0 | 1 / 1 / 0（该红） |
| S3b B 绿 × A-then-B / B-then-A / B-inside-A / A-inside-B | 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 |
| S3b B 红 × 同上 | 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 |

r3 下 S3a「途中」那一格，收尾那一趟跑的过程中标记是「[标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…]」；r2 下是「[标记不在]」。

**S2**（`s2-trigger-gaps.out`，两种起法的退出码）：r2、r3 逐格相同。只改 `Cargo.lock` 是 1 / 1，只改根 `Cargo.toml` 1 / 1，只改测试 1 / 1，只追加变异表 1 / 1，这一批只改 `NOTES.md` 77 / 77。

**S4**（`s4-marker-provenance.out`）：
- r1：`--staged` 退 0，带着 `exhaustive=false` 判绿。
- r3，收尾仍在主工作区跑：退 1，红在「LAYER0 与 LAYER0B 两行不都是 exhaustive=true（是的只有 1 行）」。
- r3，收尾照 f5 在 worktree 里跑：`--full` 判「层 0 不是全量」、没写那一格，`--staged` 退 1。

**驱动脚本**（`r3/selftest-driver.sh`，输出 `r3/selftest-output.txt`，244 行）：第二轮那些格照旧，退出码与标记去留都相同；新加了打断的几格。
- 自证四（哈希 B 上缺 CHECKER 行 / cargo 判红）两趟都退 1，之后「[标记 1 个：…2c406d1eaecd…]」：B 那一格被 trap 删了，A 那一格还在。
- 同一批输入先绿，第二趟跑到一半发 SIGTERM：退出码 143，那一格被删；重跑补回。
- 同一批输入先绿，第二趟跑到一半发 SIGKILL：退出码 137，那一格留着；之后快档退 0。
- 驱动脚本里 SIGINT 那一格**不算数**：它是用 `&` 起的后台作业，非交互 bash 给后台作业的 SIGINT 是忽略的，所以跑完了、退出码 0，那一格照写。
- 另用 `r3/sigint-check.py` 照按 Ctrl-C 的样子给进程组发 SIGINT（不经过 `&`），原样输出在 `r3/sigint-check.txt`：

```
跑到一半时： ['singlefs-layer0-full-green.2c406d1eaecd…', 'singlefs-layer0-full-green.5e3b302b7908…']
进程组收到 SIGINT 之后，退出码 -2 （负数是被信号结束）： ['singlefs-layer0-full-green.5e3b302b7908…']
```

- HUP 只在一个最小的 trap 试验里核过（EXIT trap 跑了，退出码 129），没在 54 号上发过。

## lint（改完真文件、README、规则之后，原样；全文 `r3/lints.txt`）

```
  ✓ 门禁自检通过：83 个脚本（.sh 与 .py）、227 条拒绝都带了出路
gate-lint 退出码 0
  ✓ shell 纪律检查通过（共 76 个脚本）
shell-lint 退出码 0
bash -n 退出码 0
  ✓ 文档铁律检查通过（检查 477，跳过 0；DOC_LINT_VERBOSE=1 看全部）
doc-lint 退出码 0
```

另外照 `gate.sh`「规则纪律（项目本地）」的调法跑了 rules-lint（`r3/rules-lint.txt`）：**退出码 1**，红在 `.claude/agents/experiment-runner.md:27`（「1 行写了日期」）。那是定义文件，不是我改的，第 65 行没被点名。

## 要主 agent 知道的

- 规则 `.claude/rules/implementation-workflow.md` 第 32 行写「层 0 崩溃点重放还读字节布局表与它比对的入库产物」，与 `stage-inputs.tsv` 第 11 行现在的注释（两条流的用例不读 `.claude/kb/layout/` 与 `research/results/`）对不上。不在这一次的改动范围里，没动。
- trap 删的是「开跑时的输入哈希」那一格。「跑的过程中输入变了」判红时，删的也是开跑那一批的格；这是这一轮定的形态，从严。
- 没暂存、没提交；没真跑 `--full`；场景都在合成仓里用假 cargo 跑。


## 附录：`r3/model/outputs/s2-trigger-gaps.out` 原样

```
════ S2 [lock-only｜r2] 暂存的路径：Cargo.lock ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 8896f264c959b056…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-lock-only-r2/.git/singlefs-layer0-full-green.8896f264c959b0563a497d21e258fa1a170fc38465398ae5802c4e86cb449918）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:43Z），与这一次的输入比：
             内容不同：Cargo.lock
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 8896f264c959b056…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-lock-only-r2/.git/singlefs-layer0-full-green.8896f264c959b0563a497d21e258fa1a170fc38465398ae5802c4e86cb449918）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:43Z），与这一次的输入比：
             内容不同：Cargo.lock
    [gate --staged 里 54 号的退出码 1]
════ S2 [lock-only｜r3] 暂存的路径：Cargo.lock ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 8896f264c959b056…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-lock-only-r3/.git/singlefs-layer0-full-green.8896f264c959b0563a497d21e258fa1a170fc38465398ae5802c4e86cb449918）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:43Z），与这一次的输入比：
             内容不同：Cargo.lock
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 8896f264c959b056…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-lock-only-r3/.git/singlefs-layer0-full-green.8896f264c959b0563a497d21e258fa1a170fc38465398ae5802c4e86cb449918）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:43Z），与这一次的输入比：
             内容不同：Cargo.lock
    [gate --staged 里 54 号的退出码 1]
════ S2 [toml-only｜r2] 暂存的路径：Cargo.toml ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 d1cd05492c98b660…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-toml-only-r2/.git/singlefs-layer0-full-green.d1cd05492c98b660dfd2ffcc46538bff89e3edbd7940557b881dec22813b7693）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:43Z），与这一次的输入比：
             内容不同：Cargo.toml
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 d1cd05492c98b660…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-toml-only-r2/.git/singlefs-layer0-full-green.d1cd05492c98b660dfd2ffcc46538bff89e3edbd7940557b881dec22813b7693）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:43Z），与这一次的输入比：
             内容不同：Cargo.toml
    [gate --staged 里 54 号的退出码 1]
════ S2 [toml-only｜r3] 暂存的路径：Cargo.toml ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 d1cd05492c98b660…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-toml-only-r3/.git/singlefs-layer0-full-green.d1cd05492c98b660dfd2ffcc46538bff89e3edbd7940557b881dec22813b7693）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：Cargo.toml
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 d1cd05492c98b660…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-toml-only-r3/.git/singlefs-layer0-full-green.d1cd05492c98b660dfd2ffcc46538bff89e3edbd7940557b881dec22813b7693）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：Cargo.toml
    [gate --staged 里 54 号的退出码 1]
════ S2 [test-only｜r2] 暂存的路径：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 b21d2a103829ec5c…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-test-only-r2/.git/singlefs-layer0-full-green.b21d2a103829ec5cb7a756d4be380b992f2585d8a1df877cdb6f76395cd89936）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 b21d2a103829ec5c…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-test-only-r2/.git/singlefs-layer0-full-green.b21d2a103829ec5cb7a756d4be380b992f2585d8a1df877cdb6f76395cd89936）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
    [gate --staged 里 54 号的退出码 1]
════ S2 [test-only｜r3] 暂存的路径：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 b21d2a103829ec5c…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-test-only-r3/.git/singlefs-layer0-full-green.b21d2a103829ec5cb7a756d4be380b992f2585d8a1df877cdb6f76395cd89936）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 b21d2a103829ec5c…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-test-only-r3/.git/singlefs-layer0-full-green.b21d2a103829ec5cb7a756d4be380b992f2585d8a1df877cdb6f76395cd89936）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
    [gate --staged 里 54 号的退出码 1]
════ S2 [mutations-only｜r2] 暂存的路径：crates/mutations.tsv ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2cfd45abffe5a5a2…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-mutations-only-r2/.git/singlefs-layer0-full-green.2cfd45abffe5a5a237c549137a94ec6d08430da8c8efc86682395dbe004fb86b）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：crates/mutations.tsv
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2cfd45abffe5a5a2…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-mutations-only-r2/.git/singlefs-layer0-full-green.2cfd45abffe5a5a237c549137a94ec6d08430da8c8efc86682395dbe004fb86b）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：crates/mutations.tsv
    [gate --staged 里 54 号的退出码 1]
════ S2 [mutations-only｜r3] 暂存的路径：crates/mutations.tsv ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2cfd45abffe5a5a2…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-mutations-only-r3/.git/singlefs-layer0-full-green.2cfd45abffe5a5a237c549137a94ec6d08430da8c8efc86682395dbe004fb86b）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：crates/mutations.tsv
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2cfd45abffe5a5a2…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s2/repo-mutations-only-r3/.git/singlefs-layer0-full-green.2cfd45abffe5a5a237c549137a94ec6d08430da8c8efc86682395dbe004fb86b）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:21:44Z），与这一次的输入比：
             内容不同：crates/mutations.tsv
    [gate --staged 里 54 号的退出码 1]
════ S2 [other-session-crates｜r2] 暂存的路径：NOTES.md ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 1 个改动路径，没有一个落在 crates/ Cargo.toml Cargo.lock 底下
    [gate --staged 里 54 号的退出码 77]
  ── 起门禁：gate-staged
      ! 本阶段跳过（复用上一次整轮全绿的判定）：与上次整轮全绿那棵树（8850cf9cd811）在这几条路径上逐字相同：crates/ Cargo.toml Cargo.lock .claude/gate.d/stage-inputs.tsv .claude/gate.d/54-layer0-replay.sh
    [gate --staged 里 54 号的退出码 77]
════ S2 [other-session-crates｜r3] 暂存的路径：NOTES.md ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 1 个改动路径，没有一个落在 crates/ Cargo.toml Cargo.lock 底下
    [gate --staged 里 54 号的退出码 77]
  ── 起门禁：gate-staged
      ! 本阶段跳过（复用上一次整轮全绿的判定）：与上次整轮全绿那棵树（bc5ae54134b2）在这几条路径上逐字相同：crates/ Cargo.toml Cargo.lock .claude/gate.d/stage-inputs.tsv .claude/gate.d/54-layer0-replay.sh
    [gate --staged 里 54 号的退出码 77]
```

## 附录：`r3/model/outputs/s3-marker-slot.out` 原样

```
════ S3a [r2]
── [r2｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：before] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:21:44Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [r2｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：during] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
    (收尾 --full 正在跑：[标记不在])
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s3/a-r2-yes-during/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [r2｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：after] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:21:47Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [r2｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：before] 起门禁前 [标记不在]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s3/a-r2-no-before/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [r2｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：during] 起门禁前 [标记不在]
    (收尾 --full 正在跑：[标记不在])
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s3/a-r2-no-during/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [r2｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：after] 起门禁前 [标记不在]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:21:51Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
════ S3b [r2]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 0（主工作区）
── [r2｜B 的结果：green｜先后：A-then-B] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:21:51Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 0（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r2｜B 的结果：green｜先后：B-then-A] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:21:51Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 0（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r2｜B 的结果：green｜先后：B-inside-A] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:21:54Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 0（主工作区）
── [r2｜B 的结果：green｜先后：A-inside-B] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:21:55Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 1（主工作区）
── [r2｜B 的结果：red｜先后：A-then-B] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:21:57Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 1（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r2｜B 的结果：red｜先后：B-then-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:21:57Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 1（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r2｜B 的结果：red｜先后：B-inside-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:00Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 1（主工作区）
── [r2｜B 的结果：red｜先后：A-inside-B] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:01Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
════ S3a [r3]
── [r3｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：before] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:03Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [r3｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：during] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
    (收尾 --full 正在跑：[标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）])
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:04Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [r3｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：after] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:07Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [r3｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：before] 起门禁前 [标记不在]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s3/a-r3-no-before/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [r3｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：during] 起门禁前 [标记不在]
    (收尾 --full 正在跑：[标记不在])
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s3/a-r3-no-during/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [r3｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：after] 起门禁前 [标记不在]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:10Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
════ S3b [r3]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 0（主工作区）
── [r3｜B 的结果：green｜先后：A-then-B] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:10Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 0（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r3｜B 的结果：green｜先后：B-then-A] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:10Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 0（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r3｜B 的结果：green｜先后：B-inside-A] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:13Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 0（主工作区）
── [r3｜B 的结果：green｜先后：A-inside-B] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:14Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 1（主工作区）
── [r3｜B 的结果：red｜先后：A-then-B] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:16Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 1（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r3｜B 的结果：red｜先后：B-then-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:16Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 1（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r3｜B 的结果：red｜先后：B-inside-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:20Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 1（主工作区）
── [r3｜B 的结果：red｜先后：A-inside-B] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:21Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
```

## 附录：`r3/model/outputs/s4-marker-provenance.out` 原样

```
════ S4 [r1] 工作区 vs 暂存区： M .claude/gate.d/54-layer0-replay.sh M  crates/singlefs-harness/src/crash.rs ；B 的改动命中 1 行
  ── 收尾：主工作区 bash .claude/gate.d/54-layer0-replay.sh --full
      ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
      ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
      ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r3/runs/s4/repo-r1/.git/singlefs-layer0-full-green（输入哈希 ad967f0e6db52cb2…，8 个文件；开跑 2026-09-24T00:22:23Z，跑完 2026-09-24T00:22:23Z）
    [标记 1 个：singlefs-layer0-full-green（记 ad967f0e6db5…；exhaustive=false,exhaustive=true）]
  ── gate.sh --staged
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（ad967f0e6db52cb2…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:22:23Z，标记里的计数行原样：
          LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
    [gate --staged 里 54 号的退出码 0]
════ S4 [r3-main] 工作区 vs 暂存区： M .claude/gate.d/54-layer0-replay.sh M  crates/singlefs-harness/src/crash.rs ；B 的改动命中 1 行
  ── 收尾：主工作区 bash .claude/gate.d/54-layer0-replay.sh --full
      ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
      ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
      ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r3/runs/s4/repo-r3-main/.git/singlefs-layer0-full-green.ad967f0e6db52cb2e138bb90236f89b66f458ecef8dae326c8450af88ce3aa5b（输入哈希 ad967f0e6db52cb2…，8 个文件；开跑 2026-09-24T00:22:23Z，跑完 2026-09-24T00:22:23Z；common-dir 里现有 1 格）
    [标记 1 个：singlefs-layer0-full-green.ad967f0e6db5…（记 ad967f0e6db5…；exhaustive=false,exhaustive=true）]
  ── gate.sh --staged
      ✗ 这批输入那一格全绿标记里，LAYER0 与 LAYER0B 两行不都是 exhaustive=true（是的只有 1 行）：写它的那一趟 --full 没把「不是全量」判红
             LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
             LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
    [gate --staged 里 54 号的退出码 1]
════ S4 [r3-f5] 工作区 vs 暂存区： M .claude/gate.d/54-layer0-replay.sh M  crates/singlefs-harness/src/crash.rs ；B 的改动命中 1 行
  ── 收尾（f5）：bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>；worktree 里那一份 54 号带 B 的半成品吗：0 行
      ✗ 层 0 不是全量：LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
    [标记不在]
  ── gate.sh --staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 ad967f0e6db52cb2…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/runs/s4/repo-r3-f5/.git/singlefs-layer0-full-green.ad967f0e6db52cb2e138bb90236f89b66f458ecef8dae326c8450af88ce3aa5b）
    [gate --staged 里 54 号的退出码 1]
```

## 附录：`r3/selftest-output.txt` 原样

```
════ 准备：--full 写这批输入那一格（哈希 A）
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:22:24Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 2c406d1eaecd2706…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green.2c406d1eaecd27061928685e1828c386709d3806ed2a4aa16ad72183d8a8c3d4（输入哈希 2c406d1eaecd2706…，3 个文件；开跑 2026-09-24T00:22:24Z，跑完 2026-09-24T00:22:24Z；common-dir 里现有 1 格）
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ 自证一：这批输入那一格在、相等 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2c406d1eaecd2706…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:24Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

改一个字节：/tmp/claude-1000/gate54-tiering/r3/lib.rs.before-byte-change crates/singlefs-harness/src/lib.rs differ: byte 13, line 1

════ 自证二：改一个输入文件的一个字节 ⇒ 这批输入（哈希 B）没有自己那一格 ⇒ 判红，列出最近一格与这一次不同的文件
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 a1be3d5c2e6f5859…，3 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green.a1be3d5c2e6f58592d6ea725fa7aa9b981622e40f54c58e77a729d196847c97d）
       common-dir 里最近写的一格是 singlefs-layer0-full-green.2c406d1eaecd27061928685e1828c386709d3806ed2a4aa16ad72183d8a8c3d4（跑完于 2026-09-24T00:22:24Z），与这一次的输入比：
       不同的文件共 1 个（最多列 20 个）：
         内容不同：crates/singlefs-harness/src/lib.rs
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"
[退出码 1]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ 准备：哈希 B 上 --full 全绿 ⇒ 两格并存，A 那一格不删
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:22:24Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 a1be3d5c2e6f5859…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green.a1be3d5c2e6f58592d6ea725fa7aa9b981622e40f54c58e77a729d196847c97d（输入哈希 a1be3d5c2e6f5859…，3 个文件；开跑 2026-09-24T00:22:24Z，跑完 2026-09-24T00:22:24Z；common-dir 里现有 2 格）
[退出码 0]
[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.a1be3d5c2e6f…]

════ 自证四：哈希 B 上 --full 喂缺 CHECKER 行的合成日志 ⇒ 判红、B 那一格删掉不写，A 那一格还在
$ env FAKE_CARGO_DROP_CHECKER=1 bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:22:24Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 a1be3d5c2e6f5859…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✗ 用例跑过了，LAYER0 计数行的下一行却不是以 CHECKER 开头的逐条不变量行：成功句里「逐条不变量」那一句会是空话
     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例打完 LAYER0 行要紧跟着 println! checker_line(&tally) 那一行；
                两行之间插进了别的输出，就把 CHECKER 那一行挪回 LAYER0 行的正下方。
[退出码 1]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ 自证四（另一支）：哈希 B 上 cargo 判红 ⇒ 判红、B 那一格不写，A 那一格还在
$ env FAKE_CARGO_FAIL=1 bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:22:24Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 a1be3d5c2e6f5859…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
running 6 tests (fake cargo, test=first_transaction_step_seven_layer0, include_ignored=1)
test full_enumeration ... FAILED
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）
     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture
                oracle 报的第一条违例在断言消息里。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。
[退出码 1]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ 改回那个字节（回到哈希 A）⇒ 快档判绿（A 那一格没被 B 上的几趟 --full 删掉）
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2c406d1eaecd2706…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:24Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ 自证三（兼 f3 旧名字）：删掉这批输入那一格，只留一份内容与这批输入相同、但不带哈希的旧名字标记 ⇒ 判红，点名旧名字不再认
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2c406d1eaecd2706…，3 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green.2c406d1eaecd27061928685e1828c386709d3806ed2a4aa16ad72183d8a8c3d4）
       common-dir 里一格全绿标记都没有。
       不带哈希的旧名字标记（/tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green）是按输入哈希分格之前写的，不再认：它罩不到任何一批，可以删掉。
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"
[退出码 1]
[标记 1 个：singlefs-layer0-full-green]

════ 准备：--full 重写 A 那一格
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:22:24Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 2c406d1eaecd2706…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green.2c406d1eaecd27061928685e1828c386709d3806ed2a4aa16ad72183d8a8c3d4（输入哈希 2c406d1eaecd2706…，3 个文件；开跑 2026-09-24T00:22:24Z，跑完 2026-09-24T00:22:24Z；common-dir 里现有 1 格）
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

改那一格的 LAYER0B 行：LAYER0B states=2104413 closed_form=2104413 exhau … exhaustive=false

════ f4a：这批输入那一格里 LAYER0B 行是 exhaustive=false ⇒ 判红
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 这批输入那一格全绿标记里，LAYER0 与 LAYER0B 两行不都是 exhaustive=true（是的只有 1 行）：写它的那一趟 --full 没把「不是全量」判红
         LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
         LAYER0B states=2104413 closed_form=2104413 exhaustive=false violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"
                那一趟判「层 0 不是全量」就是这一批让枚举退化了，去 crash.rs 的 enumerate_layer0 看。
[退出码 1]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ f4a：这批输入那一格里 LAYER0 行是 exhaustive=false ⇒ 判红
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 这批输入那一格全绿标记里，LAYER0 与 LAYER0B 两行不都是 exhaustive=true（是的只有 1 行）：写它的那一趟 --full 没把「不是全量」判红
         LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
         LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"
                那一趟判「层 0 不是全量」就是这一批让枚举退化了，去 crash.rs 的 enumerate_layer0 看。
[退出码 1]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ f4a 换回原样 ⇒ 判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2c406d1eaecd2706…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:24Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ --full 跑到第二条流时改了一个输入 ⇒ 判红、出路给 HEAD + 暂存区 worktree 的建法、不写这一格
$ env FAKE_CARGO_TOUCH_DURING_SECOND=/tmp/claude-1000/gate54-tiering/r3/selftest-repo/crates/singlefs-harness/src/lib.rs bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:22:24Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 2c406d1eaecd2706…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✗ 全量跑的过程中这一道的输入变了（开跑 2c406d1eaecd2706…，跑完 c8c6be913438c63a…）：两条流读到的不一定是同一版，不写全绿标记
       不同的文件共 1 个（最多列 20 个）：
         内容不同：crates/singlefs-harness/src/lib.rs
     → 怎么办：别在有人改这些路径的树里跑。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"
[退出码 1]
[标记不在]

════ f5：照快档出路句里打出来的命令原样建 HEAD + 暂存区的 worktree、在里面跑 --full，再仿 gate.sh --staged 起快档
 M Cargo.toml
M  crates/singlefs-harness/src/lib.rs
A  crates/singlefs-harness/src/new_module.rs
?? crates/singlefs-harness/src/other_session_wip.rs

  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 fff20e92dedbcecb…，5 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green.fff20e92dedbcecbc16c11abbbb7ff1513eb8d32550c1633035a348653bf0ad5）
       common-dir 里一格全绿标记都没有。
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"

──── 从出路句里抠出来的命令（原样照跑）：
layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"

Preparing worktree (detached HEAD 8f70feb)
HEAD is now at 8f70feb base
  · --full 开跑（2026-09-24T00:22:24Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 5e3b302b7908e0d2…（4 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green.5e3b302b7908e0d228d14029c3148766c03d9e2aefc0f764f61317a149a0f0b9（输入哈希 5e3b302b7908e0d2…，4 个文件；开跑 2026-09-24T00:22:24Z，跑完 2026-09-24T00:22:24Z；common-dir 里现有 1 格）
[出路命令的退出码 0]
[标记 1 个：singlefs-layer0-full-green.5e3b302b7908…]

════ 仿 gate.sh --staged：临时 worktree 里跑快档 ⇒ 判绿（工作区里别的会话的 other_session_wip.rs 与 Cargo.toml 改动都不在）
$ env SINGLEFS_GATE_FULL=1 bash /tmp/claude-1000/gate54-tiering/r3/selftest-staged-worktree/.claude/gate.d/54-layer0-replay.sh /tmp/claude-1000/gate54-tiering/r3/selftest-staged-worktree
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（5e3b302b7908e0d2…，4 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:24Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.5e3b302b7908…]

════ 对照：主工作区（带别的会话没暂存的改动）跑快档 ⇒ 判红
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 fff20e92dedbcecb…，5 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green.fff20e92dedbcecbc16c11abbbb7ff1513eb8d32550c1633035a348653bf0ad5）
       common-dir 里最近写的一格是 singlefs-layer0-full-green.5e3b302b7908e0d228d14029c3148766c03d9e2aefc0f764f61317a149a0f0b9（跑完于 2026-09-24T00:22:24Z），与这一次的输入比：
       不同的文件共 2 个（最多列 20 个）：
         内容不同：Cargo.toml
         只在这一次里：crates/singlefs-harness/src/other_session_wip.rs
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"
[退出码 1]
[标记 1 个：singlefs-layer0-full-green.5e3b302b7908…]

══ git worktree list（出路命令最后一步 remove 过，应当只剩主工作区）
/tmp/claude-1000/gate54-tiering/r3/selftest-repo  8f70feb [master]

════ r3 新加：同一批输入先绿、第二趟 --full 被 TERM / INT 打断或被 SIGKILL 杀掉，那一格去留
════ 准备：--full 全绿，写这批输入那一格
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:22:24Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 2c406d1eaecd2706…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r3/selftest-repo/.git/singlefs-layer0-full-green.2c406d1eaecd27061928685e1828c386709d3806ed2a4aa16ad72183d8a8c3d4（输入哈希 2c406d1eaecd2706…，3 个文件；开跑 2026-09-24T00:22:24Z，跑完 2026-09-24T00:22:24Z；common-dir 里现有 2 格）
[退出码 0]
[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.5e3b302b7908…]

── 第二趟 --full 跑到一半（pid 91855）时：[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.5e3b302b7908…]
   发 SIGTERM 之后退出码 143：[标记 1 个：singlefs-layer0-full-green.5e3b302b7908…]
   重跑一趟 --full 补回那一格：[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.5e3b302b7908…]
── 第二趟 --full 跑到一半（pid 92003）时：[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.5e3b302b7908…]
   发 SIGINT 之后退出码 0：[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.5e3b302b7908…]
   重跑一趟 --full 补回那一格：[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.5e3b302b7908…]
── 第二趟 --full 跑到一半（pid 95474）时：[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.5e3b302b7908…]
selftest-driver.sh: line 80: 95474 Killed                  FAKE_CARGO_SLEEP_FULL=5 bash "$stage" --full > "$base/interrupted-$signal_name.log" 2>&1
   发 SIGKILL 之后退出码 137：[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.5e3b302b7908…]
════ SIGKILL 之后留下的那一格照样认 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2c406d1eaecd2706…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:22:30Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.5e3b302b7908…]

```

## 附录：`r3/sigint-check.txt` 原样

```
跑到一半时： ['singlefs-layer0-full-green.2c406d1eaecd…', 'singlefs-layer0-full-green.5e3b302b7908…']
进程组收到 SIGINT 之后，退出码 -2 （负数是被信号结束）： ['singlefs-layer0-full-green.5e3b302b7908…']
```

## 附录：`r3/lints.txt` 原样

```

══ 门禁自检（每条拒绝都要给出路） ══

  ✓ 门禁自检通过：83 个脚本（.sh 与 .py）、227 条拒绝都带了出路
gate-lint 退出码 0

══ shell 纪律检查 ══

  ✓ shell 纪律检查通过（共 76 个脚本）
shell-lint 退出码 0
bash -n 退出码 0

══ 文档铁律检查 ══
  ! 按 .claude/doc-lint-exclude 绕开 382 个 .md —— 原样保存的证据，改了就断证据链：
  !   research/prompts/  （330 个）—— 当时原样发给模型的提示，与 research/results/ 的产物一一对应；改提示只能连同重跑一起改（文件与目录路径除外，见 .claude/rules/path-moves.md）
  !   .claude/gate.d/fixtures/43-owed-table-shape.sh/  （2 个）—— 门禁阶段「欠账表两张登记表的行形状」的判别力样本：故意造的登记表，与真表同号不同名，扫它只会报登记冲突
  !   .claude/gate.d/fixtures/51-admission-terms-covered.sh/  （4 个）—— 门禁阶段「准入不等式的每一项都有人维护」的判别力样本：造了带登记标题的 D28 / D5 样本文件，扫它只会报登记冲突
  !   .claude/gate.d/fixtures/53-format-const-placeholders.sh/  （4 个）—— 门禁阶段「格式常量文件里的占位」的判别力样本：造了带登记标题的 D22 样本决策与 C323 样本欠账行，扫它只会报登记冲突
  !   .claude/gate.d/fixtures/78-owed-cited-tests.sh/  （2 个）—— 门禁阶段「已还清的欠账行里点名的测试名还在不在」的判别力样本：造的欠账表，与真表同号不同名，扫它只会报登记冲突
  !   .claude/gate.d/fixtures/20-kb-shape.sh/  （8 个）—— 门禁阶段「kb 形状」的判别力样本：正文里故意写「此前写的是 X」这类历史陈述，扫它只会报那条本来就要它红的违规
  !   .claude/gate.d/fixtures/79-tree-table-reserve.sh/  （4 个）—— 门禁阶段「树表条目预留的认购表」的判别力样本：造了带登记标题的 D8 样本决策与 C271 / C272 / C144 样本欠账行，扫它只会报登记冲突
  !   .claude/gate.d/fixtures/81-audit-contradictions.sh/  （4 个）—— 门禁阶段「总审核第五节每行都有去向」的判别力样本：造的总审核记录与欠账表，与真表同号不同名，扫它只会报登记冲突
  !   .claude/gate.d/fixtures/82-clause-enum-pairs.sh/  （2 个）—— 门禁阶段「条文列的集合与代码里的枚举对得上」的判别力样本：造了带登记标题的 D99 样本决策，扫它只会报登记冲突
  !   .claude/gate.d/fixtures/83-closeout-row-numbers.sh/  （2 个）—— 门禁阶段「收口表行号只许顺序号」的判别力样本：造的里程碑收口表，与真表同号不同名
  !   .claude/gate.d/fixtures/16-freeze-layer-membership.sh/  （14 个）—— 门禁阶段「冻结层归属登记表」的判别力样本：造了带登记标题的 D15 / D18 / D21 / D8 / D19 / D2 样本决策与一份样本登记表，扫它只会报登记冲突
  !   .claude/gate.d/fixtures/40-results-cited.sh/  （3 个）—— 门禁阶段「实验产物有没有写回」的判别力样本：造了带登记标题的 E98 样本实验页，与真表同号不同名，扫它只会报登记冲突
  !   .claude/gate.d/fixtures/52-segment-registry.sh/  （2 个）—— 门禁阶段「段序列登记表与 E142 产物逐字比对」的判别力样本：合成的最小 layout 表只有「八、」一节，没有历史节，扫它只会报 kb 形状
  !   .claude/gate.d/fixtures/12-no-prime-marks.sh/  （1 个）—— 门禁阶段「全仓不许用撇号当角标」的判别力样本：造的最小 kb 文件只为让 12 号有东西可扫，没有历史节，扫它只会报 kb 形状

  ! 上下文指代与自指称呼这两条是**启发式**：只认句首或标点之后的常见说法，
  !   「按本决策办」这种嵌在句中的漏得掉。绿不代表这两条穷举过了。
  ✓ 文档铁律检查通过（检查 477，跳过 0；DOC_LINT_VERBOSE=1 看全部）
doc-lint 退出码 0
```

## 附录：`r3/rules-lint.txt` 原样

```
  ✗ 1 行写了日期——规矩没有日期，日期属于案卷：
     .claude/agents/experiment-runner.md:27：2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml`
     → 怎么办：把这一句的经过删掉（共享规则的历史进 CHANGELOG.md 一版一节；项目本地规则的经过进项目已有的 records/、kb/），正文只留该怎么做；
               日期是引的小节名的一部分，就把小节名放进「」里原样引；判据的起算日、路径里的日期放进反引号。
rules-lint（照 gate.sh「规则纪律（项目本地）」的调法）退出码 1
```

## 附录：`r3/README.diff` 原样

```
--- /tmp/claude-1000/gate54-tiering/r3/README.md.before	2026-09-23 22:45:57.498576937 +0000
+++ README.md	2026-09-24 00:23:39.059231819 +0000
@@ -101,11 +101,11 @@
 提交前跑门禁：
 
 ```bash
-bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含层 0 全量与 QEMU 真设备，要十几分钟）
+bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含层 0 快档与全绿标记核对、QEMU 真设备，要十几分钟；层 0 全量不在里面）
 
 cargo test --workspace                    # 平时的单测；层 0 全量标 ignored，这里只跑缩小版
 bash .claude/gate.d/54-layer0-replay.sh   # 单跑层 0 崩溃点重放快档，并核这批输入有没有全量的全绿标记
-bash .claude/gate.d/54-layer0-replay.sh --full  # 层 0 全量（release），判绿写全绿标记；主 agent 在任务收尾时跑
+bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>  # 层 0 全量（release）：暂存之后在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（建法见快档判红时的出路句），判绿按输入哈希写一格全绿标记
 bash .claude/gate.d/55-qemu-first-transaction.sh  # 单跑 QEMU 两块 virtio 盘上的第一个事务
 bash .claude/scripts/lkmm.sh              # 单跑 LKMM，需要 herd7 与一棵内核树
 bash research/scripts/vm-bench.sh --selftest  # 单跑虚机装置自检（装置归项目）
```

## 附录：`r3/implementation-workflow.diff` 原样

```
--- /tmp/claude-1000/gate54-tiering/r3/implementation-workflow.md.before	2026-09-23 22:45:57.519028008 +0000
+++ .claude/rules/implementation-workflow.md	2026-09-24 00:23:39.072530226 +0000
@@ -62,4 +62,4 @@
 - **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
 - **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
 
-**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档、再核这批输入有没有层 0 全量的全绿标记；全量由 `bash .claude/gate.d/54-layer0-replay.sh --full` 跑，没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（`.claude/gate.d/54-layer0-replay.sh`；红的那一支只拿合成日志核过）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
+**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再按这批输入的内容哈希核那一格层 0 全量的全绿标记（两条流都是 `exhaustive=true` 才算）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（`.claude/gate.d/54-layer0-replay.sh`；红的那一支只拿合成日志核过）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
```

## 附录：`r3/r3.diff` 原样

```
--- /tmp/claude-1000/gate54-tiering/r3/54-layer0-replay.sh.r2	2026-09-24 00:13:58.764383241 +0000
+++ /home/fy5090/code/singlefs/.claude/gate.d/54-layer0-replay.sh	2026-09-24 00:23:21.997688540 +0000
@@ -8,7 +8,8 @@
 #     两条流的全量枚举（「全量」「多线程」两段说的就是它），不问复用、不问改动范围，照跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
 #     开跑与跑完各算一次输入的内容哈希，对不上（跑的过程中输入被改了）判红。全绿标记按输入哈希分格：
 #     `$(git rev-parse --git-common-dir)/singlefs-layer0-full-green.<输入哈希>`，不进工作树；放 common-dir，各 worktree 读写的是同一组。
-#     开跑只删这批输入那一格（判红、被打断都不留这一格），别的格不动；全绿才写这一格。
+#     开跑一格都不删：同一批输入的结果是确定的，前一趟写下的那一格在这一趟跑的过程中照样算数。这一趟没写成标记就退出（判红、跑的过程中输入变了、
+#     被 TERM / INT / HUP 打断）时，退出前删这批输入那一格；被 SIGKILL 杀掉来不及删，前一趟那一格留着。别的格不动；全绿才写这一格。
 #     标记里有输入哈希、逐文件的「sha256  路径」、开跑与跑完的 UTC 时刻、工作线程数、两条流的计数行与 CHECKER 行原样。
 #   bash .claude/gate.d/54-layer0-replay.sh [项目根]           整轮门禁的默认（gate.sh 只传项目根）
 #     快档：两条流的测试二进制在 release 下只跑不标 ignored 的用例，一条都没通过判红；再按这批输入的哈希找那一格：
@@ -34,7 +35,7 @@
 # --full 里没显式把 SINGLEFS_LAYER0_THREADS 设成 1、而本机多于 1 核却只起了 1 个工作线程：判红（多半是线程数没传进去）。
 # --full 里 LAYER0 行的下一行不是 CHECKER 逐条不变量行：判红（不然成功句里「逐条不变量」打出来是空串，整道照样绿）。
 # 这两支判红都只拿合成日志核过：把判定段抽进临时脚本，喂一份缺那一行的日志。
-# 标记那一半（相等判绿、改一个输入字节判红、没有标记判红、--full 判红不写标记、分格互不删、只改 Cargo.lock 判红、标记里 exhaustive=false 判红）拿临时仓加一个打合成日志的假 cargo 核过，同样不在 fixtures 里。
+# 标记那一半（相等判绿、改一个输入字节判红、没有标记判红、--full 判红不写标记、分格互不删、同一批输入两趟 --full 撞车不误红、同一批输入先绿后红删掉那一格、只改 Cargo.lock 判红、标记里 exhaustive=false 判红）拿临时仓加一个打合成日志的假 cargo 核过，同样不在 fixtures 里。
 set -uo pipefail
 # 参数：`--full` 与项目根，顺序不限；gate.sh 只传项目根，于是整轮门禁走快档。
 layer0_tier="quick"
@@ -291,14 +292,16 @@
   exit 0
 fi
 
-# ── --full：记下开跑时的输入清单，只删这批输入那一格（别的格不动；判红、被打断都不留这一格），跑完对一遍 ─────────
+# ── --full：记下开跑时的输入清单，跑完对一遍；开跑一格都不删，这一趟没写成标记就退出时才删这批输入那一格 ─────────
 full_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
 manifest_at_start="$layer0_scratch_directory/manifest-at-start"
 write_layer0_input_manifest "$manifest_at_start" || fail_without_input_manifest
 input_hash_at_start="$layer0_input_hash"
 full_green_marker_path="$full_green_marker_prefix.$input_hash_at_start"
-rm -f -- "${full_green_marker_path:?}"
-echo "  · --full 开跑（${full_started_utc}）：这批输入那一格旧标记已删（别的格不动）；这一道的输入哈希 ${input_hash_at_start:0:16}…（${layer0_input_file_count} 个文件，登记路径 ${layer0_input_paths_text}）"
+# 这一趟判红（任何一处 exit）或被打断：同一批输入先绿后红，前一趟那一格不再作数，退出前删掉它；写成了就留着
+full_marker_written=0
+trap 'if [[ "$full_marker_written" != 1 ]]; then rm -f -- "${full_green_marker_path:?}"; fi; rm -rf -- "${layer0_scratch_directory:?}"' EXIT
+echo "  · --full 开跑（${full_started_utc}）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 ${input_hash_at_start:0:16}…（${layer0_input_file_count} 个文件，登记路径 ${layer0_input_paths_text}）"
 
 log="$(mktemp)"
 if ! run_layer0_test_binary first_transaction_step_seven_layer0 "$log"; then
@@ -389,5 +392,6 @@
   echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full（没有标记，整轮门禁的快档会一直红）。"
   exit 1
 fi
+full_marker_written=1
 marker_slot_total="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green.*' ! -name '*.partial.*' | grep -c .)"
 echo "  ✓ 全绿标记写进 $full_green_marker_path（输入哈希 ${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件；开跑 ${full_started_utc}，跑完 ${full_finished_utc}；common-dir 里现有 ${marker_slot_total} 格）"
```
