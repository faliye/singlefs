# m3-b44-code-r2 判决：代码欠账批 4-4 修红批合入的产品代码，第二轮（2026-10-17）

被判的对象：W8、W10、W11 对 `crates/singlefs-core/src/` 下 `mount.rs`、`recovery.rs`、`transaction.rs`、`mounted_session.rs` 的改动（4 个文件 +641/−246，全文 `research/prompts/_m3-b44-code-r2-diff.md`）。正文 `research/prompts/_m3-b44-code-r2-body.md`，冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r2/repo/`。

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 K4 K6 W8 W9 W10 W11 W12 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本 `sha256sum -c --quiet research/prompts/m3-b44-code-r2-snapshot/crates-sha256.txt`；主工作区同法；kb 快照 `kb-sha256.txt` 同法 | 冻结副本 118 份全对上，kb 在两条腿交回时全对上（之后主 agent 改了 `.claude/kb/checks-owed.md` C794 那一行的两处措辞，腿没引那一行）。主工作区有 7 份对不上，都在 `crates/singlefs-checker-tier/src/`（`bin/crash_library_reader.rs`、`crash_amplification.rs`、`crash_identity.rs`、`crash_library.rs`、`crash_state_enumeration.rs`、`judging_units_entries.rs`、`verdict_store.rs`），是别的会话在快照之后改的（它来信说明改的是判定库的结论种类与版本）；被判的四份文件与三份报告引的行都不在这 7 份里，不用倒推 |
| 正推 `research/prompts/m3-b44-code-r2-sonnet-output.md`（sha256 a4b74279…，373 行）、攻方 `research/prompts/m3-b44-code-r2-opus-output.md`（sha256 0a61e09a…，312 行） | `cite-check.py <两份> --root <冻结副本> --background research/prompts/_m3-b44-code-r2-background.md` | 「✓ 核了 95 处引文、210 处裸行号，对上 80 处，没判 225 处」；没对上的 15 处全是正推报告里少写 `crates/` 前缀、路径认不出（`model_comparison.rs`、几份 harness 测试），「对不上」0 处；承重的裸行号（`recovery.rs` 第 2549、3259 行，`mount.rs` 第 6980、6992 行，`pointer.rs` 第 325 行）主 agent 在冻结副本里现读 |
| 攻方第二次抽样 `research/prompts/m3-b44-code-r2-opus-s2-output.md`（sha256 e97d6e2b…，184 行） | 同上 | 「✓ 核了 17 处引文、4 处裸行号，对上 17 处，没判 4 处」 |
| 攻方模型 | 拷进会话草稿目录，经 `run-with-memory-cap.sh 24G` 跑 `tools/rerun.sh`（冻结副本拷贝上编、跑） | 退 0。J1：`panicked at crates/singlefs-core/src/pointer.rs:325:9`，栈 `NodePointer>::write_to` ← `JournalRecord>::to_bytes` ← `rebuild_version_in_the_unit_area_starting_at`，「writable mount: Err」「read-only mount: Ok」。J2：起点 340、960 宽 2 都是「first mount C841-after-writes(persisted=19) … disk_changed=true」，之后两次不带故障各「C841-before-acquisition … disk_changed_by_it=false」。J3：「unchanged start=340 width=2: Ok … failed_reads=2 disk_same_as_fault_free=true」。写账那一行与腿的 `out/run-bytes.log` 逐字相同（16 步合计 6594560 字节）。两段窗口复扫打中 67 次；腿 `out/b2-*.log` 的汇总行相加 2144 次挂载、`C841-after-writes` 70 次，与报告相同 |
| 攻方第二次抽样模型 | 同上，`run-with-memory-cap.sh 6G` | 退 0，「name=compare_window_2 …」之后「same」；起点 339、340、341、343 带「persisted=19 failed=0 changed=true」。腿的 `out/window-2.tsv` 764 行里 `changed=true` 62 行，`out/window-1.tsv` 764 行里 0 行 |
| 正推模型 | 腿没跑量法，`rerun_steps` 只打「无复跑」 | 不用复跑 |

## 二、逐格判

| 格 | 攻方（两次抽样） | 正推 | 判 |
|---|---|---|---|
| J1 | 第一次打中（量过，副本）：所选那一版 journal 记录新根段里树表指针的两条位置条目对调、重算头校验和，可写挂载在 `recovery.rs` 第 2549 行 `record.to_bytes()` 重新编码时 panic 在 `pointer.rs` 第 325 行写者断言上，I-2.5 那一判（第 3259 行 `pointers_of_the_rebuilt_version_ascend_by_device`）排在它之后。第二次没打中（试的是 extent 上段叶里的逆序指针，先被 I-1.1 拒） | 对得上：三棵容器树的两类指针都并进了 | **打中，坐实**（主 agent 复跑与现读）。不是 W8 带进来的：journal 记录新根段那几条指针本来就不在这一判里，且重新编码排在这一判之前。处置：修红批 |
| J2 | 两次都打中（量过，副本）：只进内存那一趟里连着两次读错，`mount.rs` 第 6980 行 `.ok()?` 把取号之后那一串的错吞成 `None`、第 6992 行只认 `MountError::Publish`，取号之前那一判放行；真做读得通，落盘 19 次发布之后交 C841。第一次 2144 次挂载 70 次，第二次窗口宽 2 的 764 个起点 62 个；单次读错两次都 0 次 | 对得上（只拒 C841、别的错留给续做）；读码推出预演第一趟两处与真做的差（不放开扣住位、不读盘核校验和），没造镜像 | **打中，坐实**。按跑前条款第三行判：反例落在条款没写的地方——用户定的「只拒 C841，别的错照旧留给续做」没写「预演自己被别的错打断、没走到会不会撞 C841 那一步」该怎么算。交用户：岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一 |
| J3 | 两次都打中（量过，副本）：同一读错窗口落在没改坏的镜像上，被预演整个吃掉、真做一次读错都没见，盘与无故障那一次逐字节相同；挂一次 1070 次读，764 次落在只进内存那一趟（第二次抽样）。只进内存的盘只增不删，窄盘每步约 0.41 MB（16 步合计 6594560 字节，量的），删 1 TiB 文件约 95 GB（推的） | 对得上（同一段代码、同一份计划与分配器值）；读序数被推进、内存没上界，条款没写 | **打中，坐实**。读被预演吃掉并入岔路一（「读一次两边共用」那一候选是它的改法）；内存没上界交用户：岔路二 |
| J4 | 两次都没打中：查了 8 处按 `MountError::Publish` 分路的调用方，写账空的这一拒每一处都走得通 | 对得上：38 行匹配逐处列过，没有调用方把它当成写了一半 | **没打中**（两次抽样都没打中，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」记）。第二次抽样附带一句：同一成员、同一原因的写账今天可能全空也可能有 19 条，调用方只看原因——不是错，记着 |
| J5 | 第一次：`mount.rs` 第 6941、6945 行「续做报的错就是真发时会报的那一个」被 J2 证伪 | 五处没跟着改：`mount.rs` 第 5081 行 `dry_run_of_the_publishes_after_acquisition` 文档、第 809、811 行 `PublishSequenceFailed` 类型文档、第 5327 行行内注释的理由只讲了一半、预演文档没写扣住位那一差；另五处指向判决、岔路单的句子 | **对不上（文档）**：六处文档改成今天代码做的事，并进修红批。指向判决、岔路单的五处（`mount.rs` 第 5237、5238、5245 行，`transaction.rs` 第 6259 行，`recovery.rs` 第 5135 行）写的是文件加节、不带日期，是规则依据，照 W10 派发时的口径留着 |

## 三、处置

**修红批**（并进下一批实现员，照「写与验分两段」）：

1. J1：可写挂载在重新编码所选那一版 journal 记录之前，按 I-2.5 判那条记录新根段里的每条指针，坏的交类型化错误、两块盘不变。用例照攻方原型那一形（树表指针两条位置条目对调、重算头校验和），另三条新根段指针各一份；变异「拿掉这一判」让它们红。
2. J5：六处文档。
3. 变异轻型第五趟的无效 2 条：`crates/mutations.tsv` 第 444、538 行的替换文写进 W8–W11 改过的 `mount.rs` 之后类型对不上（`expected PublishesRehearsedBeforeAcquisition, found Option<_>`），改成编得过、真会改行为的写法。
4. 岔路一、岔路二用户定了之后的改法。

**交用户**：岔路单 `research/prompts/m3-b44-code-r2-forks.md` 两条岔路。各候选都被攻过零轮；代价数只有窄盘一种几何上量的（764 次读落在只进内存那一趟、每步约 0.41 MB），删大文件的内存是推的。

**推的、没造、进下一轮清单**：正推读码推出预演第一趟两处与真做的差——不放开扣住位（空间判得更紧，随后走只进内存那一趟，方向安全）、不读盘核校验和（空间判得更宽，要同时满足坏盘、池几乎满、退役容器号到顶才可能误拒）；两次攻方抽样都没造这一形。

## 这一轮被判过的文件

- `crates/singlefs-core/src/mount.rs`
- `crates/singlefs-core/src/recovery.rs`
- `crates/singlefs-core/src/transaction.rs`
- `crates/singlefs-core/src/mounted_session.rs`

## 回看决策

不涉及决策：这一轮判的是代码对不对得上用户 2026-10-16 三次定案（岔路单与调度记录）与 I-2.5，没有拿依据推翻或确立 `.claude/kb/decisions.md` 里的分项（回看：2026-10-17 不受影响：岔路一、二交用户，定了之后再看要不要立分项）
