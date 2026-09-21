# 背景材料：增补 3 第 3 件（崩溃注入）的实现，代码轮第二轮

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 -->

判的是里程碑「第二个事务」增补 3 第 3 件在第一轮判决之后改出来的代码，还没提交。开工快照 `research/prompts/m2-supp3-item3-code-r2-start-snapshot.sha256`。

## 一、这一轮为什么开

第一轮判决 `research/prompts/m2-supp3-item3-code-r1-main-verification.md` 第四节交了七题，第 7 题是「要不要再攻两轮」，候选 ① 是「先按这一轮的判决修 1、3、4、6，再开第二轮攻新形态」。那几题 2026-09-20 由用户定案、改法落地，`crates/mutations.tsv` 第 181–192 行按「用户 2026-09-20 定案第 N 条」各留一条会红的变异。定案原话在仓里没有落点，反推出来的六条记在 `m2-supp3-item3-user-verdicts.md`（2026-09-21 随门禁 91 号归档进了版本库历史，取法 `git show 8186d5b^:research/prompts/m2-supp3-item3-user-verdicts.md`）。

**这一轮攻的就是那几条改法本身。** 按 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算没被攻过」，第一轮攻方提的四条改法（补段内子集 / 补记录核对器 / 放开失败那一步 / 放开起点）与本地腿提的已知红两个方向的修法，全部**被攻过零轮**。

## 二、实现今天的样子（主 agent 的观测，读的是工作区那一版）

- `crates/singlefs-harness/src/crash_injection.rs` 从第一轮判过的 1040 行长到 1415 行（+599 / −224，37 个 hunk）。结构性改动是 `CrashPoint`：第一轮是「截在流的第几个写之后」一个下标，今天是「第几段 + 那一段里哪几个写持久了」（`segment_index` 加 `persisted_within_the_segment: Vec<bool>`）。文件头写明枚举域与层 0（`crates/singlefs-harness/src/crash.rs`）同一个：更早的段整段持久、当前段任意真子集、更晚的段一个都没持久，撕裂并进「没持久」（D13（验证路线） 已定项 4）。
- 段内子集怎么抽：`CrashPointDraw` 两支——`Sampled { crash_points_per_history }` 按种子抽固定个数，`EveryProperSubsetOfShortSegments { segment_writes, sampled_in_longer_segments }` 把短段整段枚举。段内写数不超过 `WRITES_A_SUBSET_MASK_HOLDS`（63）时子集掩码用一个 u64 抽，更长的段逐写各抽一次。`CrashPoint::withholds_a_write_before_a_persisted_one()` 判这个状态是不是「后发的写先持久」那一类（只截前缀摆不出来的那一类）。
- 起点段放开：崩溃状态摆在「起点那一段起、最后一个跑完的操作为止」，`CrashPoint::operation_kind` 落在起点里的是 `None`；失败那一步的写仍然不摆。
- 记录核对器接进来：`crash_injection.rs:697` 调 `crate::crash::check_records_against(&image, writes_up_to_this_segment, report.effective_root)`，与层 0 那一路同一份判据；`:706` 把「没违例、模型不反对、`record_check == RecordCheck::default()`」三条合起来当这个崩溃状态通过。`CrashInjectionTally` 里记核对器跑过几次、两类各判红几次。
- 已知红两个方向：`crash_injection.rs:906` 那一段不再把 `raised_floor_lands_only_on_abandoned_roots` 写死成 `None`（第一轮 K6 的假阳那一半）；假阴那一半靠 checker 在 I-3.1 的说明文字里多写一项机理标识，判读的一方按那个标识认机理（`crates/mutations.tsv` 第 187 行钉着 checker 那一侧，第 185、186 行钉匹配器这一侧）。
- 种子基：`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE = 7_463_871_032_432_355_113`（`crash_injection.rs:70`），随机历史那五段与崩溃注入这三档的种子区间都从它起。文档注释写明一个测试周期从「开一个新里程碑」与「用户显式说从零开始测试 / 重建测试」里先到的那一件算起，开新周期要做三样：重抽这个数、删掉全部旧镜像重建、`crates/mutations.tsv` 整表在新基上重验一遍。
- 用例两份：`crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs`（603 行，八条：种子基对不对、快档、一段写死的历史全崩溃点、抬 F 进空档那一形、崩溃点落进回退与抬 F、线程数 1 与 4 逐字相同、大档按环境变量、抽崩溃点不改变这段历史怎么跑）与 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（1272 行，第 1、2 件的五段取样点与边沿用例）。
- `crates/mutations.tsv` 这一轮新增 37 行，其中第 181–192 行按「用户 2026-09-20 定案第 N 条」各留一条（N 取到 9）。
- `crates/singlefs-harness/src/lib.rs` 只加一行 `pub mod crash_injection;`。

## 三、要判的问题

**K1（段内真子集抽得够不够）**：一段有 n 个写就有 2ⁿ−1 个真子集，快档每段只抽固定几个。第一轮 K1-b 打中的是「同段镜像双写只持久后一份」这一整类零概率（快档 24 段里同段镜像对 2782 对）。今天摆得出来了，但**抽得到吗**：快档一轮里 `withholds_a_write_before_a_persisted_one()` 为真的崩溃状态有几个？同段镜像对里被摆出「只持久后一份」的有几对？举一个今天仍然零概率或接近零概率的子集形态，能写成可跑的构造更好。

**K2（起点段放开之后，那 627 次写真的被抽到了吗）**：第一轮 K1-d 量到快档 7219 次写里 627 次永远不是候选。今天起点段进了候选域。逐条核：快档一轮里 `operation_kind == None` 的崩溃点有几个？`assert_every_crash_injection_path_was_exercised` 里有没有一条断言钉住它 **> 0**？没有的话，这项修补与没修在报告上分不出来——报告里两种情形长得一模一样。

**K3（记录核对器接上之后判过什么）**：第一轮 K4-4 打中「`root_without_record` 与 `claimed_state_missing_unit` 在这一件里一次都没判过」。今天 `:706` 把 `record_check == RecordCheck::default()` 当通过。问：快档一轮里核对器跑过几次、两类各判红几次？要是某一类在崩溃状态上**恒返回 default**，这一格与「没接核对器」逐字相同，而报告不会说。举一个能让它判红的崩溃状态。

**K4（种子基按周期重抽之后，复现还成不成立）**：`crates/mutations.tsv` 第 191、192 行钉的是「换成别的数就红」，证明的是两个二进制同基，不是「这一批判红在新基上还成立」。文档注释写开新周期要做三样，第三样（整表在新基上重验）没有任何东西盯着。逐格判：这三样里哪几样有会红的检查、哪几样只写在注释里；一个周期换了之后，`crates/mutations.tsv` 里点名随机历史与崩溃注入的那些行，判红是不是掷骰子。

**K5（已知红的判别力挂在哪）**：假阴那一半的修法是 checker 在 I-3.1 的说明文字里多写一项机理标识、判读的一方按那个标识认机理。逐格判：这个标识是不是字符串匹配；checker 的输出格式改了（换措辞、加一项、调顺序）之后，判读的一方是报成新发现（假阳）还是接走（假阴）；`crates/mutations.tsv` 第 185、186、187 行三条各钉住了哪一侧、哪一侧没钉住。

**K6（那几条断言够不够）**：第一轮 K1-g 打中「全是存在性断言，`crash_points >= 24` 允许 23 段全 0（实际就有 1 段 0 个）」。今天 `assert_every_crash_injection_path_was_exercised` 查的是什么：哪几项是存在性断言、哪几项钉了绝对值或下界；七类操作里今天还有几类实测是 0；一段 0 个崩溃点今天还可不可能。

## 四、分工

| 腿 | 攻击面 |
|---|---|
| 云端攻方（Opus） | K1、K3：段内真子集的实际覆盖、记录核对器接上之后判没判过东西。要举得出具体的崩溃状态或子集形态，能写成一段可跑的构造更好；副本上量出来的数要写明是副本 |
| 云端正推（Sonnet） | K2、K6：逐条核「起点段放开之后报告里看得出来」与「那几条断言够不够」，引代码行号与产物读数，不引结论句 |
| 本地攻方（英文，逐格填表） | K4、K5：种子基按周期重抽之后复现还成不成立、已知红的判别力挂在哪。按表格逐格填，不许只答 yes / no |

两条攻方腿的攻击面不重叠：Opus 攻**新开的枚举域实际覆盖到了没有**（段内子集、记录核对器），本地攻**判别力挂在一个常量与一段人写的说明文字上**（种子基、机理标识）。

判决写 `research/prompts/m2-supp3-item3-code-r2-main-verification.md`，按路径点名这一轮改过的每个 `crates/*/src/*.rs`（门禁 56 号判形式）。
