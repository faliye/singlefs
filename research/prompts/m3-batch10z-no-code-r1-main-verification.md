# 根环实现第 10z 批第十四、十五块：代码轮第一轮判决（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 N1 H2 R C P Q F -->

正文 `research/prompts/_m3-batch10z-no-code-r1-body.md`。两腿报告：
- 正推：`research/prompts/m3-batch10z-no-code-r1-sonnet-output.md`，sha256 e18f141f…28a5，538 行；
- 攻方：`research/prompts/m3-batch10z-no-code-r1-opus-output.md`，sha256 e5ae2ed5…0d5b，221 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree/` 对 `research/prompts/m3-batch10z-no-code-r1-snapshot/crates-sha256.txt` `sha256sum -c` 全过；kb 对 `kb-sha256.txt` 全过 |
| 引文 | `cite-check.py` 两份各跑，`--root` 冻结副本与仓根：正推 `✓ 核了 19 处引文，对上 19 处，没判 0 处`，攻方 17 / 17 |
| 主 agent 现读 | 冻结副本 `crates/singlefs-core/src/mount.rs` 第 1895–1910 行：没施加的发布那一圈读它的账，读不出而记录齐时 `None => continue`，那次发布的账一个落点都不交（攻方 K1-1 旁证那一形的病根） |
| 正推复核不了的三条 checker 档行为用例 | 第十五块合入验证带 gpu 特性那一趟里都跑过：`~/.local/share/singlefs/work/m3-r1/merge-10z-o/checker-lib.log` 里 `test crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it ... ok`、`test crash_amplification::tests::the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken ... ok`，`gpu-tests.log` 里 `test a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest ... ok` |
| 复跑 | 没复跑：两腿的打中都是「改回去照绿」一类判别力缺口与一处 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 已有的形，判决只据它们定下一批补用例，不引腿副本上的数进 kb |
| 腿的数 | 都是腿的模型或副本上的数，不进 kb |

被判的源文件：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K1-1 回收路补隔离 | 攻方：入库与第十四块改前副本随机历史 2168 段逐段配对，红的段一个不差；把抬 F 回收改回不补隔离的两条臂各 2 段被探针看到「隔离还护着却空闲」（阳性对照）。正推：产品回收入口 5 处静态全罩到，自写读数对 9 个漏补变异抓到 8 个，没变异的树上 1288 段 0 个 C 落点被发出去 | 没打中（两腿各一次，都是造出来的扫法、带阳性对照） |
| K1-1 旁证 | 攻方：R 的根槽与账挂上之后一直读回全零、R 的记录齐、不再挂：R 引用的单元不进 C，回收后发出去，I-7.4 红，68 段里 16 段；第十四块改前副本同批逐段同红。主 agent 现读 `mount.rs` 第 1910 行坐实 | 不是这一批带进来的，归 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）：题面补「没施加的发布记录齐、账读不出时那次发布的账不进 C」一形，下一份 kb 写回 |
| K1-2 按槽跳过与起点槽豁免 | 两腿没打中；正推：C 那一圈 `!isolated.insert(key)` 按起点槽去重，同起点较宽的那份少罩一槽，改前同形 64/640，被漏的槽没有候选根合法引用、池级 checker 没红；攻方同形两臂相同、多出那一槽是陈旧记录 | 没打中；同起点去重那一形记进 C554 同一条写回的旁注 |
| K1-3 第九项与已分配 | 两腿：C 那一圈双扣改前 540 / 104 槽，改后 0；剩下的双扣在被抛弃根那一圈（第十六块已改）。正推：条款字面「仍分配着」读成「分配位置着」靠括注「那些算在已分配里」才同义 | 没打中；第九项写回时把「分配位置着（含已释放而还在 defer 窗口里的）」写进正文 |
| K2 第十四块判别力 | **两腿都量到**：① `slots_counted_both_as_allocated_and_as_isolated` 在用例里全是 `== 0`、没有阳性对照，访问器恒交 0 时用例文件 24 条照绿；② 抬 F 那一串卸载那一支（`ImmediatelyBelowTheNewestValidRootOnEveryDevice`）改回不补隔离，入库全绿，探针在卸载之后看到 C 护着的槽空闲（攻方 72/108、100/100 段，正推 48/72） | **打中（窄）**，补用例：访问器要一条阳性对照（造一个同时置两位的槽、读得出非 0）；卸载那一支要一条照攻方 `jobs-u1.txt` 那一形的用例 |
| K3-1、K5-1 多给 | 攻方：3000 种子 × 300 步、189 708 次借，多给 0、重起被拒 0；第十五块之前的走法多给 63 次（阳性对照）。正推：18 个账层变异在 123 条库单测上全红，4000 种子过 | 没打中 |
| K3-2、K5-2 少借与压并发 | 两腿：少给只在用户已认或文档写明只会少借的几类；没加新的串行段 | 没打中 |
| K4 第十五块判别力 | 账层七种改法与 18 个变异全红。扫源码用例旁「拦不住的写法」没写全：攻方在卡池模块里加同名 `mod wgpu` 或 `mod std` 遮住方法体里的路径、方法体一字不动，`cargo check` 0 警告编过、扫源码照绿；正推 N1（`check_selected_blocks` 里真调用把实参换成 `&|| false`，钉住的那一行留在 `if false` 死分支里，编得过、扫源码照绿）与 `GpuCard::device()` 返回哪台设备不在整句里 | **打中（窄）**：能拦的补到会拦（扫源码拒卡池模块里与外部 crate 同名的模块、拒死分支包着钉住的那一行），拦不了的写进用例旁的名单 |

## 三、下一批（第 10z 批第十七、十八块）

- 第十七块（core 测试）：K2 两处补用例与变异，落在 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`（第十六块也改这一份，第十七块在第十六块合入之后的树上做）。
- 第十八块（GPU 测试）：K4 那几种写法能拦的补到会拦，拦不了的写进名单；落在 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`。
- 第十六块（显式推环入口与带出路的错、被抛弃根那一圈 defer 双扣）这一轮判完之后合入，与第十七、十八块合成一批开代码轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-09 改了：第九项「只除去现行账里仍分配着的槽」写回时写明「分配位置着（含已释放而还在 defer 窗口里的）」与两条补隔离的回收路 |
| D16（发布语义） 已定项 1 | 备料 | 2026-10-09 不受影响：判的是双扣改法与 GPU 账，第 37 行写回随第十六块一起 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-09 不受影响：第 390 行与 I-7.4 照旧 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-09 不受影响：判的是 G2 乙补法的实现与测试 |
