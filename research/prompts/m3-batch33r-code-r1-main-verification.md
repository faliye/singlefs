# 删除批第 33 批 R 合入的代码：代码轮第一轮判决（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 R1 R2 R3 R4 H1 H2 H3 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-batch33r-code-r1-body.md`、`_m3-batch33r-code-r1-diff.md`、`_m3-batch33r-code-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-batch33r-code-r1-snapshot/crates-sha256.txt`（4 份）；冻结副本 `~/.local/share/singlefs/work/m3-batch33r-code-r1/tree/` | 腿跑着的时候主 agent 没改这 4 份：第 33 批 S 等这一轮判决写完才合；主 agent 这期间只往 `.claude/gate.d/stage-inputs.tsv` 插了一行登记（不在快照里） |
| 云端正推（Sonnet） | `research/prompts/m3-batch33r-code-r1-sonnet-output.md`（352 行，sha256 a4977b62…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-batch33r-code-r1-opus-output.md`（sha256 9701bb38…），模型 `…-opus-model/` | — |
| 引文 | 两份腿报告 | `cite-check.py --root <冻结副本> --root . --background <背景材料>`：正推「核了 108 处引文，对上 108 处，没判 0 处」；攻方「核了 15 处引文，对上 15 处，没判 0 处」 |
| 主 agent 复跑：正推 | 模型拷进会话草稿目录 `m3b33r-sonnet-model/`，`SINGLEFS_REPO=<仓根> SINGLEFS_THREADS=6 rerun.sh ~/.local/share/singlefs/work/m3b33r-sonnet-rerun`，再 `compare.sh <summary.txt> <summary-expected.txt>` | 「SAME 131 行」，汇总 sha256 562d030b… 与腿报告相同；`JUDGE states=62 bad=0`、`MODEL equivalence_new_vs_old tables_with_a_file_only judged=81920 differing=0`、`MODEL new_vs_spec_all_tables judged=3739770 differing=0 first_diff=None`、`MODEL file_read_on_a_root_in_the_without_a_file_group judged=787320 accepted_as_ok=0`、两行 `SELFRED … =yes` 原样都在 |
| 主 agent 复跑：攻方 | 模型拷进会话草稿目录 `m3b33r-opus-model/`，`SINGLEFS_REPO=<仓根> rerun.sh ~/.local/share/singlefs/work/m3b33r-opus-rerun all`，再 `compare.py <复跑> <它的 run/>` | `compared=45 differing=0`；`name=alternative_summary stream=h1-2dev roots=8 judged_pairs=32 differing_pairs=0 old_differing_pairs=4`、`… stream=h2-2dev roots=13 judged_pairs=52 differing_pairs=0 old_differing_pairs=9`、`… stream=h3-2dev roots=9 judged_pairs=36 differing_pairs=0 old_differing_pairs=4`；`name=threads_case … case=two_shards_merged states=33 observed_states=33 … equals_reference=true` |

被判的 4 份文件（K3），每份一格判到。

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| K1 R1.1 新判法在文件从不消失的表上与旧判法等价 | 一致：靠集合恒等式，不靠取样；单测两张表没有回退表，偏窄不漏判；违例原文点名的版本从「第一个」换成「最新的」，不进签名 | — | **采** |
| K1 R1.2 新流版本表对 D8（核心索引结构） 已定项 16 第 4 条 | 一致：A、B、C 是实例 1 第 3–5 代，意图第 6 代，各批第 7–9 代不进表，探针逐项对上 | — | **采** |
| K1 / K2 R1.3 「没有文件」组被读出内容判成立 | 不成立：读回内容那一支不比内容，一律 `FileReadWhereThisRootHasNone` | 没打中：同一根同时进两组建表时就停 | **采**；没打中记「没打中（一次抽样）」 |
| K2 R1 新旧判法分歧而旧的才对 | — | 没打中：H1 / H2 / H3 三段历史 30 个根、每根 4 种结局，新判法与真值 0 格不同，旧判法 17 格不同、都是新的对；要让旧的才对需同 txg 两实例分叉，推的是那张表在正确实现上自己先红 | **记「没打中（一次抽样）」**（复跑原样同上） |
| K1 R2.1 工作线程与调用线程是同一判法 | 一致 | — | **采** |
| K1 / K2 R2.2 漏核、重核 | 能分辨，但靠计数名与调用线程那一趟比；「每个状态都看过」那句在工作线程版近似同义反复（`observed_states` 在调 `observe` 之后无条件加一）；续跑、分片下的工作线程观察者没有用例 | 没打中：33 个状态的流 13 种跑法（1/4/6 线程、调用线程、三种跨线程续跑、panic 之后再续、两片分片再并）逐项全等，截半再续那一趟观察者恰调 17 次 | **采**：漏核由计数名那条比较抓（变异 3109 是证据），那句断言不单独当证据；续跑与分片在攻方副本上量过一次，入库用例不补（第三节不做那一条） |
| K1 R3.1 / R3.2 新流的期望与三种崩点 | 声明范围内一致，62 个代表状态在真代码上逐项成立；声明范围外 5 格没钉：全量用例没登记 `crash-case:`、续做各批的事务号、挂完的盘没过池级 checker 也没与不崩溃的终态比、①b 两份盘逐字节相同、快档单元写落一半 | 没打中：最后一批之后读出内容、意图之前报没有文件都判红；H2 意图之后两截（挂载写行 262 164 态、两次暖机 131 095 态）全量逐状态新判法与真值分开 0 个 | **采**；5 格的去向见第三节 |
| K1 R4 变异 | 9 行锚点各中一次、替换后编得过；读码推 9 行都红；3105、3107 只库单测抓得到；3103 改坏后「走到空」那句不红（`crates/singlefs-core/src/mount.rs` 第 5416 行续做结果初值就是 `WalkedToEmpty`），红在步数与队列空；新代码三处没有变异行盖着（`FileRead` 落在没有文件组那一支、计划哈希里「没有文件那几版」的分支、全量用例） | — | **采**；补行与那句断言的去向见第三节 |
| 攻方观察 ① 计划哈希只拼有没有观察者 | — | R 之前就是这样，靠输入指纹把关 | **记**：不改 |
| 攻方观察 ② 删除那几版上 oracle 看不见 size 与数据单元 | — | 冷走读遇到 nlink = 0 就答没有文件，替它看的是池级 checker 的 I-4.6 等 | **记**：正推 R3.3 第 3 项（挂完的盘过池级 checker）补上之后这一格由 checker 罩 |
| K3 | `crates/singlefs-checker-tier/src/crash.rs`、`crates/singlefs-harness/src/memory_pool.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`、`crates/mutations.tsv` 各一格 | — | **采** |

## 三、去向

1. 全量用例的 `crash-case:` 登记：已办（主 agent 2026-10-07 插进 `.claude/gate.d/stage-inputs.tsv` 第 43 行，`crash-case-check.py --only placement,modules` 全绿）。
2. 进删除线下一批（S 与根环线 10x 合入之后，同批并根环线托的两件对拍）：新流观察者在每个状态挂完之后跑池级 checker、核续做各批落成第几代（D8 已定项 16 第 2 条末句、第 7 条 ③）；变异补行盖住 `FileRead` 落在没有文件组那一支与计划哈希里「没有文件那几版」那一支。
3. 进 core 下一批（第 33 批 T 正在改 `mount.rs`，不给在跑的追加）：续做结果的初值不用 `WalkedToEmpty`，让「走到空」那句断言对「续做被跳过」有判别力。
4. ①b（同一份崩溃镜像各自重挂载续做、两份盘逐字节相同）：这条流每状态只挂一次；D8 已定项 16 第 7 条那组判据另有 harness 层用例，这一轮没人读它钉没钉，记进下一批派发时先读、缺了再补。
5. 不做：快档里单元写落一半（快档只展开写数小于 10 的段是设计，全量里有）；「每个状态都看过」那句另加独立计数（漏核由计数名比较抓，变异 3109 证过）；工作线程观察者加续跑、分片的入库用例（攻方副本上 13 种跑法量过一次全等，回扫随下一次动续跑与分片那一批）。

## 四、交用户

不交：两条腿都没打中，没有岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-07 不受影响：oracle 认「这一版起没有文件」照第 4 条读法实现，两条腿与复跑都对上；第三节第 2、4 条是测试补钉，不改条款 |
| D8（核心索引结构） 已定项 5 | 不影响 | 2026-10-07 不受影响：三种崩点读回哪一版、挂完走到空与三态表对上 |
