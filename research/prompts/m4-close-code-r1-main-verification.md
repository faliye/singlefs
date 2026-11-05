# m4-close-code-r1 判决：里程碑四收口那 14 份补丁的代码轮（主 agent，2026-10-30，第一轮）

正文 `research/prompts/_m4-close-code-r1-body.md`，diff `research/prompts/_m4-close-code-r1-diff.md`，背景材料 `research/prompts/_m4-close-code-r1-background.md`。腿：正推（Sonnet）`research/prompts/m4-close-code-r1-sonnet-output.md`，攻方（Opus）第一次抽样 `research/prompts/m4-close-code-r1-opus-output.md`、第二次抽样 `research/prompts/m4-close-code-r1-opus-s2-output.md`。正推、攻方第一次抽样两条都中途停下过，各由一条同立场的新腿抽交接摘要接着写完。

被判的产品代码（冻结副本 `~/.local/share/singlefs/work/m4-close-code-r1/repo/`，开工快照 `research/prompts/m4-close-code-r1-snapshot/crates-sha256.txt`，30 份）：`crates/singlefs-core/src/address.rs`、`crates/singlefs-core/src/block_device.rs`、`crates/singlefs-core/src/bytes.rs`、`crates/singlefs-core/src/checksum.rs`、`crates/singlefs-core/src/container_tree.rs`、`crates/singlefs-core/src/deleted_inodes.rs`、`crates/singlefs-core/src/extent_tree.rs`、`crates/singlefs-core/src/inode_tree.rs`、`crates/singlefs-core/src/journal.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-core/src/lineage_tree.rs`、`crates/singlefs-core/src/logged_operations.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/pointer.rs`、`crates/singlefs-core/src/records.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/retirement_tree.rs`、`crates/singlefs-core/src/root_record.rs`、`crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-core/src/tombstone_tree.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/truncate.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-core/src/verified_model.rs`、`crates/singlefs-core/src/write_request_split.rs`、`crates/singlefs-reference-filesystem/src/lib.rs`、`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs`、`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs`、`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs`、`crates/singlefs-replay-prefix/src/lib.rs`。

## 一、核查

- 开工快照：`python3 scripts/agents/snapshot-tree.py m4-close-code-r1 --out /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-judge/m4-close-code-r1-tree`，「清单点名 36 份，倒推出 36 份，sha256 对上 36 份」（30 份 crates 加 6 份 kb）。写判决时 `sha256sum -c research/prompts/m4-close-code-r1-snapshot/crates-sha256.txt` 30 份全过，腿跑着的时候被判文件没被改过。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-close-code-r1-sonnet-output.md research/prompts/m4-close-code-r1-opus-output.md --root /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-judge/m4-close-code-r1-tree --background research/prompts/_m4-close-code-r1-background.md`：「核了 29 处引文、40 处裸行号，1 处对不上，43 处没判」。对不上那 1 处：正推腿第 304 行引 `gc_crash_proofs.rs:9-15`「这是分配器与 floor 的来历要另证的义务，不在这里证」，冻结副本里是第 8 行「两条前提是分配器与 floor 的来历要另证的义务，」，转述、意思对得上，不改结论。没判的 43 处多是「冻结:路径:行」这种它认不出的写法与快照树之外的文件（`crates/mutations.tsv`、没进快照的 `partial_write_proofs.rs` 等），判决要引的那几处逐处在冻结副本里现查过（第二节各格的「现查」）。攻方第二次抽样的报告，腿交回前自跑 cite-check「核了 19 处引文、8 处裸行号，对上 19 处，没判 8 处」。
- kb 快照：`sha256sum -c research/prompts/m4-close-code-r1-snapshot/kb-sha256.txt` 只有 `.claude/kb/checks-owed.md` 对不上（腿跑着时别的会话往欠账表加了行），C145 那一行从第 920 行挪到第 921 行；正推腿第 71 行引的 `checks-owed.md:920` 就是这一行，照第 921 行读。
- 别的会话动过这一轮的材料与快照里的文件：
  - 同一会话恢复后另开的一个进程派的接手正推腿，在正推腿交回之后改了报告一处（它会话记录里 `sed -i '71s/kb:checks-owed.md:920/kb:checks-owed.md:921/'`，改前 grep 输出第 71 行是 `:920`）；被改后的那一份存在 `~/.local/share/singlefs/work/m4-close-code-r1-judge/sonnet-output-at-1538.md`，主 agent 把第 71 行改回 `:920`，前后只差这一处（diff 核过），现 sha256 ee9e5b03…、436 行。攻方第一次抽样的报告没被动过（sha256 96f6d882…）。
  - GPU 垃圾回收那一路的第 54 批要往 `crates/singlefs-core/src/mounted_session.rs` 加四处小块（+13/−3），打之前的原样与改动块存在 `~/.local/share/singlefs/work/m4-gpugc-code-r1-fix-writer/before-main-apply/`（打之前那一份 sha256 前缀 b85124b79ff20ba1，与开工快照相同）。写判决时还没打进主工作区；腿引 `mounted_session.rs` 的行（第 5128–5166、5460–5470 行一带）不落在那四处（第 2383、3203、3674、3688 行一带）。
- 主 agent 现查（冻结副本）：第二节「现查」各处；T10 的 8 行锚点逐行数过命中数（冻结副本 0、HEAD 1、主工作区此刻 0）。
- 核不动的：两条腿的探针与差分复跑（`tools/rerun.sh`）没在副本里重跑——用户经别的会话转达，新起的脚本任务先不起，等统一任务启动器；腿交回前各自在新 shell 里整份跑过（攻方第一次抽样末行 `name=rerun_attack_m4_close_code_r1 compared=5 differing=0`，第二次抽样 7 段都退 0，正推腿三个 `same_as_out=yes`），判决引它们的数都标「腿量的」。T8 两腿都没起 Verus，按 spec 文本手推。

## 二、逐格判定

抽样规矩照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」：打中的一次就核；「没打中」要攻方两次抽样都没打中才记。

| 格 | 正推 | 攻方第一次 | 攻方第二次 | 现查 | 判定 |
|---|---|---|---|---|---|
| T1 多 inode 改 size | 成立（T1.2 ② 判决括注字面有歧义） | 没打中（读码） | 没打中（读码） | — | 成立；两次没打中，可记「没打中」 |
| T2 部分写 core | 成立（探针 2205 例、移植版 56964 例，腿量的） | 没打中（12000 例，腿量的） | 没打中（300000 组，腿量的） | 冻结副本 `write_request_split.rs` 的计划已改成调 `partial_write_proofs.rs`，不在这一轮的 diff 里，正推腿报两版公式相同 | 成立 |
| T3 模型与 RefFS | T3.1 条款没说（F2 读法，D4（校验和位置） 已定项 4 字面不分一份与两份），T3.2 成立 | 没打中（读码） | 没攻 | — | T3.2 成立；「没打中」只抽一次，不支撑；T3.1 交用户（第四节） |
| T4 发布触发 | 成立；T4.4 窄角（推的）；T4.5 条款没说 | 没打中（冻结时 `accept` 拒） | 没打中（读码） | `mounted_session.rs:5126-5153` 的 `publish_the_pending_changes`：没发成且冻结时待发不放回，注释写「冻结了时待发并进了冻结的那一次」；冻结的若是空间被拒之后那一次抬 F 的空发布，它里面没有待发的数据。攻方第一次抽样说的「冻结时 accept 拒」管的是 `accept`（第 5168 行起），碰不到这一支 | T4.4 打中的可能性读码相容，未造出；第五节 C 簇先写会红的用例坐实 |
| T5 块设备 | 成立 | 没打中（执行体逐行相同） | 没攻 | — | 成立的那一半（执行体相同）是读码观测；「没打中」只抽一次 |
| T6 编解码同义改写 | 成立（抽样） | 没打中（153800 例，腿量的） | 没打中（1082244 行差分逐字节相同，腿量的） | — | 成立 |
| T6 往返契约 | 条款没说 | 三处取不到（journal 序号 0 等），只写在 `requires` | 树表条目读者收下、写者 `pointer.rs:709` 断言 panic | `pointer.rs:708-711` 写者断言位置条目严格升序，说明写「盘上读来的指针在 recovery::rebuild_version 判过」；改前同样 | 不在这一轮的 diff 里；第五节 E 簇核读者一侧有没有走那一判 |
| T7 容器小树行为 | 成立 | 没打中（腿量的） | 没打中（642453 行发布序列差分）；改后交 None、改前 panic 的输入是任何一层求和越界（9338 组），合法镜像走不到 | — | 成立；None 换 panic 只在走不到的输入上，更安全，收下 |
| T7 容器小树代价 | — | — | 打中：`ancestors_of_containers` 按全部容器数开标记表 | `container_tree.rs:1885-1894` 每调一次 `while marked_below.len() < container_total { marked_below.push(false) }` | 打中：每次发布的代价随盘上容器总数涨，与 `.claude/rules/fs-design.md`「运行时决策路径…代价不许随盘容量增长」相抵；第五节 B 簇改 |
| T8 G7 前提 | 不成立：协议没写「整理发布只在现行内容与最后一个根相同时发」，core 靠屏障满足 | 没攻 | 打中（手推反例：用户改现行、整理发布、崩溃） | `gc_ring_interval_proofs.rs:244`「RunRingStep::CompactionCheckpoint => true,」；第 1303–1307 行 `compaction_publish_is_accepted` 要 `head_logical == roots[last].logical`；core 一侧只有 `mounted_session.rs:5465-5466` 的文档注释 | 打中：两腿独立打到同一处；第五节 D 簇改 |
| T8 Y5 桥 | 条款没说（桥的谓词写了没接） | 没攻 | 打中（推的，两份模型之间的差） | 没现查到底 | 推的，第五节 D 簇连同 G7 一起核 |
| T8 门槛引理 | 两条前提全仓没有调用点 | 没攻 | 没打中 | — | 记账：门槛引理的两条前提由分配器与 floor 的来历另证，`gc_crash_proofs.rs:8` 自己写明是另证的义务；不在这一轮改 |
| T9 journal 预分配 | 成立 | 没打中 | 没打中 | — | 成立 |
| T10 锚点 | 不成立：14 行锚点失效、射程内 8 行 | 打中：8 行 | — | 冻结副本 `crates/mutations.tsv` 第 4387、4404、5894、5896、5901–5904 行原文在冻结副本与主工作区此刻 0 次、HEAD 各 1 次；起因是补 Verus 那几轮把 `write_request_split.rs`、`truncate.rs` 改成调 `partial_write_proofs.rs`、`file_range_proofs.rs`，不在这一轮的 diff 里 | 打中；第五节 A 簇重钉 |
| T10 覆盖 | 不成立：T1、T2、T4、T6/T7 多处改写没有变异行 | — | 打中：容器小树三处新写法没有行；副本上三条变异 core 库测试红 3、2、12 条（腿量的），第四条等价 | `crates/mutations.tsv` 里 `marked_below`、`while low < high`、`noted_so_far`、`insertion_point`、`checked_children_count_sum`、`pieces_of_one_width`、`put_six_byte_unsigned`、`copied_into` 各 0 行 | 打中；第五节 A 簇补 |

## 三、交给下一批的实现员（按文件聚簇）

腿提的改法都只在腿自己的模型上量过或推过，被攻过零轮。

- **A 簇**（`crates/mutations.tsv`，只追加与整行换）：① 第 4387、4404、5894、5896、5901–5904 行按此刻源码重钉锚点（改写之后部分写计划与截断分批的判断落在 `crates/singlefs-replay-prefix/src/partial_write_proofs.rs`、`file_range_proofs.rs` 与 `write_request_split.rs`、`truncate.rs` 里的调用处），逐行重新证红；② 补攻方第二次抽样证过红的三行（模型目录 `research/prompts/m4-close-code-r1-opus-s2-model/tools/mutations.tsv` 的前三行，必红测试名照抄），第四行等价的不收；③ 照正推腿报告「T10」一节逐处给的取样点补 T1、T2、T4、T6 的变异行（编解码的 `pieces_of_one_width`、`put_six_byte_unsigned`、`copied_into`，部分写的胶水与残留读取盘选择，`publish_the_pending_changes` 的冻结处理，「第一版不收」的拒），每行先证红。
- **B 簇**（`crates/singlefs-core/src/container_tree.rs`）：`ancestors_of_containers` 不再按全部容器数开标记表，改成只随这一次点名的容器数与树高长的写法（例：点名的容器排序去重后逐层往上取父），Verus 证明照旧过（形式化轻型核心部分加这一项）；补一条测试钉代价（大容器总数、少量点名时不开整张表），先证红。
- **C 簇**（`crates/singlefs-core/src/mounted_session.rs`，等 GPU 垃圾回收那一路第 54 批打进主工作区之后再派）：先写一条会红的用例：空间被拒、抬 F 的空发布写失败而冻结，看待发的那批是不是既没进冻结的那一次、又没放回；红了就照 D16（发布语义） 已定项 17 让待发并进重发的那一次或放回，绿了在报告写清为什么走不到。
- **D 簇**（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs`、必要时 core 一侧的屏障）：G7 的前提接上：协议里整理那一步（第 244 行）带上 `compaction_publish_is_accepted` 那一判，或在 core 一侧给屏障一条会红的测试与运行期拒（整理发布时现行内容与最后一个根不同就拒），两样里选证明与实现都对得上的那一种；Y5 桥那两条成形条件（持久 F 之上的根数不超过 window、内存 F 至少留一个候选）照攻方第二次抽样报告第 147 行一节在 core 一侧核，核实缺着就补不变式与测试。
- **E 簇**（`crates/singlefs-core/src/pointer.rs` 与树表条目的读者）：核树表条目的读者有没有在照抄写回之前走 `recovery::rebuild_version` 那一判（位置条目严格升序，I-2.5）；没有就让读者照写者的前提拒，补一条会红的用例（读者收下非升序条目、写回 panic）。

## 四、交用户

| 事 | 出处 | 被攻过几轮 |
|---|---|---|
| T3.1：部分写读回那一份读错时，D4（校验和位置） 已定项 4 的字面不分一份与两份，前一轮判决采了 F2 的读法，用户没确认过 | 正推腿报告 T3.1 | 零轮（只有正推腿判「条款没说」） |
| T4.5：两个号的写攒进一次发布，今天拒并（比条款窄） | 正推腿报告 T4.5 | 零轮 |
| 第三节五簇的改法 | 腿报告各节 | 零轮 |

## 五、后续

- 第三节 A、B、D、E 四簇文件互不相交，同一批派；C 簇等 `mounted_session.rs` 那四处打进主工作区之后派。
- 合入后验证与这一批的代码轮，等用户转达的统一任务启动器可用之后跑。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D4（校验和位置） 已定项 3 | 不影响 | 2026-10-30 不受影响：T2 两次没打中，读盘与切分照条款 |
| D4（校验和位置） 已定项 4 | 支撑 | 2026-10-31 改了：T3.1 用户定案写回定案、射程与欠（C946） |
| D8（核心索引结构） 已定项 16 | 支撑 | 2026-10-31 改了：T7 代价读法的用户定案写回射程（只管准入之前） |
| D13（验证路线） 已定项 5 | 不影响 | 2026-10-30 不受影响：这一轮没有格判池级 checker 与实现的依赖 |
| D13（验证路线） 已定项 14 | 不影响 | 2026-10-30 不受影响：T8 的 G7 前提与 Y5 桥是证明与实现之间的接法，条款不改 |
| D13（验证路线） 已定项 18 | 不影响 | 2026-10-30 不受影响：T5 执行体逐行相同 |
| D16（发布语义） 已定项 10 | 支撑 | 2026-10-31 改了：T4.5 用户定案写回射程（两个 inode 不并进一次发布）；实现边界改成现状 |
| D16（发布语义） 已定项 17 | 不影响 | 2026-10-30 不受影响：T4.4 是实现有没有照条款做，C 簇先用用例坐实 |

## 用户定案

- 2026-10-31 T3.1（弹窗）：「这个问题很严重。 两份副本应该按照最新的来。要返回能读的副本。但是这种情况要降低到最低。理论上我们现在不会出现这个情况。」；修复选「计数加搬迁修复」（选项原文：读到一份坏就计数，并把这个单元走已有的搬迁路径整个搬到新槽、两份新写，旧落点照释放前核隔离；走事务层，不开第二条写路径）。写回 D4（校验和位置） 已定项 4。
- 2026-10-31 T4.5（弹窗）：「这一版按照不并行。 如果我们有记错，并发读写时里程碑6的内容。如果当前里程碑中写了并发相关的内容，在不影响实现的情况下 移动到并发相关的里程碑中去」。写回 D16（发布语义） 已定项 10 射程；里程碑四只是计划的并发内容缩成指针挪进里程碑六。
- 2026-10-31 T7 容器小树代价的条款读法（弹窗）：条款读法「管，只管准入之前」；排法「五处加准入现在改，根源先量」。写回 D8（核心索引结构） 已定项 16 射程。
