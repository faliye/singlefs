# m2-layer0-scale-r1 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

抽样引用：`.claude/kb/invariants.md:54`（I-7.4 行），抄的原文「回退候选集里每一个根……所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头」。
对第 54 行现查：命中（`grep -o` 取出该句原样）。
把行号从 54 改成 55（+1，草稿副本 `/tmp/claude-1000/m2-layer0-scale-r1-verifier/invariants-selftest.md`），再核同一句原文：

```
$ awk 'NR==55' invariants-selftest.md
| I-7.5 | 根槽按判定宽度对齐 | 每个根槽的**起始偏移**是挂载时探测的 `physical_block_size` 的整数倍。……
$ awk 'NR==55' invariants-selftest.md | grep -c '回退候选集里每一个根'
0
```

第 55 行不含被引原文，判 ✗。核查方法能分辨行号错位，往下做。

## 输入核对

- 三条腿报告齐全：`m2-layer0-scale-r1-sonnet-output.md`、`m2-layer0-scale-r1-opus-output.md`、本地攻方 `-output-s1.md`〜`-s4.md`（`-s2.md` 带损坏，`-output-void1.md` 作废，均已读，见运行记录）。
- 报告 sha256 与派发提示给的比对：
  - sonnet-output.md：给的 `1e90db887d7e9856a56897329e3b568af55e017be18d3a4fbc9eafa535c8a49c`，现算相同。
  - opus-output.md：给的 `d3c9105952e0e74a94c4af537294400a9e031ec18e9119fbe2bcc60d19e396bb`，现算相同。
  两份均与交回时给的一致，不落入「整份记后一种」的分支。
- 冻结副本完整性：`sha256sum -c /tmp/claude-1000/l0scale-r1-frozen-sha256.txt`，133/133 行 `OK`，无一处不符。
- Opus 模型目录 `SHA256SUMS`：`sha256sum -c SHA256SUMS`，20/20 行 `OK`；报告正文里贴的哈希列表（第 15–34 行）与 `SHA256SUMS` 文件逐字节 `diff` 相同。
- `.claude/kb/` 本轮无快照，按设计轮规则对主树核；发现 `.claude/kb/decisions/16-发布语义.md` 当前有未提交改动（`git status --porcelain` 显示 `M`），但该文件 mtime（2026-09-26）早于 Sonnet 交回窗口（从背景材料落盘到报告落盘），且下面核到的具体引用在 HEAD 版与工作区版里位置相同——即 Sonnet 写报告时读到的就是今天这一份，不是「交回之后被改过」，下面的不符记 ✗ 不记「分不清」。

## 一、Sonnet 腿（正推，L1/L2/L4/L5/L7）

| 引用 | 核的结果 | 命令/证据 |
|---|---|---|
| `segments.rs:20-29` StepKind 六种 | ✓ | `sed -n '20,29p'`，六个变体逐字匹配 |
| `segments.rs:58-87` classify | ✓ | `sed -n '58,87p'`，四类判据顺序一致 |
| `segments.rs:81-83` UnitWrite 兜底 | ✓ | 精确匹配 else 分支 |
| `.claude/kb/invariants.md:54` I-7.4，两处引文（回收谓词定义、C556/H6 两句） | ✓ | 行内容与两处引文均逐字命中 |
| `segments.rs:94-127` split_into_segments | ✓ | 屏障关段、FUA 关段、尾部并段逻辑与描述一致 |
| `segments.rs:104-109` 屏障关段代码 | ✓ | 精确匹配 |
| `crash.rs:786-826` reuse_is_not_proven_illegal_by_the_reclaim_predicate | ✓ | 函数体与三个具名变量（root_in_each_ring_slot 等）逐字匹配；区间内 `singlefs_core::` 零命中，与「不共用被测代码」的论证一致 |
| `crash.rs:1851-1866`/`1858-1865` enumerate_layer0_versions | ✓ | 闭包 `&\|_segment_index, _segment\| true` 精确落在 1863 行，属所引区间 |
| `crash.rs:1870-1885`/`1877-1884` enumerate_layer0 | ✓ | 同上，闭包在 1881 行 |
| `crash.rs:1489,1497` expand 钩子 | ✓ | 两行精确匹配所引代码 |
| `second_transaction_step_zero_layer0.rs:457` 快档 expand | ✓ | 精确匹配 `segment.len() < 10` |
| `:4,453,581` 三处"18 写段"计数矛盾（2/3/8个） | ✓ | 三行原样核实，确为 2、3、8 |
| `:478` `tally.states, 108` | ✓ | 精确匹配 |
| `:581,588` 闭式 2_104_413 | ✓ | 精确匹配；反算 279 次写、54 段的算术（8×18+7×10+2×4+20×2+17×1=279，8+7+2+20+17=54）现算无误 |
| `reduc\|约简` 在 segments.rs/crash.rs 零命中、src/ 全目录 8 命中全在 e158_root_choice_repair.rs | ✓ | 原样重跑同一条 grep，结果相同 |
| `Cargo.toml:5-9` 4 个 crate | ✓ | 精确匹配 |
| `singlefs-harness/Cargo.toml:8-11` 三个依赖 | ✓ | 精确匹配 |
| `stage-inputs.tsv:20` | ✓ | 精确匹配（主树，此文件无快照，按设计轮规则核） |
| `54-layer0-replay.sh:148` cargo test 命令 | ✓ | 精确匹配（主树） |
| 9 份 layer0 用例 `mod common;`/`mod common_tree_split;`（含 `:25-26`、`:21-22` 两处双 mod） | ✓ | 原样重跑同一组 grep，逐文件逐行核对，与报告表述完全一致 |
| `crash.rs:522-529`、`segments.rs:146-158` 两份同名 closed_form_state_count | ✓ | 两处公式与代码一致 |
| `crash.rs:955-1015` absorb_following_slice | ✓ | 函数存在，签名与位置一致 |
| `crash.rs:1375,1377` 两个常量（64、16） | ✓ | 精确匹配 |
| `crash.rs:1556-1578`、`:1561-1565` state_slices 与 slice_count 公式 | ✓ | 精确匹配 |
| `54-layer0-replay.sh:337` trap | ✓ | 精确匹配（主树） |
| `:524-579`/`:575-578` one_state_slices 测试 | ✓ | 函数名与 assert_eq 精确匹配 |
| `crash.rs:1617-1625` evaluate_state_slice 签名 | ✓ | 精确匹配（base/writes/plan 只读，slice: Range<u64>） |
| `crash.rs:891` derive(PartialEq) | ✓ | 精确匹配 |
| `59-crates-mutation-replay.sh:32-33,97` GATE_CROSS_RUN_TMPDIR | ✓ | 精确匹配（主树） |
| `54-layer0-replay.sh:29,332` 全绿标记按输入哈希命名 | ✓ | 精确匹配（主树） |
| `crates/mutations.tsv` 第 54、23 行内容 | ✓ | 主树与冻结副本两份文件逐字节相同，内容与报告转述一致 |
| `.claude/kb/decisions/16-发布语义.md:27`「已定项 1 的回收谓词」引文 | **✗** | 第 27 行是「#### 已定项 1」标题行；抄的原文「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」实际在第 33 行（HEAD 版与工作区版均如此，非交回后被改） |
| `.claude/kb/decisions/16-发布语义.md:174`「已定项 7」持久顺序引文 | **✗** | 第 174 行是标题行；抄的原文「COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽」实际在第 176 行 |
| `.claude/kb/decisions/13-验证路线.md:69`「已定项 4」崩溃状态定义引文 | **✗** | 第 69 行是标题行；抄的原文「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」实际在第 71 行 |
| `.claude/kb/decisions/23-journal的角色与格式.md:351`「候选集」引文 | **✗** | 第 351 行是「已定项 14」标题行；抄的原文（候选集表格行）实际在第 378 行 |
| 同一行号 351，「边角三样」①条引文 | **✗** | 抄的原文（R_old 同槽同分配代……F 回落）实际在第 400 行 |
| `.claude/rules/fs-design.md:17`「运行时与 checker 必须用不同的算法」引文 | **✗** | 第 17 行是「## 记账是事务的副产品」标题行；抄的原文实际在第 36 行 |
| `.claude/rules/implementation-workflow.md:30`「判据是那一道读的全部输入自上次跑绿以来变没变」引文 | **✗** | 第 30 行是标题行；抄的原文实际在第 32 行 |

Sonnet 计：核 42 处，✓ 35，✗ 7。**七处 ✗ 是同一种模式**：所引行号是该「已定项 N」/该节标题所在行，而实际抄的那句话在其后若干行（2〜49 行不等）；内容本身逐字核对无误，只是行号没有指向被抄句子自己的位置。`invariants.md:54` 之所以精确，是因为该文件每条不变量整条压在一个表格行里，没有"标题行"与"内容行"的结构性分离；决策类文件（decisions/、rules/）每个「已定项」是标题+多段内容，Sonnet 的行号看起来是取自 grep 标题得到的行号，未对被抄句子本身重新 `grep -n` 核实。


## 二、Opus 腿（云端攻方，L1/L3/L6/L7）

| 引用/产物/复跑 | 核的结果 | 命令/证据 |
|---|---|---|
| `walk.rs:1709` 注释原文 | ✓ | 逐字节相同 |
| `walk.rs:1741`、`:1643`（scanned_tree_identifiers）、`:3614`（scanned_content_units） | ✓ | 三处函数/代码存在且与描述吻合 |
| `crash.rs:714` copy_is_missing 闭包 | ✓ | 精确匹配 |
| `mount.rs:831` shadow_ledger match | ✓ | 精确匹配，是变异补丁的实际锚点 |
| `mount.rs:1327` 抬 F 扣住刚回收槽的注释 | ✓ | 内容与转述一致 |
| `recovery.rs:668,680` choose_root/rollback_witness.abandons | ✓ | 精确匹配 |
| `54-layer0-replay.sh:341,374` 两条流的 run_layer0_test_binary 调用 | ✓ | 精确匹配（主树） |
| `:147` include-ignored 判据 | ✓ | 精确匹配 |
| `:156-160` worker_threads_of_full_run 全函数 | ✓ | 五行函数体逐字节相同 |
| `crash.rs:1740` spawned_worker_threads | ✓ | 精确匹配 |
| `first_transaction_step_five_publish.rs:375`、`first_transaction_step_seven_layer0.rs:386,354,466` | ✓ | 四处精确匹配 |
| `second_transaction_step_three_acquisition_barrier_layer0.rs:186-189` | ✓ | 四行逐字匹配，含 Opus 自己标注的"第187行中间行、188行提示串" |
| `second_transaction_supplement_two_rollback_witness_layer0.rs:17` | ✓ | 精确匹配 |
| `crates/mutations.tsv` 第五列含 layer0 的行数 = 32 | ✓ | 原样重跑 `awk` 命令，结果相同 |
| static/thread_local 三处（checksum.rs:11、lib.rs:60、block_device.rs:559,562-567） | ✓ | 原样重跑同一条 grep，三处行号与代码内容均相同 |
| `.claude/kb/checks-owed.md:477` C554 | ✓ | 精确匹配 |
| `.claude/kb/decisions/23-journal的角色与格式.md:384` | ✓ | 内容与转述一致（此为转述非逐字引用，语义相符） |
| `.claude/rules/implementation-workflow.md:73`「合并要确定」 | ✓ | 精确匹配，是该条款自己的内容行（非标题行），与 Sonnet 那七处形成对照 |
| 9 份 layer0 用例的 `#[test]`/`#[ignore]` 计数表 | ✓ | 原样重跑 `grep -c`，9 行数字（6/1、9/1、2/1、1/0、2/1、1/0、3/0、6/3、3/1）逐行相同 |
| `grep -rn include-ignored .claude/gate.d/*.sh .claude/agents/*.md` 只命中 54 号自己 | ✓ | 原样重跑，只在 54-layer0-replay.sh 命中 5 处，别处零命中 |
| `.claude/agents/crash-verifier.md` 第 26 行起第 2 步 | ✓ | 第 26 行确为步骤"2."，内容与转述一致 |
| 模型目录 `SHA256SUMS` 20 个文件 | ✓ | `sha256sum -c` 全部 OK |
| **复跑：L3 第 304 行三个数**（`PROBE_WHICH=C` on m304 补丁副本） | ✓ 精确复现 | `ARMS C arm_a_states=78 arm_a_bad=0 arm_a2_states=108 arm_a2_bad=12 full_sampled_cow_states=2596 full_sampled_bad_beyond_arm_a=828`；`ARMSEG`/`ARM_A2_BAD` 摘要行与报告「六、原样输出」逐字节相同（`FULL_DIFF` 随机采样的具体样例行不同，属预期内的随机子集展示，不影响判据数） |
| **复跑：L1 H1 那一格** | ✓ 精确复现 | `ARMS H1 arm_a_states=78 arm_a_bad=7 arm_a2_states=93 arm_a2_bad=10 full_sampled_cow_states=1561 full_sampled_bad_beyond_arm_a=124`；`OVERWRITE H1 write#137 dev0 slot50182 txgSome(8) overwrites: stream=#41(unit_write,txgSome(4))` 与报告逐字节相同；`mount_under_fault chosen=(1,3) effective=(1,3) instance=2 …row_publish_txg=5 warmups=[6, 7]` 与报告转述的元组一致 |
| **复跑：L7 严格读法那一格**（p8，续跑截断/行边界） | ✓ 精确复现 | `L7 byte_prefixes order=forward prefixes=53693 strict_wrong=0 …lenient_accepted_a_half_line=53530 …fewer_violations=6433`；`order=reverse …53563…6433`；`L7 line_boundaries order=forward boundaries=44 …全 0`；`order=reverse …file_order_first_violation_wrong=14`——与报告表格「53693/53530/53563/6433」「44 个行边界」「错 14/44」全部吻合 |

复跑细节：均在草稿目录 `/tmp/claude-1000/m2-layer0-scale-r1-verifier/opus-rerun/` 下用冻结副本的新拷贝跑（未用 Opus 自己留下的目录），命令按模型目录 `rerun.sh` 里的 `make_copy`/`run` 展开，`run-with-memory-cap.sh 12G` + `capped.sh 10`（线程上限 10）。唯一发现的差异：我这次跑出的 `p6-C-m304` 与模型目录 `outputs/p6-C-m304.log` 相比，`filtered out` 测试数不同（我 7、原日志 5），且随机采样展示的 `FULL_DIFF` 具体样例行（哪个 imask/盘/槽）不同——后者是 `PROBE_RANDOM=32` 随机抽样、无固定种子的预期波动，不影响任何一个判据数字；前者提示 `outputs/p6-C-m304.log` 捕获时探针文件可能处于比现存 `SHA256SUMS` 里那份更早的迭代版本（Opus 报告第 11 行已自述"日志名与模型目录 outputs/ 里的略有不同"，但没有点名 p6 这一处），记一笔观察，不影响本次要核的三个判据数字（均精确复现）。

Opus 计：核 30 处（含 3 项复跑），✓ 30，✗ 0。


## 三、本地攻方腿（L5，逐格算数）与转述核对表

### 转述核对表（`m2-layer0-scale-r1-local-attack-translation-audit.md`）逐条核

| 原文文件:行 | 核的结果 |
|---|---|
| `.claude/kb/decisions/16-发布语义.md:176` 持久顺序 | ✓ 精确匹配（对比：Sonnet 报告引同一句话时写成 `:174`，是标题行，见上表 ✗；本地材料这里写 `:176`，是内容行本身，精确） |
| `:180` 根槽不镜像 | ✓ 精确匹配 |
| `.claude/kb/decisions/13-验证路线.md:71` 镜像双写按设备算 | ✓ 精确匹配 |
| `.claude/kb/decisions/23-journal的角色与格式.md:449` 取号屏障 | ✓ 精确匹配（该行本身就是"- **取号那一步的屏障**：……"那一条项目符号行，不是标题行） |
| `.claude/kb/decisions/16-发布语义.md:215` 空发布也是发布 | ✓ 精确匹配 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:70` 第一版 2 块盘 | ✓ 精确匹配 |
| `segments.rs:90,91,92` 三条切段规则 | ✓ 三行逐字精确匹配 |
| `segments.rs:145` 闭式公式注释 | ✓ 精确匹配 |
| `second_transaction_step_zero_layer0.rs:252-253`（Quote A） | ✓ 精确匹配 |
| `:269-270`（Quote B） | ✓ 精确匹配 |
| `:286`（Quote C） | ✓ 精确匹配 |
| `:287-295`（Table 5 旧数组 assert_eq! 整块） | ✓ 九行逐字精确匹配，且与 `records/2026-09-24-里程碑二收尾调度.md:162`「second_transaction_step_zero_layer0.rs:287 的断言当场红」指向同一行 |
| `records/2026-09-24-里程碑二收尾调度.md:162` Table 4 各数字 | ✓ 精确匹配（数组、423、54、5575802973、96%、2104413 全部逐字核对无误） |
| Table 5 Row 2「15 处差异，位置 7,10,14,17,20,23,27,30,33,36,39,42,45,48,51」（本 agent 自算） | ✓ 用 python 独立比对 Table4/Table5 两个数组，结果相同（15 处，位置列表逐个相同） |

本地攻方转述核对表 14 行全部 ✓，无一处摘句、无遗漏限定词的可疑迹象（多处加字均已在"定稿说明"栏自证为可读性衔接词或消歧限定词，未引入新事实，抽查属实）。

### 本地攻方四份样本的算术复核

用 Table 6 的六条规则对 Table 4 Row 1 的 54 段数组独立编程复算 k_i/c_i（未参考任何一份样本的答案）：

```
$ python3 -c "..."   # 见草稿目录 /tmp/claude-1000/m2-layer0-scale-r1-verifier/
sum w = 423, sum k = 91, sum c = 332
Q3 每段 (2^k_i-1) 之和 = 146，加 1 = 147（grand total）
```

| 样本 | Q1 表（k_i 序列） | Q3 grand total | 判定 |
|---|---|---|---|
| s1 | 与独立复算完全一致 | 147 | ✓ |
| s2 | 与独立复算完全一致 | 147 | ✓ |
| s3 | 与独立复算完全一致 | 147 | ✓ |
| s4 | 与独立复算完全一致（Q1 表 k_i 序列逐行相同） | **145** | **✗**（内部算术错误：自己 Q1 给出的 k_i 表逐段代入 `2^k_i-1` 求和 +1 应为 147，s4 却报 145；差 2，找不到与「漏 1 段 k=1（贡献 1）」或类似单一漏项吻合的解释，只记数字对不上，不猜测它具体错在哪一步） |

四份样本的 Q1 表（54 行 k_i/c_i）彼此完全一致、且与独立复算完全一致；Q2 的公式推导（`2^k_i-1`，COW 子集不产生新状态，空子集已计入全局 +1 项）四份表述一致且与题面定义相符；Q4 自检（`c_i=0` 退化到基线公式、代入 Table4 Row1 的 w_i 得 5575802973）四份都能重新推出该数并确认与 Table 4 Row 3 相符。**唯有 Q3 的最终求和，s4 单独算错**（147→145）。运行记录里 s4 被标"判定：干净"（指词形损坏检测，不判内容对错），与这里发现的算术错误不矛盾——运行记录明确写"这属于答案内容层面的差异，不是词形损坏，按『不解读、不判它答得对不对』不在这里评判"，本核查员在此按共用约束把它核出来。

本地腿计：核 14 处转述引用 + 4 份样本的完整算术复核，✓ 17（14 转述 + 3 份样本），✗ 1（s4 的 Q3 求和）。

## 四、另核一条：三个数各自出自哪里、口径是不是同一个量

| 数 | 出处 | 现查结果 |
|---|---|---|
| 423 次写 / 54 段 / 闭式 5575802973（背景材料第 9 行） | `records/2026-09-24-里程碑二收尾调度.md:162`「层 0 规模与 E142 第二段」行 | ✓ 逐字精确匹配；该行注明是 2026-09-25 主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 在**当时的主工作区**（回退改形态 实一/实二/实三 尚未开始，"层 0 规模那一轮的约束"行的时间戳排在其后）上量出来的，且该行明确写"`second_transaction_step_zero_layer0.rs:287` 的断言当场红"——测出的数组已经比测试里钉死的旧数组新 |
| 2104413（钉在测试断言里的旧数组，54 段/279 次写反推） | 冻结副本 `second_transaction_step_zero_layer0.rs:581,588` | ✓ 精确匹配；独立编程验证：旧数组 `[2,2,1,...]` 代入 `1+Σ(2^w_i-1)` 得 2104413，段数 54、写数 279，与 Sonnet 报告的反算一致 |
| 5575606380（Opus 在冻结副本上实测，425 次写/55 段） | `research/prompts/m2-layer0-scale-r1-opus-model/outputs/p1.log:22`：`STREAM E writes=425 segments=55 sizes=[...] closed_form=5575606380` | ✓ 精确匹配；独立编程验证：把该行 `sizes=[...]` 数组代入同一公式，算出 425 次写、55 段、closed_form=5575606380，与日志逐字一致 |

三个数的口径确认是同一个量（第二条流"固定脚本到 E"、`1+Σ(2^|段|-1)` 闭式），差异全部可用形状变化解释，不是三个各说各话的量：

- **2104413 → 5575802973**：测试文件里的断言是旧形状，早于 2026-09-25 那次实测；`records:162` 行本身就说这条断言"当场红"，即代码的真实写流已经比断言钉的旧数组大，79→423 次写、多出的段全部来自脚本本身在多轮迭代中变长（Sonnet 报告里同一份文件三处注释"两个/三个/八个 18 写段"互相矛盾，佐证脚本确实迭代过、注释没跟上），不是测的量不对。
- **5575802973 → 5575606380**：Opus 的算术解释——冻结副本比 2026-09-25 那次测量多做了 实一/实二 的 SysPre 实现（抬 F 前先写一次系统配置、过一道屏障），把原来"两个系统配置槽写与 16 个单元写合并成一段（18 写）"拆成"独立的 4 写系统配置段 + 16 写单元段"，多出 2 次写、多出 1 段；差值应为 `(2^18-1)-(2^4-1)-(2^16-1)=196593`。现算：`5575802973-5575606380=196593`，与该公式算出的值完全相同。**Opus 的解释不是事后凑数，是可独立复核的精确算术恒等式。**

三个数分别对应"脚本还没长到今天这么长时钉的旧值"→"实一/实二 SysPre 实现之前当天量的真实值"→"实一/实二 SysPre 实现之后冻结副本上的真实值"，三者按时间线递进、差值都可用具体的形状变化精确解释，没有发现口径不一致或任何一处是凭空报出来的数。


## 总计数

| 腿 | 核了几处 | ✓ | ✗ | 说明 |
|---|---|---|---|---|
| Sonnet | 42 | 35 | 7 | 全部 7 处 ✗ 同属一种模式：引用行号指向「已定项 N」/章节标题所在行，抄的原文实际在其后 2〜49 行；内容本身核对无误 |
| Opus | 30（含 3 项复跑） | 30 | 0 | 含 SHA256SUMS 全量核对（20 文件）、复跑三格全部精确复现 |
| 本地攻方（转述核对表） | 14 | 14 | 0 | |
| 本地攻方（四份样本算术复核） | 4 | 3 | 1 | s4 的 Q3 grand total 145 应为 147 |
| 另核：三个数出处与口径 | 3 | 3 | 0（记「已解释的差异」，非 ✗） | 差值均可用具体形状变化的精确算术恒等式解释 |
| **合计** | **93** | **85** | **8** | |

判别力自证：1 项，判 ✗（方法有效）。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 没有逐字核对 Sonnet/Opus 报告里的**全部**引用——两份报告合计引用远超 100 处，本次挑了两腿正文里每一节判定表都用到的承重引用与全部可机械核验的 grep/awk 复现命令，未覆盖的多是纯散文转述或与判定无关的背景说明。
- 没有重跑 Opus 模型目录 `rerun.sh` 的全部内容（约一小时）：只按派发提示挑了 L3 第 304 行三个数、L1 H1 那一格、L7 严格读法那一格三处复现，其余（p2、p7 穷举对照、p9 真杀、p10 弱键、p11 用户动作放开扫等）未复跑，按 Opus 报告自陈的挂钟与产物哈希核对认可，未独立验证其数值。
- 没有验证 Opus 报告里"没打中"的各项（如 1.2/1.4/2.1/2.3/2.4 节、170 条流放开扫的 27/0 计数）——按三方推论纪律，"没打中"类结论的可信度判断归主 agent，不归核查员复核范围；只核了其中可机械复现的 3 格。
- 没有读 `crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-checker/src/walk.rs` 全文来验证 Sonnet/Opus 各自留白的"复核不了"条目（如 L1 推翻条件里"没有跑 allocator.rs 单测"、L7 推翻条件里"没有逐行读完 recovery.rs/walk.rs"）——两条腿都已在正文里明确标注这些是自己没做到的部分，不需要核查员代为补做。
- 没有核实本地攻方运行记录（runlog）里关于 `ask-local.sh` 退出码、字词损坏检测器判绿判红的具体过程细节（第 1 次调用作废、s2 带损坏的人工通读发现）——这些是过程叙述，不是可独立复核的「文件:行+抄的原文」或产物引用；只核了核对表本身与四份样本内容。
- 没有判断 s4 算术错误的具体成因（是否漏算某一段、是否符号错误）——只核出「结果与自证的复算不符」，不代主 agent 诊断病根。
- 未核 `m2-layer0-scale-r1-local-attack-output-void1.md`（作废样本）的内容细节，只确认其存在与运行记录里对它的描述一致（文件权限 600，按运行记录本身的自述采信，未强行读取）。
- 未做任何 git 写操作，未采纳任何一条腿的结论，未修改 `crates/`、`.claude/kb/`、`.claude/gate.d/` 等项目文件。
