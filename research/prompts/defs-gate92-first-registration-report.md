# 门禁 92 号（layout-checker-sync）：首次登记不算改动

## 关的是

`records/2026-09-16-subagent拆分提案.md` 第四十节相关记录里的判决（书记员报告
`research/prompts/m2-kb-writeback-batch2-scribe-report.md`「92 号：新登记的 5 个 format-const 触发 checker 同步要求」一节）：
第二批 kb 写回给 `.claude/kb/layout/01-first-txn.md` 里 5 个既有格式常量首次挂上 `format-const` 标记
（`DATA_UNIT_PAYLOAD_OFFSET`、`NONCE_MAC_ALGORITHM_RESERVED_BYTES`、`PACKED_UNIT_RECORDS_OFFSET`、
`JOURNAL_RING_DEFAULT_BYTES`、`ROOT_RING_CHUNK_BYTES`），值与 `crates/singlefs-format/src/lib.rs` 里
的 `pub const` 一直相同，92 号却判成「常量变了、checker 没跟」，报红。给的两条出路（改 checker /
登记进 `layouts-checker-lag.tsv`）都不对症：常量没变，checker 不欠它，登记滞后表会立一笔不存在的账。

## 改了什么

`.claude/gate.d/92-layout-checker-sync.sh`：④ 条新增一条豁免——一个 `(格式定义路径, 常量名)` 键这次
才出现时，若同一套布局的**另一条**格式定义路径这次没变、且原值的底层数值就等于这个新出现的值，判定
为「首次登记」，不计入这次的变化（不再要求改 checker 或登记滞后）。数值比较用 `same_underlying_value`：
先各自去掉数字分隔符 `_` 当整数比，两边都不是纯整数字面量才退回原文本比——`.rs` 的整数字面量允许
`805_306_368` 这种写法，`format-const` 标记的文法只认 `-?\d+`（不许下划线），字面不同、数值相同，
不按数值比就会把这种情形误判成「变了」（真仓 5 个常量里 `JOURNAL_RING_DEFAULT_BYTES`、
`ROOT_RING_CHUNK_BYTES` 正是这种写法，只判数值等价而不管字面能让这两个正确豁免）。

「值真变了照旧红」不受影响：该豁免只在**另一条路径的值本身也没变**（`baseline == current`，逐字比，
不放宽）时才生效；如果格式定义的源值在这次改动里也变了，新登记的标记不会被任何路径的「没变」证明是
「一直如此」，照旧判红——见下方 `first-registration-changed-value` 样本。

同时改了头部注释：④ 条描述、「为什么」段落、判别力段落，补上这条豁免的说明与新增样本的用途；
`admission:` / `run-condition:` 两行与 `source .../preflight.sh`、`preflight ...` 那一行一字未动。

## 判别力样本（新增，`.claude/gate.d/fixtures/92-layout-checker-sync.sh/` 下）

三个新目录，`red`/`green` 两个既有样本一字未动：

| 目录 | 判 | 场景 |
|---|---|---|
| `first-registration-same-value` | 绿 | 常量值一直是 16384，`.rs` 这次没变，`.md` 第一次挂标记记的也是 16384 |
| `first-registration-changed-value` | 红 | 同一次改动里 `.rs` 的值也从 16384 变成 32768，新标记记的是新值 32768 |
| `first-registration-underscored-literal` | 绿 | `.rs` 字面量带下划线分隔（`805_306_368`），标记只能写 `805306368`——复现真仓 `JOURNAL_RING_DEFAULT_BYTES`/`ROOT_RING_CHUNK_BYTES` 撞上的那种字面不同、数值相同 |

## 判红 → 判对（改前改后的原样输出）

改之前（用上一次提交的版本 `36770683` 跑同样两个新样本，验证「该红的输入」确实在改前判错）：

```
=== BEFORE FIX: first-registration-same-value ===
exit=1
  ✗ 1 个格式常量在这次改动里变了，而它所属布局的 checker 判定路径一个都没被碰（基准 HEAD）：
      样本布局线（checker 判定路径：checker/src/lib.rs）
        新增 SAMPLE_UNIT_BYTES = 16384（format/布局.md）
     → 怎么办：两条出路。① 在同一次改动里改这套布局的 checker 判定路径，让它按新格式判——
               格式走了一步而 checker 停在旧口径时，它会拿旧宽度去解新字节，而且全绿；
               ② checker 今天确实判不了它，就把常量名登记进 .claude/gate.d/layouts-checker-lag.tsv：
               三列写常量名、一条开着的欠账编号、为什么今天不判。账在册才有人排期。

=== BEFORE FIX: first-registration-changed-value ===
exit=1
  ✗ 2 个格式常量在这次改动里变了，而它所属布局的 checker 判定路径一个都没被碰（基准 HEAD）：
      样本布局线（checker 判定路径：checker/src/lib.rs）
        改值 SAMPLE_UNIT_BYTES：16384 → 32768（format/src/lib.rs）
        新增 SAMPLE_UNIT_BYTES = 32768（format/布局.md）
```

改之后（同两个样本 + 新加的下划线字面量样本）：

```
=== first-registration-same-value ===
exit=0
  ✓ 布局清单 1 套布局、3 条路径都在……这次改动比 HEAD，2 个格式常量里变了 0 个……
    另有 1 个是第一次登记标记，源里的值这次没变，不算这次的变动
      首次登记：SAMPLE_UNIT_BYTES = 16384（format/布局.md，同一套布局里别的格式定义路径这次没变、原值就是这个）

=== first-registration-changed-value ===
exit=1
  ✗ 2 个格式常量在这次改动里变了，而它所属布局的 checker 判定路径一个都没被碰（基准 HEAD）：
        改值 SAMPLE_UNIT_BYTES：16384 → 32768（format/src/lib.rs）
        新增 SAMPLE_UNIT_BYTES = 32768（format/布局.md）
（保持红，未受豁免影响）

=== first-registration-underscored-literal ===
exit=0
  ✓ ……另有 1 个是第一次登记标记，源里的值这次没变，不算这次的变动
      首次登记：SAMPLE_RING_BYTES = 805306368（format/布局.md，同一套布局里别的格式定义路径这次没变、原值就是这个）
```

真仓上从红转绿（`bash .claude/gate.d/92-layout-checker-sync.sh` 原样末行，基准 `faf255e235300d129ede6d6f85af31d686a88519`）：

```
  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 faf255e235300d129ede6d6f85af31d686a88519，111 个格式常量里变了 0 个（checker 在同一次改动里跟了 0 个，按滞后表放行 0 个），都不欠 checker 跟进；另有 5 个是第一次登记标记，源里的值这次没变，不算这次的变动
    这 5 个是首次登记，不是这次改动引入的变化：
      首次登记：DATA_UNIT_PAYLOAD_OFFSET = 134（.claude/kb/layout/01-first-txn.md，同一套布局里别的格式定义路径这次没变、原值就是这个）
      首次登记：JOURNAL_RING_DEFAULT_BYTES = 805306368（.claude/kb/layout/01-first-txn.md，同一套布局里别的格式定义路径这次没变、原值就是这个）
      首次登记：NONCE_MAC_ALGORITHM_RESERVED_BYTES = 29（.claude/kb/layout/01-first-txn.md，同一套布局里别的格式定义路径这次没变、原值就是这个）
      首次登记：PACKED_UNIT_RECORDS_OFFSET = 136（.claude/kb/layout/01-first-txn.md，同一套布局里别的格式定义路径这次没变、原值就是这个）
      首次登记：ROOT_RING_CHUNK_BYTES = 1048576（.claude/kb/layout/01-first-txn.md，同一套布局里别的格式定义路径这次没变、原值就是这个）
    没抽到常量的格式定义路径 1 条（第 ④ 条对它们没有对象可判，只受第 ②③ 条管）：
      第一条纯 SSD 布局线（……）：.claude/kb/layout/02-second-txn.md
exit=0
```

## 改过的文件与 git diff --stat

```
$ git diff --stat -- .claude/gate.d/92-layout-checker-sync.sh
 .claude/gate.d/92-layout-checker-sync.sh | 57 ++++++++++++++++++++++++++++++--
 1 file changed, 54 insertions(+), 3 deletions(-)
```

新增（未跟踪，`git status --short`）：

```
?? .claude/gate.d/fixtures/92-layout-checker-sync.sh/first-registration-changed-value/
?? .claude/gate.d/fixtures/92-layout-checker-sync.sh/first-registration-same-value/
?? .claude/gate.d/fixtures/92-layout-checker-sync.sh/first-registration-underscored-literal/
```

各目录 4 个文件：`setup.sh`、`expect`、`.claude/gate.d/layouts.tsv`、
`.claude/kb/decisions/15-格式冻结政策.md`（合计 91 行，见上方 `wc -l`）。

`gate-similar` / `hook-events`：这次没建新的门禁阶段或钩子文件，只改已有阶段 92 号自己的判定
逻辑与它自己的判别力样本目录，不适用 `gate-similar` / `hook-events`（那两行只管新建的门禁/钩子）。

## 第 7 步各项末行与退出码

- `bash .claude/gate.d/92-layout-checker-sync.sh`（真仓）：见上方，exit=0
- `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d`（全量 76 个阶段）：
  `✗ 2 个样本判错（判对 151 个，1 个阶段没有样本）`，exit=1；红的两条都是 `55-qemu-first-transaction.sh`
  （`git status` 显示这个文件是别的会话未提交的改动，我没碰过），`54-layer0-replay.sh` 报「本次未跑」
  （重型测试，按定义不跑）——92 号自己的 5 个样本（`red`、`green`、三个新目录）全部判对。
- `GATE_LINT_DIR=.claude/gate.d bash .claude/singlefs-ai-sop/scripts/gate-lint.sh`：
  `✓ 门禁自检通过：85 个脚本（.sh 与 .py）、310 条拒绝都带了出路`，exit=0
- `SHELL_LINT_DIR=.claude/gate.d bash .claude/singlefs-ai-sop/scripts/shell-lint.sh`：
  `✓ shell 纪律检查通过（共 76 个脚本）`，exit=0
- `python3 .claude/singlefs-ai-sop/scripts/preflight-lint.py`：
  `✓ 准入与运行条件：判了 138 个脚本……`，exit=0
- `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py`：
  `✓ 相对 262c02d3e47a：新加的门禁与钩子 0 个（无）都写明了比过谁，改过的 3 份脚本对照已有的 134 份没有整段相同`，exit=0
- `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .`：
  `✓ 文档铁律检查通过（检查 521，跳过 0；DOC_LINT_VERBOSE=1 看全部）`，exit=0
- `RULES_LINT_DIR=.claude/rules RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md" bash .claude/singlefs-ai-sop/scripts/rules-lint.sh .`：
  `✓ 规则只写怎么做……词法说明判了 1842 行……命中 0`，exit=0（这一轮没改任何规则/agent 定义文件，跑它只是走全套检查列表）
- `bash .claude/gate.d/47-research-script-selftests.sh`：红，
  `✗ bash research/scripts/ask-local-selftest.sh 没过` 与
  `✗ python3 research/scripts/check-segment-registry.py --selftest 没过`；
  两处点名的文件（`research/scripts/ask-local-selftest.sh`、`research/scripts/ask-local.sh`、
  `research/scripts/check-segment-registry.py`、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`）
  `git status --short` 均无改动，不在这一轮里；唯一相关的 `.claude/kb/layout/01-first-txn.md` 是
  `M`（早先 kb-scribe 批次 2 会话改的，也不是我这一轮改的）。按「红了先看点名的文件在不在这一轮的
  改动里；不在的不修」照写，未处理。
- `bash .claude/gate.d/62-stage-owners.sh`：`✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）`，exit=0
- `bash .claude/gate.d/63-agent-write-scope.sh`：`✓ 写范围闸、Bash 检出 hook……都注册着、自证通过`，exit=0
- `bash .claude/gate.d/73-research-gate-lint.sh`：
  `✓ 研究脚本、hook 与 .claude/scripts 的门禁纪律：扫了 3 个目录……拒绝都带出路、shell 纪律守住`，exit=0
- 阶段归属表登记给 `tooling-writer` 的阶段：
  `awk … -v me="tooling-writer" … stage-owners.tsv` 输出为空——没有登记给我的阶段，无需另跑。
- 54、55、57、59、87 号：未真跑。54 号只在 `stage-selftest.sh` 里报「本次未跑（本次无对象可判）」；
  55、57、59、87 号这一轮没有单独调用（92 号与这几道无关，未涉及）。

## 没做什么

- 没跑重型测试本身（54/55/57/59/87 号、`gate.sh` 整轮、全量 `cargo test`、`check.sh`）。
- 没修 47 号红的两处：`research/scripts/ask-local-selftest.sh`、`research/scripts/check-segment-registry.py`
  点名的问题都不在这一轮改动的文件里（见上）,按规矩不修、照实写。
- 没走定义三方（这一轮不是改 agent 定义/共用约束/规则，是门禁阶段本体，不适用该流程）。
- 没提交、没推送，没有任何 `git add`/`commit` 落地（过程中误跑过一次 `git add -N` 把三个新样本目录
  标成 intent-to-add，随即用 `git restore --staged` 撤回，现状是 `??` 未跟踪，已核实）。
- 没有动 `red`/`green` 两个既有样本目录的任何文件；`checks-owed.md`、`decisions-history.md` 等 kb 文件
  一个字没改（这次修的只是 92 号自己的判定逻辑与它的判别力样本）。
