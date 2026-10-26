# 第十批 landing-audit-b10-defs-sync：合入后验证那一步的例外、调度表两行同步（2026-10-23）

## 结论

K2-1、K2-2、K2-3 三条的验收都做到了。改了四份文件，都只用定点替换。`.claude/main-agent.md`「一轮怎么开、怎么收」第 11 条、`.claude/agent-common.md` 第 85 行一带没碰；拿 `diff <备份> <现文件>` 核过，变的只有下表这几行。
`.claude/hooks/heavy-test-guard.sh` 经 `scripts/gate/hook-swap.py` 换上（草稿验过、换上之后再验过，退 0）。换上后在主工作区跑 `--selftest` 退 0；打开新开关 `merge-verify-quick-step-dropped` 后退 1，红的只有新加的那一格。

会推翻这个结论的现象：主 agent 照验证段第 1 步的写法起 `merge-and-verify.sh`，仍被重型闸或范围声明闸拒；或者不带开关跑 `bash .claude/hooks/heavy-test-guard.sh --selftest` 不退 0。

## 改过的文件与 git diff --stat

```
$ git diff --stat -- .claude/main-agent.md .claude/agent-common.md .claude/rules/implementation-workflow.md .claude/hooks/heavy-test-guard.sh
 .claude/agent-common.md                  |  2 +-
 .claude/hooks/heavy-test-guard.sh        | 11 +++++++++--
 .claude/main-agent.md                    | 10 +++++-----
 .claude/rules/implementation-workflow.md |  4 ++--
 4 files changed, 17 insertions(+), 10 deletions(-)
```

改前改后的 sha256：

| 文件 | 改前 | 改后 |
|---|---|---|
| `.claude/main-agent.md` | `3aa8c2adad2356f358223fbf7a03307165331897baedc715283836237f0ee32b` | `a4e47554bdfff1e769baaff8c628651490da715ca1061af9f58e3b43097cff83` |
| `.claude/agent-common.md` | `1fe5d1cc589b34b0f9694d382c53d793e985fe033aece0237cef5aa475cd5a57` | `d8c4c78739dd099e4dbf37d2548858e78e25cec3d34aa57d2293a59840183cb2` |
| `.claude/rules/implementation-workflow.md` | `3ae3d752739cc509a832d261d49c1c25fdb69b1e8ffb531f1dd86ac8b638e4b3` | `bc87d0562a62bb0472e0bb5d87dcc6d5b0ff6dd47d6593158b5a759fb755dd5f` |
| `.claude/hooks/heavy-test-guard.sh` | `7964d2bca6ea111fe37675c80dbdec374eb16af702bd1a0914568d7dd1d41d40` | `dc7c21bc85fdf7a2aa384c3391f1cf5fecc4fc88da56e163d1be7770e012372f` |

三份 `.md` 的改前原文备份在 `/home/fy5090/.local/share/singlefs/work/landing-audit-b10-defs-sync/originals/`（清单 `manifest.tsv`），逐行 diff 在同一目录的 `diff-main-agent.txt`、`diff-agent-common.txt`、`diff-implementation-workflow.txt`。按 diff 的头行数，改了这些行：main-agent `7c7`、`22c22`、`29c29`、`96c96`、`102c102`；agent-common `58c58`；implementation-workflow `45c45`、`47c47`。

## 第二种活改过的定义、共用约束与规则（交 landing-audit-r5 三方）

- `.claude/main-agent.md`（第 7、22、29、96、102 行）
- `.claude/agent-common.md`（第 58 行）
- `.claude/rules/implementation-workflow.md`（第 45、47 行）

钩子 `.claude/hooks/heavy-test-guard.sh` 也在这一批的 diff 里。

## K2-1 合入后验证那一步的例外

每一处都写这同一个例外：合入后验证里由 `scripts/gate/merge-and-verify.sh` 起、带 `SINGLEFS_HEAVY_TESTS=user-request` 的 checker 档快档那一步不弹窗；跑法指到 `.claude/main-agent.md`「写与验分两段」验证段第 1 步，不另写判据。别的场合照旧先弹窗。规则文件里没写来历，也没写日期。

**`.claude/main-agent.md` 第 22 行（「一轮怎么开、怎么收」第 3 条）**
- 改前：`提交之外任务确实要跑，先弹窗问用户，同意了才跑。`
- 改后：`提交之外任务确实要跑，先弹窗问用户，同意了才跑；合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步除外，不弹窗，照「写与验分两段」一节验证段第 1 步跑。`

**`.claude/rules/implementation-workflow.md` 第 45 行（场合表「用户要求」那一行，末尾）**
- 改前：`命令带 `SINGLEFS_HEAVY_TESTS=user-request`；任务确实要跑时主 agent 先弹窗问用户 |`
- 改后：`命令带 `SINGLEFS_HEAVY_TESTS=user-request`；任务确实要跑时主 agent 先弹窗问用户（合入后验证里 `scripts/gate/merge-and-verify.sh` 起的 checker 档快档那一步除外：不弹窗，跑法照 `.claude/main-agent.md`「写与验分两段」验证段第 1 步） |`

**`.claude/rules/implementation-workflow.md` 第 47 行（场合表「其余任何时候」那一行）**
- 改前：`| 其余任何时候 | 重型不跑；子 agent 只跑 harness 与形式化那一条轻型 |`
- 改后：`| 其余任何时候 | 重型不跑（合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步除外，跑法照 `.claude/main-agent.md`「写与验分两段」验证段第 1 步）；子 agent 只跑 harness 与形式化那一条轻型 |`

**`.claude/agent-common.md` 第 58 行（「不做」一节重型测试那一条，中段那一句）**
- 改前：`重型测试只由主 agent 在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`），做法在 `.claude/skills/crash-test/commit-stages.md` 与 `.claude/skills/gate/staged-gate.md`。`
- 改后：`重型测试只由主 agent 在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`），做法在 `.claude/skills/crash-test/commit-stages.md` 与 `.claude/skills/gate/staged-gate.md`；合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步是例外：同样只由主 agent 跑，不弹窗，跑法照 `.claude/main-agent.md`「写与验分两段」验证段第 1 步。`

**`.claude/hooks/heavy-test-guard.sh` 拒绝说明那一句**（改前在第 857 行，换上后在第 861 行；子 agent 被拒时 stderr 里「→ 规矩」下面那几行之一）。换法照 DEVICE_TEST_CASE 那一句的写法：例外句是一个常量，有一个弄坏开关拿掉它，再加一格自证核它。共五处定点替换，规格在 `/home/fy5090/.local/share/singlefs/work/landing-audit-b10-defs-sync/spec/heavy-test-guard.json`：

1. 第 409 行 `KNOWN_GUARD_BREAK_ITEMS` 末尾加 `"merge-verify-quick-step-dropped"`。
2. 文件头开关表在 device-test-case-dropped 那一项后面加两行（第 320、321 行）：
   `#                                   #   HEAVY_TEST_GUARD_BREAK=merge-verify-quick-step-dropped  子 agent 看到的拒绝说明里拿掉「合入后验证里 scripts/gate/merge-and-verify.sh 带 =user-request 起的 checker 档快档那一步不另弹窗……」这一句`
   `#                                   #       「stdin:拒绝时 stderr 写了合入后验证里 merge-and-verify.sh 起的 checker 档快档那一步不另弹窗、指到验证段第 1 步」（共 1 格）`
3. 第 846、847 行新常量：
   `MERGE_VERIFY_QUICK_STEP = "" if guard_break_is_set("merge-verify-quick-step-dropped") else "合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步不另弹窗，跑法照 .claude/main-agent.md「写与验分两段」验证段第 1 步"`
4. 拒绝说明那一句：
   - 改前：`"→ 提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑；" + (DEVICE_TEST_CASE + "；" if DEVICE_TEST_CASE else "") +`
   - 改后：`"→ 提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑；" + (MERGE_VERIFY_QUICK_STEP + "；" if MERGE_VERIFY_QUICK_STEP else "") + (DEVICE_TEST_CASE + "；" if DEVICE_TEST_CASE else "") +`
5. 自证在 device-test-case 那一格后面加一格（第 1976 到 1978 行）：`stdin:拒绝时 stderr 写了合入后验证里 merge-and-verify.sh 起的 checker 档快档那一步不另弹窗、指到验证段第 1 步`。这一格核的是 `refused.stderr` 里有没有整句例外原文（refused 是实现员跑 `cargo test --all` 被拒那一次）。

核拒绝说明原文的那几格：「拒绝时 stderr 写了出路与前缀」核 `弹窗` 两个字，「入库装置……」核 DEVICE_TEST_CASE 那句原文。两样都还在 stderr 里，所以这两格不用改。

按派发提示，「只改拒绝说明里那一句」。加开关与自证格这几处，依据是 tooling-writer 定义「做什么」第 2 步：每条改法都要有一个该红的输入。开关名派发提示里没给，是我起的。

## K2-2 调度表里的命令补前缀

`.claude/main-agent.md` 里写出 merge-and-verify 整条命令的地方有四处：第 7 行（「职责」）、第 29 行（「一轮怎么开、怎么收」第 10 条）、第 52 行（验证段第 1 步本身）、第 96 行（调度表）。第 44 行（写段第 3 步）只写了脚本名，没写整条命令，不改。
第 7、29、96 三处的命令前面都补上两个前缀，补成同一个样子：`SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=合入后验证; 范围=整轮; 为什么=<这一批补丁碰了哪些包>' SINGLEFS_HEAVY_TESTS=user-request bash scripts/gate/merge-and-verify.sh …`。种类写 whole、为什么写这一批补丁碰了哪些包，都照 `.claude/hooks/run-scope-guard.sh` 文件头「合入后验证」那一行（第 32、33 行）。`.claude/hooks/lib_run_scope.py` 第 778、779 行那一格「merge-and-verify 声明成 whole：放」用的也是 `whole / 合入后验证 / 整轮`。

**第 7 行**
- 改前：`由主 agent 跑 `bash scripts/gate/merge-and-verify.sh [--skip-apply] <补丁目录…>`（打全部补丁`
- 改后：`由主 agent 跑 `SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=合入后验证; 范围=整轮; 为什么=<这一批补丁碰了哪些包>' SINGLEFS_HEAVY_TESTS=user-request bash scripts/gate/merge-and-verify.sh [--skip-apply] <补丁目录…>`（打全部补丁`

**第 29 行**
- 改前：`主 agent 跑 `bash scripts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>` 做「合入后验证」`
- 改后：`主 agent 跑 `SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=合入后验证; 范围=整轮; 为什么=<这一批补丁碰了哪些包>' SINGLEFS_HEAVY_TESTS=user-request bash scripts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>` 做「合入后验证」`

**第 96 行（两处定点）**
- 改前：`主 agent 跑 `bash scripts/gate/merge-and-verify.sh [--memory 上限] [--threads N] <补丁目录…>`：打全部补丁`
- 改后：`主 agent 跑 `SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=合入后验证; 范围=整轮; 为什么=<这一批补丁碰了哪些包>' SINGLEFS_HEAVY_TESTS=user-request bash scripts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>`（打过与没打过的补丁分两条、依次跑，照「写与验分两段」一节验证段第 1 步）：打全部补丁`
- 改前：`的测试、fmt、clippy、build，红的逐条列出并给只重跑那几条的命令；崩溃与变异两条轻型`
- 改后：`的测试、fmt、clippy、build，再跑 checker 档快档里单元键对不上的测试函数，红的逐条列出并给只重跑那几条的命令；崩溃与变异两条轻型`

后一处「再跑 checker 档快档……」用的是判决 K2-2 一格采纳的攻方改法里「列举里补 checker 档快档」那一半，没另写判据。快档那一步怎么跑、什么时候报本次未跑，照旧只写在验证段第 1 步。

第 52 行（验证段第 1 步）本身没改：命令那一段写的是裸的 `bash scripts/gate/merge-and-verify.sh …`，两个前缀写在同一步后面的正文里（「两条命令开头都带 `SINGLEFS_HEAVY_TESTS=user-request`」「开头都带范围声明 `SINGLEFS_RUN_SCOPE`」）。要不要把前缀也写进那一段命令里、让四处一字不差，交主 agent 定。

## K2-3 写回员那一行

**`.claude/main-agent.md` 第 102 行（调度表「定案之后写回 kb」那一行，末尾）**
- 改前：`例外是 `crates/singlefs-checker/invariants/*.md`（不变量清单，只限这些 `.md`，写回员可以写）`
- 改后：`例外是 `crates/singlefs-checker/invariants/*.md` 与 `crates/singlefs-checker/invariants/**/*.md`（不变量清单，下一层与各层子目录里的都算，只限这些 `.md`，写回员可以写）`

措辞跟 `.claude/agents/kb-scribe.md` 第 42 行一样。范围与 `.claude/hooks/agent-write-scope.tsv` 第 12、13 行的两个模式一致。

改完全仓又 grep 了一遍（`-o` 只打命中那一段，原样）：
```
$ grep -rnoF 'invariants/*.md' CLAUDE.md .claude/rules .claude/agents .claude/agent-common.md .claude/main-agent.md .claude/skills .claude/hooks scripts .claude/gate.d | grep -v '/fixtures/'
.claude/agents/kb-scribe.md:26:invariants/*.md
.claude/agents/kb-scribe.md:42:invariants/*.md
.claude/main-agent.md:102:invariants/*.md
.claude/hooks/agent-write-scope.tsv:12:invariants/*.md
$ grep -rnoF 'invariants/**/*.md' .claude/agents .claude/main-agent.md .claude/hooks/agent-write-scope.tsv
.claude/hooks/agent-write-scope.tsv:13:invariants/**/*.md
.claude/agents/kb-scribe.md:26:invariants/**/*.md
.claude/agents/kb-scribe.md:42:invariants/**/*.md
.claude/main-agent.md:102:invariants/**/*.md
```
写了下一层的三份定义行（kb-scribe.md 第 26、42 行，main-agent.md 第 102 行），同一行里也都写了 `**/*.md`；写范围表是第 12、13 两行。

- 定义、规则里还说「只这一层」的：**没有**。`.claude/rules/` 零命中，`.claude/skills/` 零命中，别的 `.claude/agents/*.md` 也零命中。
- `CLAUDE.md`：`grep -nF 'invariants/*.md' CLAUDE.md` 零命中。只有第 80 行目录表 `| `crates/singlefs-checker/invariants/` | 不变量清单（按类拆开），checker 是它的可执行形式 |`，没说写回员能写哪一层，不用交主 agent 改。
- 门禁样本 `.claude/gate.d/fixtures/code-tooling.sh/kb-directories-write-scope-red/.claude/hooks/agent-write-scope.tsv` 只写 `notes/invariants/*.md`。这是那一格的必红样本，故意只盖一层，不算。
- 别的命中都是 `.claude/kb/`（checks-owed、decisions-history 三语，都是 `grep -n … invariants/*.md` 这类当依据用的命令）、`records/` 和 `research/prompts/`（历史材料）。这些都不是写范围的说法，没逐个列；完整的文件单在 `/home/fy5090/.local/share/singlefs/work/landing-audit-b10-defs-sync/grep-invariants-files.txt`（共 101 份）。
- 钩子 `.claude/hooks/write-guard.sh` 第 674 到 677 行四格已经覆盖下一层和子目录里的 `.md` 与 `.rs`，不用改。

## 钩子的样本、自证格与弄坏开关

- 换上之前：先跑 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list`，确认这句拒绝说明归 `.claude/hooks/heavy-test-guard.sh`（挂 PreToolUse[Bash]）管。没有新加门禁或钩子，所以不用写 `# gate-similar:` / `# hook-events:`。
- 草稿：`python3 scripts/gate/hook-swap.py <规格> --hook .claude/hooks/heavy-test-guard.sh --scratch <草稿目录> --dry-run` 退 0，末行原样：
  `✓ 草稿里验过：1 份文件语法检查、内嵌 python 1 段编译、喂了 28 条`
- 换上：同一条命令不带 `--dry-run`，退 0，末两行原样：
  `✓ 草稿里验过：1 份文件语法检查、内嵌 python 1 段编译、喂了 28 条`
  `✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过：1 份文件语法检查、内嵌 python 1 段编译、喂了 28 条`
  反向规格：`/home/fy5090/.local/share/singlefs/work/landing-audit-b10-defs-sync/hook-swap/reverse-20261023T012942Z-3951253.json`。要撤回，就拿它跑一遍 hook-swap.py。
- 绿：主工作区里 `bash .claude/hooks/heavy-test-guard.sh --selftest` 退 0，没有 ✗ 行（`grep -c '✗'` 是 0）。末行开头原样：`✓ 自检通过（查了 1402 种，其中该拒 266 种）：…`。另两行 `✓ lib_heavy_tests 自检通过（查了 787 种…）`、`✓ lib_workspace_cargo_lock 自检通过（查了 52 格）…`。全文在 `selftest-green.txt`。
- 红（新开关）：`HEAVY_TEST_GUARD_BREAK=merge-verify-quick-step-dropped bash .claude/hooks/heavy-test-guard.sh --selftest` 退 1，只有一条 ✗，原样：
  `  ✗ 自检：stdin:拒绝时 stderr 写了合入后验证里 merge-and-verify.sh 起的 checker 档快档那一步不另弹窗、指到验证段第 1 步 应当是 1，实际 0`
  全文在 `selftest-break-merge-verify.txt`。
- 对照（旁边那个开关，看那两格有没有被这次改动带坏）：`HEAVY_TEST_GUARD_BREAK=device-test-case-dropped … --selftest` 退 1。两条 ✗，就是文件头写的那两格（「拒绝时 stderr 写了入库装置……」「主 agent 没带前缀被拒时「→ 照抄」那一行也写了……」）。全文在 `selftest-break-device.txt`。
- 语法：`bash -n .claude/hooks/heavy-test-guard.sh` 退 0。内嵌 python 由 hook-swap 编译过。
- 登记表：`.claude/hooks/selftest-registry.tsv` 第 161 行把这份钩子登记成 light，所以自证是我自己跑的。登记表没改。

## 顺手看到、不归这一批的（交主 agent 定）

1. **`.claude/hooks/heavy-test-guard.sh` 第 958 行，主 agent 被拒时的「→ 照抄」那一行**：`return [f"→ 照抄（提交流程里跑；用户要求时把 commit 换成 user-request，提交之外要跑先弹窗问用户{device_hint}{reason_hint}）：", candidate]`。
   主 agent 没带前缀被拒时，看到的是这一行，不是第 861 行那一串规矩（自证「主 agent 被拒时规矩只留一行出处（不整段贴）」那一格要的就是不整段贴）。所以第 861 行加的例外只有子 agent 看得到；主 agent 不带前缀起 merge-and-verify 被拒时，读到的仍是「提交之外要跑先弹窗问用户」。
   要让主 agent 也看到，改法可以照 `device_hint` 那一处：在 `{device_hint}` 前面拼进 MERGE_VERIFY_QUICK_STEP。另加一格核 `main_lines[0]`，同一个开关下也红。派发提示限定只改第 857 行那一句，这里我没动。
2. **验证段第 1 步（`.claude/main-agent.md` 第 52 行）那一段命令本身没带前缀**：见 K2-2 一节末尾那一段。
3. 判决 K2-1 一格写的是「指到定案」，派发提示要的是「指到 `.claude/main-agent.md`「写与验分两段」验证段第 1 步、不另写一份判据」。我照派发提示写了。规则文件按 `.claude/singlefs-ai-sop/rules/rules-discipline.md`「1. 正文只写四样」，不写谁定的、哪天定的，所以四处都没指到定案原文。

## 要主 agent 跑的命令

工作目录都是主工作区 `/home/fy5090/code/singlefs`。这些都是提交前那一趟的事，不归我交回前跑：

- 门禁 doc-process-records 的 agent-def-adversarial-review 格要用到的那一轮三方（landing-audit-r5），点名三份：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/implementation-workflow.md`。
- 规则纪律（项目本地）阶段与 doc-lint：改了 `.claude/rules/implementation-workflow.md`、`.claude/main-agent.md`、`.claude/agent-common.md`，归提交前 `scripts/gate/gate-staged.sh` 整轮判（按 `.claude/skills/gate/staged-gate.md`，带 `SINGLEFS_HEAVY_TESTS=commit`）。
- 门禁查重、工具层的闸、门禁 code-tooling 那几格：钩子只改了文件内的说明与自证，没改注册，同样归提交前那一趟。
- 复核钩子自证（light，主 agent 可以直接跑）：`bash .claude/hooks/heavy-test-guard.sh --selftest`，看末行「✓ 自检通过（查了 1402 种」；再跑 `HEAVY_TEST_GUARD_BREAK=merge-verify-quick-step-dropped bash .claude/hooks/heavy-test-guard.sh --selftest`，应当退 1，只红新加的那一格。

## 没做什么

- 没跑重型测试（`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）。没跑门禁阶段与各类 lint（命名检查也没跑：这次没改 `.rs`）。没走定义三方。没提交。
- 没编译，没起 cargo。开工时 `ps` 看到别的会话在跑 `cargo test -p singlefs-harness …` 和 `cargo test --release … -p singlefs-checker-tier …`，这一批不编译，没等它们。
- 没碰的：`.claude/main-agent.md`「一轮怎么开、怎么收」第 11 条、`.claude/agent-common.md` 第 85 行一带、`CLAUDE.md`、`.claude/hooks/` 下除 `heavy-test-guard.sh` 以外的文件（`runner-dispatch-guard.sh`、`handback-guard.sh`、`completion-cleanup-guard.sh`、`lib_three_way_legs.py` 都没动）、`scripts/`、`.claude/gate.d/`、别的定义、`records/`、`research/`。
- `heavy-test-guard.sh` 第 958 行没改（理由见「顺手看到、不归这一批的」第 1 条）。
- 草稿目录里没有仓副本、编译目录或库拷贝。hook-swap 的草稿树它自己跑完就删了。留下的是报告、规格、喂法日志、反向规格、三份 `.md` 改前的备份（`originals/`）和几份输出文本，整个目录 `du -sh` 是 512K。
