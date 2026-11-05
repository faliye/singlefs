# m4-close-code-r1 攻方腿第二次抽样（Opus，s2）

轮名：m4-close-code-r1；立场：云端攻方（正文第四节）；攻击面：正文第三节 T1–T10，先 T6、T7，再 T1、T2、T4，T8 必攻。
冻结副本：`/home/fy5090/.local/share/singlefs/work/m4-close-code-r1/repo/`（只读；量法是拷进草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack-s2/` 再编）。
前几轮判决：无（这一题第一轮）。本份不读禁读清单里的任何文件。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T6 同义改写 | 没打中 | 改前改后 1082244 行差分逐字节相同（读者拒收结局、panic 位置、写者字节） |
| T6 往返契约 | 打中（规格射程，不在这一轮 diff 里） | 树表条目读者收下、写者在指针位置条目升序断言上 panic，改前就有；规格把它写成写者前提，往返引理只管一个方向 |
| T6 写者收严的前提 | 没打中 | 执行调用点传的宽都是常量对，没有一处条目宽窄于 key 宽、记录宽 0 |
| T7 改写前后同不同 | 打中一半（报告射程句窄了） | 改后交 `None`、改前 panic 或收下的输入不只「层级 1 求和越过 usize」，每一层都是；合法镜像走不到 |
| T7 代价 | 打中（代价，不是结局） | `ancestors_of_containers` 改成按全部容器数开标记表，每次发布 O(容器总数)；造的大形状上 4G 撞顶，改前当场交回 |
| T7 panic / 静默截断 | 没打中 | 64 万行发布序列差分逐行相同，改后没多出 panic |
| T8 G7 前提 | 打中 | G7 要「整理那份也接受每一步」，整理被接受要现行等于最后一个根；协议对整理的发布不设这道门，core 一侧只有注释，没证 |
| T8 Y5 桥 | 打中（推的，模型差） | core 一侧根环模型的「成形」比协议少两条（持久 F 之上的根装得进窗口、内存 F 至少留一个候选） |
| T8 其余几份 | 没打中 | 只读了 requires / ensures |
| T1 | 没打中（只读代码） | 四处「供三方核」与 C145 断言各攻一个形状，没找到合法历史触发 |
| T2 | 没打中 | 读盘与一份读错的判法读代码没找到误判；切分 300000 组差分逐行相同 |
| T3、T5 | 没攻 | 余力给了 T6、T7、T8、T10 |
| T4 | 没打中（只读代码） | 触发函数是纯函数、只看交进来的量；脏量口径与 P2 用户定案同一件事 |
| T9 | 没打中 | 坏盘给大项数时先判 ≤ 64 再分配，差分里取到 u32 最大 |
| T10 | 打中（登记缺口） | 容器小树新写法（祖先求法、二分查找、重写清单）在 `crates/mutations.tsv` 里没有一行；三条变异 core 库测试都红得了，缺的是登记 |

## 复跑与文件

模型目录：`research/prompts/m4-close-code-r1-opus-s2-model/`。复跑（每段一个新的草稿目录，前台限时里一段跑得完；不设 `RERUN_PART` 依次跑全部）：

```
RERUN_PART=codec bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-close-code-r1-opus-s2-model/tools/rerun.sh --repo <仓根> --frozen /home/fy5090/.local/share/singlefs/work/m4-close-code-r1/repo --work <新草稿目录>
RERUN_PART=trees …（同上，4G）
RERUN_PART=split …
RERUN_PART=mutation:ancestors-only-the-first-child-counts …
RERUN_PART=mutation:rewritten-list-keeps-duplicates …
RERUN_PART=mutation:binary-search-drops-the-last-candidate …
RERUN_PART=mutation:retired-ties-go-before-the-existing …
```

交回前在新 shell 里逐段跑过一次，七段都退 0，打出的行与正文贴的相同（codec：`differential: identical`；trees：`unexplained 0`、`publish-sequence differential: identical`；split：`differential: identical`；四条变异的 `lib rc=` 与差分结论与 T10 那张表相同）。改前那棵取提交 `59ce5a86550de2c2c9bc6b51b34b4f881313682c` 的三包。

```
46785382d92dc246b6c60e48d1014c07fbcf519f16d037834601498413bb5240  ./tests/attack_codec_differential.rs
3839bdf0cf12c9e0342ab8886a3dd07bc67ecd017ac2293c5214b21a00ebf88b  ./tests/attack_container_tree_differential.rs
ecc789c6fd130b228fa8e958f4bfe082fbfd68680c6178a41576ebb575c83b2f  ./tests/attack_partial_write_split_differential.rs
1de711c188265ea9cb1351133d03c44048ffa1e643c5e5ca007c3473de8cba42  ./tools/apply-mutation.py
8f103e16ca469db94b5107490622ef723c1db92864cafe38112072384e839805  ./tools/check-shape-differences.py
016df28fbd0a2aa924c7efc30c5298ef1698c33b9340a9469cde4b931480c7d2  ./tools/differential-codec.sh
26ac1bc6c8203bb28199c47206cdff81c306faf94344ed5d3b45ab4ad6c79491  ./tools/differential-container-trees.sh
707ad8037032ac428c849c10d4c81f1fdfd3a615c197464e9b1ebcc45c3df67c  ./tools/differential-partial-write-split.sh
a6d7d5dc8ecb831bab103becdfd1e3d86f7df2e5785861f60114671203f13de0  ./tools/mutations.tsv
b11ce26ca9a459a04b14e1848a2438296019319374eb662373dc9cf367cdb5a5  ./tools/rerun.sh
288b7edba823491ff0627369e017602dce94126352e4bbccf9ffa300d83a30fa  ./tools/run-one-mutation.sh
```

## 量法（T6、T7 共用）

改前取 `git show HEAD:` 的 `crates/singlefs-core`、`singlefs-format`、`singlefs-replay-prefix`（14 份补丁都没提交，HEAD 那一份就是改前；这三包之外不带），改后取冻结副本同三包；两棵树各拷进草稿目录 `old/`、`new/`，工作区成员删到这三包，各用一个编译目录（`target-old/`、`target-new/`）。同一份差分用例（模型目录 `tests/`）拷进两棵树的 `crates/singlefs-core/tests/`，各编各跑，同一个种子、同一串输入，逐行打出结局（panic 记成 `PANIC`），两份输出逐行 `cmp`。用例只调两边签名相同的公开函数。数都是副本上量的（「量的」，量的是两棵副本树编出来的测试二进制），不是入库装置上的数。

复跑（在草稿目录建好 `old/`、`new/` 两棵树之后）：

```
bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G bash <模型目录>/tools/differential-codec.sh <草稿目录>
bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G bash <模型目录>/tools/differential-container-trees.sh <草稿目录>
python3 <模型目录>/tools/check-shape-differences.py <草稿目录>
```

## T6　编解码与记录格式

### 同义改写：差分没有打出改前改后不同的一行（没打中）

取样（量的，`out-new.txt` 按第二列计数）：定长读者 16 种（inode 140、tombstone 40、lineage 24、retirement 34、accounting 34、树表条目 200 两种读法与 `root_pointer_bytes`、inode 内部条目 120 两种读法、extent 112、映射条目 55、映射 key 27 的 `birth_txg_of_a_mapping_key` 与 `mapping_key_sort_key`、logged operation 44、deadlist 43、位置条目 14、节点指针与数据指针三种读法），每种在「宽 −0…宽 +3」每个长度上取随机字节，正好等宽的另取 4000 条，再从读回成功的样本上改 1–4 个字节取 40000 条，读回成功的连同 `to_bytes` 一起比；journal 记录 60000 条（造好头、点名项数取 0、1、63、64、65、u32 最大与随机，八成重封载荷与头校验和，一半从读回成功的样本上改字节），其中读回成功 13066 条，`to_bytes` 读回字节相同 9833 条；根记录槽 60000 条（长 465、466、467、512、4096，八成重封头校验和，六分之一把期望 fsid 改一位），读回成功 19381 条；`slot_after_the_journal_ring` / `record_offset` 20000 组（含 u64 最大附近）；码 1、码 2、码 3 写者各 6000 次（宽 0、1、8、24、34、40、140、4000、32661，key 宽 0…16000，条目宽窄于 key 宽的也取），每个写出的单元再改头里 1–3 个字节、按头末尾重封头校验和、读 6 次；`logged_operation_entries_of_the_root_entries` 20000 组；`unit_filesystem_identifier`、`data_key_tail`（事务号取到 2⁶⁴ 附近）、`node_key_tail` 6000 组。

```
$ bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G bash …/model/tools/differential-codec.sh …/m4-close-code-r1-attack-s2
   1082244 /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack-s2/out-new.txt
   1082244 /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack-s2/out-old.txt
   2164488 total
differential: identical
```

结论：这 1082244 行上改前改后逐字节相同，含读者拒收的结局（拒因的种类）、panic 的位置（同一批输入两边都 panic）与写者写出的字节。报告点名请三方带上的几处（`parse_slot` 取期望 fsid、`data_key_tail` 的 `% 2⁴⁸`、`put` 逐字节写、`InodeRecord::parse` 两段恒零、journal 点名项数先比 64 再乘、`pieces_of_one_width`、`div_ceil` 换 `is_multiple_of`）都在取样里，没打出不同。推翻条件：换一份能造出更深合法输入的生成器（例如整份镜像经 `make_filesystem` 与发布写出再逐字节改），打出一行不同。

### 往返契约：读者收下、写者 panic 的一处，改前就有，不在这一轮的 diff 里

差分里 `tree_table` 那一行有 4144 次 `PANIC`（改前改后同一批输入）：`TreeTableEntry::parse` 收下的条目，`to_bytes` 在节点指针写者那一句断言上 panic——读者不判位置条目按设备升序，写者判：「self.location_entries_ascend_by_device().is_ok(),」（`crates/singlefs-core/src/pointer.rs:709`）。改后的规格把这一点写成了 `to_bytes` 的前提「&&& self.root.has_location_entries_the_writer_accepts()」（`crates/singlefs-core/src/records.rs:1337`），所以证明成立；往返引理只管「写者写出的读者收下、读回相同」那一个方向，读者收下而写者不收的那一半在规格里是前提，不是结论。这不算同义改写的打中，记成规格射程的一句：「读回再写回」这条路径（照抄写回盘上读来的指针）靠调用方先判过，正文 T6 第二问的射程到这里为止。

### 码 2、码 3 写者收严的前提：没打中

`spec_packed_unit_writer_accepts` 要「&&& record_width > 0」（`crates/singlefs-core/src/unit.rs:691`），`spec_index_node_writer_accepts` 要「&&& key_width <= entry_width」（`crates/singlefs-core/src/unit.rs:931`）。core 里的执行调用点（`transaction.rs` 9 处、`extent_tree.rs` 4 处、`allocation_record_tree.rs`、`instance_table.rs`、`make_filesystem.rs`、`mount.rs`、`region_scan.rs`）传的记录宽是常量 88、140、40、24、34，key 宽与条目宽是常量对（8 对 120、8 对 8、key 宽对 44、树表 key 宽对 200、extent key 宽对 112 与 110）或码 2 树的字段宽表；没有一处取运行时的值、没有一处条目宽窄于 key 宽。差分里写者在条目宽窄于 key 宽、记录宽 0 上的结局改前改后相同（前提只进 `requires`，运行时不挡，与 codec-tighten 报告的说法一致）。

（上一段补注：`allocation_record_tree.rs` 传 key 宽 `key_bytes_width()` 与按叶、内部节点分的条目宽；`mount.rs`、`region_scan.rs` 里那几处在测试模块里，不是执行调用点——按代码读的，「推的」。）

## T7　容器小树

### 改前改后不同的只有「某一层孩子数之和越过 usize」这一种输入，报告只写了层级 1（打中一半：报告的射程句窄了）

`ContainerTreeShape::from_children_of_each_node_at_each_level` 喂 200000 组随机形状（0–4 层、每层 0–4 个节点、孩子数取 0、1–4、usize 最大、2⁶³），一半按下一层的个数凑成自洽的。两边都取到的 70377 组输入里，结局不同的 9338 组，全部是「有一层的孩子数之和 ≥ 2⁶⁴、改后交 `Ok(None)`」，改前其中 8215 组 panic、1123 组收下（量的，`check-shape-differences.py` 的原样输出）：

```
shape inputs old 74716 new 74761 in both 70377 differing 9338
explained by an overflowing level sum (old result -> count): {'Err("PANIC")': 8215, 'Ok(Some(Cont': 1123}
unexplained 0
```

实现员报告只写了层级 1 那一半（事实表该节：「层级 1 的孩子数之和越过 `usize` 的，现在交回 `None`」）。量出来还有上面几层：例如 `[[], [3, 18446744073709551615, 1, 3]]`、`[[], [2, 18446744073709551615, 0, 0], [4, 2, 4], [9223372036854775808]]`，改前在求和处 panic（开着溢出检查，release 也开，见工作区 `[profile.release]` 的 `overflow-checks = true`），改后交 `None`。这一处的不同对合法镜像与坏盘都走不到：执行调用点只有 `recovery.rs` 一处（「let shape = ContainerTreeShape::from_children_of_each_node_at_each_level(」，`crates/singlefs-core/src/recovery.rs:5673`），孩子数取自盘上码 2 节点的条目数，每个不超过一个节点的条目容量，求和越不过 usize（推的）。打中的是「报告列的行为不同」少了一类，不是行为错；交主 agent 判要不要在报告与 kb 的说法里补「每一层」。

### `ancestors_of_containers` 从「只看被问的容器」换成「按全部容器数开一张标记表」：代价从 O(被问数) 变成 O(容器总数)（打中，代价，不是结局）

改后的写法：「let mut marked_below: Vec<bool> = Vec::new();」（`crates/singlefs-core/src/container_tree.rs:1886`），随后「marked_below.push(false);」（`crates/singlefs-core/src/container_tree.rs:1893`）一直推到全部容器数；改前是对被问的那几片容器逐层往上收进一个 `BTreeSet`。结局相同（差分里形状不大时逐行相同，见下一节），代价不同：差分里第一组样本就是形状 `[[9223372036854775808, 2], [2]]`（合法性判法收下它），问一片容器的祖先，改前当场交回，改后在 4G 上限里撞内存被停：

```
run-with-memory-cap: 撞了内存上限 4G（scope singlefs-memory-cap-3567020-275077497 的 Result=oom-kill，命令退出码 143）
```

这个形状不是盘上来的（每个节点的孩子数受条目容量管），所以合法镜像上不会撞内存；但它是每次发布都走的路（「self.shape.ancestors_of_containers(self.rewritten_containers.as_slice())」，`crates/singlefs-core/src/container_tree.rs:2402`，`rewritten_nodes` 每次发布调），改前每次 O(重写片数 × 层数)，改后每次 O(容器总数) 的分配与两遍扫。容器总数随池里的 tombstone、lineage、retirement 记录数长（一片 815 / 1359 / 959 条），十亿条 tombstone 约一百二十万片，每次发布多开一张约 1.2 MB 的表、多扫一遍（推的，按条数除，没量挂钟）。条款里没有这一条路径的代价约束（没在 D8 / E 编号里查到，推的），交主 agent 判：算不算「改写前后行为同」之外要记的一笔，要不要登一个计数模型量它。

### 发布序列差分：没打中

`append_records_to_the_open_container`（一片 1–4 条、节点条目封顶 2–4 或按格式，1500 段历史各 20–80 次发布，每次 0–5 条；单条时另比 `append_record_to_the_open_container` 与批量版结局相同；每步比 `carried`、`rewritten_nodes`、`node_origins`、`grew_a_level_this_publish`、`previous_root_is_carried_down_unchanged`）与 inode 树 `write_records_into_leaf_containers` / `remove_record_from_leaf_containers`（1200 段历史，每段 10–60 步；写一批新号接在末尾，偶尔一批 200–599 条跨片，连同几条已有号的覆写，偶尔塞不升序或重复的号；删多半删在的号，偶尔删不在的号，偶尔一步连删 1–150 个号，走分裂、合并、借位、删空叶；每步比整份结果、`retired_identities`、各片的容器号、最小最大号、分隔 key、空槽）。形状那一段两边收下的集合不同，后两段各用自己的种子，不被它带偏。去掉形状那一段的行之后两边逐字节相同（量的）：

```
  642453 …/tree-rest-old.txt
  642453 …/tree-rest-new.txt
cmp=0
```

取样里 inode 写 22126 次（拒收 831 次「RecordsNotInStrictlyAscendingInodeOrder」）、删 166693 次（拒收 10961 次「RecordToRemoveIsNotInTheTree」），1092 步有退役的片；容器树追加 74188 次、没有一次拒收（量的，`tree-rest-new.txt` 按行首计数）。

### `assert!` / `unreachable!` / 静默截断在合法镜像或坏盘上走得到吗：没打中

差分里两边 panic 的输入逐行相同，改后没有多出一处 panic（形状那一段改后反而少 panic）。按代码读：`records_below_which_a_leaf_borrows_from_its_right_neighbour` 从 `div_ceil(2)` 换成「usize::try_from((INODE_LEAF_RECORDS + 1) / 2).expect("117")」（`crates/singlefs-core/src/inode_tree.rs:264`），233 时同为 117；`retire` 从「push 后稳定排序」换成按上一版叶序插入、同序号排在已有的之后（`crates/singlefs-core/src/inode_tree.rs:2008` 起），与稳定排序交同一份清单。取样的限度：容器树最深只量到几层（节点条目封顶 2–4），没造 255 层（实现员报告第七节第 1 条那一格是坏镜像上的，本腿没造盘上镜像）。

## T8　只动规格与证明的五份

### G7 的前提「整理那份也接受每一步」把「整理的发布只在现行内容等于最后一个根时发」放进了 requires，而协议的允许条件里没有这一条（打中：规格比条款窄，前提靠 core 没证的一句注释）

G7 定理的 requires：「forall|index: int| 0 <= index < events.len() ==> #[trigger] event_is_accepted_by_both(start, events, index),」（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:2205`），整理那一步「被接受」的定义是「ObservedEvent::CompactionPublish => compaction_publish_is_accepted(observation_with_compaction(start, events, index)),」（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:2193`），即「&&& observation.head_logical == observation.roots[observation.roots.len() - 1].logical」（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:1305`）。可是根环协议本身对整理的发布不设门：「RunRingStep::CompactionCheckpoint => true,」（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:244`），它推进去的根带的是现行视图「state.roots.push(RingRoot { view: state.head, name: RootName::CompactionPublish }),」（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:312`）。

协议允许、G7 不罩的一段历史（按 spec 文本手推，「推的」，没起 Verus）：起点一个用户根 v0、现行 = v0 的内容；`UserPublish { new_view: V1 }`（只换现行，不推根，`gc_ring_interval_proofs.rs` 第 290 行起那一支）；`CompactionCheckpoint`（协议放行，推一个带 V1 的整理根）；`Crash`（现行回到最后一个根）。不整理那份崩溃后现行是 v0 的内容，整理那份是 V1——`compaction_relation` 里「崩溃之后重现的内容相同：两份最后一个根的内容相同」那一条不成立。G7 对这段历史不说话，因为第二步 `compaction_publish_is_accepted` 为假。

条款那一侧：判决 Z1 第二行把 G7 定成「整理不改现行内容；回退候选集是不整理那份的后缀、至多少 k 个，抬 F 的上限恰多 k（k 为这一段整理占的发布数），按版本号点名」（`research/prompts/m4-exit5-relocation-proof-r3-main-verification.md:20` 那一行的判决列），用户定「core 直验，连胶水一起证」。G7 成立要靠「整理只在现行等于最后一个根时发」，这一条今天只在 core 的文档注释里：「那一趟起跑之前先把待发排空」（`crates/singlefs-core/src/mounted_session.rs:5466`），既不在协议的允许条件里，也没有 core 一侧的证明把它接到 `RunRingStep` 上（`verified_model.rs` 只有 spec 定义与「对到 gc_ring_interval_proofs 的 RunRingState」的注释，没有一条 ensures 引 `run_ring_next`；按 `grep -rln 'RunRingStep\|run_ring_next'` 查，core 里只有 `verified_model.rs` 提到，且只在注释里）。

四句：分辨不分辨臂——这一格没有臂，问的是规格对不对得上条款；系统看不看得到——core 的后台整理入口看得到「待发空不空」，判得出；满足的是 T8 第一问「一条 requires 比实现的调用点强，使证明成立而条款没被证」那一分句（前提里的 `compaction_publish_is_accepted` 在协议层没有对应的门，core 的调用点有没有满足它没人证）；改法（只在我的模型上推过、被攻过零轮）：把 `run_ring_step_is_enabled` 里整理那一支改成要求「现行视图等于最后一个根的视图」，让投影引理把它带到观察层，G7 的 requires 就能从协议推出来、不再是额外假设；core 一侧要证 `run_compaction_in_the_background` 的每一次整理发布之前待发为空。推翻条件：找到一条已证的引理，从协议的可达状态推出整理的每一步都满足 `compaction_publish_is_accepted`。

### Y5 那座桥缺着：core 一侧的根环模型比协议的「成形」条件少两条（打中：推的，模型之间的差，不是运行时反例）

协议的成形条件有「&&& state.roots.len() - state.persisted_floor <= state.window」（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:130`，持久 F 之上的根装得进根环）与 `in_memory_floor < roots.len()`（内存 F 至少留一个候选）；core 一侧的模型 `root_ring_is_shaped` 只有 `roots.len() >= 1`、`window >= 1`、按 checkpoint 号严格升、持久 F 等于最后一个根带的 F 与「&&& ring.persisted_floor.0 <= ring.in_memory_floor.0」（`crates/singlefs-core/src/verified_model.rs:207`）。按 checkpoint 号的模型里，内存 F 大于最后一个根的号、持久 F 之上的根比 window 多，都还算「成形」，到了协议那一侧却不成形；所以哪天把桥建起来，这两条要在 core 一侧另证成不变式，否则「回退候选至少一个」「持久 F 之上的根都在环里」这两条结论在 core 的模型上推不出来。另一处：协议里 F、版本号、根数都是 `nat`，core 是 `CheckpointTxg(u64)`；「抬 F 的上限恰多 k」在 `nat` 上永远成立，到 u64 的版本号用尽时 core 怎么办，协议不说（推的，没造到 2⁶⁴ 的历史）。

### 别的几份：没打中

`compaction_plan_proofs.rs`、`gc_crash_proofs.rs`、`address.rs` 的 newtype、codec-tighten 用 `hide` 换的两行必红世界：只读了 requires / ensures，没找到比条款弱的一条（没起 Verus，按正文要求不复验空真与必红）。

## T10　变异钉不钉得住

`crates/mutations.tsv` 6332 行里，锚点落在容器小树新写法上的（`marked_below`、二分查找的 `while low < high`、`noted_so_far`、`insertion_point`、`checked_children_count_sum`）一行都没有；编解码那一侧 `pieces_of_one_width`、`put_six_byte_unsigned`、`copied_into` 也没有，`every_byte_is_zero` 5 行、`is_multiple_of` 11 行、`precedes` 2 行（量的，`grep -F … | wc -l`）。journal 读者点名项数那一判有行，且库测试 `record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser` 同时钉住 64 收、65 拒两边。

我在改后那棵副本上打了四条变异（模型目录 `tools/mutations.tsv`），每条跑 core 库测试（372 条）与容器小树差分（量的，副本上的数）：

| 变异 | 改哪一行 | core 库测试 | 发布序列差分 |
|---|---|---|---|
| 祖先只认每个节点的第一个孩子 | `container_tree.rs:2011` `if marked_below[child] {` 加 `&& child == first_child` | 红 3 条（`appending_several_records_rewrites_every_touched_container_and_all_their_ancestors` 等） | 红 |
| 重写清单不去重 | `inode_tree.rs` `note_rewritten` 里相等那一支短路掉 | 红 2 条（`one_write_that_both_replaces_and_appends_lists_each_touched_container_once` 等） | 红 |
| 二分查找漏最后一个候选 | `inode_tree.rs:671` `while low < high` 改成 `while low + 1 < high` | 红 12 条（`writing_a_number_already_in_the_tree_replaces_that_record_and_keeps_the_container_identity` 等） | 红 |
| 退役清单同序号排到已有的前面 | `retire` 里 `<=` 改成 `<` | 绿 | 绿（等价：上一版叶序各片不同，第 1971 行按位置取，没有同号） |

结论：这三处新写法的库测试是红得了的，缺的是登记（表里没有点名它们的行，变异门禁覆盖不到）；补的取样点就是上表三行，必红的测试名照写。T6、T7 的差分本身作判据时也证过会红（同一批变异都打红了它）。

## T1　多 inode 改 size

只读代码，没造历史（「推的」）。四处「供三方核」的写法与 C145 的断言各攻了一个形状，都没打中：

- 冷恢复「有记录、没有条目、size > 0」对 nlink = 0 的记录同判（R:246）：要在合法历史里造出「nlink = 0、size > 0、上段没有条目」的记录，得有一次发布在删除中途把条目去掉而留下记录。删除那一族里只有最后一批不推条目（`plan_the_files_other_than_the_first_after_this_publish` 里「ChangeOfAFileOtherThanTheFirst::FinalDeletionBatch { inode } => {」，`crates/singlefs-core/src/transaction.rs:7618`，那一支不往 `files` 里推），而同一次发布把记录删掉：「remove_record_from_leaf_containers(&containers_before, inode)?」（`crates/singlefs-core/src/transaction.rs:8217`）。中间各批截到前 `data_units_kept` 个单元（多于一批才有这一步，留下的至少一个），size 改成「剩下单元数 × 净荷」、条目留着。没找到能触发这一拒的合法历史。拒因只罩第一个文件之外的号（`recovery.rs` 那一处先判 `record.inode != FIRST_INODE_NUMBER`），第一个文件走它自己那一套——这是「两条路」的事实，判不判它推得出归正推腿。
- unlink 的批截带洞的别的文件时尾部换回 1、2（R:244）：批之后 size = 剩下单元数 × 净荷；带洞的文件在变长那次已经把最后一个存在的单元补零重写到 min(N − 起点, 净荷)（`truncate.rs` 第 131 行那一段说明），所以留下的单元都是满的，「没有洞」与盘上一致。没打中。
- C145 删快照放块那一句断言：放掉的条目从上一版 deadlist 里挑（`deadlist_entries_the_destruction_of_a_snapshot_releases`），`entries_after` 从同一份建，断言要红只能是上一版 deadlist 里有两条键相同的条目。冷恢复读 deadlist 树传的判法是 `EveryHeaderAgainstItsReference`（`crates/singlefs-core/src/recovery.rs:4270`，每个头对它的参照判），叶里判严格递增；跨叶靠头里的 key 区间对父节点的分隔 key（推的，没造跨叶重复的坏镜像去量）。没打中，限度写在末节。

## T2　部分写 core 一侧

只读代码（「推的」）。读旧净荷改成读盘之后，两份副本一份读错的判法是「两条位置提示逐条读、整单元 CRC-32C 等于那条位置条目里的才算读到」：「.filter(|bytes| crc32_castagnoli(bytes) == location.unit_checksum)」（`crates/singlefs-core/src/transaction.rs:11410`），第一份读不出或 CRC 不对就读第二份，两份都不算数才按映射再读一遍，再不成才交回失败成员。读到 CRC 对得上而单元头与查找路径不符时不再试另一份——CRC 对得上意味着字节与写下时相同，两份是同一份字节，另一份同样不符，这一处不算误判。声明长度那一判按「写之前的 size」算：截到洞里、变长补零、unlink 各批之后，最后一个存在单元的声明长度都与 size 按净荷算出来的一致（见 T1 第二条），没找到合法镜像在这里被拒。`write_request_split.rs` 的切分没造性质测试（时间留给了 T6、T7 的差分），这一格的切分取样点没攻到，写在「没打中的形状」。

## T3　部分写进模型与 RefFS

没攻（按派发的先后，余力不够）。

## T4　发布触发

只读代码（「推的」）。`publish_trigger_cause` 是纯函数，分支只看「待发空不空」「离上次发布的间隔」「待发改动的 journal 字节合计」与阈值（「if pending_is_empty {」，`crates/singlefs-core/src/mounted_session.rs:4886`），都是调用方算好交进来的，函数里没有读时钟、读设备；间隔的读数倒退当 0。脏量取「每次改动单独发布要写的 journal 字节」相加，同一单元覆盖写 K 次计 K 次，与判决 P2 的用户定案「批内覆盖写 K 次按 journal 字节计 K 次的记录（每次改动的记录照实算），不折叠」（`research/prompts/m4-publish-trigger-r1-main-verification.md:21`）是同一件事。屏障、进门预判与冻结之后那一次的处理没攻到。没打中。

## T5、T9

T5 没攻。T9：journal 读者先判点名项数不超过 64 再 `with_capacity`，差分里点名项数取到 u32 最大，改后与改前同一个结局（拒收），没有先按大项数分配（量的，差分那一节；读者先判 `named_count > (record_bytes - header_bytes) / named_entry_bytes`）。循环不变式与 vstd 规格那一半没起 Verus，不判。

### T2 补：部分写切分的差分（没打中）

`PartialWriteOfAFile` 那一族（`new`、`end_exclusive`、`size_after_in_bytes`、首末写到的单元、`data_units_after`、`first_rewritten_unit`、`rewritten_units`，重写段里前 8 个单元各自的 `data_unit`、`declared_length_after_the_write`、`bytes_written_into`，再加段外两头各探一个单元）与 `data_unit_count_of_a_sequential_write`、`data_units_read_within_the_file_size` 两种、`split_sequential_write_into_one_unit_transactions`，300000 组输入：写之前 0–5 个单元，size 取「最后一个单元满或不满」「越过单元（带洞）」「恰好整单元」与边界附近随机；偏移与长度偏向 k × 净荷 ± 2、u64 顶端附近、长度 1、净荷、净荷 + 1。改前改后逐行相同（量的，副本上）：

```
   300000 …/out-split-new.txt
   300000 …/out-split-old.txt
   600000 total
differential: identical
```

取样里 `PartialWriteOfAFile::new` 收下 260031 组；五种单元来源各出现 OldUnitReadBack 128995、OldUnitCoveredWhole 18532、NewUnitReadingTheResidueOfItsSlot 226769、NewUnitCoveredWhole 142552、GapFilledWithZeros 759513 次，panic 0 次（量的，`out-split-new.txt` 里按名字计数）。限度：这只说改写前后同义，不说切分合不合判决 R2–R7（改前也错的话两边一起错）；按条款另写一份参照模型去比，没做。

## 打中的几格：四句与改法

| 打中 | 分不分辨臂 | 系统当时看不看得到 | 满足的是哪一分句 | 改法（全是我提的，只在我的模型上量过或推过，被攻过零轮） |
|---|---|---|---|---|
| T7 每一层求和越界 | 不分臂（无臂；问的是报告列的行为不同全不全） | 看得到：判法在构造时就有全部孩子数 | T7 第一问「`from_children` 这类构造在空输入或越界时交什么」 | 报告与 kb 的说法写成「任一层孩子数之和越过 usize 交 None」（推的，没实现） |
| T7 祖先求法的代价 | 不分臂 | 看得到：被问的容器与形状都在手里 | T7 第一问「改写前后行为同不同」——结局同、代价不同，按字面不在「行为」里，交主 agent 判归不归 | 改回按被问容器往上收（标记表只开被问容器能到的那几层节点），或登一个计数模型量容器总数对每次发布的代价（推的，没实现、没量挂钟） |
| T8 G7 前提 | 不分臂 | 看得到：后台整理入口知道待发空不空 | T8 第一问「一条 requires 比实现的调用点强，使证明成立而条款没被证」 | 协议里整理那一支加门「现行视图等于最后一个根的视图」，G7 的这条 requires 由投影引理带出；core 一侧证整理起跑前待发为空（推的，没起 Verus） |
| T8 Y5 桥 | 不分臂 | 不适用（模型之间） | T8 第二问「int 与 u64 的桥缺着会让哪一条结论在实现上不成立」 | 建桥时在 core 一侧另证两条不变式：持久 F 之上的根数 ≤ window、内存 F 不大于最后一个根的号（推的） |
| T10 登记缺口 | 不分臂 | 不适用 | T10「红不了的，补哪一个取样点」——这里是红得了、没登记 | 照 T10 表的三行登记变异，必红测试名照抄（量过：副本上那三条各自红，见表） |
| T6 往返的另一半 | 不分臂 | 看得到：读者手里有位置条目 | T6 第二问「往返契约在格式常量边界上取不取得到」——读回再写回那一半不在契约里 | 不改代码；规格射程句写明「往返只管写者写出的；读者收下而写者不收的靠调用方先判」（推的） |

## 没打中的形状与取样

- T6：16 种定长读者（每种宽 −0…+3 每个长度、正好等宽 4000 条、从读回成功的样本上改 1–4 字节 40000 条）、journal 记录 60000 条、根记录槽 60000 条、环几何 20000 组、三种单元写者各 6000 次加每个写出单元改头重封 6 次、logged operation 根 20000 组、key 尾 6000 组。没造：整份镜像经 `make_filesystem` 与发布写出再改字节；`crc32_castagnoli` 的查表与硬件两路（D13（验证路线） 已定项 14，正文说不在这一格）。
- T7：形状 200000 组（0–4 层、每层 0–4 节点、孩子数含 0、1–4、2⁶³、usize 最大）；容器树追加 1500 段历史 × 20–80 次发布；inode 树 1200 段历史 × 10–60 步，含跨片批写、连删 150 个号、删不在的号、不升序与重复号。没造：255 层的树、盘上读回的坏镜像。
- T2：切分 300000 组（见 T2 补）；读盘那一半只读代码。没造：两份副本一份坏、映射也坏的盘上镜像。
- T1、T4：只读代码，没造历史，也没跑 harness 档的场景。
- T10：四条变异，三条红、一条等价（同号退役在合法输入上不出现）。

## 这条腿自己的限度

- 差分只说「改写前后同义」：改前那一份要是本来就错，两边一起错，差分看不见（T2、T6、T7 都是）。
- 改前那棵是提交 59ce5a865 的三包，没带同时在主工作区里改着的别的会话的改动；只调两边签名相同的公开函数，私有函数只经公开入口间接罩到。
- T8 的两处打中是按 spec 文本手推的，没起 Verus（正文第五节不许）；G7 那一段反例历史没写成必红世界。
- `ancestors_of_containers` 的代价只在一个造出来的大形状上量到「撞 4G」，合法镜像上的挂钟与内存没量，「每次发布约 1.2 MB」是按条数除出来的（推的）。
- T3、T5 没攻；T1、T2 读盘一半、T4 只读代码。这几格的「没打中」都只有这一次抽样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」那一节不能单独撑「没打中」。
- 副本上的数不算入库装置上的数；全部原型都不调崩溃枚举函数，没有崩溃状态数要报。

## 跑法记录与没做什么

- 起编译与跑之前看过负载：资源提醒报本机 loadavg 在 21 到 124 之间、可用 16–26 线程，内存够，都在本机跑；没有性能测量在跑。每条 cargo 经 `run-with-memory-cap.sh`（4G 或 8G）、`capped.sh 31`、`nice -n 19`，同一时刻一条，两棵树各用一个编译目录。
- 被钩子拒过的写法（照出路改了，没绕）：`eval "$(python3 …/rocksdb-prebuilt.py env)"`（命令替换当命令词，按认不出拒），改成照它打出的四行 `export` 字面写；同一条命令里先 `sed` 写出再 `bash` 起的脚本，改成 heredoc 写出、下一条命令起；python 代码里 `eval(` 被拒，改用 `json.loads`；不经内存包装起的 `cargo test --no-run` 被拒，加了包装；脚本里的 `cargo test --lib` 没带范围声明被拒，补了声明。
- 没做：不跑门禁阶段、不跑 checker 档测试、不起 GPU、不起 Verus、不跑重型；没写 kb；没提交。
- 删了的（交回前，`du -sh` 量的）：改后那棵副本 `new/` 5.8M、改前那棵 `old/` 4.8M、`git archive` 解出来的 `head/` 4.8M、编译目录 `target-new/` 1.3G 与 `target-old/` 668M；差分输出 `out-new.txt` 846M、`out-old.txt` 846M、`out-tree-new.txt` 83M、`out-tree-old.txt` 85M、`out-split-new.txt` 248M、`out-split-old.txt` 248M、`tree-diff.txt` 166M、`tree-rest-new.txt` 29M、`tree-rest-old.txt` 29M；复跑时建的七个草稿目录（`rerun-codec/` 2.8G、`rerun-trees/` 1.3G、`rerun-split/` 1.6G、`rerun-mutation-ancestors/` 1.7G，其余三个跑完即删）。草稿目录里留下的是几份小日志（共约 480K）。
- 要主 agent 跑的命令：无（这一腿没改仓里的代码）。
