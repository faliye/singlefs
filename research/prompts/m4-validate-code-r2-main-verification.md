# m4-validate-code-r2 判决：标签 0 写者照写、隔离的盘上记号落点甲与测试临时目录守卫的代码轮（主 agent，2026-10-24，第二轮）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 K5 S1 -->

被判的产品代码（冻结副本 `~/.local/share/singlefs/work/m4-validate-code-r2/repo/`，开工快照 `research/prompts/m4-validate-code-r2-snapshot/judged-sha256.txt`）：`crates/singlefs-core/src/extent_tree.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/truncate.rs`、`crates/singlefs-core/src/allocation_record_tree.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/region_scan.rs`、`crates/singlefs-core/src/block_device.rs`。

## 一、核查

- 开工快照：`python3 scripts/agents/snapshot-tree.py m4-validate-code-r2 --out /tmp/claude-1000/-home-fy5090-code-singlefs/361bb28f-a369-4461-967b-925319d2cf5d/scratchpad/m4-validate-code-r2-tree`，末行「清单点名 10 份，倒推出 10 份，sha256 对上 10 份（……10 份取工作区此刻那一份）」：腿跑着的时候被判文件没被改过，没有要倒推的改动。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-validate-code-r2-opus-output.md research/prompts/m4-validate-code-r2-sonnet-output.md --root /tmp/claude-1000/-home-fy5090-code-singlefs/361bb28f-a369-4461-967b-925319d2cf5d/scratchpad/m4-validate-code-r2-tree --background research/prompts/_m4-validate-code-r2-background.md`，末行「核了 67 处引文、60 处裸行号，对上 50 处，没判 77 处（其中裸行号没有引文可比 30 处；逐处列在上面）」，对不上 0 处。没判的 77 处：47 处是路径不在快照树里（kb、`research/prompts/`、快照之外的源码）或只写了文件名，30 处是裸行号。判决要用的那几处我逐行现查：
  - 冻结副本：`write_request_split.rs:61`「content_length_in_bytes.div_ceil(payload_capacity).max(1)」、`:103`「DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 && data_units_on_disk == 0 => {」、`mounted_read.rs:809`「None => DataUnitsOfTheInode::NoneWritten,」，三份与主工作区逐字节相同。
  - 快照树：`truncate.rs:238`「let keeps_at_least_one_unit_or_had_none = data_units_after >= 1 || data_units_before == 0;」、`truncate.rs:273`「|| tail_before != tail_after;」、`extent_tree.rs:335`「if data_units < size_in_bytes.div_ceil(payload_capacity_in_bytes()).max(1) {」、`transaction.rs:8947`、`:8948`（两端尾部由 `TailOfAFile::of_data_units_and_size` 算）、`transaction.rs:12339`「(None, None) => size_of_the_first_file_after_this_publish」、`recovery.rs:7305`、`:6692`、`:5962`、`:5968`、`allocator.rs:2227`「if devices_whose_record_stays_allocated.contains(&device.device) {」，与腿写的相符。
  - `extent_tree.rs:407-418`：文档注释写「size > 0 写标签 0，size 0 不写条目——与 `transaction::publish_new_inodes` 建出来、一次都没写过内容的 inode 同一形，读者按「没有条目」收」，函数体 `if size_in_bytes > 0 { Some(ExtentUpperLeafTarget::NoDataUnit) } else { None }`。
  - kb：`.claude/kb/decisions/19-块指针的结构与宽度预算.md:117`（隔离的盘上记号四句）、`:119`「不认记号的旧读者读到 0x4001、0x4002 都过不去——分配记录树读者拒整棵树（挂载与恢复失败）……」、`.claude/kb/decisions/08-核心索引结构.md:451`「标签 0 全零：第一个文件一个数据单元都没有而 size > 0（整个文件都是洞）时，写者重写上段叶照写这一条」、`:477`（用户定案「丑」）、`research/prompts/m4-validate-code-r1-main-verification.md:73`（大 size 冷读用户定这一版不改，C878）、`.claude/kb/checks-owed.md` 的 C878 行，与腿写的相符。
- 复跑：
  - 攻方腿的模型拷进 `~/.local/share/singlefs/work/m4-validate-code-r2/verify/opus-model/`，`SHA256SUMS` 全 OK。经它的 `tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <草稿>` 跑三组，都是 `same_as_model_out=yes exit=0`：`name=main`、`name=old-readers`、`name=fix-candidate`。
  - 辩方腿的模型拷进同目录 `sonnet-model/`，报告第 0 节的 12 个 sha256 全对。经内存包装（8G）跑它的 `tools/rerun.sh`，退 0，79 行标准输出与 `out/rerun-stdout.txt` 去掉草稿目录前缀后逐行相同。
  - 两边副本里的编译目录与仓副本都当场删了。
- 背景材料有一处写错：正文 T5 把 r1 判决第四节第二条岔路写成「还开着」，可同一份判决第 73 行写的是用户 2026-10-22 定这一版不改（C878）。辩方指出了这一处，两条腿都按已定判，不影响判定。

## 二、逐格判定

| 格 | 辩方腿 | 攻方腿 | 判定 |
|---|---|---|---|
| T1 标签 0 配 size > 0 | N > 0 的三种改 size 站得住；**N = 0 打中** | **N = 0 打中**（12 种后续放开扫，5 种中） | **打中**。起点是标签 0、size 3000 的第一个文件，可写挂载截到 0。那一版上段整棵照抄，标签 0 留着，三处读者都收。之后只要有一次发布重写第一片上段叶（覆盖写或顺序写号 2、上段长高、写号 2 再删、写号 2 再变长到 7000），写者就按 size 0 经 `extent_tree.rs:413` 取 `None`，把第一个文件那一条丢掉。写出的这一版，冷走读、可写重挂、冷恢复都拒成「这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符」，挂载态读者收（变长到 7000 那一种也拒）。进程重启之后可写挂载不退回更早的根，池只能只读挂；池级 checker 不多判一条红。两条腿独立量到，我复跑两边的输出都逐字相同。病根是 `of_a_file_without_data_units` 的文档注释那句「读者按没有条目收」：对号 ≥ 2 的空 inode 成立，对第一个文件不成立（冷读那一路只给「第一个文件删完」留了没有条目的口子，`recovery.rs:5962`）。同一病根的第二个表现：截到 0 再删第一个文件，上段叶里留下一条孤条目（辩方 rerun-stdout 第 67、68 行，攻方扫描表「unlink 第一个文件」一行）。两条腿各自在副本上量了同一个改法（size 0 也写标签 0），5 种拒收与孤条目都消失，自带的四个标签 0 测试目标照绿 |
| T2 隔离那次发布置记号 | 站得住 | 没打中 | 两条腿各抽样一次，都没打中：一块盘读坏时两块盘都置 0x4002，分配代不变；同一次发布隔离两个单元，四条记录都带记号；写者只写得出 0x0、0x4000、0x8000 三种 |
| T3 清记号 | 站得住 | 没打中 | 两条腿各抽样一次，都没打中：回退到引用被隔离单元的版本时清记号，不引用的保留；回退被拒时记号不变。「释放隔离记录」一句今天没有活路径走到，只有单测钉着，是落点甲实现员已记的欠账，整理与扫描重建没意外走到（攻方 40 次覆盖写里整理没起，只量了这么多） |
| T4 读者认记号 | 站得住 | 没打中 | 两条腿各抽样一次，都没打中：核心层只有 `AllocationRecord::parse` 一个解析点，已去掉记号位，0xC000 判坏。区域扫描与常驻隔离集不从记号起种，是已记的欠账，这一批没修 |
| T5 第一轮的改法本身、旧读者 | 打中一处条款措辞 | 打中一处条款措辞 | **打中（措辞）**。两条腿拿 HEAD 841b3beab 导出的旧代码读带记号的镜像：分配记录树读者、冷走读、冷恢复拒成 `AllocationRecordOutsideThePoolGeometry`，旧池级 checker 多判 I-1.1、I-5.4 等红，与 E187 一致；**只读挂载照收**（两条腿都量到 `只读挂载=Ok` / `mount_read_only=ok`，带记号的与对照一样）。D19 已定项 5 第 119 行「都过不去……（挂载与恢复失败）」说多了，E187 读码那五处里没有只读挂载这一路。旧代码的可写挂载那一列两张都拒，对照也拒，分辨不了记号，不引。第一轮 T5 甲的改法（`truncate.rs:141`、`:240` 的前提，`transaction.rs:12339` 那一臂照写标签 0）在 size > 0 上都对，缺口只在 N = 0（T1）。大 size 冷读（C878）两条腿说法不一：攻方推「标签 0 的文件能变长，冷读多了一条到达路径」，辩方推「写路径写不出标签 0 的文件，入口的量没变」。两边都没量，又都归用户已定的 C878，这一轮不判 |
| T6 测试临时目录守卫 | 站得住（量过：守卫好时剩 0 份，弄坏「删文件」那一支剩 25 份、两条新用例红） | 没打中（只读码） | 两条腿各抽样一次，都没打中 |

T2、T3、T4、T6 的「没打中」各只有每条腿一次抽样，立场不同，不算同一立场的第二次抽样（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」）。这一轮不拿它们支撑「没有问题」，第三轮攻改法甲时一并再抽。

攻方腿报的起点镜像带一条手造带来的已分配无引用记录（I-3.1、I-3.11、I-1.13、I-3.15 四条红）。拒挂的那几层判的是 extent 树对 size，不读分配记录，与这条泄漏无关；辩方腿用同一种起点，`checker_violated` 与基线行相同。

## 三、打中之后的四句（T1）

1. 分不分辨臂：分辨。r1 判决第四节 T5 甲三个候选里，子（`publish_size_change` 在取 txg 之前拒）与寅（读者不收这一形）都到不了「截到 0 的标签 0」这一步；这一格只打中「丑」今天的实现，不打中「丑」这个选择本身。
2. 被判的系统看不看得到判别它的东西：看得到。重写上段叶那次发布手里有第一个文件这一版的记录（size 0）与空的数据指针，只是选成了冷读不收的那一形。
3. 满足的是判据哪一句：D8 已定项 14 第 477 行「写者接得住这一形」。写者写出了冷走读与可写重挂都拒的一版。
4. 每个改法在打中的格上还中不中：甲（size 0 也写标签 0）在 5 种上都不中，两条腿各在副本上量过；乙（冷读者放宽）在「写 inode 2 再变长到 7000」上照中，是攻方推的；丙（拒截到 0）全不中，但与「丑」相反，也是推的。

## 四、交用户

| 岔路 | 候选 | 用户定 | 被攻过几轮 |
|---|---|---|---|
| T1：标签 0 的第一个文件截到 0 之后重写上段叶丢条目 | 甲：第一个文件没有数据单元时 size 0 也写标签 0；乙：冷走读、可写重挂、冷恢复放宽成也收「没有条目、size 0」；丙：拒截到 0 | 甲 | 零轮（攻方腿提的，两条腿各在自己副本上量过一次） |

## 五、写回与后续

- 实现（并进写段下一批，派 `implementation-writer`）：
  - `crates/singlefs-core/src/extent_tree.rs` 的 `of_a_file_without_data_units` 改成一律交 `Some(ExtentUpperLeafTarget::NoDataUnit)`，文档注释那句「size 0 不写条目……读者按没有条目收」改掉；写者装上段叶与算上段形状都经这里，`transaction.rs` 的 `inodes_with_an_extent_upper_leaf_entry` 跟着变，核一遍它的调用方。
  - harness 档测试照两条腿量到的序列写，改前红、改后绿：标签 0、size 3000 起点 → 可写挂载截到 0 → 建 1 个 inode → 覆盖写 inode 2，冷走读、可写重挂、冷恢复、挂载态读者都收；再加「建 142 个、覆盖写 inode 143」「写 inode 2 再变长到 7000」「截到 0 再删第一个文件，上段叶不留孤条目」三条。
  - `crates/mutations.tsv` 留一条「改回 size 0 取 None 就红」的变异。
- 写回 kb（并进写回批）：
  - D8 已定项 14 第 451 行补成「第一个文件一个数据单元都没有（size 0，或 size > 0 整个文件都是洞）时，写者重写上段叶照写这一条」，变更史记用户定「甲」。
  - D19 已定项 5 第 119 行那句收窄到 E187 量过的几层：分配记录树读者、冷走读与冷恢复拒，池级 checker 判违例，旧读法在一片叶里收下 0 个。只读挂载那一路写明 E187 没量；两条腿在副本上量到照收，只当线索，不写成量过的数。
  - C900（池级 checker 判不出第一个文件有记录而上段叶没它的条目）已登进 `.claude/kb/checks-owed.md`，英日两份并进写回批。
- 第三轮：实现改完、合入后验证绿之后开 m4-validate-code-r3，攻改法甲本身。不许重复这一轮攻过的角度；T2、T3、T4、T6 再抽一次样。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 14 | 支撑 | 2026-10-24 改了：第 451 行补「size 0 也写标签 0」，用户定「甲」，并进写回批 |
| D19（块指针的结构与宽度预算） 已定项 5 | 支撑 | 2026-10-24 改了：第 119 行旧读者那句收窄到 E187 量过的几层，只读挂载写明没量；隔离记号的写、清、读三段两条腿都没打中 |
| D3（空间分配） | 不影响 | 2026-10-24 不受影响：跨度段读法（去掉记号位）只有 `AllocationRecord::parse` 一处，两条腿都没打中 |

## 用户定案

- 第四节 T1：用户 2026-10-24 弹窗选「甲：一律写标签 0 (Recommended)」。被攻过零轮，改完开第三轮。
