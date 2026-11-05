# m4-close-code-r1 攻方腿（Opus）报告（2026-10-30）

立场：照正文第四节「云端攻方」一行，在冻结副本 `~/.local/share/singlefs/work/m4-close-code-r1/repo/` 的拷贝上造反例。这是这一题第一轮（攻过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱）。前几轮判决：无。

（报告分段落盘；「各格判定一览」在全部格写完后追加在本段之后的第一节，复跑命令与模型文件 sha256 见「复跑」一节。）

## 各格判定一览

| 格 | 判定 | 一句话（依据见各节） |
|---|---|---|
| T1 多 inode 改 size | 没打中（静态，没跑历史） | C145 新断言要上一版 deadlist 有重复 key，恢复路径先拒了（`recovery.rs:4337`）；「供三方核」第三处要的合法中间态造不出来 |
| T2 部分写 core 一侧 | 切分没打中（量的）；读回与落盘次序没打中（静态）；变异打中，归 T10 | 性质格 12000 例 0 不一致；读回的「声明长度按写之前的 size 算」在尾部有洞的文件上也对得上 |
| T3 部分写进模型与 RefFS | 没打中（静态） | RefFS 收空写会拉长文件、core 收不下空写，但翻译交不出空写 |
| T4 发布触发 | 没打中（静态） | 一次调用只取一次时间差；冻结时 `accept` 拒、别的入口先排空，丢待发那一支走不到 |
| T5 块设备信任边界 | 没打中（静态比对） | 三个被验函数的执行体改前改后逐行相同 |
| T6 编解码改写 | 同义没打中（量的，153,800 例 0 不一致）；往返在序号 0、记录宽 0、条目宽 < key 宽三处取不到（量的），合法写路径走不到（推的） | 三处都写在 `requires` 或往返引理前提里、执行时不判，改前同样 |
| T7 容器小树改写 | 没打中（量的，0 不一致） | 形状、追加、inode 叶的分裂借位合并、容量四格；没有一处改成静默截断 |
| T8 只动规格的五份 | 没攻 | 时间给了 T6、T7、T10 |
| T9 journal 预分配 | 没打中（静态 + 差分 journal 格） | 项数先比 64 再 `with_capacity` |
| T10 测试与变异 | 打中（量的） | 变异表 8 行锚点在冻结副本里失效（6 行在 `write_request_split.rs`、2 行在 `truncate.rs`），门禁会整道判红而这几处一行都不跑；新写的编解码与容器小树辅助函数多数没有变异行 |

最脆弱的一条：T10。T2 的部分写切分在冻结副本里被改写成调 replay-prefix 的证明函数（不在这一轮的 diff 里），而钉它的 6 行变异仍按改写之前的原文写，今天不跑。

## 复跑

复跑命令（整条经内存包装，`--work` 给一个新的空目录；它从冻结副本拷 `crates/` 等到 `<work>/new`、从 `git archive 59ce5a865` 取改前那一份到 `<work>/old`，跑锚点核对、差分探针、三个证红变异体，五份输出的 `name=`/`case=`/`anchor` 行与 `out/` 里的逐行比，编译目录与树跑完删掉）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-close-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-close-code-r1/repo --work <新的草稿目录>
```

末行 `name=rerun_attack_m4_close_code_r1 compared=5 differing=0` 是对上；退非 0 是哪一份对不上（`<work>/<名>.diff`）。模板来自 `scripts/agents/three-way-rerun-template.sh`，只换了 `rerun_steps`；模板自带的 `--selftest` 用的是假仓，换过 `rerun_steps` 之后跑不通，没跑。

模型目录 `research/prompts/m4-close-code-r1-opus-model/` 每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，交回前现算）：

```
d0347cc1ed129559a58bb2e60ba72ec037937349c6a53d35bb8467801211349b  ./out/anchor-check.txt
cfa32576074763e9cd75d15d734cc531e4fefff4f66ad7fbbbbba4e2d5bed661  ./out/mut-out-container-tree-grows-only-past-the-fanout.txt
bc039b86a9466e52025fa6c4ab4484db75472a8b9425a1f1c39e7d7b35672aa9  ./out/mut-out-every-byte-is-zero-skips-the-first-byte.txt
0840809914e75f32903b27fb38efe2bdca095ad32b98c139338ee34c2a2ed323  ./out/mut-out-inode-leaf-borrow-rounds-up.txt
0040035cdefae7e0669c00d50e5e511b1e4720a87d20890905ea99ef56418401  ./out/probe-out.txt
5d478eb1c38cda72fbb1c3392ddf96eb6664407c36a3fdb34baf380a54e0c738  ./probe/Cargo.toml
1b791b82b804431635658e3300bbe09180b19836cb8425eeefecd7901ed042dc  ./probe/src/main.rs
bb091392f80ff708541f700faef3b0383dbac2b0ebf8f14f63639fbc1e5fca17  ./probe/src/suite.rs
08fd1f59bbc8eb70cfdc02618268d121ec0a57fe0b157641fb206985c0192a9c  ./probe/src/trees.rs
a0737ee085ad46f99b735410d515af63cb0cd796acf459567188752d80a7a1ac  ./tools/anchor_check.py
c377d02500389e9ca17f4747dee7875f4adb452b1213166dc8c88c912f7e1aaf  ./tools/make_mutant.py
8bafeca1bb9ea7edb1e4e429a15b24b7f06d73d8658e2215e117c6d2e0cc4239  ./tools/rename_old_core.py
07861c2d4c405eaff6f0c4220d3740e0a5c18196a735c0cded68df0e42105d50  ./tools/rerun.sh
2eff89253567c2897e56efc1e63cc53530dc6fef69fd39665c8c4f10c8bf8bfb  ./tools/run-all-as-run-in-the-scratch-directory.sh
```

`tools/run-all-as-run-in-the-scratch-directory.sh` 是我在草稿目录里实际跑的那一份（路径写死在草稿目录上），只留作对照，复跑走 `rerun.sh`。上一条攻方腿留下的 `probe/Cargo.toml` 与 `probe/src/main.rs` 是没写完的骨架，这一条换成了现在这一份。

## T6　编解码与记录格式的执行代码改写

**同义：没打中（量的，差分 153,800 例里行为不同 0 例）。往返：写者没崩、自己的读者却不收的输入有三种（量的），都在写者前提（`requires`）或往返引理前提之外、执行时不判；照我核的调用点，合法写路径走不到（推的）。前提收严那一问：没打中（推的）。**

做法：`probe/` 是一个工作区外的 crate，同一份 `suite.rs`、`trees.rs` 用 `include!` 编进两个模块，一边的 `c` 指冻结副本的 `singlefs-core`，一边指 HEAD（59ce5a865）那一份、包名改成 `singlefs-core-old`（`tools/rename_old_core.py`，两份共用冻结副本的 `singlefs-format` 与 `singlefs-replay-prefix`：格式那一份两版逐字节相同，replay-prefix 两版只差证明模块）。同一份输入喂两边，交出 Debug 串逐字比；崩了记 `PANIC`，两边都崩时只比崩没崩、不比那句话（那句话另数，见下）。

覆盖（量的，`out/probe-out.txt` 的 `case=` 行，「改前那一边收下的例数」）：码 2 1013、码 3 967、码 1 998、根槽 1080、journal 681、截断意图条目 1109、树表 561 与带销毁标记 570、映射条目 2785、inode 内部条目 192、extent 记录 225、墓碑 1620、谱系 1640、退役 1582、空间账 1550、deadlist 1609、inode 记录 288（各 3000 例）。写者 9 格各 3000 例，改前那一边崩的例数（写者前提之外的输入故意占一部分）：journal 1245、根槽 1302、码 3 453、码 2 1734、码 1 410、指针与映射 key 1205、截断意图根 1369、`ByteWriter` 2695。

结果（量的，同一份文件）：

```
name=codec_differential cases=153800 mismatches=0
per_case_mismatches={}
```

两边都崩、只是那句话不同的例数（不算行为不同；量的，`panic_message_only_differences` 一行）：`wide_checksum_zeroed` 2246、`pinned_ring_slot` 1888、`w_writer_ops` 1546、`tree_shape` 906、`w_records` 731、`reader_ops` 489、`node_pointer_judgement_shadow` 244、`mapping_sort_key` 236、`w_index` 218、`w_journal` 254 等。抽看过的几类：slice 越界（改前 `range end index …`）换成下标越界（改后 `index out of bounds`，`ByteWriter::put_little_endian` 逐字节写）；`assert_eq!` 换成 `assert!` 加同一句话。没有一类是「改前崩、改后不崩」或反过来。

往返（量的，改后那一边，`new_side_round_trip_breaks` 与 `ROUND_TRIP` 行）：

| 写者格 | 写者没崩而自己的读者不收的例数 | 抽看的样子 | 前提里写着的那一句 |
|---|---|---|---|
| `w_journal` | 393（全部是本次发布内序号为 0：`of_which_at_journal_ordinal_zero=393`） | `("len=4096 fnv=32a30bb2fcf150de", false, false) \| ordinal=0` | 往返引理另要序号不为 0：「self.ordinal_within_publish.0 != 0,」（`crates/singlefs-core/src/journal.rs:1363`），文档那一句「读者拒「本次发布内序号为 0」，所以这里要序号不为 0；写者不判它。」（`crates/singlefs-core/src/journal.rs:1359`） |
| `w_packed` | 652（抽看的前 3 例都是 `PackedUnitRecordWidthZeroWithRecords`，其余没逐例归类） | 记录宽 0、带记录 | 写者前提「&&& record_width > 0」（`crates/singlefs-core/src/unit.rs:691`） |
| `w_index` | 118（抽看的前 3 例都是 `IndexNodeEntryWidthBelowTheKeyWidth`，零条目） | 条目宽小于 key 宽、一条条目都没有 | 写者前提「&&& key_width <= entry_width」（`crates/singlefs-core/src/unit.rs:931`） |

三种在改前那一份上一模一样（差分 0 不一致），不是这一轮引进的；改写把它们写成了 `requires` 或引理前提，可平常的 `cargo build` 擦掉 `requires`，执行时照旧不判（码 2 那一种：改前只在逐条条目上断言「条目里 key 一律是前 key 宽 个字节」，零条目时不断言，改后同样）。

前提收严在执行调用点成不成立（推的，读了全部调用点的实参）：`build_packed_unit` 在 core 执行代码里的 4 个调用点（`make_filesystem.rs`、`transaction.rs` 三处；`instance_table.rs`、`region_scan.rs`、`mount.rs` 里的几处在 `#[cfg(test)]` 模块里，不算）记录宽都是格式常量（`u16::try_from(INODE_RECORD_BYTES).expect("140"),`（`crates/singlefs-core/src/transaction.rs:13739`）、`u16::try_from(to_build.record_width_in_bytes).expect("记录宽是 40、24 或 34"),`（`crates/singlefs-core/src/transaction.rs:13934`）、实例表两处 88），没有 0；`build_index_node` 在 core 执行代码里的 12 个调用点 key 宽与条目宽都是成对的格式常量（inode 树 8/120、容器树 8/120、deleted_inodes 8/8、截断意图 key/44、树表 8/200、extent 树 24/112、110、113、分配记录树 key 10、码 2 通用树按 `internal_entry_width_in_bytes(key_width)` 算的条目宽），没有一处条目宽小于 key 宽。journal 记录的序号只经 `of_record_at_offset`（偏移 + 1）与常量 `FIRST`（「pub const FIRST: Self = Self(1);」（`crates/singlefs-core/src/journal.rs:138`））造，产品路径上没有 0。推翻条件：找到一个调用点把运行时算的宽交给这两个写者，或一条路径把 `JournalRecordOrdinalWithinPublish(0)` 交给 `to_bytes`。

四句：不分辨臂（这一格没有臂）；写者在写的那一刻看得到序号、记录宽、条目宽（三样都是参数），能当场拒，只是没写成运行时判；满足的是 T6 第二问「往返契约在格式常量的边界上取不取得到」——在 0 这个边界上取不到，契约（引理）自己把 0 除掉了；改法：三处各加一句运行时 `assert!`（或交类型化错误），只在我的模型上量过「现在不拒」，加了之后的样子没实现没跑，被攻过零轮。

## T7　容器小树的执行代码改写

**没打中（量的，差分里 0 例行为不同）。**

| 格 | 例数 | 走到的（改前那一边） |
|---|---|---|
| `tree_shape`：`from_children_of_each_node_at_each_level` 在乱造与合法形状上收拒，`root_level`、`internal_levels`、`containers`、`nodes_at_level`、`positions_in_bump_order`、`root_position`、`has_a_node_at`、`is_the_root`、`children_of`（含越界位置）、`ancestors_of_containers`（含越界容器号）、`after_a_container_is_opened_at_the_right_end` 连开 1–30 次（扇出 0–2031） | 3000 | 1857 例里至少一处崩（越界位置、扇出小于 2 这类前提之外的问法），两边崩在同样的问法上 |
| `tree_append`：`append_records_to_the_open_container` 连续 1–12 次发布，每片 0–8 条，扇出 2–5 或格式值，四分之一从号是 u64 最大值附近的那一片起 | 3000 | 1000 例里至少一次长一层（`grew=true`）；237 例崩（每片 0 条那一种前提之外的输入） |
| `inode_tree`：`write_records_into_leaf_containers` 写 0–235 条（含改已有的、乱序的），`remove_record_from_leaf_containers` 一串最多 150 条 | 500 | 245 例里至少一次并掉一片（退役叶）；0 例崩 |
| `tree_capacities`：三种小树的容量与测试旋钮、四种叶序号的 `of_position` | 3000 | 572 例崩（序号装不进 u32 那一种） |

「改成 `assert!`、`unreachable!` 或静默截断」那一问（量的 + 静态）：改后执行代码里新出现、改前没有的断言消息，容器小树一簇只有两条——「问祖先的容器 {container} 在这一版里（{container_total} 片）」（改前同一个断言换了措辞）与「照抄的容器树节点……在上一版里有」（同上）；inode 树一条（借位那条，改前同一断言换了续行写法）；三种小树 0 条。探针在这些断言的前提之外问过（越界容器号、越界位置），两边都崩，没有一处改成静默交一个值。

没打中的余量：探针的形状都是自己造的，没有从盘上读一份合法镜像喂进去；inode 树的 245 例「并掉一片」没有区分走的是借位还是合并。

## T1　多 inode 改 size

**没打中（静态读码，没跑历史）。** 攻了三样：

1. C145 第三条那条新断言（「released_entry.is_some(),」（`crates/singlefs-core/src/transaction.rs:6546`））能不能在合法历史里触发。它红的唯一办法是 `deadlist_entries_the_destruction_of_a_snapshot_releases` 交出两条 key 相同的 deadlist 条目（第一次 `remove` 拿走、第二次拿到 `None`）。那个函数只从 `version.deadlist_entries` 里 `filter` 后 `cloned`（同文件 `deadlist_entries_the_destruction_of_a_snapshot_releases` 函数体），所以要上一版的 deadlist 里本来就有重复 key。两个来源都堵着：会话内的 deadlist 每一版从 `BTreeMap<CodeTwoTreeKey, DeadlistEntry>` 收出来（同一函数里 `entries_after` 那一段），key 不重；挂载读进来的那一份，恢复时「|| deadlist_mapping_keys.len() != deadlist_entries.len()」（`crates/singlefs-core/src/recovery.rs:4337`）把「两条 deadlist 条目同一把映射 key」拒成 `UnitMalformed`，在交给发布路径之前。坏盘输入也走不到这条断言。推翻条件：找到一条不经 `mapping_entries_of_the_version` 就把盘上 deadlist 交给发布路径的挂载入口。
2. 「供三方核」第三处（冷恢复对 nlink = 0 的记录同判「有记录、没有条目、size > 0」）：要打中得造出一个合法的中间态——nlink = 0、size > 0、上段没有它的条目。读了别的 inode 在 unlink 批里的那一支：删除的一批把 size 改成「剩下单元数 × 净荷」、标签换回 1、2（`plan_the_files_other_than_the_first_after_this_publish` 的 `DeletionBatch` 臂，冻结副本 `crates/singlefs-core/src/transaction.rs:7577` 起），剩下单元数大于 0 时条目留着；最后一批（`FinalDeletionBatch` 臂）记录与条目一起去。我在代码里没找到「条目先去、记录后去」的一步，所以造不出这个态（推的）。没扫用户动作的全部次序（unlink 走到一半被空间拒、再改 size 这类），这一条的否定只是一次抽样。
3. 第一个文件与别的 inode 的单元数来源：`publish_size_change` 只经 `data_units_of_the_file_in_the_version(previous, inode)` 取单元数（冻结副本 `crates/singlefs-core/src/transaction.rs:11164` 起的函数体），第一个文件与别的文件在这一处不分叉。没再往下核 `publish_deletion_step` 里两条路径各自的上段计划。

没做：S2/S3/S4/S6 的跨 inode 判法一致性没有造历史去跑（照第 3b 步要在内存稀疏盘上造历史再枚举崩溃态，这一条腿把时间给了 T6、T7 的差分）；「供三方核」第一、二、四处只读到上面那几行，没给推得出、推不出的判断（那是正推腿的格）。

## T2　部分写 core 一侧

**切分：没打中（量的，改后那一份上的性质格）；读回：没打中（静态）；变异：打中，见 T10。**

1. 切分（`write_request_split.rs`）：差分探针里 `partial_write_oracle` 一格按字节的理想读法逐单元算「写之后的 size、单元数、第一个与最后一个写到的单元、重写的那一段、每一片怎么装（五种）、写之后的声明长度、这次的字节落在净荷的哪一段」，与 `PartialWriteOfAFile` 的对应方法逐项比。取样：写之前的 size 取 0、整单元倍数、整单元倍数 ±1–2、随机，单元数取 ⌈size ÷ P⌉ 或更少（尾部有洞），偏移取单元边界、边界 − 1、size 本身与 size ±2、随机，长度取 1–3、P、P − 1、几倍 P ± 1、随机（P = 32634）。结果见「差分探针的原样输出」一节的 `case=partial_write_oracle` 那一行。理想读法把「盖满」定成罩住整个净荷容量（类型文档的那一句），把「写落在最后一个存在的单元之后」时的起点定成「它不满就从它起，否则从写之前的单元数起」——这两条是照 `crates/singlefs-core/src/write_request_split.rs` 的类型文档写的，不是从判决原文独立推的，所以这一格只核「实现与它自己的文档一致」，不核文档与 R4、R5 一致。
2. 一份读错时的判法（`read_back_the_old_payload_of_a_data_unit`）：两条位置提示逐条读，整单元 CRC-32C 等于那条位置条目里的才算读到（`find_map` 加 `filter`），两条都不算再按上一版中央映射的落点读；读到之后对查找路径核九样，声明长度按写之前的 size 算（「if u64::from(header.declared_length) != declared_length_within_a_size(unit, record_before.size)」（`crates/singlefs-core/src/transaction.rs:11452`））。我想打的形状是「尾部有洞的文件，最后一个存在的单元盘上声明长度小于按 size 算的那个」，那会让合法镜像上的部分写被拒成 `DisagreesWithTheLookupPath`。造不出来：变长时 `tail_data_unit_rewrite_of_an_extension` 重写尾单元到新的声明长度（`publish_size_change` 里那一支），尾部有洞时最后一个存在的单元按公式就是满的 P。推的，没跑历史。
3. 新错误成员在落盘之前交回：读回在 `publish_size_change` 里先于 `publish_deletion_step`（读回失败直接 `?` 返回 `OldDataUnitToRewriteNotReadBack`），读到这一处为止没有分配与写。部分写入口 `publish_partial_write` 的同一顺序没逐行核。

## T4　发布触发

**没打中（静态读码）。**

- 触发判据只在不变的输入上分支：时间是调用方交进来的读数（`CallerMonotonicClockReading`），一次调用里只取一次差（「Some(last) => now.0.saturating_sub(last.0),」（`crates/singlefs-core/src/mounted_session.rs:5115`）），`publish_trigger_cause` 是纯函数；脏量是待发里累计的 journal 字节，进门之后才加。读数倒退按 0 算（饱和减），`note_a_publish` 记 `max(before, now)`，倒退的读数不会把「上次发布」往回拨。没找到一次调用里先后两次取读数的地方。
- 报错之后并进冻结：`publish_the_pending_changes` 出错时「if self.session.allocator.frozen_publish().is_none() {」（`crates/singlefs-core/src/mounted_session.rs:5147`）才把待发放回，冻结了就丢掉待发（它已在冻结的那一次里）。我想打的形状是「冻结早于这一次、这一次又在读写之前被拒」，那会把这一批没进冻结的待发丢掉。造不出来：`accept` 一进门就查冻结（「if let Some(frozen) = self.session.allocator.frozen_publish() {」（`crates/singlefs-core/src/mounted_session.rs:5175`）），冻结着不收新的；屏障、单发、后台整理三条路径都先排空待发再发自己那一次，所以冻结出现时待发已经是空的。推的；没核后台整理那条路径排空之后、跑整理之前有没有别的入口往待发里加。
- 两个号攒进一次发布（R:66 那一问）：没攻，留给正推腿。

## T5　块设备信任边界

**没打中（静态比对改前改后）。** `check_aligned_and_in_range`、`length_of`、`zero_fill_chunk_length` 三个函数的执行体改前（HEAD 的 `block_device.rs`）与改后（冻结副本）逐行相同，只加了 `ensures`；`write_zeroes_in_chunks` 的执行体相同，闭包约束从 `FnMut` 收成 `Fn`。物理块大小为 0、`zeros` 为空而长度大于 0 两种情形两版同样（前者只有偏移与长度都是 0 才算对齐，后者两版都死循环；改后把后者写进 `requires`，执行时照旧不判）。差分测试罩不罩得住信任清单登记的那几样没核。

## T9　journal 读者预分配

**没打中（静态）。** 项数从盘上读出（「let named_count = usize::try_from(reader.get_u32()).expect("点名项数");」（`crates/singlefs-core/src/journal.rs:878`）），先过「if named_count > (record_bytes - header_bytes) / named_entry_bytes {」（`crates/singlefs-core/src/journal.rs:948`）这一判（496 字节头、56 字节一项，上限 64）交 `None`，再过点名项区的 CRC，到「let mut named: Vec<NamedUnit> = Vec::with_capacity(named_count);」（`crates/singlefs-core/src/journal.rs:971`）时 `named_count ≤ 64`，坏盘给一个大项数分不到一大块。差分探针的 `journal` 格把改前改后两个读者放在同一批输入上比（含改了项数字段、重封头校验和的记录），结果见差分一节。循环不变式与 vstd 的 `with_capacity` 规格没核（没起 Verus）。

## T10　测试与变异钉得住这一段的执行代码改动吗

**打中（量的，冻结副本上）：变异表里 8 行的原文在今天的实现里一处都找不到，锚点失效；其中 6 行落在部分写切分（T2）的 `write_request_split.rs` 上（这份文件在表里共 10 行）。**

做法：`tools/anchor_check.py` 把 `crates/mutations.tsv` 里文件落在这一轮射程（core 的编解码、容器小树、部分写、截断、发布等 21 份文件）上的每一行取出来，原文的 `\n` 换成换行，数它在冻结副本那份文件里命中几次，再在 HEAD（59ce5a865）那一份里数一次。命令与原样输出（输出里的长名在 260 列处截断，是 `cut` 截的，原样文件 `out/anchor-check.txt` 不截）：

```
python3 research/prompts/m4-close-code-r1-opus-model/tools/anchor_check.py ~/.local/share/singlefs/work/m4-close-code-r1/repo <HEAD 那一份的根>
name=mutation_anchor_check checked=1059 anchors_not_exactly_once=8
```

| mutations.tsv 行 | 文件 | 冻结副本命中 | HEAD 命中 | 点名的测试 |
|---|---|---|---|---|
| 4387 | truncate.rs | 0 | 1 | `truncating_a_three_unit_file_into_unit_zero_in_batches_of_one_publishes_the_intent_two_batches_and_replaces_only_unit_zero_at_the_intent` |
| 4404 | truncate.rs | 0 | 1 | `the_target_size_limit_and_the_three_branches_of_a_size_change` |
| 5894 | write_request_split.rs | 0 | 1 | `a_write_past_a_gap_zeroes_the_gap_units_without_reading_them` |
| 5896 | write_request_split.rs | 0 | 1 | `a_write_into_the_hole_after_an_extension_fills_the_gap_and_reads_only_the_residue` |
| 5901 | write_request_split.rs | 0 | 1 | `a_write_inside_existing_units_rewrites_every_unit_from_the_offset_and_reads_back_what_it_does_not_cover` |
| 5902 | write_request_split.rs | 0 | 1 | `a_write_into_an_inode_without_units_reads_the_residue_of_the_units_it_writes_and_zeroes_the_gap` |
| 5903 | write_request_split.rs | 0 | 1 | `a_write_after_the_last_existing_unit_rewrites_that_unit_only_when_it_is_not_full` |
| 5904 | write_request_split.rs | 0 | 1 | `the_end_of_a_write_past_u64_is_refused_and_a_write_across_a_boundary_lands_in_two_units` |

来历（量的）：这 6 行部分写变异的原文是这一轮 diff 里 m4-partial-write-writer 加的行（`_m4-close-code-r1-diff.md` 第 3768、3785、3819、3821、3837 行是其中五行的原文，`grep -nF` 现查）；冻结副本的 `write_request_split.rs` 把这一段改写成调 `singlefs_replay_prefix::partial_write_proofs::classify_partial_write_unit`、`file_range_proofs::checked_bytes_in_indexed_unit` 等（冻结副本 `crates/singlefs-core/src/write_request_split.rs:355` 调 `classify_partial_write_unit`、`crates/singlefs-core/src/write_request_split.rs:377` 调 `checked_bytes_in_indexed_unit`），这一改写与 `crates/singlefs-replay-prefix/src/partial_write_proofs.rs`、`file_range_proofs.rs` 两份新文件**不在这一轮的 diff 里**（diff 里 `partial_write_proofs`、`file_range_proofs` 命中 0 次），是主工作区里别的没提交的改动；`git diff --stat HEAD` 现查这份文件 176 行改动。主工作区当时（2026-10-30 现跑同一个脚本）仍是 `anchors_not_exactly_once=8`、8 行行号相同；表里的行号是冻结副本那一份 `crates/mutations.tsv` 的，主工作区那一份交回前现查已与冻结副本不同（sha256 前 16 位 `672da69a3dd4a43b` 对 `6ddbcf958a169aae`，别的会话在改），主 agent 引行号时以冻结副本为准。

四句：
- 分不分辨臂：这一格没有臂；它说的是「T2 的切分在今天的实现上没有一行活的变异」，与哪种改法无关。
- 被判系统看不看得到：门禁 checker-tier-crates-mutation-replay 的规矩（表头第 2 行「原文在文件里必须恰好命中一次（不是的那一行记锚点失效、不跑，别的行照判，整道判红）」）看得到，它会整道判红；但这一轮正文第二节记的变异轻型只跑了 journal 预分配那一趟（「抓到 302 条、没红 0 条、无效 0 条」），没跑到这 8 行。
- 满足的分句：正文第三节 T10「把改动改回改前、或改成另一种错法……有没有一行变异会红」——这 6 行钉的是 R5 空隙不读残留、写进洞里不重写满的最后一个单元、盖满不读回、写之后的单元数、最后一个单元满不满、字节落点这几处，今天一行都不会红（不跑）；另外 4 行没核它们钉的是哪几处。
- 改法：每行按改写后的代码重取锚点（候选取样点：`classify_partial_write_unit` 的五个臂在 core 一侧的映射 `crates/singlefs-core/src/write_request_split.rs:362` 到 366 行，`first_rewritten_unit` 的 `tail_is_partial` 那一判 `crates/singlefs-core/src/write_request_split.rs:313`）。只在我的模型上量过锚点、被攻过零轮；重取锚点之后那几条测试红不红没跑（推的）。

推翻条件：主 agent 在冻结副本上跑 `checker-tier-crates-mutation-replay --quick` 而这 8 行不报锚点失效。

另：射程内新写的 codec 与容器小树辅助函数，变异表原文里点名它们的行数（量的，`awk -F'\t' 'index($3,名)>0'` 数冻结副本的 `crates/mutations.tsv`）：`every_byte_is_zero` 5、`copied_into_an_array` 0、`copied_into_a_vector` 0、`put_little_endian` 0、`get_little_endian` 0、`copy_bytes_into` 0（另有一行以 `copy_bytes_into` 函数体为原文，第 6051 行）、`children_count_sum_of_the_first` 0、`checked_children_count_sum` 0、`every_node_has_a_child` 0、`each_level_counts_the_nodes_below` 0、`copy_of_one_level` 0。没有变异行的这几处，差分探针（T6、T7 两节）是现成的取样点：改坏其中一处，探针的对应格会出不一致（推的，没对每一处做变异实测）。

## T3　部分写进模型与 RefFS

**没打中（静态）。** 攻的形状是「RefFS 收下一次实现做不出来的按偏移写」：RefFS 的 `WriteAtOffset` 对空的 `bytes` 也照收，偏移大于文件长度时先把文件补零拉长到偏移（「let mut written_over = bytes.clone();」（`crates/singlefs-reference-filesystem/src/lib.rs:680`）之后那一段 `resize(end, 0)`，`end = offset + 0`），而 core 一侧空写交不进来（「(!bytes.is_empty()).then_some(PartialWriteBytes { bytes })」（`crates/singlefs-core/src/transaction.rs:11322`））。两边在空写上判得不一样，但翻译只在执行器报了这次写的偏移与长度之后才交 `WriteAtOffset`，字节按那个长度现算（「bytes: FileContent(choice.bytes_of_length(requested.length_in_bytes)),」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:546`）），实现收不下空写，翻译就交不出空写，允许集合里不会多出这一态（推的；翻译那份文件不在这一轮射程里，只读了这一支）。F2 的读法（「两份都读不回才失败」）是不是 D4（校验和位置） 已定项 4 推得出，是正推腿的格，这里只记：已定项 4 的定案句「旧单元读不回来、或读回来 MAC 验不过时，那个部分写失败并返错」（`.claude/kb/decisions/04-校验和位置.md:71`）说的是「旧单元」读不回，没说两份副本各自怎么算，攻不出它与「一份读错仍成功」的冲突。

## T8　只动规格与证明的五份

**没攻。** 这一条腿的时间给了 T6、T7、T10 的执行代码与变异表。没有读 `gc_ring_interval_proofs.rs`、`compaction_plan_proofs.rs`、`gc_crash_proofs.rs`、`verified_model.rs` 的 `requires` / `ensures`，也没核 Y5 那一座桥。只顺带记一处与这一格相邻的观测（量的，差分探针输出）：改写后的执行代码里，`requires` 在平常的 `cargo build` 下被擦掉，执行时只剩写成 `assert!` 的那几句；探针在越过 `requires` 的输入上（例如 `ByteWriter` 写过缓冲末尾、`children_of` 问不在形状里的位置、`ancestors_of_containers` 问越界的容器号）比了改前改后，两边都崩、崩的地方相同，只是 panic 的那句话不同（`panic_message_only_differences` 那一行），没有一处从「崩」变成「静默截断」。

## 证红：探针自己会不会红

照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@校验路径本身也要证明它会红`：在改后那一份 core 的拷贝上各改一处（`tools/make_mutant.py`），探针连这份变异体、对改前那一份再跑一次（每格 1000 例，inode 树 166 例）。三个都红（量的，`out/mut-out-*.txt` 末行）：

| 变异体 | 改哪一行（冻结副本行号） | 探针末行 | 红在哪几格 |
|---|---|---|---|
| `every-byte-is-zero-skips-the-first-byte` | `crates/singlefs-core/src/bytes.rs:210` 的 `let mut checked_count: usize = 0;` 改成 `= 1`（全零判法漏看第一个字节） | `name=codec_differential cases=51466 mismatches=66` | 根槽、journal、截断意图条目、树表、inode 记录、extent 记录、三种指针读法等 14 格 |
| `container-tree-grows-only-past-the-fanout` | `crates/singlefs-core/src/container_tree.rs:2138` 的 `>=` 改成 `>`（最高那一层满了不长） | `name=codec_differential cases=51466 mismatches=617` | `tree_append` 356、`tree_shape` 261 |
| `inode-leaf-borrow-rounds-up` | `crates/singlefs-core/src/inode_tree.rs:2262` 的 `difference / 2` 改成 `(difference + 1) / 2`（借位向上取整） | `name=codec_differential cases=51466 mismatches=26` | `inode_tree` 26 |

这三处在变异表里都没有点名的行（`every_byte_is_zero` 有 5 行原文提到它，都是调用它的那几句，不是它的函数体；后两处的原文 0 行，按 `awk -F'\t' 'index($3,原文)>0'` 现数）。仓里的测试在这三个变异体上红不红没跑（照第 3b 与共用约束，这一条腿不在仓的测试上跑变异）。

## 差分探针的原样输出

`out/probe-out.txt` 里的 `case=`、`per_case`、`name=` 行（每行在 230 列处截断，量过没有一行长到 230 列，即原样）：

```
case=unit_index widths=[] examples=3000 old_side_some_or_ok=1013 old_side_panics=0
case=unit_packed widths=[] examples=3000 old_side_some_or_ok=967 old_side_panics=0
case=unit_data widths=[] examples=3000 old_side_some_or_ok=998 old_side_panics=0
case=unit_declared_length widths=[] examples=3000 old_side_some_or_ok=0 old_side_panics=1
case=unit_payload widths=[134, 135, 136, 137, 138, 139, 140, 141] examples=3000 old_side_some_or_ok=449 old_side_panics=0
case=root_slot widths=[] examples=3000 old_side_some_or_ok=1080 old_side_panics=12
case=inode_record widths=[140] examples=3000 old_side_some_or_ok=288 old_side_panics=0
case=tombstone widths=[40] examples=3000 old_side_some_or_ok=1620 old_side_panics=0
case=lineage widths=[24] examples=3000 old_side_some_or_ok=1640 old_side_panics=0
case=retirement widths=[34] examples=3000 old_side_some_or_ok=1582 old_side_panics=0
case=accounting widths=[34] examples=3000 old_side_some_or_ok=1550 old_side_panics=0
case=deadlist widths=[43] examples=3000 old_side_some_or_ok=1609 old_side_panics=0
case=tree_table widths=[] examples=3000 old_side_some_or_ok=561 old_side_panics=21
case=tree_table_mark widths=[] examples=3000 old_side_some_or_ok=570 old_side_panics=25
case=root_pointer_bytes widths=[] examples=3000 old_side_some_or_ok=0 old_side_panics=87
case=inode_internal widths=[120] examples=3000 old_side_some_or_ok=192 old_side_panics=0
case=inode_internal_refusal widths=[120] examples=3000 old_side_some_or_ok=55 old_side_panics=119
case=extent_record widths=[112] examples=3000 old_side_some_or_ok=225 old_side_panics=0
case=mapping_entry widths=[55, 56, 57, 58, 59, 60, 61, 62] examples=3000 old_side_some_or_ok=2785 old_side_panics=0
case=mapping_sort_key widths=[] examples=3000 old_side_some_or_ok=0 old_side_panics=252
case=birth_txg_of_mapping_key widths=[27] examples=3000 old_side_some_or_ok=1576 old_side_panics=0
case=logged_entry widths=[] examples=3000 old_side_some_or_ok=1109 old_side_panics=0
case=journal widths=[] examples=3000 old_side_some_or_ok=681 old_side_panics=5
case=back_chain widths=[] examples=3000 old_side_some_or_ok=0 old_side_panics=46
case=wide_checksum_holds widths=[] examples=3000 old_side_some_or_ok=0 old_side_panics=2234
case=wide_checksum_zeroed widths=[] examples=3000 old_side_some_or_ok=0 old_side_panics=2260
case=node_pointer_judging widths=[86, 87, 88, 89, 90, 91, 92, 93] examples=3000 old_side_some_or_ok=292 old_side_panics=227
case=data_pointer_judging widths=[88, 89, 90, 91, 92, 93, 94, 95] examples=3000 old_side_some_or_ok=338 old_side_panics=217
case=head_judging widths=[50, 51, 52, 53, 54, 55, 56, 57] examples=3000 old_side_some_or_ok=311 old_side_panics=197
case=node_pointer_judgement_shadow widths=[86, 87, 88, 89, 90, 91, 92, 93] examples=3000 old_side_some_or_ok=2628 old_side_panics=372
case=node_pointer_judgement_first widths=[86, 87, 88, 89, 90, 91, 92, 93] examples=3000 old_side_some_or_ok=316 old_side_panics=222
case=location_entries widths=[28, 29, 30, 31, 32, 33, 34, 35] examples=3000 old_side_some_or_ok=1597 old_side_panics=119
case=reader_ops widths=[] examples=3000 old_side_some_or_ok=0 old_side_panics=515
case=w_journal examples=3000 old_side_panics=1245 new_side_round_trip_breaks=393 of_which_at_journal_ordinal_zero=393
case=w_root examples=3000 old_side_panics=1302 new_side_round_trip_breaks=0 of_which_at_journal_ordinal_zero=0
case=w_packed examples=3000 old_side_panics=453 new_side_round_trip_breaks=652 of_which_at_journal_ordinal_zero=0
case=w_index examples=3000 old_side_panics=1734 new_side_round_trip_breaks=118 of_which_at_journal_ordinal_zero=0
case=w_index_capacity examples=3000 old_side_panics=7 new_side_round_trip_breaks=0 of_which_at_journal_ordinal_zero=0
case=w_data examples=3000 old_side_panics=410 new_side_round_trip_breaks=0 of_which_at_journal_ordinal_zero=0
case=w_records examples=3000 old_side_panics=1205 new_side_round_trip_breaks=0 of_which_at_journal_ordinal_zero=0
case=w_logged_root examples=3000 old_side_panics=1369 new_side_round_trip_breaks=0 of_which_at_journal_ordinal_zero=0
case=w_writer_ops examples=3000 old_side_panics=2695 new_side_round_trip_breaks=0 of_which_at_journal_ordinal_zero=0
case=tree_shape examples=3000 old_side_examples_with_a_panic=1857 old_side_examples_with_a_merge_or_a_new_level=0
case=tree_append examples=3000 old_side_examples_with_a_panic=237 old_side_examples_with_a_merge_or_a_new_level=1000
case=inode_tree examples=500 old_side_examples_with_a_panic=0 old_side_examples_with_a_merge_or_a_new_level=245
case=tree_capacities examples=3000 old_side_examples_with_a_panic=572 old_side_examples_with_a_merge_or_a_new_level=0
case=partial_write_oracle examples=12000 mismatches=0
per_case_mismatches={}
name=codec_differential cases=153800 mismatches=0
```

## 没打中的形状（与取样范围）

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| T6 | 改前改后的读者在同一段字节上交的东西不同（收/拒、拒因、读出的值、崩不崩）；写者在同一份值上写出的字节不同；写者写出、自己的读者不收 | 33 个解码格各 3000 例，9 个写者格各 3000 例（见差分一节原样输出）；输入一半以上取自改前那一边写者写出的合法单元或记录再改 0–3 个字节、四分之三重封校验和，其余是三种疏密的随机字节，长度偏到读者认得的宽 ±1 |
| T7 | 形状的构造在坏输入上收拒不同、几个查询与「最右开一片」的结果不同、追加跨片与长高时重写清单与来历不同、inode 叶的写入分裂与删除借位合并结果不同 | 形状 3000 例（三分之一是乱造的、三分之二是自下而上造的合法形状再偶尔改坏一格），追加 3000 例（每例最多 12 次发布、每片 0–8 条、扇出 2–5 或格式值、四分之一从号接近 u64 顶的那一片起），inode 树 500 例（每例最多 25 次发布，写 0–235 条、删一串最多 150 条），容量 3000 例 |
| T2 | 部分写切分与按字节的理想读法不一致 | 12000 例，取样见 T2 一节 |
| T1 | C145 断言在合法历史上触发；nlink = 0、size > 0、没有条目的合法中间态 | 只读码，没跑历史 |
| T4 | 一次调用里先后两次取时间读数；冻结早于这一次而丢掉待发 | 只读码 |
| T5 | 三个被验的执行函数改前改后不同 | 逐行比对 |
| T9 | 大项数先分配 | 只读码；差分探针的 journal 格 |
| T3 | RefFS 收下空写 | 只读码 |

「没打中」这几格各只抽了一次样（照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`，要用它支撑结论得再抽一次）。

## 这条腿自己的限度

- 差分探针拿改前（HEAD 59ce5a865）那一份当对照。HEAD 不全是「14 份补丁之前」：`write_request_split.rs` 在 HEAD 里已经有部分写那一套（T10 一节量的），所以 T2 的切分不能用差分判，只用了性质格；编解码与容器小树的文件在 HEAD 里还没进 `verus!`（`bytes.rs`、`container_tree.rs` 等的 HEAD 版没有 `requires`），差分对它们有意义。
- 冻结副本里还有不归这一轮的改动（正文第二节那一条），差分出的不一致要逐处归属；这一次没有不一致，所以没归属的活。
- 两边都崩时只比「崩没崩」，不比 panic 的那句话；所以「两边都崩但崩在不同的那一步」分不出来（例：改前是 slice 越界，改后是下标越界）。
- 往返只在改后那一边数了「写者没崩、自己的读者不收或收回来的不同」，前提（`requires`）是不是条款的那一层没判（那是 T8）。
- 探针的值大多从随机字节经两边相同的 `read_from` 读出来，位置条目照 I-2.5 修成合法的占八分之七；没有按格式常量逐一扫边界值（例：每一种记录宽的 ±1 只靠「读者认得的宽」那一步自动找到）。
- T1、T2 读回、T4、T5、T9、T3 只读了码，没有造历史；第 3b 步要求的「内存稀疏盘上造历史、每段历史的崩溃态一个不落地枚举并跑 checker」这一条腿没做（崩溃态 0 个，原型没调枚举函数）。
- 证红：三个变异体（见「证红」一节）只证了探针会红，没证仓里的测试与变异表会红；仓里的测试在变异体上红不红没跑。

## 复跑的结果（交回前在新 shell 里整份跑的那一次）

```
bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-close-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-code-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/rerun-1
rc=0
name=mutant_made mutant=every-byte-is-zero-skips-the-first-byte file=crates/singlefs-core/src/bytes.rs
name=mutant_made mutant=container-tree-grows-only-past-the-fanout file=crates/singlefs-core/src/container_tree.rs
name=mutant_made mutant=inode-leaf-borrow-rounds-up file=crates/singlefs-core/src/inode_tree.rs
name=rerun_attack_m4_close_code_r1 compared=5 differing=0
```

退非 0 的件数：0（这一次只起了这一件）。输出留在草稿目录 `rerun-1/`（五份 `.txt` 与五份空的 `.diff`），编译目录与树由 `rerun.sh` 自己删了。

## 跑在哪、等了多久

全部在本机经 `scripts/hosts/run-with-memory-cap.sh` 8G 起，线程 `capped.sh 31`。我起的几件一次只起一条；上一条攻方腿留下、一直排着的那一件 `cargo build -p singlefs-core`（pid 1770836）我停不了（停它的命令被权限拒了），它后来自己排到、在改过名的改前树上解析失败退出、没编出东西，日志最后一次写是在我第一次编探针前后，两件可能有一小段同时在跑（推的，按两份日志的写入时刻）。排队最长那一次等了 1692 秒（`probe-run-1-done.txt` 里内存包装打的「已等 1692 秒」），之后几次 0–533 秒。包装几次提示「第二台此刻放得下」，没去：探针要读草稿目录里的树，第二台上没有。第一次编译因为我的探针源码有两处借用错误红了（`probe-build-1.txt`），改了重排。

## 崩溃态的取样

这一条腿没有造历史、没有调 `enumerate_crash_states`，原型跑的崩溃态 0 个；接手的腿不必从这里扣量。

## 没做什么

- T8 没攻；T1 的跨 inode 判法、「供三方核」第一、二、四处、T4 的「两个号攒进一次发布」、T5 的差分测试罩不罩得住信任清单，都没攻或只读到上面写的那几行。
- 没在内存稀疏盘上造任何历史，没枚举崩溃态，没跑 checker；T1、T2 读回、T4、T9、T3 的「没打中」都只是读码。
- 没跑仓里的任何测试，没跑门禁、checker 档、GPU、Verus；三个证红变异体只喂给了我自己的探针。
- 没读 codec-writer-2 报告第二节的「同义改写逐处列表」，也没读事务员事实表的 T6、T7 两节逐条对照；差分是黑盒地比两版公开函数，私有函数只经公开函数间接比到。
- 「没打中」各格只抽了一次样。

## 删了哪些（交回前删，删之前 `du -sh` 量的）

| 路径（草稿目录 `~/.local/share/singlefs/work/m4-close-code-r1-attack/` 下） | 是什么 | 大小 |
|---|---|---|
| `new/` | 冻结副本 `crates/` 等的拷贝（上一条腿建的） | 44M |
| `old/` | HEAD 那一份 `crates/` 等的拷贝（上一条腿建的，我改了 core 的清单） | 41M |
| `target-probe/` | 差分探针的编译目录 | 462M |
| `target-mut/` | 三个变异体共用的编译目录 | 1.2G |
| `mut-every-byte-is-zero-skips-the-first-byte/`、`mut-container-tree-grows-only-past-the-fanout/`、`mut-inode-leaf-borrow-rounds-up/` | 三个变异体的树 | 各 44M |
| `probe-ws/`、`probe-mut-*/`（三份） | 探针工作区（源码已在模型目录） | 各 80K |

`rerun-1/` 里的树与编译目录由 `rerun.sh` 跑完自己删了。草稿目录里留下的只有日志、草稿与输出（删完之后整个目录 748K）。

## 要主 agent 跑的命令

T10 那 8 行锚点失效，要在门禁上坐实的话（轻型，子 agent 不起，交主 agent）：

```
SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=crates 变异轻型; 范围=冻结副本上 write_request_split.rs 与 truncate.rs 的变异行; 为什么=核攻方腿报的 8 行锚点失效' bash scripts/hosts/run-with-memory-cap.sh <上限> bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick <冻结副本根>
```

看它对 `crates/mutations.tsv` 第 4387、4404、5894、5896、5901–5904 行报不报「锚点失效」。`--item` 的项名我没查，按门禁自己的列法取。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-close-code-r1-opus-output.md` 末行（交回前现跑，退 0）：

```
  ✓ 核了 17 处引文、13 处裸行号，对上 17 处，没判 13 处（其中裸行号没有引文可比 13 处；逐处列在上面）
```
