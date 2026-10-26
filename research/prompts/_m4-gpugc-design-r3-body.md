# m4-gpugc-design-r3：里程碑四七·2 GPU 垃圾回收设计轮，第三轮正文（2026-10-24）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 -->

## 一、题面

这是这一题的第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：第三轮之后停）。被判的是第一轮正文 `research/prompts/_m4-gpugc-design-r1-body.md` 的七格 G1–G7 与第二轮正文 `research/prompts/_m4-gpugc-design-r2-body.md` 新加的 G8。前两轮判决：`research/prompts/m4-gpugc-design-r1-main-verification.md`、`research/prompts/m4-gpugc-design-r2-main-verification.md`；岔路单 `research/prompts/m4-gpugc-design-r1-forks.md`。

第二轮判决：
- 打穿（两轮都中）：G1-①③、G2、G4 ①、G6；
- 挂着进这一轮：G3、G5、G7、G8，G4 ② 与 N，G6 的改法。

第二轮判决第三节第 1 条要在入库装置上量的已经量了：E185（GC一批搬迁的挂钟拆分） 第 4 次重跑，登记 `research/prompts/e185-r4-prereg.md`，问题单 `research/prompts/m4-gpugc-measure-r4-forks.md`（第 1 行够判、第 2 行复现不了、第 3 行撤掉不量）。

第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、这一轮报出的数字能不能核。第三轮只攻前两轮站住的形态，这一轮里新冒出来的零轮形态不再为它开一轮，写进判决的交用户表、标「零轮」。

用户的前提不变：「gpu垃圾回收需要加入。不需要讨论。」值不值按 D26（后台整理与放置回收） 已定项 13 第 7 条算省下的主机 CPU 时间。

## 二、实现今天的样子（主 agent 的观测，2026-10-24）

方案按 `crates/` 今天的实现来谈。代码事实同第二轮正文第二节与第二轮判决第一节（主 agent 现读过的行）；另加：
- 核心层的 CRC-32C 是软件 slicing-by-8：`crates/singlefs-core/src/checksum.rs` 第 3 行。
- 区域扫描逐条记录在全部映射条目上线性找：`crates/singlefs-core/src/region_scan.rs` 第 286–289、309–312 行。
- 整理批的准入：`crates/singlefs-core/src/transaction.rs` 第 9663–9670 行文档注释写搬迁副本「不进需求，所以两判都不判」，与 D26 已定项 3 第 2 句（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 100 行）对不上，已登记 `.claude/kb/checks-owed.md` C906（整理批不过准入与 D26 已定项 3 对不上） 交用户。
- 本机有三张显卡（两张 NVIDIA、一张 AMD 集显），ICD 有软件 Vulkan（`lvp_icd.json`）在内。

## 三、E185 第 4 次重跑的读数（主 agent 的观测；数以产物为准）

| 段 | 产物 | 判定那一行 |
|---|---|---|
| 控漂移段 | `research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out`（13808 行） | `E7RESULT name=verdict segment=host_cpu_drift_control constant_or_anchor_mismatches=0 stop_clause_findings=0 arms_measured=2 cards_started=2 card_failures=0 card_failure_items=none voided_forms_by_the_positive_controls_or_probes=0` |
| 写法段 | `research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out`（2362 行） | `E7RESULT name=verdict segment=cpu_writings constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none` |
| 重判 | `research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz`（108438 行） | `E7RESULT name=verdict segment=baseline_rejudge stop_clause_findings=0 rejudged_lines=96696 voided_writings=none drift_control_end=measured` |

控漂移段：复现臂卡 0 轮次比 `value=1.056747 judgement=drift_not_reproduced`；受控臂卡 0 `value=1.079304`、卡 1 `value=1.000256` 都 `drift_gone`；`controlled_unstable_card_zero_over_card_one … value=8.629032`；失败条款 L5（这一天复现不出）触发。

重判的主 agent 现算透视（`baseline_rejudge_summary` 行，取 `crt_source=r4_drift_control_controlled threshold_source=direct_from_the_r4_product threshold_kind=machine_scaled clock=thread`，数 `not_losing`、`not_losing_lower_bound`、`losing`、`unstable` 四栏）：
- 写法 `today`：候选 C 卡 0 120 格不赔 113、不稳定 7；卡 1 不赔 59、赔 47。A 卡 0 不赔 44；D 卡 0 不赔 50；卡 1 上 A、D 全赔。
- 写法 `both_three_way_without_build`（三路硬件 CRC、按身份查、不计建索引）：C 卡 0 不赔 64、赔 6、不稳定 50；卡 1 不赔 10、赔 103。A、D 两张卡全赔；B 卡 0 只剩下界不赔 48。
- 写法 `both_single_with_build`：C 卡 0 120 格全不赔，卡 1 不赔 89。
- 失败条款 L1（写法换了之后判定改成 losing）在几种写法下触发，例 `writing=both_three_way_without_build … crt_source=r4_drift_control_controlled changed_to_losing=78 triggered=true`。

第二轮攻方在它模型上推的「两处都换掉之后，间隔 50 ms 下 C 一格不赔都没有」：入库装置上的读数是卡 0 仍多半不赔（这一节第二条）。腿要引的数一律在产物里现取整行，不从这一节抄。

## 四、格

### G1　GPU 上算哪几段（前两轮打穿，攻「已定」那一半）
第二轮判决把 G1-① 收严成「A、B、D 两张卡都出局；C 的赢依赖 CPU 模式今天的两处慢路径」。第三节的读数下：这句话哪一半还成立、哪一半翻了；「C 只在卡 0 赢、在卡 1 赔」给「全部显卡」（D26 已定项 13 第 4 条）带来什么，写成岔路。

### G2　攒批形态（前两轮打穿）
攻打穿那一句本身：跨批攒成不了、卡 0 的拟合不可信。第 4 次重跑受控臂的拟合（`fixed_and_per_byte_fit` 行）能不能用，改不改判。

### G3　多卡分活（第二轮一轮打中）
第三节卡 0、卡 1 判定差得多；本机另有一张 AMD 集显没量。攻正推第一轮的推荐与第二轮的反例，给最终形态或写成岔路。

### G4　两份核比对与 N（① 打穿，② 与 N 挂着）
② CPU 只改位图与 `relocation_checked_against_the_version`（`transaction.rs` 第 10138 行）的关系；N 取几、计数住哪。第二段（第 3 次重跑）零次上界只罩两份 CRC 核。

### G5　控制接口（第二轮一轮打中）
攻第二轮攻方的反例（先请求切 cpu、这一批 GPU 随后出错、GC 停下）。

### G6　三个默认值（打穿；改法挂着）

### G7　GPU 代码住哪（第二轮一轮打中，推的）
攻 `wgpu` 版本钉法（checker 档 `=30.0.1`）与产品共不共用这条翻译链。

### G8　GC 挪不挪出调用线程（第二轮第一次判）
两边的代价：不挪出前台每次净增 5.50–14.76 ms（第二轮辩方，第一段读数）；挪出要补三样保证（发布预算 `.max(1)` 无上限、`last_completed_intent` 跨趟不留、整理批不过准入）。C906 是同一件事的条款侧。

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | G1–G8 | 复核第二轮判决：每一处「打穿」在第三节新读数下撑不撑得住；每一处「挂着」该判打中还是没打中；第二轮攻方的反例与推的数，哪几条被第四次重跑推翻或撑住。第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、报出的数字能不能核 |
| 云端攻方（Opus） | G1–G8 | 第三轮分工：攻前两轮站住的形态；未定项清单漏了什么；两轮判决与第三节报出的数逐个能不能从产物核出来（给命令与输出）。不许重复第一、二轮攻方已攻过的角度（`research/prompts/m4-gpugc-design-r1-opus-output.md`、`research/prompts/m4-gpugc-design-r2-opus-output.md`）。能用算术或小模型核的，在自己的草稿目录里核 |

## 六、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写 `crates/` 下的路径与行号（主工作区，开工快照在 `research/prompts/m4-gpugc-design-r3-snapshot/sha256sums.txt`）。
- 引产物整行抄，带文件名。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；要编译的经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，用自己的编译目录，跑完删。不用第二台。
- 报告写进 `research/prompts/m4-gpugc-design-r3-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 交岔路的地方写成表：候选（各自的定义）、翻面观测、够判条件；不写倾向。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
