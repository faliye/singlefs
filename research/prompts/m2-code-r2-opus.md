你是发布 B（覆盖写 + 释放）那批代码三方对抗**第二轮**的**攻方腿**。第一轮攻方腿打中五处、主 agent 改了四处代码（判决在 `research/prompts/m2-code-r1-main-verification.md` 第三、四节）。第二轮**只攻这四处改法本身**——攻击面换了：不再问「原来的代码错在哪」，问「改法有没有改对、有没有引进新的错」。

## 先读

1. 第一轮判决：`research/prompts/m2-code-r1-main-verification.md`（第三节每一格写了改了哪、会红的用例是哪条）。
2. 附录二：`research/prompts/_m2-code-r2-diff.md`（从提交 d2aeb7d 起 crates/ 的全部 diff + 两个新测试全文）。第一轮的背景材料 `research/prompts/_m2-code-r1-background.md` 里有条款附录（D3、D5、D16、D19、D22、D28 的相关分项整段），引条款从那里抄、行号写 kb 文件自己的。
3. 自己去读代码（原仓只读）：`crates/singlefs-core/src/allocator.rs`、`transaction.rs`、`recovery.rs`、`unit.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-checker/src/walk.rs`。
4. 要改代码跑用例，把仓拷一份到 `/tmp/claude-1000/-home-fy5090-code-singlefs/e166d536-b665-4a88-8372-3091b9a0fc41/scratchpad/m2-code-r2-opus-copy/`（`rsync -a --exclude target --exclude .git`），在副本上改、在副本上 cargo test；副本上的数照实报、注明是副本。

## 要攻的四处改法

- **Y1 释放经映射**（`placements_to_release_via_mapping`、`mapping_locations_for_key`、`TransactionOutput::mapped_units`）：它查的是**上一版内存态里的映射节点字节**与**上一版内存态里的 key**——找一个「上一版内存态与盘上不一致」或「key 算错但查得到」的形态；映射树与树表两个豁免单元从根记录取指针，那两条指针在什么历史下会指错；`span_slots` 按种类推跨度与分配记录里的跨度不一致时谁先发现；报 `ReleaseNotInMapping` 之后分配器一点没动、但调用方拿着一个半新的池——下一次发布还能不能对。
- **Y2 空闲独立维护**（`DeviceFreeMap::free_slots` 字段）：找一个让「空闲 + 已分配 ≠ 单元区」在盘上出现而 checker 不红的状态，或反过来——独立计数与位图分道的路径（`mark_released` 不动它对不对；将来回收放回时该加多少；mkfs 那两个单元走的是哪条）；I-5.2 现在判的到底是什么、它与 I-3.1 一起能不能被同一条变异同时骗过。
- **Y3 装不下报错**（`index_node_entry_capacity`、`AllocationRecordsExceedOneNode`）：容量算式 (16384 − 头) / 20 = 812 对不对（头 135 从哪来、`reserved_bytes` 算不算）；「这次之后的记录数 = 现有 + 8 × 盘数」在哪种发布上不成立（空发布、多对象、跨盘数）；报错发生在动分配器之前——核实；其它树（映射、记账、extent、inode 叶）会不会更早装不下而仍然 panic。
- **Y4 oracle 带实例**（`PublishedVersion.instance`、`newest_persisted_root`、`oracle_violation_for_versions`）：(txg, 实例) 字典序与 D22 已定项 7 / D23 已定项 14 的择新序在哪一格不同（回退新根取「根环最大 + 1」、被抛弃实例的根同时在环里、设备失而复得）；直接喂 `oracle_violation_for_versions` 构造一个判错的输入；`evaluate_state`（单版本）从根槽写的偏移 24 读实例——偏移对不对，暖机空发布的根算不算一版。
- 顺带：`allocation_records_are_one_per_device` 逐盘核（槽, 跨度, 代, 已释放）集合——找一个它放过而本该拒的镜像，或它拒而本该过的（多盘、三盘、盘上记录顺序）。

## 交付

- **分段写**：报告写进 `research/prompts/m2-code-r2-opus-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。
- 每一处先写「打中 / 没打中」再写依据；打中的给出一段历史或一个输入、副本上跑出来的读数、以及哪条用例该红而没红。引条款整行抄、行号写 kb 文件的；引代码写路径与行号。不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 40 分钟内交出报告。最后回复只写一句指向报告。
