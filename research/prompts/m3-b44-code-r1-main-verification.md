# m3-b44-code-r1 判决：代码欠账批 4-4 合入的产品代码，第一轮（2026-10-16）

被判的对象：代码欠账批 4-4 写段 W1–W6 合入主工作区的产品代码，`git diff HEAD -- crates/singlefs-core crates/singlefs-format`（HEAD c1e9b324，25 个文件 +1268/−421，全文 `research/prompts/_m3-b44-code-r1-diff.md`）。正文 `research/prompts/_m3-b44-code-r1-body.md`，格 K1–K8。

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 K7 K8 A1 A2 A3 W1 W2 W3 W4 W5 W6 W7 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 主工作区 `sha256sum -c --quiet research/prompts/m3-b44-code-r1-snapshot/crates-sha256.txt`；kb 快照 `kb-sha256.txt` 同法 | kb 全对上。crates 有 6 份对不上，都是 `crates/singlefs-checker-tier/src/` 下的 `crash_amplification.rs`、`crash_injection.rs`、`crash_judge_checker_common.wgsl`、`crash_judge_checker_scan.wgsl`、`crash_judge_dispatch.rs`、`crash_judge_gpu.rs`：别的会话的补丁在取快照之后 3 分钟打进主工作区（`~/.local/share/singlefs/work/impl-crates-round-4b-checker-tier/patch/crates.patch`，sha256 1552ade5…）。把 `crates/` 拷进草稿目录、对 `crates/singlefs-checker-tier/src/*` 做 `git apply -R` 这份补丁之后，118 份逐份对上快照。被判的 25 份不在那 6 份里，两条腿读的都是冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r1/repo/`，腿引的行没有一处落在被改的文件上 |
| 攻方 `research/prompts/m3-b44-code-r1-opus-output.md`（sha256 c2009ec7…，203 行）、正推 `research/prompts/m3-b44-code-r1-sonnet-output.md`（sha256 65532413…，259 行） | `cite-check.py <两份> --root <仓根> --background research/prompts/_m3-b44-code-r1-background.md`（根取主工作区：被判的 core 与 format 文件与快照逐份相同） | 「✓ 核了 89 处引文、66 处裸行号，对上 89 处，没判 66 处（其中裸行号没有引文可比 41 处；逐处列在上面）」。没判的 66 处是只写文件名的裸行号；判决承重的几处人工核过，在冻结副本里逐行对上：`mount.rs` 第 10241、10273、10301、10551 行是 K2 点名的断言，`pointer.rs` 第 325 行是 K4 探针 panic 的那一个 `assert!`，`recovery.rs` 第 5121 行是 `fn pointers_of_the_rebuilt_version_ascend_by_device(`，`crates/mutations.tsv` 第 3813、3824、3825、3858 行的变异名与报告说的一致 |
| 正推模型 | 拷进会话草稿目录，经 `run-with-memory-cap.sh 24G` 跑 `tools/rerun.sh`（带 `BINDGEN_EXTRA_CLANG_ARGS`） | 末行「name=rerun_compare same」：18 行变异各红在点名的测试、core 库与三个 harness 目标绿，与 `out/summary.txt` 逐行相同 |
| 攻方模型 | 同上 | 末行「name=rerun_compare all_same」：`agg-enumeration.txt`、`survey.txt` 两份都「same」，各批 `nonzero=0`。K1 的数在入库的模型目录上从头重做了一次，与腿报告逐行相同 |

## 二、逐格判

| 格 | 攻方 | 正推 | 判 |
|---|---|---|---|
| K1 | 没打中（量过，副本）：四段 `urJ` 一族历史，M1 写流的崩溃状态全量 90293 个，被判挂载两种盘序；读故障 k=5 那段 17 组、另三段 6 组；换几何 232 个盘面。R 走得完、走到那一判的状态一个没放过（`walk_pass=0`，带 ` HIT` 的行 0 行）；第三轮打中 `pointer_per_kind` 的 972、324 个状态两序都在写之前拒；入库 `urJ` 盘面读故障 `none` 下走到那一判的 4 个状态两序都放过 | 一致：读哪几个槽、读多长、比哪个实例代号、空指针、读不出走哪一支逐句对上用户定案（`research/prompts/c834-r3-main-verification.md`「## 四、用户定案（2026-10-15）」）；入库 `urJ` 三条放过用例绿，三行 C834 变异各红在点名的用例 | **代码对得上条款。** 攻方的数在副本上量、副本只在那一判外面加一层记录、交回值不动；复跑见第一节。与 `.claude/kb/decisions/23-journal的角色与格式.md` 第 392 行「现算不成不隔离、不拒」并排冲突是已知的待写回（第一轮判决 G5，C834 第三轮判决第三节），不是代码错。攻方只抽一次样：K1 的「没打中」按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」要再抽一次同立场腿；用户 2026-10-16 说这一批不再派子 agent，这一条记「只抽一次，待补第二次样」，第二次样派之前不写「没打中」 |
| K2 | 不归攻方 | 一致：A1 次序互换（`crates/mutations.tsv` 第 3824 行）红在两条新用例的 `assert_eq!`（`mount.rs` 第 10273、10301 行）；A2（第 3825 行）红在 `second_rollback.is_ok()`（第 10551 行）；A3 第 40 行改坏红在第 10241 行 | 采纳。主 agent 的轻型变异第三趟点名了第 3824、3825、40 行，三行都「红了」（`scratchpad/b44/light-mutation-3.log`） |
| K3 | 没打中（读码）：合法写路径只写格式常量；读者把它当自证不过、换槽 | 一致：偏移 301、与校验和不过同一臂、别的槽照择、拒因单列 | **代码对得上条款。** 两腿都点到条款没写的一格：同一槽节点大小坏、每区槽数 S 又越界时，代码先判节点大小（换槽），改前整池拒；只有不经写者、校验和自洽的坏镜像走得到。不改判，进欠账：两种拒因的先后要条款写一句 |
| K4 | 没打中验收字面（读码）；相邻一形零轮、没造盘面：三棵容器树（墓碑、谱系、退役）的容器指针与根之下节点指针不在重建时那道 I-2.5 判里 | 写者与条款一致；**读侧漏判，探针打中**：坏镜像（墓碑树根条目里一个子指针两条位置条目同盘、读得通）上冻结副本 `mount=ok`、`publish=panic`（`pointer.rs:325:9`），把写者断言改回改前的 `<=` 后 `publish=ok`；手跑一次、`rerun.sh` 一次 | **打中，归跑前条款第二行：改法并进修红批。** I-2.5（`crates/singlefs-checker/invariants/04-校验和.md` 第 10 行）说位置条目不严格升序「一律判损坏」，读侧该交错误，不该放进来再在写者那里 panic；`pointers_of_the_rebuilt_version_ascend_by_device`（`crates/singlefs-core/src/recovery.rs` 第 5121 行起）漏了三棵树，是实现没跟上条款，不是条款没写。缺口不是这一批带进来的（两条逆序的指针改前也在同一处 panic），这一批把「同盘」也并进了 panic 的范围。改法：把三棵容器树的容器指针与 `pointers_below_the_root` 并进那张清单，正推的探针改成 harness 用例（改后在挂载处拒、不 panic），谱系树、退役树各补一份同构用例；被攻过零轮，随修红批之后的代码轮再攻 |
| K5 | 没打中（读码） | 一致：两处产品调用都取读者自己的盘表，零块盘在那之前报错返回；`index_node_entry_capacity` 的调用方都交格式常量 | **代码对得上条款。** 「有记录、盘表空」`false` 变 panic 只有直调公开 API 走得到，已写进文档的 `# Panics` |
| K6 | 发布层没打中（读码）；挂载层一形交主 agent：挂载续做里一次删除类发布报这个错时，写行那次发布已落盘，挂载拒、之后每次可写挂载再取号、写行、再拒 | 一致：checked 加法、类型化错误、计划阶段交回、三处穷举 `match` 都对；谱系树、退役树的发布路径只有读码与 core 单测 | **发布层对得上条款。** 挂载层那一形：改前在同一个位置 panic、写行同样已落，不是这一批的回归；C841 用户定案只说「开新容器那一次发布报错、读盘不变」，对挂载续做没有一句话，归跑前条款第三行，交用户（第三节）。谱系树、退役树的端到端用例缺，进修红批补 |
| K7 | 没打中（grep 与读码，抽查） | 一致（抽查）：删掉的 16 个名字零调用，`every_unlink_step_from` 是挪进测试模块；四份文件只动注释；注释核了 6 处 | 采纳。两腿都只抽查了注释（正推约 180 行没逐句核），不写「注释全对」 |
| K8 | 没打中（读码） | 一致：`leaves` 三条边与求值时机都对；inode key 宽只剩 `INODE_KEY_WIDTH_IN_BYTES` 一处定义 | 采纳。轻型变异第三趟里第 3859、3860、3861 行都红了 |

## 三、处置

**并进 4-4 验证段修红批**（一批派 `implementation-writer` 一次改全；用户 2026-10-16 说这一批不再派子 agent，派的时机等用户）：

1. K4：三棵容器树的指针并进 `pointers_of_the_rebuilt_version_ascend_by_device`；探针改成用例，三棵树各一份；每处改法一条变异行。
2. K6：谱系树、退役树开新容器报错的发布层端到端用例（同墓碑树那一条：两块盘逐字节不变、分配器记录不变）。
3. 验证段的红，与代码轮无关、同一批改：
   - `crates/mutations.tsv` 第 393、1946 行在 HEAD 上就不红（调查员报告 `~/.local/share/singlefs/work/m3-b44-inv-not-red/report.md`，sha256 5b52f36d…），取样点不敏感，只改 harness 测试。
   - `crates/singlefs-harness/tests/system_configuration_slot_with_a_foreign_node_size_is_not_chosen.rs` 里判池级 checker 的那一条挪进 `pool_checker_` 起头的新测试文件（`scripts/crash/cpu-judging-sites-check.py check .` 现在红在它第 189 行）。
4. 改完回到验证段第 1 步整体再验一次；第 3839 行（轻型变异第三趟撞了每批 4G 内存上限没判到）与轻型崩溃（第三趟被第二台 GPU 上不在清场名单里的进程挡住，退 78）随那一趟重跑。

**交用户**（零轮）：

- K6 挂载层：坏镜像上最右容器号到顶时，挂载续做那一次删除类发布报错，写行那次发布已落盘，每次可写挂载都重复取号、写行、拒。攻方给的两个候选（都被攻过零轮）：挂载在取号之前预演续做第一步的计划，这一错在取号之前拒；或续做里遇到它照「推满仍放不下」那样停下、挂载照样做成可写、删除队列留着。

**进欠账**：K3 两种拒因（节点大小、S 越界）的先后没有条款；K1 第二次同立场样本。

## 这一轮被判过的文件

这一轮判过的产品代码文件（逐个按路径点名）：

- `crates/singlefs-core/src/address.rs`
- `crates/singlefs-core/src/admission.rs`
- `crates/singlefs-core/src/allocator.rs`
- `crates/singlefs-core/src/block_device.rs`
- `crates/singlefs-core/src/code_two_tree.rs`
- `crates/singlefs-core/src/container_tree.rs`
- `crates/singlefs-core/src/extent_tree.rs`
- `crates/singlefs-core/src/inode_tree.rs`
- `crates/singlefs-core/src/instance_table_sweep.rs`
- `crates/singlefs-core/src/journal.rs`
- `crates/singlefs-core/src/lineage_tree.rs`
- `crates/singlefs-core/src/make_filesystem.rs`
- `crates/singlefs-core/src/mount.rs`
- `crates/singlefs-core/src/mounted_read.rs`
- `crates/singlefs-core/src/mounted_session.rs`
- `crates/singlefs-core/src/pointer.rs`
- `crates/singlefs-core/src/recovery.rs`
- `crates/singlefs-core/src/retirement_tree.rs`
- `crates/singlefs-core/src/root_ring.rs`
- `crates/singlefs-core/src/system_configuration.rs`
- `crates/singlefs-core/src/tombstone_tree.rs`
- `crates/singlefs-core/src/transaction.rs`
- `crates/singlefs-core/src/unit.rs`
- `crates/singlefs-core/src/unlink.rs`
- `crates/singlefs-format/src/lib.rs`

## 补记：K1 第二次抽样（2026-10-16）

同一份提示再派一条攻方腿（Opus），报告 `research/prompts/m3-b44-code-r1-opus-s2-output.md`（sha256 18e544d5…，257 行），模型 `research/prompts/m3-b44-code-r1-opus-s2-model/`；主 agent 现跑 cite-check：「✓ 核了 32 处引文、2 处裸行号，对上 32 处，没判 2 处（其中裸行号没有引文可比 2 处；逐处列在上面）」，两处裸行号都是 `pointer.rs:326`（第一次那份判决引的写者断言在第 325 行，相邻一行，不承重）。第二次样 K1 两个分句都没打中（副本上的数：换几何 232 段 × 6 组读故障 × 两序 2784 次挂载；入库 `urJ` 那一段 20516 个崩溃状态 × 7 组读故障 × 两序；前缀 `oiww` 那一段 24633 个崩溃状态 × 3 组读故障 × 两序；判错 0），拒的分布与 c834 第三轮判决的数相同（`tt1+mr0+mr1+rec` 下拒而 R 已没 11964、拒而 R 走得完 972）。与第一次样对齐到同一件事上比，K1 两次都没打中，记「没打中」。K3、K4、K5、K6 两次一致；K4、K6 的相邻形与第一次样相同，已并进修红批（W8）。第二次样的模型复跑没跑：别的会话此刻在两台跑崩溃全量、占满机器，等它跑完再跑 `tools/rerun.sh`，在那之前判决里 K1 第二次样的数只当腿报的数、没经主 agent 复跑。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-16 不受影响：这一轮不改决策文件；C834 新判法替换 Y2 收严那一判，第 392 行那一句随写回批一起改 |
| D22（单元原子性怎么合成） 已定项 26 | 不影响 | 2026-10-16 不受影响：C840 只让读者判节点大小等于格式常量，第一版没有改节点大小的路，三档可改性不动 |
