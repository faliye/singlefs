# 第十一批 landing-audit-b11-watch-wording：看门狗按提示的草稿目录取 progress.md、合入后验证那一步的例外罩上脚本给的重跑命令（tooling-writer，2026-10-23）

## 结论

- 看门狗（九十节 ㉚ 的 `agent-watch.py` 那一处、判决 K3「看门狗取别的子 agent 的 progress.md」）：`progress_file_of` 改成调 `.claude/hooks/lib_prompt_scratch_line.py` 的 `scratch_directories`，只认派发提示行首「草稿目录：」那一行，只取那个目录下的 `progress.md`。没有那一行的取不到，告警里写「派发提示里没有行首「草稿目录：」一行，取不到 progress.md（不到提示点名的别处去找）」，不到别处去猜。在真会话记录上，第一条记录在 2026-10-22 及之后的 231 份里：取到别人 progress.md 的从改前 22 份降到改后 0 份，取到自己的 40 份前后不变。只算攻方清单里的 216 份，改前读数正好复现攻方的 21 / 37 / 158，改后是 0 / 37 / 179。全部 3358 份里是 41 份降到 0 份，取到自己的 100 份不变。
- 例外说法（判决 K2「例外说法窄」、攻方 K2-4a）：五处改成同一句，「合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步与它给出的只重跑那几格的命令（那一步红了、本次未跑或没带前缀时打出的那一条）」，后面照旧指到 `.claude/main-agent.md`「写与验分两段」验证段第 1 步。五处是 `.claude/main-agent.md:22`、`.claude/rules/implementation-workflow.md:45`、`:47`、`.claude/agent-common.md:58`、`.claude/hooks/heavy-test-guard.sh:847`。别的场合照旧先弹窗，没有一处写宽。`heavy-test-guard.sh --selftest` 退 0，核拒绝说明原文的那一格（第 1979 行）跟着改了。
- 推翻条件：看门狗告警里附的 progress.md 仍然落在本提示草稿目录之外；或者派发提示里有行首「草稿目录：」一行、那个目录下也有 progress.md，看门狗却报取不到；或者主 agent 定案「单跑那两格本来就该弹窗」（攻方 K2-4a 写的推翻条件）。

## 改过的文件

`git diff --stat`（工作区对暂存区。换之前核过五份的暂存区与工作区都等于我取的基线 `base/`，所以这就是这一批的改动）原样：

```
 .claude/agent-common.md                  |  2 +-
 .claude/hooks/heavy-test-guard.sh        |  4 +-
 .claude/main-agent.md                    |  2 +-
 .claude/rules/implementation-workflow.md |  4 +-
 scripts/agents/agent-watch.py            | 77 ++++++++++++++++++++++++--------
 5 files changed, 65 insertions(+), 24 deletions(-)
```

换上之后的 sha256：

```
dc885cfbe4640a8b4656bb2893ad7d7108b2222e0bc5ca5e5c334e58ce3da1a6  scripts/agents/agent-watch.py
05e21c5a18f098c99580f7b5f12571989246251c63360573601c217831a71acc  .claude/main-agent.md
551ded653089e108849c08747ba7d0b0cd987c2167a7fcbbdcdfe36a71622ae1  .claude/agent-common.md
58958652f69ae6bd79ee3096d9be8254b0268d5ed93194a454a4398a6b119741  .claude/rules/implementation-workflow.md
3b6dd366b8b86526667f9df65e43f928bc34a46d2f005e6ba12108f138a5c10e  .claude/hooks/heavy-test-guard.sh
```

换上的办法：
- `scripts/agents/agent-watch.py` 照 b9 的做法换。先在草稿树里改拷贝（`.claude` 与 `scripts` 别的子目录都链接到主工作区），再用 `make_spec.py` 按「基线 → 草稿」的差出规格。规格施加到基线上之后核过与草稿逐字相同，反向规格也核过能回到基线。核过主工作区与基线逐字相同，先跑 `replace-batch.py --dry-run`，再真换（9 处）。换上之后核过主工作区与草稿逐字相同。为了在开关表里补一处点名，又按同样做法换了一次（1 处，规格 `spec-watch-second.json`）。
- `.claude/hooks/heavy-test-guard.sh` 经 `scripts/gate/hook-swap.py` 换（规格 `spec-heavy-guard.json`，2 处）。草稿里语法检查、内嵌 python 编译、内置喂法 28 条都过了；换上之后在主工作区里再验一遍，同样都过。
- 三份 md 用 `scripts/kb/replace-once.py` 定点改，四处各命中 1 次，回读确认过。
- 回滚用的规格：`reverse-spec-watch-second.json`、`reverse-spec-watch.json`（要先倒第二份，再倒第一份），`hook-swap/reverse-20261023T051048Z-3273233.json`。三份 md 要回滚就把下面的新旧原文对调，用 `replace-once.py` 改回去。

## 看门狗：按提示的草稿目录取 progress.md

改了什么（`scripts/agents/agent-watch.py`）：
- 文件开头另 import `lib_prompt_scratch_line`，与 `lib_work_root` 在同一个目录 `.claude/hooks`，那个目录已经在 sys.path 上了。
- 新加 `own_scratch_directories(transcript)`：直接调库里的 `scratch_directories(transcript.dispatch_prompt)`。认法全在库里，这里不另写一份。
- `progress_file_of`：候选只取 `own_scratch_directories` 列出的每个目录下的 `progress.md`。提示里写了几行草稿目录的，取改得最晚的那一份；都没有就返回 None。弄坏开关 `AGENT_WATCH_BREAK=progressanypath` 下照改前的取法走（提示里点名的工作根下的路径都算候选）。`SCRATCH_PATH_MENTION` 现在只给这个开关用，注释里写明了。
- `progress_note`：提示里没有那一行时，写「；派发提示里没有行首「草稿目录：」一行，取不到 progress.md（不到提示点名的别处去找）」。有那一行而那个目录下没有 progress.md 的，照旧写「派发提示点名的草稿目录下没有 progress.md」。
- 自证：`selftest_scratch_mentions` 的三格原来的提示写的是「跑前登记 x。草稿目录：<目录>/，报告写 …」，草稿目录不在行首，改成骨架那样的独立一行。另加 3 格（下面的红绿样本）。`selftest_session_watch_in` 的两份提示原来是句中的「，草稿目录 <目录>/」，同样改成独立一行。成功句加了一句「progress.md 只在派发提示行首「草稿目录：」那一行点名的目录下找：提示里另点名的别的子 agent 的目录不取，没有那一行的报取不到」。
- 文件头开关表：新加 `progressanypath`，点名三格。`SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` 那一条补点名新加的那一格，因为实测它在这个开关下也红。

新写的弄坏开关（原文）：

```
  AGENT_WATCH_BREAK=progressanypath  progress_file_of() 照改前的取法：派发提示里点名的工作根下的路径都算候选、取改得最晚的那一份，不只认行首「草稿目录：」那一行
      selftest_scratch_mentions（「✗ 自检：派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告…」、「✗ 自检：派发提示没有行首「草稿目录：」一行、只点名别的子 agent 的报告…」、
      「✗ 自检：派发提示没有行首「草稿目录：」一行时，告警里附的一句应当写取不到 progress.md…」）
```

先造的该红的输入（`probe_cells.py`，四格，直接调 `progress_file_of`。工作根底下建两个目录，别人那份 progress.md 改得更晚）：
- 改前基线：`probe_cells-before.out` 末行「共 4 格，红 4 格」。例：「红  本提示草稿目录一行 + 叫它先读别的子 agent 的报告（别人的 progress.md 更新）→ 取自己的：应当 …/b11-probe-own-nbybyw_w/progress.md，实际 …/b11-probe-other-lgwhz1ot/progress.md」。
- 改后草稿：`probe_cells-after.out` 末行「共 4 格，红 0 格」。
- 改后加 `AGENT_WATCH_BREAK=progressanypath`：`probe_cells-after-break.out` 末行「共 4 格，红 4 格」。

自证（在主工作区、换上之后跑；`.claude/hooks/selftest-registry.tsv` 里登记为 light）：
- `nice -n 19 python3 scripts/agents/agent-watch.py --selftest`：退 0，159 秒。最后一次换上之后又跑了一次（`watch-selftest-final.out`），同样退 0、`✗ 自检` 0 行，成功句里「--ack 落文件下一次读回、条件消失的确认作废（36 格）」。`selftest_scratch_mentions` 从 3 格加到 6 格，这一段的格数从 33 变成 36（33 是从改动推的，基线没在副本里重跑，见「没做什么」）。
- `AGENT_WATCH_BREAK=progressanypath`：退 1，红的正好是点名的三格（`watch-selftest-break-progressanypath.out`）：
  「✗ 自检：派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告（那边的 progress.md 改得更晚）：应当只取本提示草稿目录下的，应当 …」
  「✗ 自检：派发提示没有行首「草稿目录：」一行、只点名别的子 agent 的报告：应当取不到，应当 None，实际 …」
  「✗ 自检：派发提示没有行首「草稿目录：」一行时，告警里附的一句应当写取不到 progress.md，实际「；…」」
- `SINGLEFS_WORK_ROOT_BREAK=new-root-unknown`：退 1，红 5 格。点名的四格（progressstale、整点询问、工作根两格）都在，另外红的一格是新加的「派发提示有本提示的草稿目录一行…」，开关表里已经补上点名。
- `SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown`：退 1，红 1 格，就是点名的「派发提示点名过渡期旧根下的草稿目录」。
- 语法检查：`python3 -m py_compile scripts/agents/agent-watch.py` 通过。

### 真会话记录上量改前改后

量法：`measure_real.py`。读 `~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/agent-*.jsonl`，每份取第一条 user 记录当派发提示，取它的时刻当派发时刻。改前、改后两份 `agent-watch.py` 都按路径导入，分别调各自的 `progress_file_of`，文件系统取的都是此刻同一个样子。「本提示草稿目录」按库里的 `scratch_directories` 算，与攻方探针 `watch_progress.py` 同一个分法。只读。

第一条记录在 2026-10-22 及之后、限攻方清单 `research/prompts/landing-audit-r5-opus-model/out/corpus-agents.txt` 的 216 份（`measure-r5-list-since-1022.out`，每行截到例子之前）：
```
改前｜有行首草稿目录一行｜取到别的子 agent 的：21 份
改前｜有行首草稿目录一行｜取到本提示草稿目录下的：37 份
改前｜有行首草稿目录一行｜取不到：158 份
改后｜有行首草稿目录一行｜取到本提示草稿目录下的：37 份
改后｜有行首草稿目录一行｜取不到：179 份
改前改后不同｜有行首草稿目录一行｜取到别的子 agent 的 → 取不到：21 份
```
改前这三个数与攻方 `out/watch_progress.out` 的 21 / 37 / 158 一样。

第一条记录在 2026-10-22 及之后、不限清单的 231 份（`measure-all-since-1022.out`）：改前取到别人的 22 份、取到自己的 40 份、取不到 169 份；改后取到别人的 0 份、取到自己的 40 份、取不到 191 份；改前改后不同的只有「取到别人的 → 取不到」那 22 份。

全部 3358 份（`measure-all-time.out`）：有那一行的，改前取到别人的 41 份、取到自己的 100 份、取不到 1884 份，改后是 0 / 100 / 1925；没有那一行的 1333 份，改前改后都取不到。取到自己的那几份没有一份前后不同。

例（改前取到别人的，改后取不到）：`ac39106511365e719`（implementation-writer）的本提示草稿目录是 `…/m4-quarantine-mark-gpu-grouping`，改前取到的是 `/home/fy5090/.local/share/singlefs/work/m4-quarantine-mark-writer/progress.md`（提示里叫它先读的那个实现员的目录），改后是 None。例（改前改后都取到自己的）：`a21d15f91b9bc333c`（implementation-writer）→ `/home/fy5090/.local/share/singlefs/work/m4-quarantine-mark-writer/progress.md`。

这一格跟着此刻的文件系统变，过几天重跑数会不一样。不变的是「改后取到别人的为 0」：只要 `scratch_directories` 不变，改后的候选就只有本提示草稿目录下的那几份。

## 例外罩上脚本给的重跑命令

每一处改前与改后的原文都是那一行里的一段，整行太长，只抄改到的那一段。`merge-and-verify.sh` 给出的重跑命令由 `checker_quick_rerun_command` 打（`scripts/gate/merge-and-verify.sh:655` 起），本次未跑、`--copy`、没带前缀、GPU 忙、红了这几处出路都用它（第 678、684、697、710、735 行）。

`.claude/main-agent.md:22`
- 改前：「合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步除外，不弹窗，照「写与验分两段」一节验证段第 1 步跑。」
- 改后：「合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步与它给出的只重跑那几格的命令（那一步红了、本次未跑或没带前缀时打出的那一条）除外，不弹窗，照「写与验分两段」一节验证段第 1 步跑。」

`.claude/rules/implementation-workflow.md:45`（这一处原来少「带 `SINGLEFS_HEAVY_TESTS=user-request`」，为了逐处一致一起补上了）
- 改前：「（合入后验证里 `scripts/gate/merge-and-verify.sh` 起的 checker 档快档那一步除外：不弹窗，跑法照 `.claude/main-agent.md`「写与验分两段」验证段第 1 步）」
- 改后：「（合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步与它给出的只重跑那几格的命令（那一步红了、本次未跑或没带前缀时打出的那一条）除外：不弹窗，跑法照 `.claude/main-agent.md`「写与验分两段」验证段第 1 步）」

`.claude/rules/implementation-workflow.md:47`
- 改前：「（合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步除外，跑法照 `.claude/main-agent.md`「写与验分两段」验证段第 1 步）」
- 改后：「（合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步与它给出的只重跑那几格的命令（那一步红了、本次未跑或没带前缀时打出的那一条）除外，跑法照 `.claude/main-agent.md`「写与验分两段」验证段第 1 步）」

`.claude/agent-common.md:58`
- 改前：「合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步是例外：同样只由主 agent 跑，不弹窗，」
- 改后：「合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步与它给出的只重跑那几格的命令（那一步红了、本次未跑或没带前缀时打出的那一条）是例外：同样只由主 agent 跑，不弹窗，」

`.claude/hooks/heavy-test-guard.sh:847`（`MERGE_VERIFY_QUICK_STEP`）与第 1979 行（自证里核拒绝说明原文的那一格），两处的串同样改：
- 改前：「合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步不另弹窗，跑法照 .claude/main-agent.md「写与验分两段」验证段第 1 步」
- 改后：「合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步与它给出的只重跑那几格的命令（那一步红了、本次未跑或没带前缀时打出的那一条）不另弹窗，跑法照 .claude/main-agent.md「写与验分两段」验证段第 1 步」
- 第 846 行注释、第 1977 行注释、第 1978 行格名、文件头第 320–321 行开关表都没改：它们的说法没有因为这次改动变错，只是没提重跑命令；派发提示只许改那一句。

先造的该红的输入（`probe_wording.py`，6 处：四处 md 那一行里有没有那一句、那一句之后有没有指到「写与验分两段」验证段第 1 步；钩子的常量与自证那一格是不是那一句）：
- 改前基线（`base/` 拼成的树）：`probe_wording-before.out` 末行「共 6 处，红 6 处」。
- 改后主工作区：`probe_wording-after.out` 末行「共 6 处，红 0 处」。

自证（主工作区、换上之后跑；登记为 light）：
- `nice -n 19 bash .claude/hooks/heavy-test-guard.sh --selftest`：退 0，6 秒，`✗` 0 行。末行开头原样：「  ✓ 自检通过（查了 1402 种，其中该拒 266 种）：子 agent 跑 checker 档（按包判）、全量测试、check.sh、整轮门禁、…」
- `HEAVY_TEST_GUARD_BREAK=merge-verify-quick-step-dropped`：退 1，只红点名的那一格：「  ✗ 自检：stdin:拒绝时 stderr 写了合入后验证里 merge-and-verify.sh 起的 checker 档快档那一步不另弹窗、指到验证段第 1 步 应当是 1，实际 0」
- `bash -n .claude/hooks/heavy-test-guard.sh` 通过。hook-swap.py 换上前后两次都过了语法检查、内嵌 python 1 段编译、内置喂法 28 条（`hook-swap.out`）。

## 自证总数

| 自证 | 开关 | 退出码 | 读数 |
|---|---|---|---|
| `agent-watch.py --selftest` | 不设 | 0 | `✗ 自检` 0 行；换上之后跑了两次 |
| 同上 | `AGENT_WATCH_BREAK=progressanypath` | 1 | 红 3 格，就是点名的三格 |
| 同上 | `SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` | 1 | 红 5 格，点名的四格都在，新加的一格已补进点名 |
| 同上 | `SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown` | 1 | 红 1 格，就是点名的那一格 |
| `heavy-test-guard.sh --selftest` | 不设 | 0 | 查了 1402 种 |
| 同上 | `HEAVY_TEST_GUARD_BREAK=merge-verify-quick-step-dropped` | 1 | 红 1 格，就是点名的那一格 |
| `probe_cells.py`（草稿目录取法） | 改前 / 改后 / 改后加开关 | — | 红 4 / 0 / 4（共 4 格） |
| `probe_wording.py`（五处说法） | 改前 / 改后 | — | 红 6 / 0（共 6 处） |

一共起了 6 次 `--selftest`、5 次探针。

## 新写的 `# gate-similar:` / `# hook-events:` 行

没有：这一批没有新建门禁阶段、钩子或研究脚本。`gate-overlap.py --list` 跑过（`gate-overlap-list.out`，59 行），管 progress.md 取法的只有 `agent-watch.py` 这一处，例外说法只在 `heavy-test-guard.sh`，没有别的门禁或钩子判这两件事。

## 第二种活改过的定义、共用约束与规则

- `.claude/main-agent.md`（「一轮怎么开、怎么收」第 3 条）
- `.claude/agent-common.md`（「不做」一节「重型测试就是 checker 档那一批」那一条）
- `.claude/rules/implementation-workflow.md`（「checker 档只在提交时跑，harness 随时跑」一节表里「用户要求」「其余任何时候」两行）

这三份进这一批收尾的三方 landing-audit-r6，归主 agent 开（门禁 doc-process-records 的 agent-def-adversarial-review 格）。`.claude/main-agent.md`「一轮怎么开、怎么收」第 11 条与 `.claude/agent-common.md` 第 85 行一带没有碰。

## 要主 agent 跑的命令

在主工作区 `/home/fy5090/code/singlefs` 跑，不起 cargo，不用内存包装与线程上限：

1. 文件头开关表的写法（`agent-watch.py` 的开关表改过）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-headers 格; 范围=1 格; 为什么=agent-watch.py 文件头开关表新加 progressanypath 一条、改了 new-root-unknown 的点名' bash .claude/gate.d/code-tooling.sh --check break-switch-headers /home/fy5090/code/singlefs`
   看末行是不是 ✓，查了几项；红的话看点名的是不是 `scripts/agents/agent-watch.py`。
2. 提交前的整轮门禁照常跑，其中 rules-lint、doc-lint 管三份 md 的新句子，code-tooling 的 research-script-selftests 格会再起一次 `agent-watch.py --selftest`。
3. 复跑方 break-switch-replay 格：`agent-watch.py` 与 `heavy-test-guard.sh` 都不在 `.claude/gate.d/break-switch-replay-files.tsv` 里，那一格不会起它们。我把两份的新旧开关都手跑过了，读数见「自证总数」。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没跑门禁阶段与各类 lint；没走定义三方；没提交。命名检查不适用：这一批没改 `.rs`。
- 基线 `agent-watch.py` 的 `--selftest` 没重跑。草稿树里也没跑改过的自证：重型测试闸不放工作根副本里改过的脚本的自证。自证都在换上之后的主工作区里跑。成功句里 33 → 36 格是从改动推的，没量过。
- `.claude/hooks/lib_prompt_scratch_line.py` 文件头「四处用这一份」那张表还没把 `scripts/agents/agent-watch.py` 列进去。那份文件不在我能改的范围里（派发提示只许改 `heavy-test-guard.sh` 那一句），交主 agent 定，或者并进兄弟工具员的批。
- 九十节 ㉚ 另两处（`scripts/agents/dispatch-prompt.py` 的 `DRAFT_LINE`、`scripts/agents/agent-cost.py` 的草稿目录认法）照判决留在下一批，没碰。
- `.claude/main-agent.md`「写与验分两段」验证段第 1 步本身没写那条只重跑红格的命令怎么跑。五处都指到那一步，那一步的正文不在派发提示列的文件里，没改；要不要补，交主 agent。
- `.claude/hooks/selftest-registry.tsv` 里 `agent-watch.py` 那一行的档位理由没改：新加的几格只建目录与文件，不起进程，档位不变。
- 工作根底下有一份 `/home/fy5090/.local/share/singlefs/work/agent-watch-selftest-scratch-ts3nxvwv`（只有一份 67 字节的 progress.md，修改日期是 2026-10-22），不是我这一批跑出来的，没删。

## 草稿目录里有什么

`/home/fy5090/.local/share/singlefs/work/landing-audit-b11-watch-wording/`：
- 改前基线 `base/`（五份），以及第二次换之前的 `base-second/agent-watch.py`；`base.sha256`
- 规格与反向规格：`spec-watch.json`、`spec-watch-second.json`、`reverse-spec-watch.json`、`reverse-spec-watch-second.json`、`spec-heavy-guard.json`，以及 `hook-swap/` 里的反向规格、喂法日志、检出记录
- 探针与驱动：`probe_cells.py`、`probe_wording.py`、`measure_real.py`、`make_spec.py`
- 每一次跑的输出：`*.out`、`watch.diff`
- 草稿树 `tree/`（928K）与 `tree-base/`（924K）交回前已经删了（`.claude` 与 `scripts` 下别的子目录在里面都只是符号链接）。现在这里没有仓副本、编译目录或库拷贝。
