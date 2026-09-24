# 门禁 54 号第二轮改动（f2、f3、f4a、f5）：交回报告

依据：判决 `research/prompts/defs-gate54-tiering-r1-main-verification.md` 第三节 K3 那张表与第四节第 1 条；攻方报告 `research/prompts/defs-gate54-tiering-r1-opus-output.md` 第五节。

改好的文件：`/home/fy5090/code/singlefs/.claude/gate.d/54-layer0-replay.sh`（393 行，sha256 `c3f9360fc9d1ceb0d22e421e380feacf78c38351cfd5554788a3cda545cf3537`）。
- 改前是 `86ff561e868cab9ffb138a2e5fbcd668c467caea6939592c32485a162e017ec9`：动手前现核过，与攻方、开工快照里的那一份相同。备份在 `r2/54-layer0-replay.sh.r1`。
- diff 在 `r2/r2.diff`（+102 / −47）。
- 18 处都用 `research/scripts/replace-once.py` 改，每处都是「命中 1 次，已替换并回读确认」。
- 18 对旧文本 / 新文本在 `r2/edits/old*.txt`、`new*.txt`。先在草稿副本 `r2/54-layer0-replay.sh` 上改、做完全部自证，再把同样 18 对套到真文件上；套完 `cmp` 报真文件与自证用的副本逐字节相同。
- 没暂存、没提交。没碰 `stage-inputs.tsv`（它在工作区里与 HEAD 差 1 行，是主 agent 改的，登记给 54 号的三条是 `crates/ Cargo.toml Cargo.lock`）、定义文件、kb、`crates/`。

## 四处改了什么

| 改法 | 改在哪（改后行号） | 做什么 |
|---|---|---|
| f2 | 55–70 行（新）、86–91 行 | 读 `stage-inputs.tsv` 登记给本阶段的路径，挪到两问之前，结果放进 `layer0_registered_input_paths`：登记表没有这一行判红、退 1。第二问 `change-touches-crates.sh` 的前缀从 `crates/` 换成这几条，跳过句里写明前缀出自登记表。输入清单也改成读同一个数组，不再各读一遍 |
| f3 | 102–103 行、248 行、296–301 行、392–393 行 | 标记按输入哈希分格：`<common-dir>/singlefs-layer0-full-green.<输入哈希>`。快档按这批输入的哈希找那一格；`--full` 先算开跑时的输入哈希，只删那一格，别的格不动。判绿的句子多报一句「common-dir 里现有 N 格」。不带哈希的旧名字不再认：快档没找到这批的那一格时，点名说旧名字标记不再认、可以删掉（`report_newest_other_marker`，205 行起），同时拿 common-dir 里最近写的一格与这一次比，列出不同的文件 |
| f4a | 277–286 行 | 计数行恰好三行之后，另核 `LAYER0` 与 `LAYER0B` 两行都带 `exhaustive=true`（`grep -cE '^LAYER0B? (.* )?exhaustive=true( |$)'` 要等于 2），不等就判红，并原样列出两行 |
| f5 | 197–202 行（`print_staged_worktree_full_commands`），以及快档四处判红、`--full`「输入变了」那一处判红的出路 | 出路句改成「这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）」，后面三行是能直接照跑的命令：`mktemp -d`、`git diff --cached --binary`、`git worktree add --detach … HEAD`、diff 不空才 `apply --index`、在那棵树里跑 54 号 `--full`、`git worktree remove --force` |

文件头照改后的行为改写：`gate-stage` 行、「分两档」一段、原来的「工作区里有别的会话没提交的输入时…」那一行（改成主工作区跑的 --full 为什么罩不到）、二问那一句、最后一行里自证做到哪一步。门禁修复会话在 CHECKER 判红那一块的改动一字未动。

## 每一处先在副本里证明会红

装置：
- 攻方模型 `research/prompts/defs-gate54-tiering-r1-opus-model/` 拷一份到 `r2/model/`（拷来的五份 sha256 与攻方报告列的逐字相同），`lib.sh` 改成两条臂：r1＝改前、r2＝改后草稿。
- 标记改成按格列出；合成仓建在 `r2/runs/` 下。
- S2、S3、S4 三个场景照攻方脚本的历史写，输出在 `r2/model/outputs/`。
- 另有上一轮那种驱动脚本 `r2/selftest-driver.sh`，全文输出在 `r2/selftest-output.txt`（212 行）。
- 全部原样输出在附录。真仓的 common-dir 一次都没写（跑完 `ls .git/singlefs-layer0-full-green*` 报不存在）。

**f2（S2，`r2/model/outputs/s2-trigger-gaps.out`）**：每一格退出码是 gate-triage 那一种起法 / gate-staged 那一种起法。

| 这一批 | r1 | r2 |
|---|---|---|
| 只改 `Cargo.lock` | 77 / 77（「没有一个落在 crates/ 底下」） | **1 / 1**（「内容不同：Cargo.lock」） |
| 只改根 `Cargo.toml` | 77 / 77 | **1 / 1**（「内容不同：Cargo.toml」） |
| 只改测试文件 | 1 / 1 | 1 / 1 |
| 只追加 `crates/mutations.tsv` | 1 / 1 | 1 / 1 |
| 这一批只改 `NOTES.md`，别的会话改了 `crates/` 没暂存 | 77 / 77 | 77 / 77（「没有一个落在 crates/ Cargo.toml Cargo.lock 底下」） |

**f3（S3，`r2/model/outputs/s3-marker-slot.out`）**：

| | r1 | r2 |
|---|---|---|
| S3a 验证员跑过 × 门禁起在收尾 --full 的开跑前 / 途中 / 跑完后 | 0 / **1** / 0 | 0 / **1** / 0 |
| S3a 验证员没跑 × 同上 | 1 / 1 / 0 | 1 / 1 / 0 |
| S3b B 绿 × A-then-B / B-then-A / B-inside-A / A-inside-B | **1** / 0 / 0 / **1** | 0 / 0 / 0 / 0 |
| S3b B 红 × 同上 | **1** / 0 / 0 / 0 | 0 / 0 / 0 / 0 |

r1 那一行复现了攻方的误红：S3a 1 格，S3b 3 格。r2 下 S3b 8 格全绿。
**S3a「验证员跑过、门禁起在途中」在 r2 下仍然误红**。原因是派发要的形态「开跑只删自己要写的那一格」：验证员与收尾是同一批输入、同一个哈希，收尾开跑删掉的正是验证员写的那一格（输出里「收尾 --full 正在跑：[标记不在]」）。攻方量过的 f3 是「开跑一格都不删」，所以它那一格是绿的。

我另在副本里量了一个变体，**没有落进真文件**：开跑不删，这一趟没写成标记就退出（判红）时，靠 EXIT trap 删这批输入那一格。脚本 `r2/variant/54-layer0-replay.sh`，场景 `r2/model/s3a-variant.sh`，输出 `outputs/s3a-variant.out`。结果：
- S3a 6 格只剩「验证员没跑、开跑前 / 途中」两格红，这两格本来就该红。
- 同一输入先绿一趟、第二趟 cargo 判红之后，那一格被删、门禁判红。
- 代价：跑到一半被 SIGKILL，上一趟同一输入的绿格会留着。

要不要换成变体，由主 agent 定。

f3 的其余自证在 `r2/selftest-output.txt`：
- 哈希 B 上一趟 --full 全绿，A、B 两格并存。
- 接着 B 上两趟判红（缺 CHECKER 行、cargo 退 101），每趟都退出码 1，之后「[标记 1 个：…2c406d1eaecd…]」：B 那一格没了，A 那一格还在。
- 改回 A 的输入，快档退出码 0。
- 删掉 A 那一格、只留一份内容与 A 相同但不带哈希的旧名字标记，快档退出码 1，打出「不带哈希的旧名字标记（…/singlefs-layer0-full-green）是按输入哈希分格之前写的，不再认：它罩不到任何一批，可以删掉。」

**f4a（S4，`r2/model/outputs/s4-marker-provenance.out`；加 `r2/selftest-output.txt`）**：

| 臂 | 收尾那一趟 | `--staged` 下 54 号 |
|---|---|---|
| r1（收尾在主工作区，54 号带 B 的半成品） | 写了标记，`LAYER0 … exhaustive=false` | **0**（照样判绿，还把 `exhaustive=false` 那一行打了出来） |
| r2，收尾仍在主工作区 | 写了那一格，`exhaustive=false` | **1**：「✗ 这批输入那一格全绿标记里，LAYER0 与 LAYER0B 两行不都是 exhaustive=true（是的只有 1 行）」 |
| r2，收尾照 f5 在 HEAD + 暂存区的 worktree 里跑（那棵树里的 54 号不带 B 的半成品：命中 0 行） | 「✗ 层 0 不是全量：LAYER0 … exhaustive=false」，不写那一格 | **1**（没有这批输入那一格） |

另外直接改那一格：只把 `LAYER0B` 改成 `exhaustive=false` 退出码 1，只把 `LAYER0` 改成 `exhaustive=false` 退出码 1，换回原样退出码 0。

**f5（`r2/selftest-output.txt` 末段）**：
- 在合成仓里暂存这一批（改 `lib.rs`、新文件 `new_module.rs`）。工作区另有别的会话没暂存的 `Cargo.toml` 改动与未跟踪的 `other_session_wip.rs`。
- 快档判红，把它的出路句里打出来的三行命令原样抠进 `r2/outlet-commands.sh`、原样照跑：退出码 0，写出这批暂存内容那一格。
- 再照 `gate.sh --staged` 的建法仿一趟快档，退出码 0。
- 对照：主工作区（带别的会话的改动）跑快档，退出码 1，列出「内容不同：Cargo.toml」「只在这一次里：crates/singlefs-harness/src/other_session_wip.rs」。
- 跑完 `git worktree list` 只剩主工作区。
- `--full` 跑到第二条流时输入被改，判红，出路同一套命令。
- S4 那一张表的 r2-f5 一行也是 f5。

## 真仓快档

`nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`（工作区，不带 `--full`），日志 `r2/real-quick-run.log`。2026-09-24T00:14:15Z 开跑、00:14:29Z 结束，release 产物是现成的；转发的 `LAYER0_PROGRESS` 行 123 行。其余原样：

```
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 8 条通过、1 条 ignored），但这批输入（哈希 a721893541f8fd59…，94 个文件）没有层 0 全量的全绿标记（/home/fy5090/code/singlefs/.git/singlefs-layer0-full-green.a721893541f8fd59a182a7738a0390e57b8871553ee5bb49a606eb51cc81a47b）
       common-dir 里一格全绿标记都没有。
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"
exit=1
```

跑完 `ls .git/singlefs-layer0-full-green*` 报不存在：真仓里一格都没写。

## 两个 lint（改完真文件之后，原样）

```
══ 门禁自检（每条拒绝都要给出路） ══

  ✓ 门禁自检通过：83 个脚本（.sh 与 .py）、227 条拒绝都带了出路
gate-lint 退出码 0

══ shell 纪律检查 ══

  ✓ shell 纪律检查通过（共 76 个脚本）
shell-lint 退出码 0
```

上一轮是 225 条拒绝；多的 2 条是新加的「登记表里没有本阶段」与「两行不都是 exhaustive=true」。`bash -n` 通过。

## 要主 agent 判的

1. **S3a 同一输入两趟 --full 撞车那一格没修**：派发的「开跑只删自己那一格」修不到它，数与变体见 f3 一节。第二轮攻的时候，这一格可以直接拿变体比。
2. **旧格没人清**：分格之后 common-dir 里的格只增不减，一格约 120 行。--full 判绿时报现有几格，没做自动清理：按时间清会把别人刚写的格删掉，S3 就回来了。
3. **f5 的出路命令会在 `$TMPDIR` 下留一个装 `staged.patch` 的空目录**：`git worktree remove` 只删那棵树。建完 worktree 之后别人又往暂存区放东西，出路命令罩不到，这是判决点名给第二轮攻的。
4. **f4a 只挡 `exhaustive`**：线程数判定、CHECKER 判定被改坏，快档看不出来（攻方报告第五节已列）。f5 让收尾跑的是那棵树里的 54 号，挡住的是「工作区那一份 54 号被别人改坏」这一种。
5. **别的判定要跟着改的地方**（定义与文档，我没碰）：`README.md` 第 107 行；`.claude/rules/implementation-workflow.md` 第 65 行；`main-agent.md`、`crash-verifier.md` 由主 agent 在改。

## 没做什么

- 没真跑 `--full`。f2、f3、f4a、f5 的判定只在合成仓里用假 cargo 走通。
- 自证脚本在 `/tmp`，没进仓。
- 没跑整轮门禁，也没跑别的阶段。
- 变体只在副本上量过，被攻过零轮，没落进真文件。


## 附录：`r2/model/outputs/s2-trigger-gaps.out` 原样

```
════ S2 [lock-only｜r1] 暂存的路径：Cargo.lock ｜[标记 1 个：singlefs-layer0-full-green（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 1 个改动路径，没有一个落在 crates/ 底下
    [gate --staged 里 54 号的退出码 77]
  ── 起门禁：gate-staged
      ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 1 个改动路径，没有一个落在 crates/ 底下
    [gate --staged 里 54 号的退出码 77]
════ S2 [lock-only｜r2] 暂存的路径：Cargo.lock ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 8896f264c959b056…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s2/repo-lock-only-r2/.git/singlefs-layer0-full-green.8896f264c959b0563a497d21e258fa1a170fc38465398ae5802c4e86cb449918）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:11:39Z），与这一次的输入比：
             内容不同：Cargo.lock
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 8896f264c959b056…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s2/repo-lock-only-r2/.git/singlefs-layer0-full-green.8896f264c959b0563a497d21e258fa1a170fc38465398ae5802c4e86cb449918）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:11:39Z），与这一次的输入比：
             内容不同：Cargo.lock
    [gate --staged 里 54 号的退出码 1]
════ S2 [toml-only｜r1] 暂存的路径：Cargo.toml ｜[标记 1 个：singlefs-layer0-full-green（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 1 个改动路径，没有一个落在 crates/ 底下
    [gate --staged 里 54 号的退出码 77]
  ── 起门禁：gate-staged
      ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 1 个改动路径，没有一个落在 crates/ 底下
    [gate --staged 里 54 号的退出码 77]
════ S2 [toml-only｜r2] 暂存的路径：Cargo.toml ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 d1cd05492c98b660…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s2/repo-toml-only-r2/.git/singlefs-layer0-full-green.d1cd05492c98b660dfd2ffcc46538bff89e3edbd7940557b881dec22813b7693）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:11:39Z），与这一次的输入比：
             内容不同：Cargo.toml
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 d1cd05492c98b660…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s2/repo-toml-only-r2/.git/singlefs-layer0-full-green.d1cd05492c98b660dfd2ffcc46538bff89e3edbd7940557b881dec22813b7693）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:11:39Z），与这一次的输入比：
             内容不同：Cargo.toml
    [gate --staged 里 54 号的退出码 1]
════ S2 [test-only｜r1] 暂存的路径：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs ｜[标记 1 个：singlefs-layer0-full-green（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但全绿标记记的输入哈希（c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b）与这一次的（b21d2a103829ec5cb7a756d4be380b992f2585d8a1df877cdb6f76395cd89936，8 个文件）不同
             内容不同：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但全绿标记记的输入哈希（c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b）与这一次的（b21d2a103829ec5cb7a756d4be380b992f2585d8a1df877cdb6f76395cd89936，8 个文件）不同
             内容不同：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
    [gate --staged 里 54 号的退出码 1]
════ S2 [test-only｜r2] 暂存的路径：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 b21d2a103829ec5c…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s2/repo-test-only-r2/.git/singlefs-layer0-full-green.b21d2a103829ec5cb7a756d4be380b992f2585d8a1df877cdb6f76395cd89936）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:11:39Z），与这一次的输入比：
             内容不同：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 b21d2a103829ec5c…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s2/repo-test-only-r2/.git/singlefs-layer0-full-green.b21d2a103829ec5cb7a756d4be380b992f2585d8a1df877cdb6f76395cd89936）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:11:39Z），与这一次的输入比：
             内容不同：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
    [gate --staged 里 54 号的退出码 1]
════ S2 [mutations-only｜r1] 暂存的路径：crates/mutations.tsv ｜[标记 1 个：singlefs-layer0-full-green（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但全绿标记记的输入哈希（c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b）与这一次的（2cfd45abffe5a5a237c549137a94ec6d08430da8c8efc86682395dbe004fb86b，8 个文件）不同
             内容不同：crates/mutations.tsv
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但全绿标记记的输入哈希（c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b）与这一次的（2cfd45abffe5a5a237c549137a94ec6d08430da8c8efc86682395dbe004fb86b，8 个文件）不同
             内容不同：crates/mutations.tsv
    [gate --staged 里 54 号的退出码 1]
════ S2 [mutations-only｜r2] 暂存的路径：crates/mutations.tsv ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2cfd45abffe5a5a2…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s2/repo-mutations-only-r2/.git/singlefs-layer0-full-green.2cfd45abffe5a5a237c549137a94ec6d08430da8c8efc86682395dbe004fb86b）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:11:40Z），与这一次的输入比：
             内容不同：crates/mutations.tsv
    [gate --staged 里 54 号的退出码 1]
  ── 起门禁：gate-staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2cfd45abffe5a5a2…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s2/repo-mutations-only-r2/.git/singlefs-layer0-full-green.2cfd45abffe5a5a237c549137a94ec6d08430da8c8efc86682395dbe004fb86b）
           common-dir 里最近写的一格是 singlefs-layer0-full-green.c07850adb431b3cb8b56efb1ed16f6b7cbd7ee3ac64a8fb28eb05ecb3a00383b（跑完于 2026-09-24T00:11:40Z），与这一次的输入比：
             内容不同：crates/mutations.tsv
    [gate --staged 里 54 号的退出码 1]
════ S2 [other-session-crates｜r1] 暂存的路径：NOTES.md ｜[标记 1 个：singlefs-layer0-full-green（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 1 个改动路径，没有一个落在 crates/ 底下
    [gate --staged 里 54 号的退出码 77]
  ── 起门禁：gate-staged
      ! 本阶段跳过（复用上一次整轮全绿的判定）：与上次整轮全绿那棵树（0f04a085588f）在这几条路径上逐字相同：crates/ Cargo.toml Cargo.lock .claude/gate.d/stage-inputs.tsv .claude/gate.d/54-layer0-replay.sh
    [gate --staged 里 54 号的退出码 77]
════ S2 [other-session-crates｜r2] 暂存的路径：NOTES.md ｜[标记 1 个：singlefs-layer0-full-green.c07850adb431…（记 c07850adb431…；exhaustive=true,exhaustive=true）]
  ── 起门禁：gate-triage
      ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 1 个改动路径，没有一个落在 crates/ Cargo.toml Cargo.lock 底下
    [gate --staged 里 54 号的退出码 77]
  ── 起门禁：gate-staged
      ! 本阶段跳过（复用上一次整轮全绿的判定）：与上次整轮全绿那棵树（96aa7dc312cb）在这几条路径上逐字相同：crates/ Cargo.toml Cargo.lock .claude/gate.d/stage-inputs.tsv .claude/gate.d/54-layer0-replay.sh
    [gate --staged 里 54 号的退出码 77]
```

## 附录：`r2/model/outputs/s3-marker-slot.out` 原样

```
════ S3a [r1]
── [r1｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：before] 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:11:40Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [r1｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：during] 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
    (收尾 --full 正在跑：[标记不在])
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3/a-r1-yes-during/.git/singlefs-layer0-full-green）
    [gate --staged 里 54 号的退出码 1]
── [r1｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：after] 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:11:43Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [r1｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：before] 起门禁前 [标记不在]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3/a-r1-no-before/.git/singlefs-layer0-full-green）
    [gate --staged 里 54 号的退出码 1]
── [r1｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：during] 起门禁前 [标记不在]
    (收尾 --full 正在跑：[标记不在])
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3/a-r1-no-during/.git/singlefs-layer0-full-green）
    [gate --staged 里 54 号的退出码 1]
── [r1｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：after] 起门禁前 [标记不在]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:11:46Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
════ S3b [r1]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 0（主工作区）
── [r1｜B 的结果：green｜先后：A-then-B] A 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但全绿标记记的输入哈希（236ceeb156f952b4cf895ec4a5b450e7174e7587fe2d2a29415aa924958fcda2）与这一次的（0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050，8 个文件）不同
    [gate --staged 里 54 号的退出码 1]
    B 的 --full 退出码 0（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r1｜B 的结果：green｜先后：B-then-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:11:46Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 0（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r1｜B 的结果：green｜先后：B-inside-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:11:50Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 0（主工作区）
── [r1｜B 的结果：green｜先后：A-inside-B] A 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但全绿标记记的输入哈希（236ceeb156f952b4cf895ec4a5b450e7174e7587fe2d2a29415aa924958fcda2）与这一次的（0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050，8 个文件）不同
    [gate --staged 里 54 号的退出码 1]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 1（主工作区）
── [r1｜B 的结果：red｜先后：A-then-B] A 起门禁前 [标记不在]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3/b-r1-red-A-then-B/.git/singlefs-layer0-full-green）
    [gate --staged 里 54 号的退出码 1]
    B 的 --full 退出码 1（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r1｜B 的结果：red｜先后：B-then-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:11:53Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 1（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r1｜B 的结果：red｜先后：B-inside-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:11:56Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 1（主工作区）
── [r1｜B 的结果：red｜先后：A-inside-B] A 起门禁前 [标记 1 个：singlefs-layer0-full-green（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:11:57Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
════ S3a [r2]
── [r2｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：before] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:11:59Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [r2｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：during] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
    (收尾 --full 正在跑：[标记不在])
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3/a-r2-yes-during/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [r2｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：after] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:02Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [r2｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：before] 起门禁前 [标记不在]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3/a-r2-no-before/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [r2｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：during] 起门禁前 [标记不在]
    (收尾 --full 正在跑：[标记不在])
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3/a-r2-no-during/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [r2｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：after] 起门禁前 [标记不在]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:05Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
════ S3b [r2]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 0（主工作区）
── [r2｜B 的结果：green｜先后：A-then-B] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:06Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 0（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r2｜B 的结果：green｜先后：B-then-A] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:06Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 0（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r2｜B 的结果：green｜先后：B-inside-A] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:09Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 0（主工作区）
── [r2｜B 的结果：green｜先后：A-inside-B] A 起门禁前 [标记 2 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true） singlefs-layer0-full-green.236ceeb156f9…（记 236ceeb156f9…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:10Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 1（主工作区）
── [r2｜B 的结果：red｜先后：A-then-B] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:12Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 1（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r2｜B 的结果：red｜先后：B-then-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:12Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    B 的 --full 退出码 1（主工作区）
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
── [r2｜B 的结果：red｜先后：B-inside-A] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:15Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    A 的 --full 退出码 0（HEAD + 暂存区的 worktree）
    B 的 --full 退出码 1（主工作区）
── [r2｜B 的结果：red｜先后：A-inside-B] A 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:16Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
```

## 附录：`r2/model/outputs/s3a-variant.out` 原样

```
── [变体｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：before] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:54Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [变体｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：during] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
    (收尾 --full 正在跑：[标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）])
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:54Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [变体｜验证员先跑过 --full：yes｜门禁起在收尾 --full 的：after] 起门禁前 [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:12:57Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [变体｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：before] 起门禁前 [标记不在]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3-variant/a-no-before/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [变体｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：during] 起门禁前 [标记不在]
    (收尾 --full 正在跑：[标记不在])
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3-variant/a-no-during/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
── [变体｜验证员先跑过 --full：no｜门禁起在收尾 --full 的：after] 起门禁前 [标记不在]
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（0a906e4b33b85a9b…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:13:00Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
── [变体｜同一输入先绿一趟] [标记 1 个：singlefs-layer0-full-green.0a906e4b33b8…（记 0a906e4b33b8…；exhaustive=true,exhaustive=true）]
      ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）
── [变体｜同一输入第二趟 cargo 判红之后] [标记不在]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 0a906e4b33b85a9b…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s3-variant/red-rerun/.git/singlefs-layer0-full-green.0a906e4b33b85a9b1db07a72f1059b9f0b804a7451e0a614f7374247be631050）
    [gate --staged 里 54 号的退出码 1]
```

## 附录：`r2/model/outputs/s4-marker-provenance.out` 原样

```
════ S4 [r1] 工作区 vs 暂存区： M .claude/gate.d/54-layer0-replay.sh M  crates/singlefs-harness/src/crash.rs ；B 的改动命中 1 行
  ── 收尾：主工作区 bash .claude/gate.d/54-layer0-replay.sh --full
      ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
      ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
      ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r2/runs/s4/repo-r1/.git/singlefs-layer0-full-green（输入哈希 ad967f0e6db52cb2…，8 个文件；开跑 2026-09-24T00:12:18Z，跑完 2026-09-24T00:12:18Z）
    [标记 1 个：singlefs-layer0-full-green（记 ad967f0e6db5…；exhaustive=false,exhaustive=true）]
  ── gate.sh --staged
      ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
      ✓ 全绿标记与这批输入的内容哈希相同（ad967f0e6db52cb2…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-24T00:12:18Z，标记里的计数行原样：
          LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
    [gate --staged 里 54 号的退出码 0]
════ S4 [r2-main] 工作区 vs 暂存区： M .claude/gate.d/54-layer0-replay.sh M  crates/singlefs-harness/src/crash.rs ；B 的改动命中 1 行
  ── 收尾：主工作区 bash .claude/gate.d/54-layer0-replay.sh --full
      ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
      ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
      ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r2/runs/s4/repo-r2-main/.git/singlefs-layer0-full-green.ad967f0e6db52cb2e138bb90236f89b66f458ecef8dae326c8450af88ce3aa5b（输入哈希 ad967f0e6db52cb2…，8 个文件；开跑 2026-09-24T00:12:18Z，跑完 2026-09-24T00:12:18Z；common-dir 里现有 1 格）
    [标记 1 个：singlefs-layer0-full-green.ad967f0e6db5…（记 ad967f0e6db5…；exhaustive=false,exhaustive=true）]
  ── gate.sh --staged
      ✗ 这批输入那一格全绿标记里，LAYER0 与 LAYER0B 两行不都是 exhaustive=true（是的只有 1 行）：写它的那一趟 --full 没把「不是全量」判红
             LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
             LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
    [gate --staged 里 54 号的退出码 1]
════ S4 [r2-f5] 工作区 vs 暂存区： M .claude/gate.d/54-layer0-replay.sh M  crates/singlefs-harness/src/crash.rs ；B 的改动命中 1 行
  ── 收尾（f5）：bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>；worktree 里那一份 54 号带 B 的半成品吗：0 行
      ✗ 层 0 不是全量：LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
    [标记不在]
  ── gate.sh --staged
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 ad967f0e6db52cb2…，8 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/runs/s4/repo-r2-f5/.git/singlefs-layer0-full-green.ad967f0e6db52cb2e138bb90236f89b66f458ecef8dae326c8450af88ce3aa5b）
    [gate --staged 里 54 号的退出码 1]
```

## 附录：`r2/selftest-output.txt` 原样

```
════ 准备：--full 写这批输入那一格（哈希 A）
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:13:43Z）：这批输入那一格旧标记已删（别的格不动）；这一道的输入哈希 2c406d1eaecd2706…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r2/selftest-repo/.git/singlefs-layer0-full-green.2c406d1eaecd27061928685e1828c386709d3806ed2a4aa16ad72183d8a8c3d4（输入哈希 2c406d1eaecd2706…，3 个文件；开跑 2026-09-24T00:13:43Z，跑完 2026-09-24T00:13:43Z；common-dir 里现有 1 格）
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ 自证一：这批输入那一格在、相等 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2c406d1eaecd2706…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:13:43Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

改一个字节：/tmp/claude-1000/gate54-tiering/r2/lib.rs.before-byte-change crates/singlefs-harness/src/lib.rs differ: byte 13, line 1

════ 自证二：改一个输入文件的一个字节 ⇒ 这批输入（哈希 B）没有自己那一格 ⇒ 判红，列出最近一格与这一次不同的文件
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 a1be3d5c2e6f5859…，3 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/selftest-repo/.git/singlefs-layer0-full-green.a1be3d5c2e6f58592d6ea725fa7aa9b981622e40f54c58e77a729d196847c97d）
       common-dir 里最近写的一格是 singlefs-layer0-full-green.2c406d1eaecd27061928685e1828c386709d3806ed2a4aa16ad72183d8a8c3d4（跑完于 2026-09-24T00:13:43Z），与这一次的输入比：
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
  · --full 开跑（2026-09-24T00:13:43Z）：这批输入那一格旧标记已删（别的格不动）；这一道的输入哈希 a1be3d5c2e6f5859…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r2/selftest-repo/.git/singlefs-layer0-full-green.a1be3d5c2e6f58592d6ea725fa7aa9b981622e40f54c58e77a729d196847c97d（输入哈希 a1be3d5c2e6f5859…，3 个文件；开跑 2026-09-24T00:13:43Z，跑完 2026-09-24T00:13:43Z；common-dir 里现有 2 格）
[退出码 0]
[标记 2 个：singlefs-layer0-full-green.2c406d1eaecd… singlefs-layer0-full-green.a1be3d5c2e6f…]

════ 自证四：哈希 B 上 --full 喂缺 CHECKER 行的合成日志 ⇒ 判红、B 那一格删掉不写，A 那一格还在
$ env FAKE_CARGO_DROP_CHECKER=1 bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:13:43Z）：这批输入那一格旧标记已删（别的格不动）；这一道的输入哈希 a1be3d5c2e6f5859…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✗ 用例跑过了，LAYER0 计数行的下一行却不是以 CHECKER 开头的逐条不变量行：成功句里「逐条不变量」那一句会是空话
     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例打完 LAYER0 行要紧跟着 println! checker_line(&tally) 那一行；
                两行之间插进了别的输出，就把 CHECKER 那一行挪回 LAYER0 行的正下方。
[退出码 1]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ 自证四（另一支）：哈希 B 上 cargo 判红 ⇒ 判红、B 那一格不写，A 那一格还在
$ env FAKE_CARGO_FAIL=1 bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:13:43Z）：这批输入那一格旧标记已删（别的格不动）；这一道的输入哈希 a1be3d5c2e6f5859…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
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
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2c406d1eaecd2706…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:13:43Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ 自证三（兼 f3 旧名字）：删掉这批输入那一格，只留一份内容与这批输入相同、但不带哈希的旧名字标记 ⇒ 判红，点名旧名字不再认
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2c406d1eaecd2706…，3 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/selftest-repo/.git/singlefs-layer0-full-green.2c406d1eaecd27061928685e1828c386709d3806ed2a4aa16ad72183d8a8c3d4）
       common-dir 里一格全绿标记都没有。
       不带哈希的旧名字标记（/tmp/claude-1000/gate54-tiering/r2/selftest-repo/.git/singlefs-layer0-full-green）是按输入哈希分格之前写的，不再认：它罩不到任何一批，可以删掉。
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"
[退出码 1]
[标记 1 个：singlefs-layer0-full-green]

════ 准备：--full 重写 A 那一格
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:13:43Z）：这批输入那一格旧标记已删（别的格不动）；这一道的输入哈希 2c406d1eaecd2706…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r2/selftest-repo/.git/singlefs-layer0-full-green.2c406d1eaecd27061928685e1828c386709d3806ed2a4aa16ad72183d8a8c3d4（输入哈希 2c406d1eaecd2706…，3 个文件；开跑 2026-09-24T00:13:43Z，跑完 2026-09-24T00:13:43Z；common-dir 里现有 1 格）
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
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2c406d1eaecd2706…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:13:43Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.2c406d1eaecd…]

════ --full 跑到第二条流时改了一个输入 ⇒ 判红、出路给 HEAD + 暂存区 worktree 的建法、不写这一格
$ env FAKE_CARGO_TOUCH_DURING_SECOND=/tmp/claude-1000/gate54-tiering/r2/selftest-repo/crates/singlefs-harness/src/lib.rs bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T00:13:43Z）：这批输入那一格旧标记已删（别的格不动）；这一道的输入哈希 2c406d1eaecd2706…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
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

  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 fff20e92dedbcecb…，5 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/selftest-repo/.git/singlefs-layer0-full-green.fff20e92dedbcecbc16c11abbbb7ff1513eb8d32550c1633035a348653bf0ad5）
       common-dir 里一格全绿标记都没有。
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"

──── 从出路句里抠出来的命令（原样照跑）：
layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"

Preparing worktree (detached HEAD da06d84)
HEAD is now at da06d84 base
  · --full 开跑（2026-09-24T00:13:43Z）：这批输入那一格旧标记已删（别的格不动）；这一道的输入哈希 5e3b302b7908e0d2…（4 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/r2/selftest-repo/.git/singlefs-layer0-full-green.5e3b302b7908e0d228d14029c3148766c03d9e2aefc0f764f61317a149a0f0b9（输入哈希 5e3b302b7908e0d2…，4 个文件；开跑 2026-09-24T00:13:43Z，跑完 2026-09-24T00:13:43Z；common-dir 里现有 1 格）
[出路命令的退出码 0]
[标记 1 个：singlefs-layer0-full-green.5e3b302b7908…]

════ 仿 gate.sh --staged：临时 worktree 里跑快档 ⇒ 判绿（工作区里别的会话的 other_session_wip.rs 与 Cargo.toml 改动都不在）
$ env SINGLEFS_GATE_FULL=1 bash /tmp/claude-1000/gate54-tiering/r2/selftest-staged-worktree/.claude/gate.d/54-layer0-replay.sh /tmp/claude-1000/gate54-tiering/r2/selftest-staged-worktree
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（5e3b302b7908e0d2…，4 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T00:13:43Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记 1 个：singlefs-layer0-full-green.5e3b302b7908…]

════ 对照：主工作区（带别的会话没暂存的改动）跑快档 ⇒ 判红
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 fff20e92dedbcecb…，5 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/r2/selftest-repo/.git/singlefs-layer0-full-green.fff20e92dedbcecbc16c11abbbb7ff1513eb8d32550c1633035a348653bf0ad5）
       common-dir 里最近写的一格是 singlefs-layer0-full-green.5e3b302b7908e0d228d14029c3148766c03d9e2aefc0f764f61317a149a0f0b9（跑完于 2026-09-24T00:13:43Z），与这一次的输入比：
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
/tmp/claude-1000/gate54-tiering/r2/selftest-repo  da06d84 [master]
```
