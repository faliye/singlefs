# defs54-r2 实现：判决第四节六条落进 54 号与 gate-triage（副本里改，交补丁）

判决：`research/prompts/defs-gate54-tiering-r2-main-verification.md` 第四节。补丁 `/tmp/claude-1000/defs54-r2-impl/defs54-r2.patch`（sha256 c5e5fdc9fb005957b05b960c86a1290c3a9a2e72e13351aaf202f128077997d6，184 行，两份文件），在主工作区 `git apply --check -v` 退出码 0。

## 起点与终点

| 文件 | 开工时主工作区 = 副本起点 sha256 | 改后 sha256 | 改后行数 |
|---|---|---|---|
| `.claude/gate.d/54-layer0-replay.sh` | 232c8c0de11bf6561f9ca1736652745f387d05832ad0a5c99a08bc80c0386bf7（与攻方 `inputs/54-post.sh` 相同） | f845fcd011526c9790450078245b769141867a8cfc8990f613e7fe5ad0520a5b | 438（原 397） |
| `.claude/agents/gate-triage.md` | e631bc7adf9345e3f9e74559c137cb97b07c9ce5f313811ab10b83358e3c6bb7 | 3663395ae0e6170c13f70eb0f81741dea3c958f01c3881e3afa6677b5353f564 | 39（原 39，改的是第 26 行那一整行） |

副本：`rsync -a --exclude /target` 主工作区（含 `.git`）到 `/tmp/claude-1000/defs54-r2-impl/repo`，删掉副本里被 `.gitignore` 挡着的 `research/target/`（7.7G，只删副本）。原样两份另存 `orig/`，补丁用 `diff -u --label a/… --label b/…` 从 `orig/` 对 `repo/` 生成；把补丁套回 `orig/` 的拷贝，两份 sha256 与上表改后一栏相同。收尾时主工作区两份 sha256 仍是起点那两个。

真仓 git common-dir（`/home/fy5090/code/singlefs/.git`）里 `singlefs-layer0*`：开工 0 个，收尾 0 个（`find … -maxdepth 1 -name 'singlefs-layer0*' | wc -l`）。

## 每一处改了什么（54 号）

1. **g1、g2（第四节第 1 条）**：`cd "$ROOT"` 之前取 `layer0_stage_script_path`（`$(cd "$(dirname "$0")" && pwd)/$(basename "$0")`）。`write_layer0_input_manifest` 在逐文件清单之后追加两行，格式与别的行相同（64 位 sha256、两个空格、名字，`report_manifest_differences` 从第 67 列取名字）：
   - `<跑的这一份 54 号的 sha256>  <判它的 54 号：54-layer0-replay.sh>`
   - `<sha256 of "$(cd "$ROOT" && cargo -V && rustc -V)" 加换行>  <工具链：cargo …；rustc …>`（名字里换行换成「；」，两行原样都看得见）。
   只取 stdout（rustup 往 stderr 打的 info 不进哈希）；`cargo -V` 或 `rustc -V` 退出码非 0、或输出为空 ⇒ 函数返回 1 ⇒ `fail_without_input_manifest` 判红，那条红句与出路各补了一句（「或 cargo -V / rustc -V 跑不出来」「在项目根跑 cargo -V && rustc -V … bash .claude/scripts/env.sh 看缺什么」）。报的文件数 `layer0_input_file_count` 仍只数登记路径下的文件。文件头「输入」一段与函数头注释跟着改；`--full` 开跑那句 `·` 行加「连同判它的 54 号与工具链」。
2. **g4（第 2 条）**：新变量 `full_input_changed_during_run=0`；「跑的过程中输入变了」那一支第一句把它置 1；EXIT trap 改成 `if [[ "$full_marker_written" != 1 && "$full_input_changed_during_run" != 1 ]]; then rm -f …; fi`。那一支的红句加「开跑那一批已有的那一格不删」；开跑的 `·` 行加「（跑的过程中输入变了的那一种红不删）」；trap 上方注释与文件头第 11–13 行（原第 11、12 行）改成：判红、被 TERM / INT / HUP 打断时删；「跑的过程中输入变了」那一支不删；同一批输入「连同判它的 54 号与工具链」的结果是确定的。
3. **h1（第 3 条）**：原「计数行共 3 行」+「exhaustive=true 共 2 行」两个总数判法换成：`LAYER0 `、`CHECKER `、`LAYER0B ` 三种开头各数一次，各必须恰好 1 行（红句报三个数）；再各取 `LAYER0` 与 `LAYER0B` 那一行，各自 `grep -qE '^LAYER0B? … exhaustive=true( |$)'`，缺的那条流在红句里点名。出路与原来相同（`print_staged_worktree_full_commands` 三行命令）。文件头快档那一段跟着改。
4. **h2（第 4 条）**：快档分支开头、复用那一问（原第 80 行）之前，逐条 `git -C "$ROOT" ls-files -co --exclude-standard -- <那一条>`，输出为空的收进一张表；有一条就判红（退出码 1），红句列出列不出的几条与整行登记，出路指到 `.claude/gate.d/stage-inputs.tsv` 并给复核命令。只放在快档（判决写的是快档）；`--full` 不做这一问。文件头快档那一段加一句。
5. 文件头「判别力」那一段（原第 38 行）补上这一次核过的几格及用的装置。

## gate-triage.md（第 6 条，h3）

第 3 步在「文件不在暂存区 ⇒『不是这一轮』」之后、「工具没装」之前插入：

> 红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；

「一条都没碰 ⇒ 不是这一轮」是判决没写的那一半，我补上是为了不让「不按点名的标记路径判」留下空档；它给出的归属与改前那句「文件不在暂存区 ⇒ 不是这一轮」对这类红给的相同，不改任何一格的结论。

## 自证一：g1、g2、g4 用攻方的 fix-arms.sh

`fix-arms.sh` 第 12 行自己把 `POST_54_OVERRIDE` 设成 `$MODEL/fixes/54-$fix.sh`，并把输出写进 `$MODEL/outputs/`，所以没法从外面指 `POST_54_OVERRIDE`，也不能在 `research/prompts/` 里跑。做法：把模型目录整份拷到 `/tmp/claude-1000/defs54-r2-impl/model`（`sha256sum -c --quiet sha256sums.txt` 通过），把改后的真文件拷成 `model/fixes/54-impl.sh`（sha256 f845fcd0…，与改后 54 号相同），原样 54 号拷成 `fixes/54-none.sh` 当改前对照；脚本一个字不改。

命令：`cd /tmp/claude-1000/defs54-r2-impl/model && FIXES="none g124 impl" RUNS_BASE=/tmp/claude-1000/defs54-r2-impl/runs nice -n 19 bash /tmp/claude-1000/defs54-r2-impl/model/fix-arms.sh`，退出码 0，原样：

```
改法 none 跑完：/tmp/claude-1000/defs54-r2-impl/model/outputs/fix-arms-none.out
改法 g124 跑完：/tmp/claude-1000/defs54-r2-impl/model/outputs/fix-arms-g124.out
改法 impl 跑完：/tmp/claude-1000/defs54-r2-impl/model/outputs/fix-arms-impl.out
fix-arms 退出码 0
```

逐格比对用 `/tmp/claude-1000/defs54-r2-impl/extract-cells.py`（按出现次序抽 `[gate --staged 里 54 号退出码 N]`、`[退出码 N]`（H-B2 主工作区直接跑）、`[真值：… 退出码 N]`）。期望取开工时从入库 `outputs/` 拷出的 `fix-arms-g124.out`（`model-outputs-before/`）：

- `python3 extract-cells.py model-outputs-before/fix-arms-g124.out model/outputs/fix-arms-impl.out` ⇒ 「期望 42 项；实测 42 项 … 对不上 0 项」，退出码 0。
- 同一脚本对副本重跑的 g124 ⇒ 对不上 0 项（这台机器上装置复现得出入库那一份）。
- 对改前的 54 号（none）⇒ 对不上 14 项：13 格门禁与真值不一致（H-A 3 格、H-B 4 格、H-B2 2 格误绿；H-C 4 格误红），外加 H-A 多线程对照那一格 none 判 0、g124 与 impl 判 1——那一格是判决第三节写的 g1 代价（X 那一趟是旧 54 号跑的，新 54 号没跑过全量，真值 0 而门禁要求再跑一趟），不是误判。

哈希与文件名不比：impl 的清单末两行名字与 g124 不同（多了 `rustc -V`），哈希数必然不同。

| # | 格 | 哪一次 | 入库 g124（期望） | 副本重跑 g124 | 改前 54 号（none） | 改后 54 号（impl） | impl 对上期望 |
|---|---|---|---|---|---|---|---|
| 1 | H-A [post] Y 在 T1 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED | 门禁#1 | 1 | 1 | 0 | 1 | ✓ |
| 2 | H-A [post] Y 在 T1 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED | 真值#1 | 1 | 1 | 1 | 1 | ✓ |
| 3 | H-A [post] Y 在 T2 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED | 门禁#1 | 1 | 1 | 0 | 1 | ✓ |
| 4 | H-A [post] Y 在 T2 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED | 真值#1 | 1 | 1 | 1 | 1 | ✓ |
| 5 | H-A [post] Y 在 T3 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED | 门禁#1 | 0 | 0 | 0 | 0 | ✓ |
| 6 | H-A [post] Y 在 T3 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED | 真值#1 | 0 | 0 | 0 | 0 | ✓ |
| 7 | H-A [post] Y 在 T3 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED | 门禁#2 | 1 | 1 | 0 | 1 | ✓ |
| 8 | H-A [post] Y 在 T3 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED | 真值#2 | 1 | 1 | 1 | 1 | ✓ |
| 9 | H-A [post] Y 在 T1 暂存 54；X 的 crash.rs 记号=多线程 | 门禁#1 | 1 | 1 | 0 | 1 | ✓ |
| 10 | H-A [post] Y 在 T1 暂存 54；X 的 crash.rs 记号=多线程 | 真值#1 | 0 | 0 | 0 | 0 | ✓ |
| 11 | H-B [post] 漂移=54 用户动作=during | 门禁#1 | 0 | 0 | 0 | 0 | ✓ |
| 12 | H-B [post] 漂移=54 用户动作=during | 门禁#2 | 0 | 0 | 0 | 0 | ✓ |
| 13 | H-B [post] 漂移=54 用户动作=during | 门禁#3 | 1 | 1 | 0 | 1 | ✓ |
| 14 | H-B [post] 漂移=54 用户动作=during | 真值#1 | 1 | 1 | 1 | 1 | ✓ |
| 15 | H-B [post] 漂移=54 用户动作=kill9 | 门禁#1 | 0 | 0 | 0 | 0 | ✓ |
| 16 | H-B [post] 漂移=54 用户动作=kill9 | 门禁#2 | 0 | 0 | 0 | 0 | ✓ |
| 17 | H-B [post] 漂移=54 用户动作=kill9 | 门禁#3 | 1 | 1 | 0 | 1 | ✓ |
| 18 | H-B [post] 漂移=54 用户动作=kill9 | 真值#1 | 1 | 1 | 1 | 1 | ✓ |
| 19 | H-B [post] 漂移=54 用户动作=finish | 门禁#1 | 0 | 0 | 0 | 0 | ✓ |
| 20 | H-B [post] 漂移=54 用户动作=finish | 门禁#2 | 0 | 0 | 0 | 0 | ✓ |
| 21 | H-B [post] 漂移=54 用户动作=finish | 门禁#3 | 1 | 1 | 1 | 1 | ✓ |
| 22 | H-B [post] 漂移=54 用户动作=finish | 真值#1 | 1 | 1 | 1 | 1 | ✓ |
| 23 | H-B [post] 漂移=toolchain 用户动作=during | 门禁#1 | 0 | 0 | 0 | 0 | ✓ |
| 24 | H-B [post] 漂移=toolchain 用户动作=during | 门禁#2 | 0 | 0 | 0 | 0 | ✓ |
| 25 | H-B [post] 漂移=toolchain 用户动作=during | 门禁#3 | 1 | 1 | 0 | 1 | ✓ |
| 26 | H-B [post] 漂移=toolchain 用户动作=during | 真值#1 | 1 | 1 | 1 | 1 | ✓ |
| 27 | H-B [post] 漂移=toolchain 用户动作=kill9 | 门禁#1 | 0 | 0 | 0 | 0 | ✓ |
| 28 | H-B [post] 漂移=toolchain 用户动作=kill9 | 门禁#2 | 0 | 0 | 0 | 0 | ✓ |
| 29 | H-B [post] 漂移=toolchain 用户动作=kill9 | 门禁#3 | 1 | 1 | 0 | 1 | ✓ |
| 30 | H-B [post] 漂移=toolchain 用户动作=kill9 | 真值#1 | 1 | 1 | 1 | 1 | ✓ |
| 31 | H-B2 [post] 工具链 1 下提交 A → 工具链升到 2 → SINGLEFS_GATE_FULL=1 重新验 HEAD | 门禁#1 | 0 | 0 | 0 | 0 | ✓ |
| 32 | H-B2 [post] 工具链 1 下提交 A → 工具链升到 2 → SINGLEFS_GATE_FULL=1 重新验 HEAD | 门禁#2 | 1 | 1 | 0 | 1 | ✓ |
| 33 | H-B2 [post] 工具链 1 下提交 A → 工具链升到 2 → SINGLEFS_GATE_FULL=1 重新验 HEAD | 主工作区直接跑#1 | 1 | 1 | 0 | 1 | ✓ |
| 34 | H-B2 [post] 工具链 1 下提交 A → 工具链升到 2 → SINGLEFS_GATE_FULL=1 重新验 HEAD | 真值#1 | 1 | 1 | 1 | 1 | ✓ |
| 35 | H-C [post] Z=wip 先后=Zlast | 门禁#1 | 0 | 0 | 1 | 0 | ✓ |
| 36 | H-C [post] Z=wip 先后=Zlast | 真值#1 | 0 | 0 | 0 | 0 | ✓ |
| 37 | H-C [post] Z=wip 先后=overlapZlast | 门禁#1 | 0 | 0 | 1 | 0 | ✓ |
| 38 | H-C [post] Z=wip 先后=overlapZlast | 真值#1 | 0 | 0 | 0 | 0 | ✓ |
| 39 | H-C [post] Z=edit 先后=Zlast | 门禁#1 | 0 | 0 | 1 | 0 | ✓ |
| 40 | H-C [post] Z=edit 先后=Zlast | 真值#1 | 0 | 0 | 0 | 0 | ✓ |
| 41 | H-C [post] Z=edit 先后=overlapZlast | 门禁#1 | 0 | 0 | 1 | 0 | ✓ |
| 42 | H-C [post] Z=edit 先后=overlapZlast | 真值#1 | 0 | 0 | 0 | 0 | ✓ |

改后 54 号上门禁与真值不一致的只有第 9、10 行那一格（门禁 1、真值 0），与入库 g124 相同，是判决认过的代价。

## 自证二：h1、h2，外加 g1、g2、g4 的直接一格

驱动 `/tmp/claude-1000/defs54-r2-impl/selftest-driver.sh`（照 `research/prompts/defs-gate54-tiering-implementer/selftest-driver.sh` 的做法），假 cargo 自造在 `/tmp/claude-1000/defs54-r2-impl/fake-bin/cargo`：抄攻方模型那一份（认 `-V`，版本号取 `$FAKE_TOOLCHAIN_FILE`），加 `FAKE_CARGO_TOUCH_DURING_SECOND` 与 `FAKE_CARGO_FAIL` 两个开关；`/tmp/claude-1000/gate54-tiering/fake-bin/cargo` 在，但它不认 `-V`。临时仓在 `/tmp/claude-1000/defs54-r2-impl/selftest/repo-old`（原样 54 号）与 `repo-new`（改后 54 号），各自的 common-dir；`stage-inputs.tsv`、`stage-must-run.sh`、`change-touches-crates.sh` 取副本里的（与主工作区相同）。同一串步骤两份 54 号各跑一遍，所以每一格都有「改前」对照。

命令：`bash /tmp/claude-1000/defs54-r2-impl/selftest-driver.sh > selftest-output.txt`，退出码 0。逐格结论：

| 格 | 改前 54 号 | 改后 54 号 |
|---|---|---|
| h1 红：标记里 LAYER0 抄两遍、没有 LAYER0B（哈希行不动） | 0（判绿，就是 V3DUPLICATE） | **1**（「LAYER0 2 行、CHECKER 1 行、LAYER0B 0 行」） |
| h1 改回：标记还原 | 0 | 0 |
| h2-a：三条全写错 + 这一批改了 crates/，不设 SINGLEFS_GATE_FULL | 77（「没碰」，就是 V4WRONGPATH） | **1**（列出 3 条） |
| h2-a 同上，SINGLEFS_GATE_FULL=1 | 1（算不出哈希） | 1（h2 那一句） |
| h2-b：只把 Cargo.lock 写成 Cargo.lok + 这一批只改 Cargo.lock | 77 | **1**（列出 Cargo.lok） |
| h2 改回：表与 Cargo.lock 还原，SINGLEFS_GATE_FULL=1 | 0 | 0 |
| h2 改回：不设 SINGLEFS_GATE_FULL，干净工作树 | 77 | 77（改动范围那一问照旧） |
| g1：只给 54 号末尾加一行注释 | 0 | **1**（「内容不同：<判它的 54 号：54-layer0-replay.sh>」） |
| g1 改回 | 0 | 0 |
| g2：假工具链 1 ⇒ 2，仓里一字节不动 | 0 | **1**（「只在这一次里：<工具链：cargo 1.99.0 (fake toolchain 2)；rustc 1.98.0 …>」与「只在那一格里：…toolchain 1…」） |
| g2 改回 | 0 | 0 |
| g4：--full 跑到第二条流时 crates/ 一个文件多一字节 | 1，那一格被删 | 1，**那一格留着** |
| g4 之后字节改回，快档 | 1（没有那一格） | **0** |
| 对照：--full 里 cargo 判红（输入没变） | 1，那一格被删 | 1，那一格被删（别的红照旧删） |
| 对照之后快档 | 1 | 1 |

原样输出（`selftest-output.txt` 全文，行尾截在 260 列）：

```
════════════════ 54 号 old（232c8c0de11bf656…）
──── 准备：--full 全绿，写这批输入那一格
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T02:28:55Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 2175faa5f77d0325…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 viol
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persist
  ✓ 全绿标记写进 /tmp/claude-1000/defs54-r2-impl/selftest/repo-old/.git/singlefs-layer0-full-green.2175faa5f77d03252ed4337b11d24680823c2e45af1db92507418403fed5561c（输入哈希 2175faa5f77d0325…，3 个文件；开跑 2026-09-24T02:28:55Z，跑完 
[退出码 0]
    [格] 2175faa5f77d03252e
──── 基线：快档（SINGLEFS_GATE_FULL=1 越过两问）⇒ 判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2175faa5f77d0325…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:28:59Z，标记里的计数�
[退出码 0]
    [格] 2175faa5f77d03252e
════ h1：标记里 LAYER0 抄两遍、没有 LAYER0B
    改后标记里的计数行：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=2
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_
──── h1 红：快档读改过的标记
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2175faa5f77d0325…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:28:59Z，标记里的计数�
[退出码 0]
    [格] 2175faa5f77d03252e
──── h1 改回：标记还原 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2175faa5f77d0325…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:28:59Z，标记里的计数�
[退出码 0]
    [格] 2175faa5f77d03252e
════ h2-a：登记的三条路径全写错，这一批又改了 crates/
    stage-inputs.tsv 里 54 号那一行现在是：54-layer0-replay.sh crate/ Cargo.tom Cargo.lok
    工作区相对 HEAD： M .claude/gate.d/stage-inputs.tsv  M crates/singlefs-harness/src/lib.rs 
──── h2-a 红：快档（不设 SINGLEFS_GATE_FULL，照整轮门禁的样子）
$ bash .claude/gate.d/54-layer0-replay.sh
  ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 2 个改动路径，没有一个落在 crate/ Cargo.tom Cargo.lok 底下
     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 crate/ Cargo.tom Cargo.lok，判据见 research/scripts/change-touches-crates.sh。
[退出码 77]
    [格] 2175faa5f77d03252e
──── h2-a 红：快档（SINGLEFS_GATE_FULL=1）
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 算不出这一道输入的内容哈希：git 在登记的路径（crate/ Cargo.tom Cargo.lok）下一个文件都列不出来，或读文件出错
     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- crate/ Cargo.tom Cargo.lok，看列不列得出文件；
[退出码 1]
    [格] 2175faa5f77d03252e
════ h2-b：只写错一条（Cargo.lock 写成 Cargo.lok），这一批只改了 Cargo.lock
    stage-inputs.tsv 里 54 号那一行现在是：54-layer0-replay.sh crates/ Cargo.toml Cargo.lok
    工作区相对 HEAD： M .claude/gate.d/stage-inputs.tsv  M Cargo.lock 
──── h2-b 红：快档（不设 SINGLEFS_GATE_FULL）
$ bash .claude/gate.d/54-layer0-replay.sh
  ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 2 个改动路径，没有一个落在 crates/ Cargo.toml Cargo.lok 底下
     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 crates/ Cargo.toml Cargo.lok，判据见 research/scripts/change-touches-crates.sh。
[退出码 77]
    [格] 2175faa5f77d03252e
    改回之后工作区相对 HEAD：0 行 status
──── h2 改回：登记表与 Cargo.lock 还原 ⇒ 快档判绿（SINGLEFS_GATE_FULL=1）
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2175faa5f77d0325…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:28:59Z，标记里的计数�
[退出码 0]
    [格] 2175faa5f77d03252e
──── h2 改回：不设 SINGLEFS_GATE_FULL ⇒ 干净工作树，改动范围那一问答「没碰」退 77
$ bash .claude/gate.d/54-layer0-replay.sh
  ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间一个改动都没有（干净工作树；要重新验 HEAD 用 SINGLEFS_GATE_FULL=1）
     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 crates/ Cargo.toml Cargo.lock，判据见 research/scripts/change-touches-crates.sh。
[退出码 77]
    [格] 2175faa5f77d03252e
════ g1：只改判它的 54 号（文件末尾加一行注释），crates/ 不动
──── g1 红：快档
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2175faa5f77d0325…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:28:59Z，标记里的计数�
[退出码 0]
    [格] 2175faa5f77d03252e
──── g1 改回：54 号还原 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2175faa5f77d0325…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:28:59Z，标记里的计数�
[退出码 0]
    [格] 2175faa5f77d03252e
════ g2：只换工具链（假 cargo -V 从 toolchain 1 变成 2），仓里一个字节不动
──── g2 红：快档
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2175faa5f77d0325…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:28:59Z，标记里的计数�
[退出码 0]
    [格] 2175faa5f77d03252e
──── g2 改回：工具链回到 1 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（2175faa5f77d0325…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:28:59Z，标记里的计数�
[退出码 0]
    [格] 2175faa5f77d03252e
════ g4：--full 跑到第二条流时 crates/ 里一个文件被追加一个字节（这批输入那一格已在）
──── g4：--full，跑的过程中输入变了 ⇒ 判红
$ env FAKE_CARGO_TOUCH_DURING_SECOND=/tmp/claude-1000/defs54-r2-impl/selftest/repo-old/crates/singlefs-harness/src/lib.rs bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T02:29:06Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 2175faa5f77d0325…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 viol
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persist
  ✗ 全量跑的过程中这一道的输入变了（开跑 2175faa5f77d0325…，跑完 8082248ea02e5ffe…）：两条流读到的不一定是同一版，不写全绿标记
       不同的文件共 1 个（最多列 20 个）：
         内容不同：crates/singlefs-harness/src/lib.rs
     → 怎么办：别在有人改这些路径的树里跑。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
[退出码 1]
    [格] 一格都没有
──── g4 之后：那个字节改回，快档读开跑那一批那一格
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2175faa5f77d0325…，3 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-r2-impl/
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
[退出码 1]
    [格] 一格都没有
──── 对照：--full 里 cargo 判红（输入没变）⇒ 判红、照旧删这批输入那一格
$ env FAKE_CARGO_FAIL=1 bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T02:29:06Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 2175faa5f77d0325…（3 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）
     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture
[退出码 1]
    [格] 一格都没有
──── 对照之后：快档 ⇒ 判红（这批输入没有格了）
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 2175faa5f77d0325…，3 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-r2-impl/
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
[退出码 1]
    [格] 一格都没有

════════════════ 54 号 new（f845fcd011526c97…）
──── 准备：--full 全绿，写这批输入那一格
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T02:29:06Z）：不删任何一格，这一趟判红才删这批输入那一格（跑的过程中输入变了的那一种红不删）；这一道的输入哈希 ff7c9ca4e8c6ca9d…（3 个文件，登记路径 crates/ Cargo.toml 
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 viol
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persist
  ✓ 全绿标记写进 /tmp/claude-1000/defs54-r2-impl/selftest/repo-new/.git/singlefs-layer0-full-green.ff7c9ca4e8c6ca9dbf96f70c2136cf94a36c2abcef78c83e89f8333d5abf3ed2（输入哈希 ff7c9ca4e8c6ca9d…，3 个文件；开跑 2026-09-24T02:29:06Z，跑完 
[退出码 0]
    [格] ff7c9ca4e8c6ca9dbf
──── 基线：快档（SINGLEFS_GATE_FULL=1 越过两问）⇒ 判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（ff7c9ca4e8c6ca9d…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:29:06Z，标记里的计数�
[退出码 0]
    [格] ff7c9ca4e8c6ca9dbf
════ h1：标记里 LAYER0 抄两遍、没有 LAYER0B
    改后标记里的计数行：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=2
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_
──── h1 红：快档读改过的标记
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 这批输入那一格全绿标记的哈希对得上，计数行却不是 LAYER0 / CHECKER / LAYER0B 各恰好一行（LAYER0 2 行、CHECKER 1 行、LAYER0B 0 行）：标记不是 --full 写的，或被改过
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
[退出码 1]
    [格] ff7c9ca4e8c6ca9dbf
──── h1 改回：标记还原 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（ff7c9ca4e8c6ca9d…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:29:06Z，标记里的计数�
[退出码 0]
    [格] ff7c9ca4e8c6ca9dbf
════ h2-a：登记的三条路径全写错，这一批又改了 crates/
    stage-inputs.tsv 里 54 号那一行现在是：54-layer0-replay.sh crate/ Cargo.tom Cargo.lok
    工作区相对 HEAD： M .claude/gate.d/stage-inputs.tsv  M crates/singlefs-harness/src/lib.rs 
──── h2-a 红：快档（不设 SINGLEFS_GATE_FULL，照整轮门禁的样子）
$ bash .claude/gate.d/54-layer0-replay.sh
  ✗ stage-inputs.tsv 登记给 54-layer0-replay.sh 的路径里，有 3 条 git 一个文件都列不出来：crate/ Cargo.tom Cargo.lok（登记的是 crate/ Cargo.tom Cargo.lok）
     → 怎么办：多半是 .claude/gate.d/stage-inputs.tsv 里那一条写错了（拼错、目录少了斜杠、指到不存在的路径）；
[退出码 1]
    [格] ff7c9ca4e8c6ca9dbf
──── h2-a 红：快档（SINGLEFS_GATE_FULL=1）
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ stage-inputs.tsv 登记给 54-layer0-replay.sh 的路径里，有 3 条 git 一个文件都列不出来：crate/ Cargo.tom Cargo.lok（登记的是 crate/ Cargo.tom Cargo.lok）
     → 怎么办：多半是 .claude/gate.d/stage-inputs.tsv 里那一条写错了（拼错、目录少了斜杠、指到不存在的路径）；
[退出码 1]
    [格] ff7c9ca4e8c6ca9dbf
════ h2-b：只写错一条（Cargo.lock 写成 Cargo.lok），这一批只改了 Cargo.lock
    stage-inputs.tsv 里 54 号那一行现在是：54-layer0-replay.sh crates/ Cargo.toml Cargo.lok
    工作区相对 HEAD： M .claude/gate.d/stage-inputs.tsv  M Cargo.lock 
──── h2-b 红：快档（不设 SINGLEFS_GATE_FULL）
$ bash .claude/gate.d/54-layer0-replay.sh
  ✗ stage-inputs.tsv 登记给 54-layer0-replay.sh 的路径里，有 1 条 git 一个文件都列不出来：Cargo.lok（登记的是 crates/ Cargo.toml Cargo.lok）
     → 怎么办：多半是 .claude/gate.d/stage-inputs.tsv 里那一条写错了（拼错、目录少了斜杠、指到不存在的路径）；
[退出码 1]
    [格] ff7c9ca4e8c6ca9dbf
    改回之后工作区相对 HEAD：0 行 status
──── h2 改回：登记表与 Cargo.lock 还原 ⇒ 快档判绿（SINGLEFS_GATE_FULL=1）
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（ff7c9ca4e8c6ca9d…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:29:06Z，标记里的计数�
[退出码 0]
    [格] ff7c9ca4e8c6ca9dbf
──── h2 改回：不设 SINGLEFS_GATE_FULL ⇒ 干净工作树，改动范围那一问答「没碰」退 77
$ bash .claude/gate.d/54-layer0-replay.sh
  ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间一个改动都没有（干净工作树；要重新验 HEAD 用 SINGLEFS_GATE_FULL=1）
     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 crates/ Cargo.toml Cargo.lock，判据见 research/scripts/change-touches-crates.sh。
[退出码 77]
    [格] ff7c9ca4e8c6ca9dbf
════ g1：只改判它的 54 号（文件末尾加一行注释），crates/ 不动
──── g1 红：快档
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 70dfbb62a71157af…，3 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-r2-impl/
       不同的文件共 1 个（最多列 20 个）：
         内容不同：<判它的 54 号：54-layer0-replay.sh>
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
[退出码 1]
    [格] ff7c9ca4e8c6ca9dbf
──── g1 改回：54 号还原 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（ff7c9ca4e8c6ca9d…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:29:06Z，标记里的计数�
[退出码 0]
    [格] ff7c9ca4e8c6ca9dbf
════ g2：只换工具链（假 cargo -V 从 toolchain 1 变成 2），仓里一个字节不动
──── g2 红：快档
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 12e2db433b66228b…，3 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-r2-impl/
       不同的文件共 2 个（最多列 20 个）：
         只在这一次里：<工具链：cargo 1.99.0 (fake toolchain 2)；rustc 1.98.0 (88d9e12ae 2026-08-18)>
         只在那一格里：<工具链：cargo 1.99.0 (fake toolchain 1)；rustc 1.98.0 (88d9e12ae 2026-08-18)>
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
[退出码 1]
    [格] ff7c9ca4e8c6ca9dbf
──── g2 改回：工具链回到 1 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（ff7c9ca4e8c6ca9d…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:29:06Z，标记里的计数�
[退出码 0]
    [格] ff7c9ca4e8c6ca9dbf
════ g4：--full 跑到第二条流时 crates/ 里一个文件被追加一个字节（这批输入那一格已在）
──── g4：--full，跑的过程中输入变了 ⇒ 判红
$ env FAKE_CARGO_TOUCH_DURING_SECOND=/tmp/claude-1000/defs54-r2-impl/selftest/repo-new/crates/singlefs-harness/src/lib.rs bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T02:29:11Z）：不删任何一格，这一趟判红才删这批输入那一格（跑的过程中输入变了的那一种红不删）；这一道的输入哈希 ff7c9ca4e8c6ca9d…（3 个文件，登记路径 crates/ Cargo.toml 
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 viol
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persist
  ✗ 全量跑的过程中这一道的输入变了（开跑 ff7c9ca4e8c6ca9d…，跑完 33237ec21ae56deb…）：两条流读到的不一定是同一版，不写全绿标记；开跑那一批已有的那一格不删
       不同的文件共 1 个（最多列 20 个）：
         内容不同：crates/singlefs-harness/src/lib.rs
     → 怎么办：别在有人改这些路径的树里跑。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
[退出码 1]
    [格] ff7c9ca4e8c6ca9dbf
──── g4 之后：那个字节改回，快档读开跑那一批那一格
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（ff7c9ca4e8c6ca9d…，3 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T02:29:06Z，标记里的计数�
[退出码 0]
    [格] ff7c9ca4e8c6ca9dbf
──── 对照：--full 里 cargo 判红（输入没变）⇒ 判红、照旧删这批输入那一格
$ env FAKE_CARGO_FAIL=1 bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-24T02:29:11Z）：不删任何一格，这一趟判红才删这批输入那一格（跑的过程中输入变了的那一种红不删）；这一道的输入哈希 ff7c9ca4e8c6ca9d…（3 个文件，登记路径 crates/ Cargo.toml 
  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）
     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture
[退出码 1]
    [格] 一格都没有
──── 对照之后：快档 ⇒ 判红（这批输入没有格了）
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 ff7c9ca4e8c6ca9d…，3 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-r2-impl/
     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）：
[退出码 1]
    [格] 一格都没有

```

## 静态检查（都在副本里跑）

| 命令 | 退出码 | 原样成功句 / 说明 |
|---|---|---|
| `bash -n .claude/gate.d/54-layer0-replay.sh` | 0 | |
| `GATE_LINT_DIR=.claude/gate.d bash .claude/singlefs-ai-sop/scripts/gate-lint.sh` | 0 | `✓ 门禁自检通过：83 个脚本（.sh 与 .py）、228 条拒绝都带了出路` |
| `SHELL_LINT_DIR=.claude/gate.d bash .claude/singlefs-ai-sop/scripts/shell-lint.sh` | 0 | `✓ shell 纪律检查通过（共 76 个脚本）` |
| `bash gate-lint.sh "$PWD/.claude/gate.d"`（`gate.sh` 的调法，连上游包一起扫） | 0 | `✓ 门禁自检通过：109 个脚本（.sh 与 .py）、397 条拒绝都带了出路` |
| `bash shell-lint.sh "$PWD/.claude/gate.d"`（同上） | 0 | `✓ shell 纪律检查通过（共 100 个脚本）` |
| `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh "$PWD"` | 1 | 汇总句原样：`✗ 文档铁律检查失败：2 个文件违规、0 处编号引用无定义、12 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 483，跳过 0）`；✗ 行点名的是 kb 文件（`experiments-history.md`、`experiments.md`、`experiments/156-…`、`158-…`、`160-…`、`milestone/02-second-txn.md`）的编号简称，全文不提 `gate-triage`。把两份文件换回原样再跑一遍，输出逐字相同（「检查 N」那一处先抹掉再比，两次都是 483）：红不是这一次带进来的 |
| rules-lint（照 `gate.sh`「规则纪律（项目本地）」：`GATE_IN_STAGE=1 RULES_LINT_DIR=$PWD/.claude/rules RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md .claude/skills/*/SKILL.md" bash rules-lint.sh "$PWD"`） | 0 | `✓ 规则只写怎么做（扫了 29 份文件 1713 行；没扫 0 个；记录小节 0、论证小节 0、带日期的行 0 … 解释性段落 0、解释性半句 0、没带劝阻句的链接 0 …）` |
| `bash .claude/gate.d/62-stage-owners.sh "$PWD"` | 0 | `✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）` |
| `bash .claude/gate.d/63-agent-write-scope.sh "$PWD"` | 0 | `✓ 写范围闸、Bash 检出 hook、续派闸与续做闸注册着、自证通过 … 表与定义一致（3 个有 Write 或 Edit 的定义、18 条路径模式）` |

## 没做什么

- 没真跑层 0 全量，没跑真 `cargo test`，没跑整轮门禁；真 cargo 只跑过一次 `cargo -V && rustc -V`（读版本，主工作区，只读）。
- 真仓 common-dir 一格标记都没写（开工 0、收尾 0）；没在真仓建 worktree（`git worktree list` 里现有的几棵都不是这一次的路径）。
- 没改判决、没改 `research/prompts/` 下任何文件：主工作区 `defs-gate54-tiering-r2-opus-model/` 收尾时 `sha256sum -c --quiet sha256sums.txt` 通过，`outputs/` 与开工时拷出的 `model-outputs-before/` `diff -rq` 无差。fix-arms 在模型的拷贝里跑。
- 没提交、没建分支、没动主工作区；没用 `pkill -f`、`pgrep -f` 与 git 的撤销命令。
- h2 只加在快档（判决原文「同一份的快档」）；`--full` 遇到写错的登记仍会照登记算哈希、写一格，那一格快档读不到（快档先在 h2 判红）。
- 标记里 `CHECKER`、线程数、`input_file`、`finished_utc` 被手改快档照绿，判决第五节记账不改，这里也没改。

## 顺带看到的（不改，交主 agent 判）

- 只改 54 号、不碰 `crates/ Cargo.toml Cargo.lock` 的一批，快档在改动范围那一问退 77，新 54 号当场不被要求跑全量；下一批碰了 crates/ 时，因为 g1 把 54 号放进了键，快档会要求用新 54 号跑一次全量。不误绿，只是推迟。这是改动范围那一问的现有设计，g1 不改它。
- g2 新增一种快档判红：工具链跑不出 `cargo -V` / `rustc -V` 时，快档在算哈希那一步就红（改前是跑到 `cargo test` 才红），红句与出路已补。

## 文件

- 补丁：`/tmp/claude-1000/defs54-r2-impl/defs54-r2.patch`
- 改后的两份：`/tmp/claude-1000/defs54-r2-impl/repo/.claude/gate.d/54-layer0-replay.sh`、`/tmp/claude-1000/defs54-r2-impl/repo/.claude/agents/gate-triage.md`；原样两份 `/tmp/claude-1000/defs54-r2-impl/orig/`
- 自证：`selftest-driver.sh`（466f6cab…）、`fake-bin/cargo`（87992999…）、`selftest-output.txt`、`extract-cells.py`（2ef22d5d…）、`fix-arms-table.md`、`model/outputs/fix-arms-{none,g124,impl}.out`、`model-outputs-before/`，都在 `/tmp/claude-1000/defs54-r2-impl/` 下
- 静态检查原样：`doc-lint-before.txt`、`doc-lint-after.txt`、`rules-lint-after.txt`、`gate62-after.txt`、`gate63-after.txt`
