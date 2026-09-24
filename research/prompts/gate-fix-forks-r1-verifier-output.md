# gate-fix-forks-r1 三方核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

快照核验（先跑，见下）：`sha256sum -c research/prompts/gate-fix-forks-r1-snapshot/sha256sums.txt`——19 个 OK（主树 = 快照，直接对主树核），1 个 FAILED（`47-research-script-selftests.sh`，符合派发说明：另一会话在第 29 行插了 1 行；`47-research-script-selftests.sh.at-snapshot` 的 sha256 与清单一致，本轮凡引 47 号行号一律对这份副本核）。云端两条腿交回时都没给报告 sha256，这一项不核（按派发说明「这一项没有」）。

## 判别力自证

取 Sonnet 报告里的一条引用：`.claude/singlefs-ai-sop/rules/sop-first.md:26`「## 每一条拒绝都必须给出下一步」。在草稿目录副本里把行号 +1（26→27）后核对：

```
$ awk 'NR==27{print "line27:["$0"]"}' /tmp/claude-1000/gate-fix-forks-r1-verifier/discriminability-selftest/sop-first.md
line27:[]
$ awk 'NR==26{print "line26:["$0"]"}' /tmp/claude-1000/gate-fix-forks-r1-verifier/discriminability-selftest/sop-first.md
line26:[## 每一条拒绝都必须给出下一步]
```

第 27 行为空行，与引文不符 ⇒ 判 **✗**。核查方法分辨得出坏输入，往下走。

## 核对方法

「文件:行号 + 抄的原文」一律用 `awk 'NR==行号{print}'`（区间用 `awk 'NR>=A && NR<=B'`）现取，与引用处原样比对。给定 20 文件快照清单内的文件，主树即快照（`sha256sum -c` 已核过 19 个 OK），直接对主树核，行号对不上记 **✗**；清单外的文件对主树核，行号对不上记「**分不清**：文件可能在腿交回之后被改过」，不记 ✗（工作区里别的会话在改）。

## 一、云端正推（Sonnet）：`gate-fix-forks-r1-sonnet-output.md`

这条腿没有产出模型/产物/复跑命令（自陈「没有跑门禁……没有额外执行 `.claude/gate.d/*.sh`」），核的只有它引用的文件:行号与抄的原文。下表按引用出现的位置分组，重复出现的同一条引用合并成一行、注明出现次数。

| 引用（Sonnet 报告里的写法） | 核的结果 | 实际位置 / 备注 |
|---|---|---|
| `sop-first.md:26`「## 每一条拒绝都必须给出下一步」 | ✓ | 快照外，主树核，行号内容都对（判别力自证用的就是这条） |
| `agent-common.md:41`「不做 git 写操作……不提交，不推送」 | ✓ | 快照外，主树核，对 |
| `command-safety.md:10`「## 退不回去的操作，动手前先想一遍」 | ✓ | 快照外，主树核，对 |
| `command-safety.md:1-9`（头部块） | ✓ | 快照外，主树核，逐行对 |
| `test-discipline.md:114`「## 只让多条臂互相比……」（T4、T5 各引一次） | ✓ ×2 | 快照外，主树核，对 |
| `show-me-test.md:119` | ✓ | 快照外，主树核，对 |
| `show-me-test.md:122`「# 一条改动不同步铺满全仓时……」（T5 引） | ✓ | 快照外，主树核，对 |
| `kb-discipline.md:98`「## 4. 矛盾比空白更糟」（T9 引） | ✓ | 快照外，主树核，对 |
| `kb-discipline.md:132-133` / 单独 `:132`「领域术语（RAID5、SHA256）……`<!-- doc-lint:not-numbers RAID5 SHA256 -->`」（T8 段落出现 4 处：报告第 112、116、118 行） | **分不清** | 该引文实际横跨主树第 **133-134** 行（132 行是上一句「每次引用都要带着它……」），偏了 1 行；`kb-discipline.md` 不在给定的 20 文件快照清单里，无法证明腿核对时行号就已偏，按派发说明记「分不清」不记 ✗ |
| `10-kb-rot.sh:107`「判据要不要换成『标题行变成已跑的那个提交』，交三方定」 | ✓ | 快照内（sha256 OK），对 |
| `10-kb-rot.sh:134`（howto 首行） | ✓ | 快照内，对 |
| `10-kb-rot.sh:174-192`（备料判定块） | ✓ | 快照内，逐行对 |
| `fs-design.md:107`「## 借用式条款要把被借规则的每个参数都绑死」 | ✓ | 快照内，对 |
| `fs-design.md:111`「⇒ 写借用式条款时……」（T2、T3 共引 3 次） | ✓ ×3 | 快照内，对 |
| `fs-design.md:107-111`（整节） | ✓ | 快照内，逐行对 |
| `decisions/08-核心索引结构.md:68`「改名覆盖已存在的目标 = 一次删除……」 | ✓ | 快照内，对 |
| `decisions/08…md:105`「前件写的是『删大目录、截断大文件会产生海量墓碑』」 | **✗** | 该句实际在主树第 **107** 行（第 105 行是「#### 已定项 5：……」标题行）；`decisions/08-核心索引结构.md` 在快照清单里、sha256 OK，主树即快照，非「分不清」，是确定的行号错误 |
| `implementation-workflow.md:16`「## 改 agent 定义与共用约束，走同一条三步」（T3 引 3 次） | ✓ ×3 | 快照内，对 |
| `implementation-workflow.md:6`「## 三步，缺一步就不算做完」 | ✓ | 快照内，对 |
| `e156_allocation_basis_counts.rs:1`（doc comment 首句） | ✓ | 快照外，主树核，逐字对 |
| `first_transaction_device_log_check.rs:1`（省略号引用） | ✓ | 快照外，主树核，起始句对；省略号之后未再逐句核对该文件其余部分 |
| `first_transaction_region_bytes.rs:1`（省略号引用） | ✓（带一处轻微省略） | 快照外，主树核；Sonnet 的转述把「E142**（第一个事务的干跑）** 量 5 的实装一侧」中间的括注省掉了，写成「E142 量 5 的实装一侧」——不影响判定但严格说不是整句照抄 |
| `31-blocking-verdict.sh:62-64`「⚠️ 生成器按脚本自身的位置取……而这条检查看起来一切正常」 | **✗** | 该三句实际在主树第 **61-63** 行（第 64 行已经是 `GEN="$(...)"` 代码行，不是注释）；文件在快照里、sha256 OK，确定的行号错误 |
| `52-segment-registry.sh:26`（`cd "$ROOT"`） | ✓ | 快照内，对 |
| `52-segment-registry.sh:27,29`（两处相对路径调用） | ✓ | 快照内，两行都对 |
| `check-segment-registry.py:41-43`（三个路径常量） | **分不清** | `LAYOUT_RELATIVE_PATH` 实际在主树第 **40** 行，不在所标 41-43 区间内（41-42 是另两个常量，43 是空行）；`research/scripts/check-segment-registry.py` 不在快照清单里（且该文件当时在别的会话手里，Opus 报告独立提到这一点），按派发说明记「分不清」 |
| `57-lkmm.sh:15` | ✓ | 快照内，对 |
| `70-citations.sh:9`「⚠️ 源码树不在也判红，不许当跳过——『跳过』正是让上一批文献无声蒸发的那个行为」（报告第 98、102 行两处指向同一处） | **✗** | 该句实际在主树第 **10** 行（第 9 行是空的 `#` 续行）；文件在快照里、sha256 OK，确定的行号错误；同一处错误在报告里重复出现 2 次。旁证：本地攻方腿译文核对表里对同一句引用的是 `70-citations.sh:10`（见下文本地腿一节），与本核查独立核出的行号一致 |
| `70-citations.sh:26`「echo "  ✗ 找不到 $S"」 | ✓ | 快照内，对 |
| `73-research-gate-lint.sh:19` | ✓ | 快照内，对 |
| `77-test-environment.sh:68-70` | ✓ | 快照内，逐行对 |
| `lib-format-const.py:18`「normalized_text——按空白归一后的原文，只当变更探测器用，不问它等于几」（引 3 次） | ✓ ×3 | 快照内，对 |
| `92-layout-checker-sync.sh:1-2`（gate-stage 说明行） | ✓ | 快照内，对 |
| `92-layout-checker-sync.sh:13-14`「④ 这次改动让某套布局的格式定义里『常量名 → 值』的集合发生增、删或改值……」（报告第 134、140 行两处） | **✗** | 该两句实际在主树第 **10-11** 行（13-14 行是另一段「为什么」注释，内容完全不同）；文件在快照里、sha256 OK，确定的行号错误，重复出现 2 次 |
| `27-format-constants.sh:4-8`「D23 已定项 9……三处都停在旧值……没有任何东西把实验源码里的格式常量绑到 kb 的现行值上」 | **分不清** | 前半句（D23……三处都停在旧值）确在 4-8 行；但「⇒ 没有任何东西把实验源码里的格式常量绑到 kb 的现行值上」这句实际在第 **9** 行，超出所标的 4-8 区间；`27-format-constants.sh` 不在快照清单里，记「分不清」 |
| `.claude/abbreviations`（无行号）「e<数字> # 实验编号，登记在 .claude/kb/experiments.md」 | 内容不逐字对（未列入 ✓/✗/分不清 计数，见下） | 主树该行实际是「e<数字>  # 实验编号：e57 指 .claude/kb/experiments.md 里登记的 E57。research/ 下的实验二进制按「编号_简称」命名，kb 的复跑命令按这个名字调用」——Sonnet 用反引号整行引用格式给出，但文字是改写过的（省略了「e57 指……E57」示例句与「research/ 下的实验二进制……」整句），不是逐字抄；因未给行号，不纳入下方按行核的计数，单列 |

**Sonnet 一节计数**（按不同的「文件:行号+引文」条目数，重复出现的同一条只算一处）：核了 27 处；✓ 24 处；✗ 4 处（`decisions/08:105`、`31-blocking-verdict.sh:62-64`、`70-citations.sh:9`、`92-layout-checker-sync.sh:13-14`，其中后两条各重复出现 2 次）；分不清 3 处（`kb-discipline.md:132`/`132-133`、`check-segment-registry.py:41-43`、`27-format-constants.sh:4-8`）；另有 1 处无行号的内容改写（`.claude/abbreviations`）单列、不计入上述三档。

命令样例（每条都是 `awk 'NR==行号{print NR": "$0}' <文件>` 或区间形式，逐条对每一处都真的跑过，这里只贴触发 ✗ / 分不清 的那几条作代表）：

```
$ awk 'NR>=105 && NR<=107{print NR": "$0}' .claude/kb/decisions/08-核心索引结构.md
105: #### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log
106:
107: **定案**：删大目录、截断大文件会产生海量墓碑……（下略）

$ awk 'NR>=61 && NR<=64{print NR": "$0}' .claude/gate.d/31-blocking-verdict.sh
61: # ⚠️ 生成器按**脚本自身的位置**取，不按 cwd——判别力样本会把 cwd 换成一个只放着
62: # 样本决策文件的临时目录，那里没有 `.claude/scripts/`。按 cwd 取会「找不到生成器 ⇒ 跳过」，
63: # 于是红样本安静地绿掉，而这条检查看起来一切正常。生成器自己 glob 的是 cwd 下的 kb，正合样本所需。
64: GEN="$(cd "$(dirname "$0")/../.." 2>/dev/null && pwd)/.claude/scripts/gen-decision-items.py"

$ awk 'NR>=9 && NR<=11{print NR": "$0}' .claude/gate.d/70-citations.sh
9: #
10: # ⚠️ **源码树不在也判红，不许当跳过**——「跳过」正是让上一批文献无声蒸发的那个行为。
11: # 红了不等于 kb 写错，处置见脚本自己打印的下一步。

$ awk 'NR>=10 && NR<=14{print NR": "$0}' .claude/gate.d/92-layout-checker-sync.sh
10: #   ④ 这次改动让某套布局的格式定义里「常量名 → 值」的集合发生增、删或改值，
11: #      而它的 checker 判定路径一个都没被这次改动碰过 ⇒ 判红。
12: #
13: # 为什么：一套布局的格式改了而它的 checker 判定路径没跟，门禁照样全绿……
14: # 去判新字节，而「checker 没报错」被读成「镜像是好的」。
```

## 二、云端攻方（Opus）：`gate-fix-forks-r1-opus-output.md` + 模型目录

### 产物 sha256

报告里列出的 14 个模型/输出文件的 sha256，逐个用 `sha256sum research/prompts/gate-fix-forks-r1-opus-model/*` 现算，与报告里贴的值**全部一致**（14/14 ✓）。

### 文件:行号引用

| 引用 | 核的结果 | 备注 |
|---|---|---|
| `.claude/kb/invariants.md:270`（I-9.5） | ✓ | 快照外，主树核，逐字对（含括注「记录在回收完成前留在 inode 树里……」） |
| `.claude/agent-common.md:4` | ✓ | 快照外，主树核，对 |
| `.claude/gate.d/72-agent-def-adversarial-review.sh:4` | ✓ | 快照外，主树核，对 |
| `.claude/singlefs-ai-sop/scripts/lib.sh:182` | ✓ | 快照外，主树核，对 |
| `.claude/singlefs-ai-sop/rules/sop-first.md:56` 与 `:58` | ✓ ✓ | 快照外，主树核，两句都对 |
| `.claude/singlefs-ai-sop/scripts/gate.sh:117`（+122-123） | ✓ | 快照外，主树核，对 |
| `.claude/singlefs-ai-sop/scripts/stage-selftest.sh:65` | ✓ | 快照外，主树核，对（省略号处是 `env -u GATE_BASE -u GATE_STAGED_FROM`，用省略号标出，不算摘句） |
| `decisions/08:68、107、119、121、184、148` | ✓ ×6 | 快照内，sha256 OK，逐处都对；184 行核到的是「mode / uid / gid / nlink，各 4」表格行，与「nlink 就在 inode 记录里」的判定精确吻合 |
| `decisions/14-双轨大小文件-持久临时.md:142`「linkat 在 POSIX 下是 O(1)」 | ✓ | 快照外，主树核，对 |
| `decisions/13-验证路线.md:107`「POSIX 可见语义：内容、大小、目录结构」 | ✓ | 快照外，主树核，对 |
| `show-me-test.md:57-58`、`:39`、`:62` | ✓ ×3 | 快照外，主树核，全对 |
| `10-kb-rot.sh:119、135、162、98、130` | ✓ ×5 | 快照内，逐处都对 |
| `75-decision-experiment-links.sh:8`、`:14` | ✓ ×2 | 不在快照，主树核，对 |
| `61-settled-same-file.sh:34-42、46、113` | ✓ | 不在快照，主树核，逐行对 |
| `40-results-cited.sh:86-89、81` | ✓ | 不在快照，主树核，对 |
| `88-quoted-result-lines.sh:28-31` | ✓ | 不在快照，主树核，对 |
| `92-layout-checker-sync.sh:155-157` | ✓ | 快照内，对 |

**内部不一致（单列，不算 ✗，因为两处引用各自查到的位置都真实存在于主树上，问题是同一份报告两处给了不同的行号）**：报告第 98 行说 75 号「判据在第 396–417 行」，第 342 行又说「按 75 号代码读出来的（第 8、14、389–417 行）」——同一件事（`check_changed_rows`/回看逻辑的判据范围）两处给的行号不同。现查 `.claude/gate.d/75-decision-experiment-links.sh`：判据逻辑的 `if in_git:` 起始于第 389 行，396 行只是循环内部的一个赋值语句，389-395 是必要的前置（读 `in_git`、建 `result_hits`），所以「389–417」是准确的范围，「396–417」偏窄、漏了前 7 行的建表逻辑。这不影响 T1/T5 的判定内容（判据本身逐字与背景吻合），但报告自相矛盾，供主 agent 核实推理时留意。

### 复跑（拷进草稿目录、`nice -n 19` 现跑，与留存 `.out`／报告表格比对）

| 命令 | 退出码 | 与留存 `.out` 比对 | 与报告表格/正文比对 |
|---|---|---|---|
| `python3 t1-history.py .`（真仓，只读） | 0 | 逐字节相同；`sha256sum` 与报告贴的 `2d30f169…` 一致 | 汇总行「对象 144 个；甲 红 28；乙 红 13、乙 找不到提交 1；终 红 13」与报告一致 |
| `bash .claude/gate.d/10-kb-rot.sh`（真仓，只读，非模型目录内脚本，额外验证） | 1 | — | 第 3 段末行「判了 143 个……其中 28 个红；没判 1 个……E157」与报告贴的原样输出逐字一致 |
| `bash t1-scenarios.sh . <草稿>`（造 5 段临时 git 历史） | 0 | 归一化提交号/临时路径后与留存 `.out` 逐字相同 | S1/S9/S10/S2/S2(撤回)/S3/S11/S4/S5 九格判定（绿/红/不判/没判）与报告第 74-82 行的表格逐格一致 |
| `bash t5-bases.sh . <草稿>`（bare+clone 造 3 种仓状态） | 0 | 归一化临时路径后与留存 `.out` 逐字相同 | 情形 A/B/C 各 4 种基准取法的结果与报告第 209-224 行代码块一致 |
| `bash t5-61-bypass.sh . <草稿>`（跑真 61 号原文件） | 0 | 逐字相同 | 「甲：退出码 1……」「乙：退出码 77……」与报告第 231-236 行一致 |
| `python3 t5-scan.py .` 与 `--no-git-word` | 0 / 0 | 两份都逐字相同 | 命中数（17 行/7 阶段，20 行/8 阶段）与报告一致；92 号确认漏判 |
| `bash t6-model.sh . <草稿>` | 0 | 归一化临时路径后与留存 `.out` 逐字相同（含节选部分） | 甲/乙两种样本形态在 M 变异下的 stage-selftest 结果与报告第 278-290 行一致 |

7 条复跑命令全部可重现（0 处不一致）。这也顺带验证了「变异 M（`sys.exit(exit_code)` → `sys.exit(0)`）」与 `research/results/e142-first-txn-dry-run-2026-09-22-mkfs-zero-fill-resync.out`（现查存在，123164 字节）这两样报告依赖的现场条件今天仍然成立。

### Fact 10-4 式的独立数字复核（附带验证，见本报告局部三之 T10）

Opus 本身不覆盖 T10；但 Sonnet 的 T10 判定引用了「主 agent 已现查」的 72/15 常量数字，本核查用 `lib-format-const.py` 的 `read_rust_consts` 现场重跑对 `crates/singlefs-format/src/lib.rs`，独立复现出同一组数字（见文末「三、本地攻方」一节的 T10 部分），一并算进本轮核查证据。

### 顺带看到禁读文件的披露

Opus 报告第 340 行「这条腿自己的限度」一节自陈：T3 部分对全仓 `grep -rn` 时未排除禁读文件，带出了 `research/prompts/gate-fix-forks-r1-sonnet-output.md` 第 39、41、43 行的片段；该腿自称之后的 grep 未再扫 `research/prompts`，且 T3 结论只引用了 implementation-workflow.md / show-me-test.md / sop-first.md / agent-common.md / 72 号原文，没有用那三行内容。本核查未独立验证「有没有用那三行」这一断言本身（这是语义判断，不是能核的事实），只照录该腿自己披露的位置：**报告第 340 行**。

**Opus 一节计数**：核了 28 处文件:行号引用；✓ 28 处；✗ 0 处；分不清 0 处；另单列 1 处内部行号自相矛盾（不计入三档）。复跑 7 条命令，全部可重现（含产物 sha256 14/14 一致）。

## 三、本地攻方：`gate-fix-forks-r1-local-attack-translation-audit.md` + 两份干净样本

这条腿本身不写「文件:行号」（提示明令禁止：「Do not write any source file name together with a line number」），核的对象是它自己的**译文核对表**——每一行「原文文件:行」都去主树现核，逐条判「行号在不在、抄的是不是原文、英文有没有丢限定词/多加限定词」。表分 T4/T7/T8/T9/T10 共 40 行，本核查逐条核完，结果如下（按小节汇总，行号见下方命令样例）。

### T4（6 行）

| 核对表行 | 核的结果 |
|---|---|
| Background rule quote（test-discipline.md:114-126） | ✓ 行号范围精确（114 标题、115-126 正文，127 空行分节）；核对表自陈首稿只译了 7 句里的 2 句、定稿补全，本核查逐句比对 local-attack.md 第 39-59 行的英文与主树中文，**7 句全部对应到位**，无遗漏、无多加 |
| Fact 4-1（80-absolute-assertions.sh:18-19,27） | ✓ 行号、内容都对 |
| Fact 4-3 B1-B4（4 个 .rs 文件各自 `:1`） | ✓ ×4 全对，doc comment 首句逐字比对无误 |

### T7（22 行：Fact 7-1、R1-R20、Fact 7-4）

**R1-R20 全部 20 行逐条核对，行号与译文全部对上**（用一条 python 脚本批量 `awk`+比对完成，见下方命令）。特别地：Fact 7-1 核对表给的行号是 `70-citations.sh:10`——这与本核查在 Sonnet 一节独立核出的正确行号（10，而非 Sonnet 报告误写的 9）一致，互证。R8（21-decision-items-sync.sh:39）译文「this stage has nothing to judge」核对表记「首稿错译成与『跳过』相同的『this stage is skipped』，定稿改对」——现查主树该行原文正是「本阶段无对象可判」（不是「本阶段跳过」），定稿的英文措辞与原文的区分对应正确。Fact 7-4（77-test-environment.sh 本轮未提交改动）：用 `git diff -- .claude/gate.d/77-test-environment.sh` 现核，改动内容（新增头部行 + 消息从「没有 $SCRIPT，本阶段跳过」+ exit 77 改成 R6 引文原文 + exit 1）与核对表描述逐字一致。

### T8（3 行）

- Fact 8-1（gen-decision-items.py:25-47）：✓ 逐行比对，两个正则的字符类都精确对应（第二个正则的三种逗号 `,`/`，`/`、`、中间点 `·`、三个连接字 `的`/`与`/`和`，与英文 "any one of three different comma characters ... a middle dot, and three specific Chinese connector characters" 一一对应；第一个正则的 `\s*$` 与英文 "optional trailing whitespace" 对应）。核对表自陈首稿在这两处漏了/说错了，定稿已修，现查定稿（local-attack.md 第 371-377 行）确认修法落实。
- Fact 8-2（decisions/05-快照-空间记账机制.md:28 源串 + decisions.md 现存剪坏产物）：✓。现查 `decisions/05…md:28` 该行原文「待删除结构算活的还是死的」去掉星号后恰好 12 个字符，与 Fact 8-2「twelve characters long」精确吻合；`decisions.md` 第 167 行现查确认仍是剪坏状态「20. 待删除结构算活的还是死 —— 已定」（缺尾字「的」），与 Fact 8-2 描述的「已提交、未重新生成」状态吻合。
- Fact 8-4（INDEX.md:29 标记行）：✓ 逐字符比对完全一致。但 Fact 8-4 正文里「this marker line appears in twenty one different files」这个数字，本核查用 `grep -rl "doc-lint:not-numbers" .claude/kb/ | wc -l` 现查得 **19**，不是 21；`.claude/kb/` 不在给定快照范围内，记「分不清：可能是别的会话这一轮之后增删了带这个标记的 kb 文件」，不记 ✗（这是局部数字，不是文件:行号意义上的引用，不计入下方 40 行的核对表计数，单列）。

### T9（4 行）

- Fact 9-1（lib-history-brief.py:91-97）：✓ 逐行对，`decision_names()` 的正则与 fact 描述一致。
- Fact 9-2（lib-history-brief.py:75-81）：✓。核对表自陈首稿把 `what_of(entry)`（有标题用标题、没标题退回用「改了什么」快查）简化成「its title」，丢了退回条件、来源数目从 3 个数错成 2 个；现查定稿（local-attack.md 第 486-489 行一带）已改成「its own title, or, when it has no title, its own quick note for what changed, plus its own quick notes for what came before and what came after」，与主树 `mentions()` 函数逐字对应（title-or-改了什么 + 改前 + 改后，三个来源）。另外 `MENTION` 正则 `(?<![\w-])D(\d+)（` 现查也与 Fact 9-2 描述的否定回顾断言、全角括号要求一致。
- Fact 9-3（lib-history-brief.py:143-146,178-186）：✓ 逐行对，`report_lost()` 的三句 howto 与 Fact 9-3 译文逐字对应。
- Fact 9-4（"no memory of any other day"）：✓（独立复核）：`grep -n "git log\|git show\|subprocess\|\.git" .claude/gate.d/lib-history-brief.py` 零命中，独立确认该生成器确实不读任何 git 历史。

### T10（5 行）

- Background（lib-format-const.py:17-18 docstring）：✓ 逐字对。
- Fact 10-1（27-format-constants.sh:87 调用行 + lib-format-const.py:119 默认参数）：✓ 两处都对；`only_scalar_types=True`、不带 `only_top_level_public`，与函数签名默认值 `value_reading="integer_literal"` 逐一核实相符。
- Fact 10-2（39-field-table-sum.sh:59）：✓，`parse_marks(body)` 而非 `read_rust_consts`，对。
- Fact 10-3（92-layout-checker-sync.sh:25-26,66-67）：✓ 两处引文逐字对。
- Fact 10-4（现查数据，非某一行的转述）：**✓ 独立复现**。本核查现场重跑：

```python
# 现场重跑（草稿目录内执行），对 crates/singlefs-format/src/lib.rs
# 用 92 号实际调用参数（only_top_level_public=True，不带 only_scalar_types）
normalized_text 读法：72 个声明，0 个 None
integer_literal 读法：72 个声明，15 个 None
15 个里 14 个标量 u64、1 个数组 [u32; 3]（ROOT_RING_REGION_DEVICES = [0, 1, 0]）
样例值文本：DATA_UNIT_PAYLOAD_OFFSET = "DATA_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES"
          JOURNAL_RING_DEFAULT_BYTES = "768 * 1024 * 1024"
```

与 Fact 10-4 全文（72/15/14+1/两个具体值文本样例）逐项精确吻合，这也间接印证了 Sonnet T10 判定里引用的「主 agent 已现查」的 72/15 数字今天仍然成立。

### 两份干净样本与运行记录

- 提示文件 `gate-fix-forks-r1-local-attack.md` 现查 sha256 `9256574228da4…`、736 行，与运行记录声称的一致，且与 s1、s2 所用提示文件相符（现场确认，无需假设）。
- 用草稿目录复制的 `corruption-check.py`／`oov-check.py`（需要连带 `research/data/en-words.txt` 才能跑，已按原相对路径布局复制）重跑 s1、s2：两份均判**绿**（cjk/fffd/复读/粘连/实词自复读/缩写自粘全 0），`oov-check.py` 判绿、生词清单 s1=`apparatus measurement's exemption vocabulary`（5 个，含重复计数）、s2=`exemption apparatus`（10 次出现、2 个不同词），与运行记录逐字段一致。
- void1/void2/s3 按派发说明不核（前两者是损坏闸判红作废的样本，s3 是 0 字节的中断调用）。运行记录里 s1/s2/s3 三份文件的 mtime（14:53:21 / 14:57:53 / 14:58:30）现查与其自述的 confirm/调用时刻（14:53:29 / 14:57:59 / 14:58:27）顺序一致、量级相符（mtime 略早于/略晚于自述时刻属正常写入延迟），未见时序矛盾。

**本地攻方一节计数**：核了 40 处译文核对表条目（T4 6 + T7 22 + T8 3 + T9 4 + T10 5）；✓ 40 处；✗ 0 处；分不清 0 处（另有 1 处不计入 40 行本身的局部数字——Fact 8-4「21 个文件」，现查为 19，记「分不清」，单列不计）。另独立复核 corruption-check/oov-check 2 项、Fact 9-4 与 Fact 10-4 两项现查数据，均与该腿自述一致。

## 四、总计

| 腿 | 核了几处 | ✓ | ✗ | 分不清 | 复跑命令 |
|---|---|---|---|---|---|
| Sonnet | 27（+1 处无行号改写单列） | 24 | 4 | 3 | 无（该腿没有产物需要复跑） |
| Opus | 28（+1 处内部矛盾单列） | 28 | 0 | 0 | 7/7 全部可重现（含 14/14 产物 sha256 一致） |
| 本地攻方（译文核对表） | 40（+1 处局部数字单列） | 40 | 0 | 0 | 2 项独立复核（corruption/oov）+ 2 项现查数据（Fact 9-4/10-4）全部一致 |

Sonnet 一腿的 4 处 ✗ 都发生在**快照清单内**、sha256 已核对为「主树=快照」的文件上（`decisions/08`、`31-blocking-verdict.sh`、`70-citations.sh`、`92-layout-checker-sync.sh`），都是**行号偏离实际引文位置 1-3 行**，但抄的文字内容本身都逐字准确——四处的共同形态是「引对了句子，标错了行号」，且其中两处（`70-citations.sh:9`、`92-layout-checker-sync.sh:13-14`）在报告里各自重复出现了 2 次，说明这不是偶发笔误而是同一处行号在多处引用时被系统性地记错。特别值得注意：本地攻方腿的译文核对表独立核出的 `70-citations.sh:10` 与本核查独立核出的正确行号一致，两条不同的腿在同一处条款上给出了互相印证的正确行号，只有 Sonnet 这一处错了 1 行——这是三方交叉验证本身抓出分歧点的一个实例，不是核查员单方面断言。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 不判 Sonnet/Opus/本地攻方三条腿在 T1-T10 上给出的判定（选定/交用户/选定但另立欠账）对不对、该不该被主 agent 采纳——那是主 agent 的事。
- Opus 报告里「T1 丙与 75 号冲突」「T1 丁的 75 号代价」两处，该腿自己标注「推的，按代码读，没跑 75 号」；本核查现查了 `75-decision-experiment-links.sh` 相关代码段的存在与大致逻辑（见二节「内部不一致」），但没有像 T1/T5/T6 那样造一份最小样本去实跑 75 号验证这条推论——按派发范围，复跑要求只到「每一条复跑命令」，该腿自己没有把这条列为「复跑命令」（只是读代码），所以不额外造样本去跑。
- 没有对 Sonnet 报告里每一处未给出具体行号的泛泛引用（例如「10-kb-rot.sh 第 98 行子串问题也在，不分辨臂」这类转述性总结句）逐句去核对措辞是否精确对应原文语气——只核「文件:行号 + 抄的原文」这一形态的引用，不核纯粹复述结论的句子。
- 没有重新核实 Opus 报告「没打中的形状」「我自己提的改法」两节里那些**推的（没有模型、没有跑）**的内容——这些本来就自陈是没有量过的猜测，不构成「文件:行号+抄的原文」或「复跑命令」，不在核查范围内。
- 本地攻方腿的 void1/void2（损坏闸判红作废）与 s3（0 字节中断调用）按派发说明不核，没有去读它们的内容或重跑损坏检测。
- 没有去判断 T1-T10 里各腿之间的判定是否一致、分歧点是什么——三方一致或分歧的判断属于「判决」，不属于核查员的观测职责。
- 云端两条腿交回时都没给报告本身的 sha256（派发说明已明确「这一项没有」），因此没有做「腿交回给的报告 sha256」与「报告文件现在的 sha256」的比对；本核查独立核过的只是模型目录内产物文件（Opus 14 个 `.py`/`.sh`/`.out`）的 sha256，这些都在 Opus 报告自己贴出的值范围内、且逐一核对一致。
- 没有编译 Rust、没有跑 `gate.sh` 全量、没有跑 54/55/57/59/87 号门禁（按定义禁止）；额外跑过的门禁只有只读的 10 号（真仓）与草稿目录里的临时仓上的 61 号，均属于验证 Opus 复跑命令的必要动作，不越出核查范围。
