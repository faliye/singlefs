# 第九批 landing-audit-b9-defs-verify：定义改法与合入后验证加 checker 档快档（tooling-writer，2026-10-23）

依据：`records/2026-09-16-subagent拆分提案.md` 第九十节 ⑥、⑦、⑧、⑲、㉒、㉓、㉔ 七行；用户定案在 `research/prompts/landing-audit-r3-main-verification.md` 节「用户定案（2026-10-23，弹窗）」。

## 结论

派发提示列的六条都做完了，改动都已定点换进主工作区，换上之后在主工作区验过。有一处要主 agent 接着办（下面第 1 点）：

1. ⑧ 写范围表加了 `**/*.md` 一行。`.claude/hooks/write-guard.sh` 的自证里有一格还按旧表断言，所以它的 `--selftest` 现在红 1 格：「✗ 自检：范围:kb-scribe 不变量清单的子目录越界 应当是 2，实际 0」。这份钩子不在我的写范围里，没改。我起草了一份定点规格，在草稿里用 `hook-swap.py --dry-run` 验过、没换上，命令见「要主 agent 跑的命令」第 1 条。
2. ㉓ `scripts/gate/merge-and-verify.sh` 在编译、测试与重编核对之后加了一步「checker 档快档」：
   - 起门禁 `checker-tier-crash-point-replay.sh --check crash-case-replay,feature-gated-tests <树>`。键怎么算、复用怎么做，全由那一道现读，这里不另算一份键。
   - 起它的命令要带 `SINGLEFS_HEAVY_TESTS=user-request` 或 `=commit`；没带就不起，记一项红，给出路。
   - 补丁没碰 checker 档包与它的依赖闭包时，报「本次未跑」并写明原因。
   - 红了逐条列出 FAILED 的测试函数和变了的单元，给只重跑那几格的整条命令，整条退 1。
   - 自证从 38 种加到 45 种，全绿。新加 7 个弄坏开关，逐个打开都红在点名的格。
3. ㉒ `.claude/main-agent.md` 第 8 条加了闭窗口结局账的命令，`--out` 落在 `records/<轮>-outcome-<这一天>.txt`。
4. ⑥ ⑦ `.claude/agent-common.md` 交回正文闸那一条补上了交回闸的 ⑤、⑥ 两样。
5. ⑲ `.claude/gate.d/break-switch-replay-files.tsv` 加了两行。加之前照那一格的读法逐条复跑了 100 条开关，都记「对」。
6. ㉔ `.claude/agents/implementation-writer.md` 第 4 步「验证只到这几样」的列举补上了两条命名检查；同一行后面「除外」那一处改成指这同一样。

什么现象会推翻这些结论：
- ㉓：在主工作区带前缀跑 merge-and-verify，补丁改了一条快档测试函数闭包里的函数，checker 档快档那一步却报「本次未跑」或「现判 0」；或者那一道判红，这一步却报过、退 0；或者补丁只改了 checker 档依赖闭包之外的包，这一步却起了那一道。
- ⑧：kb-scribe 写 `crates/singlefs-checker/invariants/<子目录>/x.md` 还被写闸拒；或者写同一处的 `.rs` 被放行。
- ⑲：门禁 code-tooling 的 break-switch-replay 格复跑这两份时，有一条记成「不符」「没红」或「超时」。

## 改过的文件

| 文件 | 改法 | 条 |
|---|---|---|
| `scripts/gate/merge-and-verify.sh` | 草稿改完，按基线到草稿的差出三份定点规格（`mav-spec-1/2/3.json`，15 + 2 + 3 处），每次都先核主工作区与上一版逐字相同，再用 `scripts/kb/replace-batch.py` 换上，换完核主工作区与草稿逐字相同 | ㉓ |
| `.claude/hooks/agent-write-scope.tsv` | `scripts/gate/hook-swap.py`，喂 `write-guard.sh`；草稿里验过、换上之后主工作区里再验过 | ⑧ |
| `.claude/agents/kb-scribe.md` | `replace-once.py` 两处（第 26 行、第 42 行） | ⑧ |
| `.claude/main-agent.md` | `replace-once.py` 三处（第 8 条、第 10 条、验证段第 1 步） | ㉒ ㉓ |
| `.claude/agent-common.md` | `replace-once.py` 一处（「执行前拒绝的写法」交回正文闸那一条） | ⑥ ⑦ |
| `.claude/agents/implementation-writer.md` | `replace-once.py` 一处（第 4 步） | ㉔ |
| `.claude/gate.d/break-switch-replay-files.tsv` | Edit 两处，各插一行 | ⑲ |

`git diff --stat -- <这七份>` 原样（工作区对暂存区）：

```
 .claude/agent-common.md                      |   4 +-
 .claude/agents/implementation-writer.md      |   2 +-
 .claude/agents/kb-scribe.md                  |   4 +-
 .claude/gate.d/break-switch-replay-files.tsv |   3 +
 .claude/hooks/agent-write-scope.tsv          |   1 +
 .claude/main-agent.md                        |   8 +-
 scripts/gate/merge-and-verify.sh             | 308 ++++++++++++++++++++++++---
 7 files changed, 287 insertions(+), 43 deletions(-)
```

上表里有别的会话的改动，不是这一批的，我没碰：
- `.claude/main-agent.md` 第 11 条，加了 run-build-directory.sh 那几句；
- `.claude/agent-common.md` 第 85 行，主工作区里起 cargo 那一句；
- `break-switch-replay-files.tsv` 的 `scripts/hosts/run-build-directory.sh` 一行。

这三处都是我开工之后才出现的：拿我开工时留的底（`base/originals/`）逐份比过。我自己的改动核过都还在。

终版 sha256（主工作区）：

```
3aa8c2adad2356f358223fbf7a03307165331897baedc715283836237f0ee32b  .claude/main-agent.md
1fe5d1cc589b34b0f9694d382c53d793e985fe033aece0237cef5aa475cd5a57  .claude/agent-common.md
2cdfabcaa1122088e4f3830955e24faca9148bf579eec69dacd9e1276e6aea0f  .claude/agents/implementation-writer.md
cca39fa9145eb2251088841021aecdf6d906cad7cf9a2b2b5b864d0fdd78b300  .claude/agents/kb-scribe.md
f2ebff14ec69546a6a84f181c82b985c149287332450181fe3fc05ebf4a79162  .claude/hooks/agent-write-scope.tsv
0f8bacd75e0db06b4c03735ca9824b8d2bdcdc049b50c43acbd83a74233ecdcb  scripts/gate/merge-and-verify.sh
390e93c5e2e712424bc1eeb50d787e67b6081a9e3b22c12e34aaf684037271c0  .claude/gate.d/break-switch-replay-files.tsv
```

开工时的 sha256 在草稿目录的 `base-sha256.txt`，原文件留底在 `base/originals/`（`copy-originals.py`）。`break-switch-replay-files.tsv` 的那一份留底是在我插了第一行之后才拷的，所以它的改前样子要看 `git diff`：我加的只有那两行。

## ㉓ 合入后验证加 checker 档快档那一步

### 做法（`scripts/gate/merge-and-verify.sh`）

- 文件头新加「checker 档快档」一段（第 99 行起），写全了判法、本次未跑的几种、出路，以及两个只给自证用的变量：`MERGE_AND_VERIFY_CHECKER_QUICK_STAGE` 换掉那一道，`MERGE_AND_VERIFY_CHECKER_QUICK_ROOT` 换掉交给它的根。用法首行、test 一段说 checker 档包归谁的那一句、退出码一段，也同步改了。
- 起不起，在 `select_test_packages` 那段 python 里同一趟判，函数是 `checker_verdict`（第 457 行）：
  - 给了补丁目录时都判，带 `--all-packages` 也判；
  - 照跑的几种：改到工作区根的 `Cargo.toml`、`Cargo.lock`、`.cargo/`；改到不落在任何包里的路径；补丁目录里有非空的 `mutations-append.tsv`；读不出成员清单；改到的包落在 checker 档包的依赖闭包里（成员 Cargo.toml 的三张依赖表，连同 `[target.*]` 下的，一层层往下找）；
  - 改到的包都不在闭包里，或补丁目录里没有改到文件的 `crates.patch`，判「本次未跑」；
  - 没给补丁目录的，照跑。
  这段 python 原来那几条路（ALL、改到的包、依赖它们的包）的输出不变，只多打一行 `CHECKER<TAB>…`，bash 那一侧读完就删掉这一行。
- `checker_quick_step`（第 667 行），在 `verify()` 里重编核对之后调（第 888 行），次序如下：
  1. `--peer` 报本次未跑，`--copy` 也报本次未跑（副本没有 `.git`，判定库读不到）；
  2. 补丁没碰的，报本次未跑；
  3. 没带前缀的，记红并给出路；
  4. 都过了，经 `run_capped`（内存包装、线程上限、两份日志）起 `env -u MERGE_AND_VERIFY_BREAK bash <门禁> --check crash-case-replay,feature-gated-tests <树>`；
  5. 结局：退 0 算过；退 3 只在交给它的根有 `.crash-point-replay-sample-tools/quick-tier` 时算过；退 77 报本次未跑；退 252、245 照内存包装的两种结局办；别的退出码判红。读不到那一道打的「按条复用」那一行，也不算过。
- 成功句把两趟「按条复用」那一行里的现判、复用、拒算加起来报。红了逐条列出 FAILED 的测试函数，连同那一道报的「红的 …」行（各自变了的单元）与 `error: test failed, to rerun pass` 行，出路是 `checker_quick_rerun_command`（第 657 行）打的整条命令，例：

```
SINGLEFS_HEAVY_TESTS=user-request SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-tier-crash-point-replay.sh 的 crash-case-replay 格; 范围=1 格; 为什么=合入后验证这几格红了，修完只重跑它们：键相同的照复用，只现判红的那几条与改后键变了的' bash <仓根>/scripts/hosts/capped.sh <N> bash <仓根>/scripts/hosts/run-with-memory-cap.sh <上限> bash <仓根>/.claude/gate.d/checker-tier-crash-point-replay.sh --check crash-case-replay <树>
```

  这条命令按复用来理解就是只重跑那几条：判绿的照键记过库，红的没记，下一趟只现判红的那几条与改后键变了的。
- 汇总行末尾加了一句「checker 档快档 <过（现判 N、复用 M）/ 红（…）/ 本次未跑（…）>」。

### 样本（真的门禁、假 cargo）

样本用的是门禁 checker-tier-crash-point-replay 自带的两份样本：`unit-key-samples-green` 与 `unit-key-samples-red`。照它们的 `setup.sh` 搭好，上一版记进判定库，这一版改了 4 条快档测试函数闭包里的单元；`-red` 让 test_t 判 FAILED。

开工时先在草稿里直接对这两份样本根起了一次那一道，看输出长什么样（命令在样本根里起，根参数写同一个目录），原样存在草稿目录：

- 绿，头一趟（`stage-sample-green.run1.out`，退 3：样本档判绿）：
  ```
    · 快档按条复用：checker 档包 8 条编得进的测试函数，现判 4、复用 4、拒算 0；这一组特性下不编 0 条
      CRASH_POINT_REPLAY_PROGRESS sample-cargo-arguments: test --release --no-fail-fast -p singlefs-checker-tier --lib --tests -- --nocapture --exact --skip f1_first --skip f1_last --skip h3_test --skip test_v
  ```
- 绿，同一个根第二趟（`stage-sample-green.run2.out`，退 3）：
  ```
    · 快档按条复用：checker 档包 8 条编得进的测试函数，现判 0、复用 8、拒算 0；这一组特性下不编 0 条
    ✓ checker 档快档跑完（release，只跑不标 ignored 的用例，全量留给 --full）：checker 档包 8 条测试函数都有键相同的判定，全部复用，这一趟没起 cargo（现判 0、复用 8、拒算 0、本次未跑 0）
  ```
- 红（`stage-sample-red.run1.out`，退 1）：
  ```
      test test_t ... FAILED
      红的 crates/singlefs-checker-tier/tests/setting.rs::test_t：键对不上（上一次 2026-10-23 判绿），变了 1 处：变了 fn crates/singlefs-checker-tier/tests/setting.rs::observe
    ✗ 快档有 1 格判红：crash-case-replay（各自的出路在上面）
  ```

merge-and-verify 这一侧的红绿，由自证新加的格拿这两份样本根真起那一道来核（下一节）。merge-and-verify 本身是 whole，子 agent 不许起，所以它那一侧的真输出要等主 agent 下一趟带前缀合入时才看得到。

### 自证新加的 7 格（38 种到 45 种）

别的格怎么跑：都换上假的那一道，它打一行按条复用、退 0，并在自证里带上 `SINGLEFS_HEAVY_TESTS=user-request`，所以原有 38 格的断言一行没动。前三格换回真的门禁，交它的样本根；合入的树都是 narrow-tree。

| 格（失败行「✗ 自检：」之后的开头） | 喂什么 | 判什么 |
|---|---|---|
| checker 档快档那一步：补丁改到 checker 档包、改了 4 条快档测试函数闭包里的单元…… | 补丁 narrow-checker（只改 checker 档包），根 unit-key-samples-green | 退 0；成功句「现判 4、复用 4、拒算 0」；原因写「补丁改到 checker 档包或它依赖的包：singlefs-checker-tier」；那一道的日志里假 cargo 收到 `--skip f1_first --skip f1_last --skip h3_test --skip test_v` |
| checker 档快档那一步：键都对上时应当现判 0…… | 同一个根再跑一次 | 退 0；「现判 0、复用 8、拒算 0」；日志里没有假 cargo 的参数行，有「全部复用，这一趟没起 cargo」 |
| checker 档快档那一步：样本里 test_t 判 FAILED 时应当退 1…… | 根 unit-key-samples-red | 退 1；「红的格：crash-case-replay」「FAILED 的测试函数 1 条」；单独一行 test_t；「红的 …setting.rs::test_t：」；「并进写段的下一批」；重跑命令带前缀与范围声明，`--check crash-case-replay <根>` |
| checker 档快档那一步：补丁改到 checker 档包依赖的 demo、起它的命令没带 SINGLEFS_HEAVY_TESTS…… | narrow-tree 的 checker 档包加一行依赖 demo，补丁 narrow-demo，前缀设成空 | 退 1；那一道一次都没被起；✗ 句与「→ 怎么办：整条 merge-and-verify.sh 命令前面加 SINGLEFS_HEAVY_TESTS=user-request」；原因写「…依赖的包：demo」 |
| checker 档快档那一步：补丁只改到不在 checker 档包依赖闭包里的 alpha、beta…… | 补丁 narrow-two，前缀设成空 | 退 0；那一道没被起；「- checker 档快档本次未跑：补丁只改到 alpha、beta，都不是 checker 档包 singlefs-checker-tier、也不在它的依赖闭包里」 |
| checker 档快档那一步：那一道退 3 而根不是它的样本根…… | 假的那一道退 3，根是 narrow-tree | 退 1，红句写「退 3 只在样本根上算过」 |
| checker 档快档那一步：带 --copy 与带 --peer 时应当报本次未跑 | 读原有 copy-green、peer-green 两格的输出 | 各有一行「- checker 档快档本次未跑：带了 --copy / --peer」 |

在主工作区跑的终版：`timeout 235 nice -n 19 bash scripts/gate/merge-and-verify.sh --selftest` 退 0，24 秒。末行开头：「  ✓ 自检通过（查了 45 种）：…」，成功句末尾两行是新加的这几样（`mav-selftest-3.out`）。

中间红过两次：
- 头一次（`mav-selftest-1.out`）红 1 格，是原有的「cargo test 那一趟包装退 245……」：那一格要求输出里没有「红的测试」四个字，而我红句里写的是「红的测试函数」。我把红句改成「FAILED 的测试函数」，那一格的断言没动。
- 跑开关轮时，`WORKSPACE_LOCK_BREAK=trust-dead-holder` 那一趟超过了 230 秒。原因是锁那几格在这个开关下留下了死掉的持有者，新加的 7 格每格都要排满 30 秒的放弃时限。改法：新格之前先删掉自证自己的那份锁文件，见下面「弄坏开关」。

### 弄坏开关（新加 7 个，逐个打开整份 `--selftest`，`switch_wheel.py` 并行 8，`wheel-final.txt`）

| 开关（都是 `MERGE_AND_VERIFY_BREAK=`） | 弄坏哪一处 | 退出码 | 红的格 |
|---|---|---|---|
| checker-quick-skipped | 整步不跑（改前的样子） | 1 | 新加的 7 格全红 |
| checker-quick-elsewhere-run | `--copy`、`--peer` 不报本次未跑 | 1 | 「带 --copy 与带 --peer……」 |
| checker-quick-always-run | 不看补丁碰没碰，一律照跑 | 1 | 「补丁只改到不在……alpha、beta……」；连带原有的「本机内存排不上（包装退 252）时……」（多出一行「本次未跑：test 那一步本机内存排不上」，数对不上） |
| checker-quick-prefix-unchecked | 没带前缀也照起 | 1 | 「补丁改到 checker 档包依赖的 demo、起它的命令没带 SINGLEFS_HEAVY_TESTS……」（那一道被起 1 次） |
| checker-quick-dependencies-ignored | 依赖闭包只算 checker 档包自己 | 1 | 同一格（报成了补丁没碰） |
| checker-quick-sample-exit-anywhere | 退 3 不看是不是样本根，一律算过 | 1 | 「那一道退 3 而根不是它的样本根……」 |
| checker-quick-red-ignored | 那一道红了也报过 | 1 | 「样本里 test_t 判 FAILED……」与「那一道退 3……」 |

文件头开关表照这张表写了（第 186 行起），格式照原有那几条：开关名与值、弄坏哪一处、下一行起点名的格。

原有的 33 个开关（29 个 `MERGE_AND_VERIFY_BREAK`、`SINGLEFS_WORK_ROOT_BREAK=new-root-unknown`、`REBUILT_PACKAGES_BREAK=any-compiling-line`、`WORKSPACE_LOCK_BREAK` 三个）也逐个打开跑了一遍（`wheel-old-a.txt`、`wheel-old-b1.txt`、`wheel-old-b2.txt`、`wheel-final-lock.txt`），都退 1。新加的格只在两个开关下跟着红：
- `peer-ignored`：「带 --copy 与带 --peer……」跟着红，因为 --peer 那一趟改在本机跑了。文件头那一条已补上「连带」。
- `WORKSPACE_LOCK_BREAK=trust-dead-holder`：同一格跟着红，因为 peer 那几格先撞锁退 1。这个开关下原有的 peer 四格与「本机内存排不上」那一格本来也跟着红，文件头一直只点名了「锁文件里的持有者已死」一格。这是改前就有的点名不全，我没补。

`WORKSPACE_LOCK_BREAK=no-wait`、`lock-in-common-dir` 两条是在超时被杀的那一趟里跑完的：输出存在 `wheel-old/no-wait.out`、`wheel-old/lock-in-common-dir.out`，可是汇总行没写出来（那一趟收尾时抛了异常）。被杀的那一趟在 /tmp 留下一个临时目录，核过没有进程开着里面的文件之后删了。

语法：`bash -n scripts/gate/merge-and-verify.sh` 过；`PY_SELECT` 那段 python 单独截出来 `python3 -m py_compile` 也过。
（这两份输出里的红格：`no-wait` 红「两个合入同时起……」与「锁文件里的持有者已死……」，`lock-in-common-dir` 红「主工作树的锁被占着时在 git worktree 加出来的树里验……」，都与文件头点名的相同，新加的格都没红。）

### `.claude/main-agent.md` 同步的两处

- 「一轮怎么开、怎么收」第 10 条：测试那一句后面加了一段，写编译与测试之后另跑的 checker 档快档：带 `SINGLEFS_HEAVY_TESTS=user-request`；照门禁快档那两格只现判键对不上的测试函数、键相同的复用；补丁没碰 checker 档包与它依赖的包时报本次未跑；没带前缀这一步拒；红了逐条列出与只重跑那几格的命令、整条退非 0；红的测试函数并进写段的下一批。原来括号里的「checker 档包的用例不直接跑，归提交时 `checker-tier-crash-point-replay`；崩溃点重放快档也归提交时」，改成「快档的别的格与 checker 档全量归提交时 `checker-tier-crash-point-replay`」。测试挑包那一句里的「除 checker 档包」「补丁只改 checker 档包时测试本次未跑」照留：它们说的是 harness 那一步的 `cargo test`，脚本里也还是这样。
- 「写与验分两段」验证段第 1 步：「两条都绿才算这一步完成」之后加了一句：两条命令开头都带前缀，脚本另跑那一步，没带前缀这一步拒、整条退非 0；这一步红了，红的测试函数并进写段的下一批。

## ㉒ 结局账的时机（`.claude/main-agent.md`「一轮怎么开、怎么收」第 8 条）

第 8 条末尾加了一句（`--out` 的写法照 `scripts/agents/agent-cost.py` 第 15 行的「每批收尾落账的写法」）：

```
每批收尾另跑一次闭窗口结局账：`python3 scripts/agents/agent-cost.py outcome --since <开工日期> --until <这一天> --summary --out records/<轮>-outcome-<这一天>.txt`（落进这一轮调度记录所在的 `records/`，文件名照 `scripts/agents/agent-cost.py` 文件头的写法；`--out` 只排他新建，那个文件已在就拒，换一个名字，不覆盖），收拢表引它的数。
```

读数：`grep -c 'agent-cost.py outcome --since' .claude/main-agent.md` 是 1。落点写成 `records/`，因为调度记录就是 `records/` 下按日期起名的平铺文件，不分子目录（`ls records` 看过）。这样会在 `records/` 下放进一份 `.txt`，`records/` 的门禁认不认这种文件，我没核，见「没做什么」。

## ⑥ ⑦ 共用约束（`.claude/agent-common.md`「执行前拒绝的写法」交回正文闸那一条）

在「书记员与执行员……绿行」之后、「全部子 agent 自己起的后台任务……」之前，补了两样。说法照 `.claude/hooks/handback-guard.sh` 文件头第 26–38 行：

```
实现员、执行员、工具员在主工作区或带 `.git` 的副本（git worktree）里用 Edit / Write / MultiEdit 改了 `crates/` 或 `scripts/` 下的 `.rs`，而那棵树最后一次改 `.rs` 之后没真跑过「命名检查交回前跑」那两条命令（只看会话记录里的命令，不看报告；rsync 拷的副本不判）；写回员规格里的目标文件在主工作区不在，交回正文、交回点名的报告与派发提示「报告：」那一份里又都没有一节标题含「找不到的目标」、节里写着这个文件；
```

## ⑧ 写范围

- `.claude/hooks/agent-write-scope.tsv` 在原来那一行（`kb-scribe	crates/singlefs-checker/invariants/*.md	…`）后面加一行，原行留着：

```
kb-scribe	crates/singlefs-checker/invariants/**/*.md	crates/singlefs-checker/invariants/ 各层子目录里的不变量清单，只限这些 .md（* 不跨目录、**/*.md 不盖下一层，与上一行合起来盖住下一层与各层子目录；门禁 code-tooling 的 kb-directories-write-scope 格要登记的 kb 目录下一层与子目录都写得到，records/2026-09-16-subagent拆分提案.md 第九十节 ⑧）
```

- 改法：`python3 scripts/gate/hook-swap.py write-scope-spec.json --hook .claude/hooks/write-guard.sh --scratch <草稿目录> --feeds write-scope-feeds-pass-only.json` 退 0：「✓ 草稿里验过：2 份文件语法检查、内嵌 python 1 段编译、喂了 2 条」「✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过……」。sha256 从 `5164248e…` 换成 `f2ebff14…`，反向规格是 `hook-swap/` 下三份 `reverse-*.json` 里文件名进程号为 1614171 的那一份。
- 只拿放行的两条喂：草稿树在工作根底下，写回员的 `$SINGLEFS_WORK_ROOT/**` 会把草稿树里的一切都放行，该拒的两条在草稿里判不出（头一次喂四条，两条拒的在草稿里退 0，停下了，主工作区没写，见 `write-scope-swap-dry.out`）。所以换上之后，在主工作区用 `--feed-only` 把四条都喂了一遍。
- 样本（`write-scope-feeds.json`，写闸的 hook 输入，`agent_type=kb-scribe`、Edit）：

| 喂的路径 | 改前（`write-scope-feed-before.out`） | 改后（`write-scope-feed-after.out`） |
|---|---|---|
| `crates/singlefs-checker/invariants/kb-directories-write-scope-probe/kb-directories-write-scope-probe.md` | 退 2（红：子目录写不到） | 退 0 |
| `crates/singlefs-checker/invariants/kb-directories-write-scope-probe.md` | 退 0 | 退 0 |
| `crates/singlefs-checker/invariants/kb-directories-write-scope-probe/x.rs` | 退 2 | 退 2（仍只许 `.md`） |
| `crates/singlefs-checker/src/x.md` | 退 2 | 退 2 |

- `.claude/agents/kb-scribe.md` 两处同步：第 26 行「写范围闸对你放行 `.claude/kb/`、`crates/singlefs-checker/invariants/*.md` 与 `crates/singlefs-checker/invariants/**/*.md`（不变量清单下一层与各层子目录里的 `.md`）与那里」；第 42 行「`crates/singlefs-checker/invariants/*.md` 与 `crates/singlefs-checker/invariants/**/*.md`（不变量清单，下一层与各层子目录里的都算，只限这些 `.md`；`crates/` 别处与 `.rs` 照旧不碰）」。
- 读这张表的三个钩子，换上之后在主工作区跑了自证：
  - `bash .claude/hooks/runner-dispatch-guard.sh --selftest` 退 0，「✓ 自检通过（查了 483 种）」；
  - `bash .claude/hooks/bash-command-detector.sh --selftest` 退 0，「✓ 自检通过（查了 764 种）」；
  - `bash .claude/hooks/write-guard.sh --selftest` 退 1，只红一格：「✗ 自检：范围:kb-scribe 不变量清单的子目录越界 应当是 2，实际 0」。这格断言的正是这一批要改掉的旧行为，自证用的就是真表的副本（write-guard.sh 第 627 行 `shutil.copyfile(… agent-write-scope.tsv …)`）。要改的是 write-guard.sh，它不在我的写范围里，修法交主 agent，见「要主 agent 跑的命令」第 1 条。

## ⑲ 复跑表加两行（`.claude/gate.d/break-switch-replay-files.tsv`）

加的两行：

```
.claude/hooks/runner-dispatch-guard.sh	120	# 每条开关逐个跑过：点名的失败行原文都在失败行里，且自证跑得快（并行 8、负载 50 到 90 时最慢一条 53.5 秒，不到上限的一半）
scripts/agents/subagent_transcript.py	60	# 每条开关逐个跑过：点名的失败行原文都在失败行里，且自证跑得快（并行下最慢一条不到上限的一半）
```

两份在 `.claude/hooks/selftest-registry.tsv` 里都登记成 light（第 166、154 行），文件头开关表都写全了（那一格跑起来没报「没写全」）。

加之前照那份表文件头的要求逐条复跑过，用的是草稿探针 `replay_probe.py`。它运行时从 `.claude/gate.d/code-tooling.sh` 按标记截出 break-switch-replay 格那一段 python 原样起（不另抄一份判法），喂一份只有这两行的临时登记表（`probe-registry.tsv`，秒数与上面相同，并行 8），在仓根跑：

```
  ✓ 脚本与钩子的弄坏开关复跑：复跑了 100 条开关（2 份），自证红且点名的项都在失败行里 100 条、自证自带开关轮 0 条；没接复跑的 57 份文件头写全的脚本：…
格程序退出码 0
计时 .claude/hooks/runner-dispatch-guard.sh：98 条，并行 8；最短 2.4 中位 4.4 最长 12.5 秒；退非 0 的 98 条
计时 scripts/agents/subagent_transcript.py：2 条，并行 8；最短 0.0 中位 0.0 最长 0.0 秒；退非 0 的 2 条
```

（原样在 `probe-replay.out`。末尾「没接复跑的 57 份」是那一段程序对全仓的扫，临时登记表里只有这两行，所以它把别的已登记的也列成「没接」，不算这一次的结论。）

派发闸那一份现在是 98 条开关，`b9-dispatch-guard.md` 交回时是 96 条。100 条都是「对」：设上开关后自证退非 0，点名的项都在失败行里。每条的退出码与耗时（`probe-timing.tsv`，并行 8，这一趟机器负载 10 到 17）：

- `subagent_transcript.py`：`SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` 退 1、0.0 秒；`SUBAGENT_TRANSCRIPT_BREAK=start-by-mtime` 退 1、0.0 秒。
- `runner-dispatch-guard.sh` 的 98 条，在下面那一段。

## ㉔ 实现员第 4 步措辞（`.claude/agents/implementation-writer.md`）

改前：「……、`cargo build --offline --all-targets`，各贴末尾原样输出。门禁阶段（第 1d 步形式化那一道门禁的 `--quick`、共用约束「门禁」一节的命名检查除外）……」

改后：「……、`cargo build --offline --all-targets`、共用约束「门禁」一节「命名检查交回前跑」那两条命名检查，各贴末尾原样输出。门禁阶段（第 1d 步形式化那一道门禁的 `--quick`、那两条命名检查除外）……」

前后说的是同一件事：列举里有了命名检查，后面「除外」那一处指的就是它。做法没改。

### 派发闸 98 条开关逐条读数（开关、退出码、并行 8 下的秒数）

```
RUNNER_DISPATCH_GUARD_BREAK=disable-check	退1	2.4 秒
RUNNER_DISPATCH_GUARD_BREAK=disable-heavy-test	退1	7.4 秒
RUNNER_DISPATCH_GUARD_BREAK=heavy-permission-unchecked	退1	7.4 秒
RUNNER_DISPATCH_GUARD_BREAK=light-tier-commands-unchecked	退1	7.4 秒
RUNNER_DISPATCH_GUARD_BREAK=light-tier-quick-counted	退1	7.6 秒
RUNNER_DISPATCH_GUARD_BREAK=disable-snapshot	退1	7.4 秒
RUNNER_DISPATCH_GUARD_BREAK=admission-ignored	退1	7.4 秒
RUNNER_DISPATCH_GUARD_BREAK=general-purpose-allowed	退1	7.4 秒
RUNNER_DISPATCH_GUARD_BREAK=general-purpose-free	退1	8.7 秒
RUNNER_DISPATCH_GUARD_BREAK=inputs-ignored	退1	10.8 秒
RUNNER_DISPATCH_GUARD_BREAK=json-error-denies	退1	9.8 秒
RUNNER_DISPATCH_GUARD_BREAK=json-error-silent	退1	9.8 秒
RUNNER_DISPATCH_GUARD_BREAK=mutation-table-free	退1	9.8 秒
RUNNER_DISPATCH_GUARD_BREAK=concurrency-no-limit	退1	9.8 秒
RUNNER_DISPATCH_GUARD_BREAK=absent-task-counted	退1	9.9 秒
RUNNER_DISPATCH_GUARD_BREAK=report-name-unchecked	退1	9.6 秒
RUNNER_DISPATCH_GUARD_BREAK=report-line-labels-split	退1	8.3 秒
RUNNER_DISPATCH_GUARD_BREAK=report-line-unanchored	退1	10.8 秒
RUNNER_DISPATCH_GUARD_BREAK=report-name-negation-judged	退1	10.8 秒
RUNNER_DISPATCH_GUARD_BREAK=report-line-position-anchored	退1	10.8 秒
RUNNER_DISPATCH_GUARD_BREAK=three-way-report-unchecked	退1	10.8 秒
RUNNER_DISPATCH_GUARD_BREAK=report-name-case-sensitive	退1	10.8 秒
RUNNER_DISPATCH_GUARD_BREAK=report-name-whole-path	退1	10.7 秒
RUNNER_DISPATCH_GUARD_BREAK=report-name-codeblock-judged	退1	9.8 秒
RUNNER_DISPATCH_GUARD_BREAK=report-name-prefixes-unchecked	退1	9.8 秒
RUNNER_DISPATCH_GUARD_BREAK=leg-types-hardcoded	退1	9.2 秒
RUNNER_DISPATCH_GUARD_BREAK=placeholders-ignored	退1	9.2 秒
RUNNER_DISPATCH_GUARD_BREAK=registry-ignored	退1	9.0 秒
RUNNER_DISPATCH_GUARD_BREAK=registry-no-prune	退1	8.5 秒
RUNNER_DISPATCH_GUARD_BREAK=runner-backfill-refused	退1	8.5 秒
RUNNER_DISPATCH_GUARD_BREAK=scope-ignored	退1	8.5 秒
RUNNER_DISPATCH_GUARD_BREAK=scope-library-unread	退1	8.5 秒
RUNNER_DISPATCH_GUARD_BREAK=scope-negation-ignored	退1	8.6 秒
RUNNER_DISPATCH_GUARD_BREAK=scope-token-unexpanded	退1	11.2 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-any-agent	退1	11.2 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-body-optional	退1	11.2 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-closed-round-open	退1	11.3 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-loose-lines	退1	10.7 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-missing-spec-denies	退1	10.5 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-no-edges	退1	10.5 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-no-normalize	退1	10.3 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-spec-unread	退1	11.6 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-unreadable-aborts	退1	11.6 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-unreadable-denies	退1	11.6 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-valve-ignored	退1	11.6 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-valve-loose	退1	11.7 秒
RUNNER_DISPATCH_GUARD_BREAK=spec-check-skipped	退1	11.6 秒
RUNNER_DISPATCH_GUARD_BREAK=valve-reason-ignored	退1	12.5 秒
RUNNER_DISPATCH_GUARD_BREAK=verdict-not-spec	退1	11.3 秒
RUNNER_DISPATCH_GUARD_BREAK=window-ignored	退1	4.3 秒
RUNNER_DISPATCH_GUARD_BREAK=window-never-lifted	退1	4.4 秒
RUNNER_DISPATCH_GUARD_BREAK=long-run-unchecked	退1	4.3 秒
RUNNER_DISPATCH_GUARD_BREAK=long-run-duplicate-label-first-wins	退1	4.4 秒
RUNNER_DISPATCH_GUARD_BREAK=long-run-codeblock-judged	退1	4.2 秒
RUNNER_DISPATCH_GUARD_BREAK=wall-clock-optional	退1	4.4 秒
RUNNER_DISPATCH_GUARD_BREAK=long-run-types-narrowed	退1	4.4 秒
RUNNER_DISPATCH_GUARD_BREAK=stop-at-first	退1	4.2 秒
RUNNER_DISPATCH_GUARD_BREAK=inputs-hint-dropped	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=check-mode-registers	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-ignored	退1	4.2 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-unchecked	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-source-any-path	退1	4.3 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-source-escapes	退1	4.2 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-line-unread	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-round-name-unchecked	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-name-substring	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-quote-short-ok	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-note-accepted	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-codeblock-judged	退1	4.0 秒
RUNNER_DISPATCH_GUARD_BREAK=code-fence-loose	退1	3.2 秒
RUNNER_DISPATCH_GUARD_BREAK=code-block-line-rule	退1	3.2 秒
RUNNER_DISPATCH_GUARD_BREAK=code-block-library-missing	退1	3.2 秒
RUNNER_DISPATCH_GUARD_BREAK=code-block-ambiguity-unchecked	退1	4.0 秒
RUNNER_DISPATCH_GUARD_BREAK=code-block-fallback-unmasked	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=hosts-valve-unchecked	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-valve-hint-merged	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-angle-dropped	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-required-six-types	退1	4.0 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-limit-unchecked	退1	3.8 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-limit-any-refused	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-valve-ignored	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-valve-loose	退1	4.0 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-formula-accepted	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-line-optional	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=thread-required-only-writer	退1	4.0 秒
RUNNER_DISPATCH_GUARD_BREAK=writer-lock-unchecked	退1	4.0 秒
RUNNER_DISPATCH_GUARD_BREAK=writer-lock-live-ignored	退1	4.0 秒
RUNNER_DISPATCH_GUARD_BREAK=writer-lock-stale-kept	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=writer-lock-retry-refused	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=writer-lock-valve-ignored	退1	4.1 秒
RUNNER_DISPATCH_GUARD_BREAK=writer-lock-root-locked	退1	4.0 秒
LIB_LONG_RUN_BREAK=partial-accepted	退1	4.1 秒
LIB_LONG_RUN_BREAK=hosts-unchecked	退1	4.1 秒
LIB_LONG_RUN_BREAK=hosts-canonical-unchecked	退1	4.0 秒
LIB_LONG_RUN_BREAK=hosts-suffix-accepted	退1	4.0 秒
LIB_LONG_RUN_BREAK=unshardable-unchecked	退1	4.0 秒
SINGLEFS_WORK_ROOT_BREAK=new-root-unknown	退1	4.0 秒
SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown	退1	3.7 秒
```

## 新写的 `# gate-similar:` / `# hook-events:` 行

没有：这一批没新建门禁阶段，也没新建钩子。开工时跑过 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list`（退 0，59 行，`gate-overlap-list.txt`），没有门禁或钩子在管 merge-and-verify 的这几条判据。

## 要主 agent 跑的命令

工作目录都是仓根 `/home/fy5090/code/singlefs`。

1. **补 write-guard.sh 的自证（⑧ 带出来的，先做这一条）**
   - 规格在草稿目录的 `write-guard-followup-spec.json`，两处：
     - 「范围:kb-scribe 不变量清单的子目录越界」那一格（期望 2）改名为「范围:kb-scribe 写不变量清单子目录里的 .md」，期望改成 0；
     - 另加一格「范围:kb-scribe 不变量清单子目录里的 .rs 越界」，期望 2；
     - 文件头 `WRITE_GUARD_BREAK=scope-new-rows-dropped` 那一条的点名补上新的那一格。在这个开关下，自证用的表副本按前缀 `kb-scribe\tcrates/singlefs-checker/invariants/` 删行，两行都会被删，新格会红。
   - 我只在草稿里 `--dry-run` 验过（`write-guard-followup-dry.out`：「✓ 草稿里验过：1 份文件语法检查、内嵌 python 1 段编译、喂了 2 条」），没换上。
   - 这份钩子不归这一批，由你或派一个工具员换：
     `python3 scripts/gate/hook-swap.py /home/fy5090/.local/share/singlefs/work/landing-audit-b9-defs-verify/write-guard-followup-spec.json --hook .claude/hooks/write-guard.sh --scratch <你的草稿目录> --feeds /home/fy5090/.local/share/singlefs/work/landing-audit-b9-defs-verify/write-scope-feeds-pass-only.json`
   - 换完跑 `bash .claude/hooks/write-guard.sh --selftest`，应当退 0。再跑 `WRITE_GUARD_BREAK=scope-new-rows-dropped bash .claude/hooks/write-guard.sh --selftest`，应当红「范围:kb-scribe 写不变量清单的 .md」「范围:kb-scribe 写不变量清单子目录里的 .md」「范围:experiment-runner 写重跑比对对象登记」三格。
   - 规格的旧串是此刻 write-guard.sh 的原文。别的会话改过它的话，hook-swap 退 3、一个字节不写，那时照新的原文重出规格。
2. **⑧ 那一格**：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 kb-directories-write-scope 格; 范围=1 格; 为什么=写范围表给 kb-scribe 加了不变量清单各层子目录 .md 的一行，核登记的 kb 目录下一层与子目录都写得到' bash .claude/gate.d/code-tooling.sh --check kb-directories-write-scope`
   - 看那一格的成功句，应当报盖住登记的 2 个 kb 目录；
   - 改前的红句（`b9-gates-a.md` 第 60 行）不该再出现。
3. **⑲ 复跑方与开关表的格式格**：
   `SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay、break-switch-headers 两格; 范围=2 格; 为什么=复跑表加了派发闸与 subagent_transcript.py 两行，merge-and-verify.sh 文件头开关表加了七条' bash .claude/gate.d/code-tooling.sh --check break-switch-replay,break-switch-headers`
   - break-switch-replay 格的成功句里，复跑的条数应当比加这两行之前多 100 条（派发闸 98 条、subagent_transcript.py 2 条），不对的条数是 0；
   - break-switch-headers 格判 merge-and-verify.sh 新加的七条三样写全（merge-and-verify.sh 不在复跑表里，它的开关我逐个跑过，读数在上面）。
4. **研究脚本自证格**（merge-and-verify.sh 的自证在这一格的 runner 表里，`.claude/gate.d/code-tooling.sh` 第 423 行）：
   `SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 research-script-selftests 格; 范围=1 格; 为什么=merge-and-verify.sh 自证加了 checker 档快档那一步的 7 格，新格在样本根上真起门禁 checker-tier-crash-point-replay（假 cargo），核守卫底下照绿' bash .claude/gate.d/code-tooling.sh --check research-script-selftests`
   - 新格起的是样本根里的假 cargo（路径上有 `.crash-point-replay-sample-tools`，守卫 ④ 不判）。
   - 那一格对别的脚本是红是绿，与这一批无关。
5. **真的那一趟（㉓）**：下一批合入时照改过的 `.claude/main-agent.md` 验证段第 1 步，两条命令都带 `SINGLEFS_HEAVY_TESTS=user-request`。
   - 这一步在 release 下编 checker 档包，内存上限现在与别的几步共用 `--memory`（默认 8G）。建议照 `.claude/skills/crash-test/commit-stages.md` 给 `checker-tier-crash-point-replay` 定的上限来给（推的，没量过）。
   - 排不上会退 252，照「本机排不上」报。
6. **定义三方**：改过的定义与共用约束，交第九批收尾的定义三方（门禁 doc-process-records 的 agent-def-adversarial-review 格）：
   - `.claude/main-agent.md`（第 8 条、第 10 条、验证段第 1 步）
   - `.claude/agent-common.md`（交回正文闸那一条）
   - `.claude/agents/implementation-writer.md`（第 4 步）
   - `.claude/agents/kb-scribe.md`（第 26、42 行）
7. **命名检查**：这一批没改 `.rs`，共用约束「命名检查交回前跑」那一条不适用，没跑。

## 没做什么

- 没跑重型测试：checker-tier-crash-point-replay（只在门禁自己的样本根上，用样本里的假 cargo 起过它的快档两格）、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`。没跑门禁阶段与各类 lint，也没走定义三方、没提交。
- merge-and-verify 本身没真跑过（它是 whole，子 agent 起不了），新那一步在真仓上跑出什么样，要等主 agent 下一趟带前缀合入时才看得到。
- `.claude/hooks/write-guard.sh` 的自证那一格没改（不在写范围里）。它现在红 1 格，修法见「要主 agent 跑的命令」第 1 条。
- `.claude/hooks/selftest-registry.tsv` 里 merge-and-verify.sh 那一行的理由还写着「PATH 前面放假 cargo……」，没提新格在门禁样本根上真起那一道（那一道里起的仍是样本的假 cargo，档位还是 light）。那份表不归我改，要不要补理由交主 agent。
- 下面几点留给定义三方当攻击面，我没定：
  - `.claude/main-agent.md`「一轮怎么开、怎么收」第 3 条写「重型测试提交之外任务确实要跑，先弹窗问用户」。这一批写的合入后验证那一步带 `=user-request` 跑 checker 档快档、不弹窗，依据是用户定案（`research/prompts/landing-audit-r3-main-verification.md` 节「用户定案（2026-10-23，弹窗）」）。第 3 条里没写这个例外：派发提示只让改第 10 条与验证段第 1 步里说 merge-and-verify 跑什么的那几句，我没往第 3 条加条款。
  - 带 `--copy`、`--peer` 时这一步报本次未跑，不判红。合入主工作区（不带这两个）是要验的那条路，这两种情形只给出路。这样会不会让人把 checker 档那一步漏掉，三方可以判。
  - 补丁没碰的判法：看依赖闭包，追加了变异行、改到不落在包里的路径都照跑。测试运行时读、却在依赖闭包外的包里的文件（比如 `include_str!` 指到别的包），这里认不出，会被判成没碰。真实的单元键里有没有这类内容输入，要看门禁那一道的键怎么组，我没核。
  - 红了给的重跑命令，是只重跑那几格（按复用只现判红的与改后键变了的），不是 `cargo test … --exact <名>` 逐条点名：那一道起 cargo 的参数在它自己里面，这里不另拼一份。
- ㉒ 的 `--out` 落在 `records/` 下的 `.txt`。门禁 doc-process-records 与 doc-text 这几道认不认 `records/` 下的 `.txt`，我没核；文件头的写法就是「<那一轮调度记录目录>/<轮>-outcome-<日期>.txt」，我照抄了。
- `WORKSPACE_LOCK_BREAK=trust-dead-holder` 那一条的文件头点名本来就不全（peer 那几格也跟着红），我没补，只确认新加的格在这个开关下只是连带红。
- 复跑表里 `scripts/agents/subagent_transcript.py` 那一行，插在了 `three-way-materials.py` 之后，没照字母序排；那一格不判次序，表里也没写次序的规矩。

## 草稿目录里有什么

`/home/fy5090/.local/share/singlefs/work/landing-audit-b9-defs-verify/`：

- 改前留底：`base/originals/`（`copy-originals.py`，清单 `base/originals/manifest.tsv`）与 `base-sha256.txt`；草稿 `draft/merge-and-verify.sh`（与主工作区终版逐字相同）；中间版本 `mav-after-spec-1.sh`、`mav-after-spec-2.sh`。
- 规格：
  - `mav-spec-1.json`、`mav-spec-2.json`、`mav-spec-3.json`：merge-and-verify；
  - `write-scope-spec.json`：写范围表；
  - `write-guard-followup-spec.json`：交主 agent 的那一份；
  - `hook-swap/reverse-*.json`：hook-swap 写的反向规格。
- 喂法与输出：`write-scope-feeds.json`、`write-scope-feeds-pass-only.json`、`write-scope-feed-before.out`、`write-scope-feed-after.out`、`write-scope-swap*.out`、`write-guard-followup-dry.out`、`hook-swap/feeds-*.log`。
- 自证与开关轮：
  - `mav-selftest-1/2/3.out`；
  - `wheel-new.txt`、`wheel-final.txt`、`wheel-final-lock.txt`、`wheel-old-*.txt`，与每个开关的整份输出 `wheel-new/`、`wheel-final/`、`wheel-old/`；
  - `selftest-after-scope-*.out`。
- 复跑探针：`replay_probe.py`、`probe-registry.tsv`、`probe-replay.out`、`probe-timing.tsv`。
- 门禁样本的原样输出：`stage-sample-green.run1.out`、`stage-sample-green.run2.out`、`stage-sample-red.run1.out` 与两份 setup 日志。
- 工具：`make_spec.py`、`switch_wheel.py`。
- 清理：
  - 门禁样本的两份拷贝 `probe-stage/`（1.3M，`setup.sh` 在里面 git init 过）交回前删了；
  - hook-swap 的草稿树由它自己删了；
  - 被杀的那两趟自证在 `/tmp` 留下的临时目录（一份 3.4M、一份没量），核过没有进程开着，删了。
  - 这里没有仓副本、编译目录或库拷贝。
