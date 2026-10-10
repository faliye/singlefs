# 第 21 批代码轮第一轮判决（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 F3 H1 H2 -->

被判的文件：`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs`、`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs`，`crates/mutations.tsv` 21-1 追加的 7 行与 21-2 追加的 4 行。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b21-r1-body.md`、`_m3-formal-code-b21-r1-diff.md`、`_m3-formal-code-b21-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-b21-r1-snapshot/`（crates 287 份，冻结副本 `/tmp/claude-1000/m3-formal-code-b21-r1/tree/`；kb 4 份） | 冻结副本对 crates 快照全过；kb 里 `.claude/kb/invariants.md` 在腿跑着时被别的会话改过，引文对现行文件核 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b21-r1-sonnet-output.md`（sha256 2aef7882…），模型 `…-sonnet-model/` | 引文 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b21-r1-opus-output.md`（sha256 79b50001…），模型 `…-opus-model/` | 主 agent 照它的 `tools/rerun.sh all` 在会话草稿 `b21-opus-rerun/` 复跑，见第四节 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本的 crates 加主仓 .claude、research、records 拼的根> --background <背景材料>`：「✓ 核了 87 处引文，对上 87 处，没判 0 处（逐处列在上面）」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1-a H2 用例 | 一致：正确实现绿，放宽 1、2、5 个 txg 都红在落点断言（左 50176、右 50180）；翻转点正确实现 k = S + 2、放宽 1 个 txg 是 k = S + 1，在 S = 5、6、7、8、12、16、默认环与 128 MiB 环上量过；脆处：常数 9、10 绑死根环槽数 S = 8（推的） | — | 站住。脆处进下一批：用例里的挂载次数从根环槽数算出，不写死 9、10 |
| K1-b 六张坏镜像、K1-c 判完再断言 | 一致：各只红 I-9.27，改动只落在点名那一条的头 ID 两字节与重封校验和；判完再断言没放宽 | — | 站住 |
| K1-d 注释对代码 | 一致，一处偏宽：「C670 那一条……复算为假、不报」只对用例那份盘面成立，「某块盘两槽都撕裂」整支代码写的是照常比 | — | 采：下一批改那一句措辞 |
| K2 F3 取样 | — | 没打中：2097120 个「段号 xor 指纹」两两不同，同一段全部子集摆出的二次崩溃并集 3 → 16–2359 个 | 站住 |
| K2 F3 内联用例判别力 | — | 打中（弱）：指纹只取第一个写、或只取第一个被扣下的写的下标时内联用例照绿，4095 个第一次崩溃态只剩 2、12 串随机数 | **坐实**（复跑 `f3.txt` 同）。改法 F3-c 进下一批：长 12 的段全部 4095 个子集的种子两两不同，配弱指纹变异 |
| K2 改法 a 判别力 | — | 打中：新 inode nlink = 0、出生代早一版、size 4096、写入时间 0 四种坏发布全放过（池级 checker 也绿；nlink = 0 那种 I-9.5 只管一个方向） | **坐实**（复跑 `k2a.txt` 同）。改法 A1 进下一批：比整条 inode 记录；nlink = 0 那种 checker 抓不到，另看 I-9.5 的方向要不要记账（下一批实现员报读码结论，主 agent 定） |
| K2 改法 b 判别力 | — | 打中：128 槽盘面上拒的是空间准入，那一支建 0 个照样记 UnitAreaWall；136 槽时正确判法 UnitAreaWall、只看 Some 是 Unexplained、建 0 个是 Published | **坐实**（复跑 `k2b.txt`、`k2b-sweep*.txt` 同）。改法 B1 进下一批：用例换 136 槽的盘面，配「建 0 个」变异 |
| K3 第四种接错 | 成立（推的，三前提）：崩溃枚举域里 23328 个三元组 0 反例，方向「更旧那一版只会更真」，接错只换支 | 打中：越屏障的组合 750 个里 9 个计数不同，正确接线 (1,1) 报红、接错 (1,0) 不报；不越屏障时两种接线都 0（推的） | **两腿不冲突，分在两个域**：屏障守住的域里两边都说计数分不开（都没在 checker 档量）；越屏障的靶向域里攻方量到计数分得开（复跑 `k3-sweep.txt`「COUNTS-DIFFER … correct=(1, 1, …) mutant=(1, 0, [])」坐实）。F2 靶向用例与接线变异用的正是越屏障的靶向摆法，所以主 agent 在 21-2 交回时判的「按支算还清」在这个域里不成立，撤回。改法 K3-a 进下一批：用攻方那个态补一条计数层的靶向用例，配「所选那一版交成第一次崩溃那一版」变异 |

## 三、去向

1. 21-1、21-2 的改动站住，不回改。
2. 下一批实现员：K3-a、F3-c、A1、B1、H2 用例挂载次数从根环槽数算出、K1-d 措辞；连同 E165 第 5 次重跑登记认定转来的 `crates/singlefs-core/src/root_record.rs` 第 26 行文档注释。
3. 提交时：21-2 与下一批的 checker 档用例、变异整表；F3 之后变异 2195、2283 的结局。

## 四、复跑核对

主 agent 在会话草稿 `b21-opus-rerun/` 照 `WORK=<草稿> THREADS=7 bash research/prompts/m3-formal-code-b21-r1-opus-model/tools/rerun.sh all` 复跑，退 0；`f3.txt`、`k2a.txt`、`k2b.txt`、`k2b-sweep.txt`、`k2b-sweep-classes.txt`、`k3.txt`、`k3-sweep.txt` 与模型目录 `out/` 逐个 cmp 相同。攻方的探针是另起的包（`opus-b21-probe`）链 checker 档的库跑注入函数，不起 checker 档的测试目标。

核不动：checker 档用例与变异没跑；正推的三条前提里第三条（二次崩溃那一版不旧于第一次崩溃那一版）只读码推；屏障守住的域里「计数分不开」两腿都没在 checker 档量。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | K3-a、F3-c、A1、B1 | 攻方 | 判别力缺口量过，改法本身没实现 | 下一批实现员；不用用户定 |

不涉及要用户定的岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 7 | 不影响 | 2026-10-04 不受影响：H2 用例只钉实现的回收谓词 |
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-04 不受影响：改法 a、b 的判别力缺口在用例，不在条款 |
