# 门禁 54 号分档：交回报告

改好的文件：`/home/fy5090/code/singlefs/.claude/gate.d/54-layer0-replay.sh`（338 行，sha256 `86ff561e868cab9ffb138a2e5fbcd668c467caea6939592c32485a162e017ec9`）。
改前备份：`/tmp/claude-1000/gate54-tiering/54-layer0-replay.sh.orig`（sha256 `5bd04b7b…`，门禁修复会话那一处改动已在里面）；与它的 diff：`/tmp/claude-1000/gate54-tiering/54.diff`（+221 / −25 行）。
没暂存、没提交。`stage-inputs.tsv` 没改（`git diff --quiet HEAD -- .claude/gate.d/stage-inputs.tsv` 退出码 0）。

## 改了哪几处（六次 `research/scripts/replace-once.py`，每次都是「命中 1 次，已替换并回读确认」）

| # | 旧文本（改前行号） | 改成什么（改后行号） |
|---|---|---|
| 1 | 文件头 1–17 行 | 1–34 行：gate-stage 行改成两档的说法；新写「分两档」「输入」两段；全量与多线程两段保留，线程判定与 CHECKER 判定两句前面加「--full 里」；门禁修复会话那两行（LAYER0 下一行不是 CHECKER 判红、两支判红只拿合成日志核过）原句保留；加一行说明标记那一半是怎么核的 |
| 2 | 18–20 行（`set -uo pipefail` / `ROOT=` / `cd`） | 35–50 行：参数解析，`--full` 与项目根顺序不限，认不出的 `-*` 参数判红退 2 |
| 3 | 22–42 行（复用判定、改动范围判定、没有装置退 77） | 52–86 行：复用与改动范围两问原样包进 `if [[ "$layer0_tier" == quick ]]`（--full 不问）；没有装置退 77 两档都保留；新加取 common-dir（取不到判红）、标记路径、登记表路径、临时目录与 EXIT 清理 |
| 4 | 53–55 行（`run_layer0_test_binary` 头） | 97–102 行：--full 带 `--include-ignored`，快档不带 |
| 5 | 86–87 行（`log="$(mktemp)"` 那两行）之前插入 | 133–250 行：`write_layer0_input_manifest`、`fail_without_input_manifest`、`report_manifest_differences`、`run_quick_tier_of_stream` 四个函数；快档整段（跑两条流、没标记判红、哈希不等判红并列出不同的文件、标记里计数行不是恰好三行判红、判绿报计数并原样带出标记里的三行计数与跑完时刻，`exit 0`）；--full 开跑段（先删旧标记、记开跑时刻与输入清单） |
| 6 | 142 行（最后一句 ✓）之后追加 | 307–338 行：--full 判绿之后再算一次输入，与开跑时不同判红不写标记；相同则先写 `<标记>.partial.<pid>` 再 `mv` 成标记，写不成判红 |

原有几样的去向：线程数判定（`worker_threads_are_acceptable`）、`LAYER0` 下一行不是 `CHECKER` 判红（门禁修复会话那一块，一字未动）、`exhaustive=true` 判定、两条流的 release 全量跑法，都原样留在 --full 那条路上；改动范围没碰输入退 77、`stage-must-run.sh` 复用判定原样留在快档上。

标记：`$(git -C <根> rev-parse --path-format=absolute --git-common-dir)/singlefs-layer0-full-green`，真仓里就是 `/home/fy5090/code/singlefs/.git/singlefs-layer0-full-green`。内容（自证临时仓里写出的一份原样见附录 A 开头）：`input_hash=`、`input_file_count=`、`input_paths=`、`started_utc=`、`finished_utc=`、`judged_root=`、`worker_threads=`、`LAYER0 …`、`CHECKER …`、`LAYER0B …` 三行原样，再逐文件一行 `input_file <sha256>  <路径>`。

输入哈希：路径取 `stage-inputs.tsv` 里 `54-layer0-replay.sh` 那一行（`crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/`）；文件集是 `git ls-files -z --cached --others --exclude-standard -- <这几条>` 里磁盘上真存在的文件，`LC_ALL=C sort -zu` 排序，逐个 `sha256sum`，整张清单再 `sha256sum` 一次。不按 git 对象算；附录 A「--staged 那条路」一段证明工作区跑的 --full 与临时 worktree 里的快档对同一份内容算出同一个哈希。

## 四样自证（临时仓 + 打合成日志的假 cargo）

做法：`/tmp/claude-1000/gate54-tiering/selftest-repo`（`git init` 的临时仓，只放 54 号、`stage-inputs.tsv`、`stage-must-run.sh`、`change-touches-crates.sh` 与五个假输入文件），`/tmp/claude-1000/gate54-tiering/fake-bin/cargo` 按 `--include-ignored` 有没有打全量或快档的合成日志，环境变量控制缺 `CHECKER` 行、cargo 判红、第二条流跑时改一个输入。驱动脚本 `/tmp/claude-1000/gate54-tiering/selftest-driver.sh`，全部原样输出 `/tmp/claude-1000/gate54-tiering/selftest-output.txt`（194 行，全文在附录 A）。快档用 `SINGLEFS_GATE_FULL=1` 越过「没碰 crates/ 退 77」那一问（临时仓相对 HEAD 没改动）。
不在真仓里做这几样，是因为合成日志跑出来的 --full 会在真仓 common-dir 里留一份假的全绿标记。

1. **标记相等判绿**：附录 A「自证一」，退出码 0，成功句报「第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored」并原样带出标记里的 `LAYER0` / `CHECKER` / `LAYER0B` 三行与 `全量跑完于 2026-09-23T17:13:31Z`。
2. **改一个输入文件一个字节判红**：`cmp` 报 `differ: byte 13, line 1`；附录 A「自证二」，退出码 1，列出「内容不同：crates/singlefs-harness/src/lib.rs」，出路句是「这批输入的层 0 全量没在收尾跑过：跑 `bash .claude/gate.d/54-layer0-replay.sh --full`」。改回那个字节再跑，退出码 0（证明红的就是那一个字节）。
3. **删掉标记判红**：附录 A「自证三」，退出码 1，同一句出路。真仓里也现跑了一次（下一节），真仓本来就没有标记，同样判红。
4. **--full 判红时不写标记**：附录 A「自证四」两支——合成日志里 `LAYER0` 下一行不是 `CHECKER`（门禁修复会话那一支判红，退出码 1）、假 cargo 退 101（「层 0 崩溃点重放的用例判红」，退出码 1）；两次跑完都是「[标记不在]」（开跑前标记在，开跑先删、判红不写）。紧接着跑快档，退出码 1、报没有标记。

外加两样（附录 A）：--full 跑到第二条流时改了一个输入 ⇒ 判红「全量跑的过程中这一道的输入变了」、列出那个文件、不写标记；照 `gate.sh --staged` 的建法（`worktree add --detach HEAD` + `apply --index` 暂存区的 diff）建临时 worktree，工作区跑的 --full（新文件已暂存）⇒ 临时 worktree 里快档读 common-dir 同一份标记判绿；工作区再多一个没暂存的输入文件、重跑 --full ⇒ 临时 worktree 里快档判红、列出「只在标记里：research/results/e999-other-session.out」。认不出的参数 `--fast` ⇒ 退出码 2 并给用法。

## 真仓里快档真跑一次

命令：`nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`（不带 `--full`，工作区，没设 `SINGLEFS_STAGED_TREE`，所以复用那一问答「要跑」，改动范围那一问答「碰了」）。开跑 2026-09-23T17:12:35Z，结束 17:16:39Z（挂钟 4 分 4 秒，大半是 release 编译）。ps 看到的 cargo 命令行是 `cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --nocapture`（没有 `--include-ignored`）。
全文 `/tmp/claude-1000/gate54-tiering/real-quick-run.log`，其中 `LAYER0_PROGRESS` 转发行 123 行（`grep -c`）；末尾原样：

```
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 8 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/home/fy5090/code/singlefs/.git/singlefs-layer0-full-green）
     → 怎么办：这批输入的层 0 全量没在收尾跑过：跑 `bash .claude/gate.d/54-layer0-replay.sh --full`
exit=1
finish=2026-09-23T17:16:39Z
```

跑完 `ls` 真仓 common-dir：`ls: cannot access '/home/fy5090/code/singlefs/.git/singlefs-layer0-full-green': No such file or directory`——真仓里没有留下标记。真仓的 --full 没有真跑（按派发，机器被编译占满；两条流合计约 4 小时）。

## 两个 lint（改完之后跑，原样）

```
══ 门禁自检（每条拒绝都要给出路） ══

  ✓ 门禁自检通过：83 个脚本（.sh 与 .py）、225 条拒绝都带了出路
gate-lint 退出码 0

══ shell 纪律检查 ══

  ✓ shell 纪律检查通过（共 76 个脚本）
shell-lint 退出码 0
```

改前基线同一对命令是「215 条拒绝都带了出路」与「共 76 个脚本」；多出的 10 条是 54 号新加的 10 处 ✗。`bash -n` 语法检查通过。

## 要主 agent 判的几件事（我没改）

1. **54 号的登记输入比它实际读的宽**：两个测试文件与 `crates/singlefs-harness/src/` 里没有读 `research/results/` 或 `.claude/kb/layout/` 的代码（`grep -rn "fs::\|File::\|OpenOptions\|env!(\|include_"` 在两个测试文件、`tests/common/mod.rs`、`src/*.rs` 里只命中删临时文件、写镜像、`include_str!("model.rs")` 与 `write` 录制流）；两处只在注释里引。按登记表算哈希，每出一份新的实验产物（写报告时 `git ls-files --others --exclude-standard -- research/results/` 数到 11 个未跟踪的产物）或改一次布局表，标记就失效、要重跑约 4 小时的全量。收不收窄登记表是主 agent 的事：它同时管 `stage-must-run.sh` 的复用判定，登记表文件头写的是「宁宽勿窄」。
2. **别的会话没提交的输入会让 `--staged` 下判红**：文件集含未跟踪的文件（今天 `crates/` 下有 20 个未跟踪的文件，其中 6 个是 `singlefs-core` 与 `singlefs-harness` 的 `lib.rs` 用 `pub mod` 引着的源文件；只看索引的话 --full 算的哈希会漏掉它们，暂存之后再比就对不上）。代价是工作区里别的会话的未跟踪、未暂存输入会进工作区跑出的哈希，暂存树里没有它们。出路句已写：在只含这一批的 worktree 里跑 `--full <它的根>`（标记在 common-dir，各 worktree 共用）。
3. **哈希里没有 54 号脚本自己和登记表文件**：按派发只算登记的路径。登记表改了会让文件集变、哈希跟着变；54 号脚本改了（比如收严线程判定）不会让标记失效。要不要把脚本自己算进去，主 agent 定。
4. **快档照旧把 `LAYER0_PROGRESS` 转到门禁输出**：真仓一次 123 行。那是给全量看进度用的，快档不需要，但删不删属于行为改动，没动。
5. **跟着要改的定义与文档**（约束里不许我碰，列给主 agent / sweep）：
   - `.claude/agents/crash-verifier.md` 第 23、24 行：照 `stage-owners.tsv` 第 37 行跑 `nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`，现在只跑快档；要它跑全量得写 `--full`。
   - `.claude/main-agent.md`：「任务收尾」是哪一步、在那一步跑 `bash .claude/gate.d/54-layer0-replay.sh --full`，还没写（记录里的要求：`records/2026-09-19-里程碑二遗留收拢.md` 第 147 行「改之前先写清『任务收尾』是哪一步（阶段同步那一步）」）。
   - `README.md` 第 107 行：`bash .claude/gate.d/54-layer0-replay.sh   # 单跑层 0 崩溃点重放全量（release）`，现在这条命令是快档。
   - `.claude/rules/implementation-workflow.md` 第 65 行说 54 号判线程数；这一判现在只在 `--full` 里。
   - `.claude/kb/milestone/02-second-txn.md` 第 379 行（收口表第 41 行）的状态要回写。
6. 文件头「池级 checker（23 条不变量）」是原文照留，没现查它是不是还对。

## 没做什么

- 没真跑 `--full`（派发说不用；两条流合计约 4 小时，机器负载开跑时 83）。--full 的判定段、标记写入、跑中输入变化判红，只在临时仓用合成日志走通；真全量跑出的 `LAYER0B` 行长度、跑 4 小时期间别的会话改输入的频率，都没在真仓验过。
- 没做 fixtures 样本（54 号原本就没有，判红要 cargo 真跑）；四样自证的驱动脚本在 `/tmp`，没进仓。
- 没改 `stage-inputs.tsv`、`crates/`、kb、定义文件；没暂存、没提交。
- 没跑整轮门禁，也没跑别的阶段。

## 附录 A：自证全部原样输出（`/tmp/claude-1000/gate54-tiering/selftest-output.txt`）

```
════ 准备：--full（假 cargo 打全绿的合成日志）写标记
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-23T17:13:30Z）：旧的全绿标记已删；这一道的输入哈希 0e710f0037556efc…（5 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/selftest-repo/.git/singlefs-layer0-full-green（输入哈希 0e710f0037556efc…，5 个文件；开跑 2026-09-23T17:13:30Z，跑完 2026-09-23T17:13:31Z）
[退出码 0]
[标记在：0e710f0037556efc…，16 行]

──── 标记全文 /tmp/claude-1000/gate54-tiering/selftest-repo/.git/singlefs-layer0-full-green
# 层 0 全量全绿标记：.claude/gate.d/54-layer0-replay.sh --full 判绿之后写，整轮门禁的快档读。不进工作树，别手改。
input_hash=0e710f0037556efc3797a5ed7502a4cba0ebd464b87f81b278d2bd70b0cd835e
input_file_count=5
input_paths=crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/
started_utc=2026-09-23T17:13:30Z
finished_utc=2026-09-23T17:13:31Z
judged_root=/tmp/claude-1000/gate54-tiering/selftest-repo
worker_threads=第一个事务那条流 32、两次发布那条流 32；SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核
LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
input_file 292064490607ece9e9bf2ee9f6b7a9b6677c1b42179c2b773cd0e747573a73a5  .claude/kb/layout/01-first-txn.md
input_file 718475cd179c5fce5f3cbaf68fb45b15017015ebeec359c040cf704e3f1c86b6  Cargo.lock
input_file 7cdf4d0b1f6a07ea6c8d8f3d40e983f38cee0bc74c941f98ca0b9ab1946ba347  Cargo.toml
input_file e35859007da098648ad7f62e4185436e7c7a37953d68c9ec7b88a9f7eae785e7  crates/singlefs-harness/src/lib.rs
input_file 5656fafa00d4f294bcb606cf4f7d4fa877390e46f583e8b3c8744ace104a31d1  research/results/e142-sample.out

════ 自证一：标记与输入相等 ⇒ 快档判绿
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 全绿标记与这批输入的内容哈希相同（0e710f0037556efc…，5 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）：层 0 全量跑完于 2026-09-23T17:13:31Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记在：0e710f0037556efc…，16 行]

改一个字节：cmp 原文件与改后：
/tmp/claude-1000/gate54-tiering/lib.rs.before-byte-change crates/singlefs-harness/src/lib.rs differ: byte 13, line 1

════ 自证二：改一个输入文件的一个字节 ⇒ 快档判红
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但全绿标记记的输入哈希（0e710f0037556efc3797a5ed7502a4cba0ebd464b87f81b278d2bd70b0cd835e）与这一次的（b144b68aca6c94849cf9edbc93e4d575e7bad9c7cb7300e6abb1ee8853b71326，5 个文件）不同
       不同的文件共 1 个（最多列 20 个）：
         内容不同：crates/singlefs-harness/src/lib.rs
     → 怎么办：这批输入的层 0 全量没在收尾跑过：跑 `bash .claude/gate.d/54-layer0-replay.sh --full`
                不同的文件里有别的会话没提交的东西时，工作区跑的全量罩不到这一批：在只含这一批的 worktree 里跑 --full <它的根>。
[退出码 1]
[标记在：0e710f0037556efc…，16 行]

════ 改回那个字节 ⇒ 快档又判绿（证明自证二红的就是那一个字节）
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 全绿标记与这批输入的内容哈希相同（0e710f0037556efc…，5 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）：层 0 全量跑完于 2026-09-23T17:13:31Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记在：0e710f0037556efc…，16 行]

════ 自证三：删掉标记 ⇒ 快档判红
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/selftest-repo/.git/singlefs-layer0-full-green）
     → 怎么办：这批输入的层 0 全量没在收尾跑过：跑 `bash .claude/gate.d/54-layer0-replay.sh --full`
[退出码 1]
[标记不在]

════ 准备：--full 全绿，标记重新写上
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-23T17:13:33Z）：旧的全绿标记已删；这一道的输入哈希 0e710f0037556efc…（5 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/selftest-repo/.git/singlefs-layer0-full-green（输入哈希 0e710f0037556efc…，5 个文件；开跑 2026-09-23T17:13:33Z，跑完 2026-09-23T17:13:34Z）
[退出码 0]
[标记在：0e710f0037556efc…，16 行]

════ 自证四：--full 喂 LAYER0 下一行不是 CHECKER 的合成日志 ⇒ 判红、不写标记（旧标记开跑时已删）
$ env FAKE_CARGO_DROP_CHECKER=1 bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-23T17:13:34Z）：旧的全绿标记已删；这一道的输入哈希 0e710f0037556efc…（5 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✗ 用例跑过了，LAYER0 计数行的下一行却不是以 CHECKER 开头的逐条不变量行：成功句里「逐条不变量」那一句会是空话
     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例打完 LAYER0 行要紧跟着 println! checker_line(&tally) 那一行；
                两行之间插进了别的输出，就把 CHECKER 那一行挪回 LAYER0 行的正下方。
[退出码 1]
[标记不在]

════ 自证四之后：快档判红（没有标记）
$ env SINGLEFS_GATE_FULL=1 bash .claude/gate.d/54-layer0-replay.sh
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/gate54-tiering/selftest-repo/.git/singlefs-layer0-full-green）
     → 怎么办：这批输入的层 0 全量没在收尾跑过：跑 `bash .claude/gate.d/54-layer0-replay.sh --full`
[退出码 1]
[标记不在]

════ 准备：--full 全绿，标记重新写上
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-23T17:13:34Z）：旧的全绿标记已删；这一道的输入哈希 0e710f0037556efc…（5 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/selftest-repo/.git/singlefs-layer0-full-green（输入哈希 0e710f0037556efc…，5 个文件；开跑 2026-09-23T17:13:34Z，跑完 2026-09-23T17:13:36Z）
[退出码 0]
[标记在：0e710f0037556efc…，16 行]

════ 自证四（另一支）：--full 里 cargo 判红 ⇒ 判红、不写标记
$ env FAKE_CARGO_FAIL=1 bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-23T17:13:36Z）：旧的全绿标记已删；这一道的输入哈希 0e710f0037556efc…（5 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）
running 6 tests (fake cargo, test=first_transaction_step_seven_layer0, include_ignored=1)
test full_enumeration ... FAILED
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）
     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture
                oracle 报的第一条违例在断言消息里。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。
[退出码 1]
[标记不在]

════ 准备：--full 全绿，标记重新写上
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-23T17:13:38Z）：旧的全绿标记已删；这一道的输入哈希 0e710f0037556efc…（5 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/selftest-repo/.git/singlefs-layer0-full-green（输入哈希 0e710f0037556efc…，5 个文件；开跑 2026-09-23T17:13:38Z，跑完 2026-09-23T17:13:40Z）
[退出码 0]
[标记在：0e710f0037556efc…，16 行]

════ 外加：--full 跑到第二条流时改了一个输入 ⇒ 判红、不写标记
$ env FAKE_CARGO_TOUCH_DURING_SECOND=/tmp/claude-1000/gate54-tiering/selftest-repo/research/results/e142-sample.out bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-23T17:13:40Z）：旧的全绿标记已删；这一道的输入哈希 0e710f0037556efc…（5 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✗ 全量跑的过程中这一道的输入变了（开跑 0e710f0037556efc…，跑完 40a66be9aae25aac…）：两条流读到的不一定是同一版，不写全绿标记
       不同的文件共 1 个（最多列 20 个）：
         内容不同：research/results/e142-sample.out
     → 怎么办：等改动停下再跑 --full；别的会话在改这些路径时，在只含这一批的 worktree 里跑 --full <它的根>。
[退出码 1]
[标记不在]

════ 外加：--staged 那条路（照 gate.sh --staged 的建法：worktree add HEAD + 套暂存区的 diff）
M  crates/singlefs-harness/src/lib.rs
A  crates/singlefs-harness/src/new_module.rs
════ 工作区里跑 --full（新文件已暂存，工作区与暂存区一致）
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-23T17:13:42Z）：旧的全绿标记已删；这一道的输入哈希 9266375194117e35…（6 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/selftest-repo/.git/singlefs-layer0-full-green（输入哈希 9266375194117e35…，6 个文件；开跑 2026-09-23T17:13:42Z，跑完 2026-09-23T17:13:45Z）
[退出码 0]
[标记在：9266375194117e35…，17 行]

════ 临时 worktree 里跑快档（读 common-dir 里同一份标记）⇒ 判绿
$ env SINGLEFS_GATE_FULL=1 bash /tmp/claude-1000/gate54-tiering/selftest-staged-worktree/.claude/gate.d/54-layer0-replay.sh /tmp/claude-1000/gate54-tiering/selftest-staged-worktree
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 全绿标记与这批输入的内容哈希相同（9266375194117e35…，6 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）：层 0 全量跑完于 2026-09-23T17:13:45Z，标记里的计数行原样：
      LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
      CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
      LAYER0B states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
[退出码 0]
[标记在：9266375194117e35…，17 行]

════ 工作区多了别的会话没暂存的一个输入文件，再跑 --full
$ bash .claude/gate.d/54-layer0-replay.sh --full
  · --full 开跑（2026-09-23T17:13:46Z）：旧的全绿标记已删；这一道的输入哈希 bfc9c3097786eb7a…（7 个文件，登记路径 crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/）
    LAYER0_PROGRESS slice=1/2 states=[0,131083) segments=0..=9 finished_slices=1/2 finished_states=131083/262165 elapsed_seconds=1.0
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-2.1=4/0
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=9 no_file=1 file_read=2 failed=0 journal_differing=0 verification_ran=1 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 first_violation=none
  ✓ 全绿标记写进 /tmp/claude-1000/gate54-tiering/selftest-repo/.git/singlefs-layer0-full-green（输入哈希 bfc9c3097786eb7a…，7 个文件；开跑 2026-09-23T17:13:46Z，跑完 2026-09-23T17:13:46Z）
[退出码 0]
[标记在：bfc9c3097786eb7a…，18 行]

════ 临时 worktree 里跑快档 ⇒ 判红，列出那一个文件
$ env SINGLEFS_GATE_FULL=1 bash /tmp/claude-1000/gate54-tiering/selftest-staged-worktree/.claude/gate.d/54-layer0-replay.sh /tmp/claude-1000/gate54-tiering/selftest-staged-worktree
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但全绿标记记的输入哈希（bfc9c3097786eb7a50ef770d03164775c56ae01aad6e3d0d70d1b6d6b7afba7d）与这一次的（9266375194117e353e537abbbbe3a99879d65fd0cae45fe8a63f6c95beb80fd3，6 个文件）不同
       不同的文件共 1 个（最多列 20 个）：
         只在标记里：research/results/e999-other-session.out
     → 怎么办：这批输入的层 0 全量没在收尾跑过：跑 `bash .claude/gate.d/54-layer0-replay.sh --full`
                不同的文件里有别的会话没提交的东西时，工作区跑的全量罩不到这一批：在只含这一批的 worktree 里跑 --full <它的根>。
[退出码 1]
[标记在：bfc9c3097786eb7a…，18 行]

════ 参数：认不出的参数判红
  ✗ 认不出的参数：--fast
     → 怎么办：只认 --full 与项目根，顺序不限：bash .claude/gate.d/54-layer0-replay.sh [--full] [项目根]
[退出码 2]
```
