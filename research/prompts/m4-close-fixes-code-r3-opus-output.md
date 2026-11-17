# m4-close-fixes-code-r3 攻方腿（Opus，第三轮，最终轮：「已定」项撑不撑得住、未定项清单的完整性、报出的数字能不能核）

<!-- doc-lint:not-numbers T1 T2 T3 T4 P1 P2 P2a P3 F1 Q1 Q4 Q5 Q6 M1 M2 M3 M4 -->

日期：2026-11-09。被判树：主工作区 `crates/singlefs-core/src/` 七份，`sha256sum -c research/prompts/m4-close-fixes-code-r3-snapshot/crates-sha256.txt` 7 个 OK（量的）。代码行号都是这棵树现查的。

模型目录 `research/prompts/m4-close-fixes-code-r3-opus-model/`。**三个模型都没编、没跑**（子 agent 不交任务，`rustfmt --check` 解析过，有排版差异、没有语法错，量的）。凡标「模型」的读数都是「推的，待主 agent 跑」；代码行与任务日志是我现查的。

| 文件 | sha256 |
|---|---|
| `tests/attack_fixes_r3_other_reads.rs` | `aa2ab3e85980f36ed2cfacf853193fb1c633c7edd0420599b093459e8a4ddcbb` |
| `tests/attack_fixes_r3_legal_disks.rs` | `0eedb7add5b6c8a852978565368f835967a08733413ef5e70f3ee081dacec3f1` |
| `tests/attack_fixes_r3_settling_fallback.rs` | `ba380764645df3ebc3c5da710a927c65b29c53385380083854194743049c500b` |
| `tools/rerun.sh`（照模板拷，只换 `rerun_steps`；模板外部分 `diff` 0 行，量的） | `6c0e2939f3bc6580a5f0d6d93e2ccaca145115838e3b7971d55c383a6862b222` |

复跑命令在「要主 agent 跑的命令」。

## 各格判定一览

| 格 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| 攻击面 1 P2a 多写 | **没打中**（一次抽样、读码） | 闭包走到的节点都要释放旧落点，读的指针就是释放用的落点 | 代码 |
| 攻击面 1 P2a 仍撞 64 | **打中（机制是推的，待跑）** | 提交内生块回落到最低空槽时，每片叶两个空槽的区段上取落点那一半不衰减；E206 产物里没有回落这一列，「至多 5 轮」没量过这一格 | 代码、模型 |
| 攻击面 1 写行两处盲区 | **打中（轻，推的）** | 「造不出」不成立：反复挂载不写文件就有那段历史；条款与算法没错，缺的是证红 | 代码 |
| 攻击面 2 拒合法的盘 | **没打中**（一次抽样、读码，待跑扫描） | 新判只在条数已过而集合不等时才多拒；写路径的 key 就是读路径算的那把 | 代码、模型 |
| 攻击面 2 GPU 少判一形 | 没打中 | 只有造坏的盘走得到，崩溃流造不出 | 代码 |
| 攻击面 2 只读路径不判 | 没打中；**留一形零轮** | 只读走不到错读（读码）。key 集合相等、两把 key 的位置对换的盘照收，交用户表 | 代码、模型 |
| 攻击面 3 `transaction.rs:668` 的依据 | **打中** | 「账够不着」不成立：写入口已有同形的账；每次发布都读现行实例表、不进账 | 代码、模型 |
| 攻击面 3 `mount.rs:1407` 一带的依据 | **打中** | `:1767` 不只挂载这一串，会话抬 F 也走，读的就是现行那一版的账 | 代码、模型 |
| 攻击面 3 六处没有变异行 | **打中（轻）** | 六处都不是等价；水位那一读有不靠注入的分辨输入 | 代码、模型 |
| 攻击面 4 账与 D4 | **打中（轻）** | 代码写实了「再读再试」，D4 第 73 行字面仍写「之后的用户改动不再接着搬」，kb 没写回 | 现查 |
| 攻击面 5 未定项清单 | **打中** | 16 条没有去向的，含 r2 判决说要登而没登的攒批一条 | 现查 |
| 攻击面 6 数字 | **打中（轻）** | 「变异都证过红」窄了 12 行；任务 11334 退 1；`29 轮` 的日志不在了；代码注释「全部取样点」宽了 | 现查 |
| 检查单：走不到的理由 | **成立** | `grep` 0 处；删掉的那一支读码走不到 | 现查 |
| 检查单：类型与名字 | **成立**（文档一处宽） | 老名字全 `crates/` 0 命中；加行没有新 `expect` | 现查 |
| 检查单：不兼容旧代码 | **成立** | `grep` 0 命中 | 现查 |
| 检查单：不新建判定库 | **成立** | 产品代码不写判定；新用例判定走给定状态引擎 | 现查 |
| 检查单：多机可用 | **成立** | 新目标没有读本机状态的写法（`grep` 0 命中） | 现查 |
| 检查单：测试优先多线程 | **成立** | 单历史逐次判，用例注释写明次序就是被测对象 | 现查 |
| 检查单：带 --item | **不成立（窄）** | 任务 11017 确是 cancelled；「变异行都走 prove-red.sh 证过」差 12 行，其中 4 行改了锚点 | 现查 |
| 检查单：新崩溃流 | **推不出，算打中（半格）** | GPU 缩域流多罩成立；坏盘 × 部分落盘只判 `recover`，可写挂载那半推不出 | 现查 |

## 攻击面 1：P2a 的前提

### 打中之前的四句（「仍撞 64」这一格）

- 分不分辨臂：分辨 P2a 与「把上界换成算出来的」或 P3 那一类；不分辨 P1 与 P2a（P1 同样走不完，只会更早）。
- 系统当时看得到吗：看得到，空闲图与重写集都在内存。
- 满足判据字面哪一句：E206 登记「上界 64 照留」与代码文档「这一半的上界没有推出来」都承认这一格，打中的是「E206 全部取样点量到至多 5 轮」被当成这一半的代表。
- 跑前条款的改法在这一格上还中不中：P2a 不碰取落点这一半，这一格上不起作用（推的）；P2b 定期整棵重写同样要落点，也不起作用（推的）。

### 机制（读码，推的，模型待跑）

- 取落点：「最低的全空段（开新段），没有全空段就回落到槽号最小的空槽」（`crates/singlefs-core/src/allocator.rs:2570`）。
- 一片叶 812 槽：「pub const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;」（`crates/singlefs-format/src/lib.rs:163`）。两盘同槽，碰到一片新叶，下一轮多重写两片叶（两块盘各一），又多要两个落点。
- 记一轮多出来的落点数 Δ，空闲槽每片叶 F 个：下一轮 Δ' = Δ × 2 / F。F = 2 不衰减，轮数 ≈ 2 × 片数 ÷ Δ，走到稠密的那一段才定；F = 1 指数涨，很快盖满那一段；F ≥ 3 几何衰减，几轮定（下表「推的」）。
- 代码文档自己写了这一半没有上界：「所以每次并进重写集都一次走到闭包，释放链不占轮数，轮数只剩取落点碰到新叶那一半。这一半的上界没有推出来，E206（发布固定点轮数的代价）全部取样点量到至多 5 轮。」（`crates/singlefs-core/src/transaction.rs:13773`）。
- E206 没量过这一格：`grep -l fallback research/results/e206-fixed-point-cost-*.out | wc -l` 是 0（量的）；16 个点都是一路顺序写、盘离满很远（登记的 unit_area 与文件数）。「至多 5」只代表有全空段的那一格。
- 撞上界的后果：`settle` 排在合取 7 之前（「let settled = settle_the_allocation_record_tree(」`crates/singlefs-core/src/transaction.rs:13255`，「if empty_cluster_segments_conjunct_is_judged {」`crates/singlefs-core/src/transaction.rs:13292`）；这个错不算空间不够，会话不推抬 F（「PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,」`crates/singlefs-core/src/mounted_session.rs:2038`）。unlink、整理这些合取 7 豁免的发布走同一个固定点，要腾空间的那几步一样被拒（推的）。

| F（每片叶空槽） | 片数 L | Δ | 轮数（推的，没量） | 撞 64 |
|---|---|---|---|---|
| 2 | 64 | 2 | ≈ 64 | 压线 |
| 2 | 120 | 2 | ≈ 120 | 撞 |
| 2 | 120 | 25（没有前缀时约等于这次的角色数） | ≈ 10 | 不撞 |
| 3 | 任意 | 任意 | 差按 2/3 衰减，≈ 对数轮 | 不撞 |
| 1 | 任意 | 任意 | 差翻倍，≈ 对数轮，但全部片一次重写 | 不撞 |

### 给的历史

**盘面（模型第一段 forged）**：空闲图直接改，只改落点去处、不写分配记录，造法同 `crates/singlefs-verification/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs`；一个全空段都没有，第 12 片叶开头一段稠密前缀，之后 L 片叶每片 F 个连着的空槽，再往后每段留段首之外的槽。扫前缀 0..160，再在轮数最多的前缀上扫 F × L。**这是机制的量，不是可达的量**：盘面没有哪段用户操作造出来。

**用户操作能不能到（模型第二段 natural，推的）**：一个单元的小文件写到被拒，隔 k 个删一个（k = 2、8、64），再随机覆盖写，隔 200 次探一次，打每片叶的空槽数、全空段有没有。我预计这段历史上 F 不均匀、走不到 64：冷的单槽节点（extent 上段、分配记录树叶）占掉槽对的一半，留下单个空槽，F 大于 2 的片会让差衰减（推的）。要造严格的 F = 2 要先把低端压实（第一遍写、重挂载再写，让数据单元填掉上一遍的聚簇段），再靠活快照下覆盖写加删快照一次放出每片叶一个槽对；这条路我写了步骤没实现。

**什么现象推翻它**：forged 扫完 `did_not_settle=0` 且 `most_rounds` 不到 16（机制错）；或 natural 全程 `free_of_each_leaf` 没有连续几十片小于等于 3（这一格只有造出来的盘走得到，那就只是 D3 已定项 8 ② 的条款空白，不是可达缺陷）。

### 没打中：闭包多写（并进 P1 不动点之外的节点）

逐步查了登记那句「被并进去的节点都在 P1 的不动点里」要的三件事：

1. 重写集里的节点旧落点一定释放：「let replaced_previous_nodes = previous_nodes」（`crates/singlefs-core/src/allocation_record_tree.rs:446`）那一段写的是重写的、这一版不再有的都换下，所以 `rewritten` 里掉出 `nodes_after` 的节点照样释放。闭包不看 `nodes_after`，这里对得上。
2. 读的指针就是释放用的落点：释放走映射，闭包读 `previous_tree.pointer_of`。两边相同靠 `take_the_mapping_positions_over_the_location_hints`：「for (_, pointer) in &mut version.allocation_record_tree.nodes {」（`crates/singlefs-core/src/transaction.rs:3937`）在重建与搬迁那一版（「take_the_mapping_positions_over_the_location_hints(&mut relocated_version, |key| {」`crates/singlefs-core/src/transaction.rs:15608`）都换成映射位置。分配记录树节点被整理搬过之后这个指针是新落点，不是提示。只供测试的 `SourceReleasedWhileTheMappingStillNamesItForTests` 那一档不算。
3. 上一版的取法：「let previous_tree = previous.map(|previous_version| &previous_version.allocation_record_tree);」（`crates/singlefs-core/src/transaction.rs:13817`），回退那次是拼出来的上一版，分配记录树取现行那一版（`mount.rs` 回退拼上一版那一段），与释放同一份。

读码另看到一处「写的角色与字节不变」的例外，方向是少写不是多写（推的，没造）：一批搬迁里点名了分配记录树节点 n，n 没进重写集时搬 n 要取一个目的地落点；P1 走到 n 进重写集之前，那个落点弄脏的叶已经并进重写集、留在里面（「多罩」），闭包让 n 提前进重写集时那片叶不再被弄脏。所以 P2a 的重写集可以比 P1 小。登记的 Q5 是 15 个点、都不发整理批（`grep -c 'ratio=1.0000 …'` 15，E206 页），新目标也只有覆盖写，这一形两边都没量。影响：崩溃用例钉值若有落在这一形上的，会变。**什么现象推翻**：入库的整理批钉值在整表崩溃点重放时全绿。

没扫的：P2a 与 P1 在别的形状上逐个比重写集（第二段 33 项没跑，主 agent 已定不跑；我也没写这个对照）。

### 打中（轻）：写行那一份两处并入的盲区

实现员写「造不出轮数分得开的历史」（`~/.local/share/singlefs/work/m4-close-fixes-r2-writer/result-reds-and-closure.md` 第 66 行）。走得到的历史（推的）：只做 mkfs、反复可写挂载不写第一个文件（每次写一行实例表 COW 整条链）。`tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs` 已经挂载 60 次；挂载次数够多、实例表链跨几片叶时，上一版的分配记录树节点旧落点在别的叶上，释放链长出来，不走闭包多几轮。轮数读得到（写行那一路同样 `.set(settled.rounds_walked)`）。**不改判定，只要补一行变异**：这一处照 `.claude/rules/mutation-sampling.md@第三类不许当等价变异留档` 不当等价留。

## 攻击面 2：映射 key 集合不等就拒

### 没打中：合法的盘（读码，一次抽样，模型待跑）

新判接在条数那一判之后（「mapping_entry_count_is_one_per_mapped_unit(」`crates/singlefs-core/src/recovery.rs:3607`，「central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots(」`crates/singlefs-core/src/recovery.rs:3637`）。所以只有条数已过而集合不等时才多拒，要造合法盘被拒得让写路径写进映射的 key 与读路径从指针算出的 key 不同，而条数碰巧相等。逐形查：

| 状态 | 查了什么 | 判 |
|---|---|---|
| 留存条目 | 读路径 key = 这一版的单元 ∪ deadlist 的 key；deadlist key 查不到更早就拒（`mapping_entries_of_the_version`）；一把 key 同时在单元与留存里，条数先不过 | 成立 |
| 搬迁中崩溃 | key 看出生（树、txg、实例、序号或写序），不看落点（`records.rs` 的 `mapping_key_for_node`）；搬迁不改 key；一批是一次发布，根在才算这一版 | 成立（推的，崩溃态没枚举） |
| 回退之后 | 回退那次的上一版 `mapped_units` 取目标的用户可见单元，映射按它重装；两版都不带快照才许回退（`mount.rs` 回退拼上一版那一段的注释） | 成立（推的，模型 `rollback`、`compaction_rollback` 两组比重扫） |
| 快照销毁 | 一批放的条目连映射条目与 deadlist 条目一起去掉 | 成立（推的，模型 `snapshots` 组扫） |
| 整理半途 | 意图在 logged_ops，`LoggedOperationsRoot` 进 `mapped_units` | 成立（推的，模型 `compaction` 组扫） |

**模型**：十六组比重、两种盘宽，每一步之后把根环里读得出的每条根重建一遍，数拒在这一判的根；历史自己的可写挂载被这一判拒的另计。**没枚举崩溃状态**，理由：这一判只看一条根封住的内容，崩溃只决定落到哪条根，发过的根都判到（环轮出去之前）；反过来的现象（只有崩溃态才有的根）判不到。全量域崩溃枚举归提交时那一轮门禁（任务 11334 只跑了 177 个态，`closed_form=123338919`）。

**什么现象推翻**：模型 `key_set_refusals` 非 0，或全量崩溃枚举红在 `CENTRAL_MAPPING_KEY_SET_DISAGREES_WITH_THE_MAPPED_UNITS`。

### 没打中：GPU 判器少判的一形

「一版里两个单元同一把 key 而映射多出一条别的 key 的那一形这里判不出」（`crates/singlefs-verification/src/crash_judge_recovery.wgsl:846`）。崩溃枚举的盘面都是合法写的前缀与子集，读路径 key 由指针算、写路径 key 由出生定，这一形要写路径本身错才有，所以它只在产品缺陷叠加时才让 GPU 与 CPU 不一致，那时对拍照样报不一致（CPU 判拒、GPU 不拒）。不算漏判。

### 只读路径不判，留下什么

- key 被换：提示新鲜时读得出；提示过期时报 `MappingMiss`（实现员读码，我现读 `recovery.rs` 那句同义：「只读路径（冷走读、`read_at`）不判」`crates/singlefs-core/src/recovery.rs:6870`）。没有错读。一把 key 换成另一个单元的 key 的形状我查的是单元头核（D4 第 71 行要出生 txg 与写序相符），推的。
- **留一形（零轮，交用户表）**：两把 key 的位置对换、key 集合相等、两道校验和都对、根重封。这一判过得去；重建把这两个单元的指针换成对方的落点（`take_the_mapping_positions_over_the_location_hints`）。这一形与「key 被换」同样要造坏的盘。模型 `a_disk_with_the_positions_of_two_keys_swapped_passes_the_key_set_judgement` 只观测挂不挂得上、读出什么，不钉结论。用户定的是「映射 key 被换的盘挂载时拒」，对换不在字面里。

## 攻击面 3：读全部接上之后

### 打中：`transaction.rs:668`（现行号 `:679`）的依据不成立

穷举表原话是「`PoolWriter` 手里没有分配器，账够不着。要接得给写入口加一份账、改每个 `PoolWriter::new` 的调用方」。两句都不对：

- 写入口已有同形的账：「mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell<Vec<Vec<u8>>>,」（`crates/singlefs-core/src/transaction.rs:376`），新的字段在 `PoolWriter::new` 里初始化，调用方不用动，发布之后由会话并账。
- 「通常紧接着的 discard 或推抬 F 也读，但不保证」说小了：「let pool_effective_floor = effective_rollback_floor_rereading_unreadable_reads_once(」（`crates/singlefs-core/src/transaction.rs:679`）在 `RotateSystemConfigurationSlots` 里每块盘一次，**每次发布都读**最新那条根的实例表，是可写挂载里最常发生的一读，没有一个读数记它。用户定案是「按字面全接上」（r1 判决第三节）。

**分辨的历史（模型 `the_rotation_read_of_the_instance_table_reads_the_damaged_copy_and_nothing_counts_it`）**：挂载之后弄坏现行实例表一片的头一份，发一次建 inode。期望（推的）：盘层数到那一份上发过读（`reads>0`），`other=0`，结局 `NotNeeded`，那一份还坏着。

四句：分辨「按字面全接上」与「只接判决点名的」；系统看得到（读者报得出）；字面分句是 D4 第 73 行「可写挂载里的读都计数并搬」的「可写挂载期间别的读」；改法「写入口加账」在这一格上修得到（**只在我读码上推的、被攻过零轮**）。

### 打中：`mount.rs:1407` 一带的依据不成立（`:1767`、`:1809`）

穷举表把这八处归成「挂载这一串建分配器、算影子账读各根」「这几处读的是别的版本，读到的也没东西可搬」。

- `:1767` 不只挂载这一串：「match isolate_slots_referenced_only_by_abandoned_roots(」（`crates/singlefs-core/src/mount.rs:5484`）在 `raise_the_floor_through` 里，会话抬 F 每次都走。
- 读的不是「别的版本」：「if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {」（`crates/singlefs-core/src/mount.rs:1767`）循环的是候选根，现行那一版就在里面；`placements_referenced_by_root` 带树的那一支读 `allocation_records_under_root_in_the_unit_area_starting_at`（`crates/singlefs-core/src/mount.rs:2826`）。读到的是现行那一版的分配记录树节点，它进映射，**有东西可搬**。
- 同一个节点在回退那一路（`crates/singlefs-core/src/mount.rs:10798`）算数、在抬 F 这一路不算，两处口径不一。

**分辨的历史（模型 `the_floor_raise_reading_the_account_of_the_current_version_reads_a_damaged_leaf_and_nothing_counts_it`）**：400 个单元的文件，建 8 次 inode；空跑一遍取一片抬 F 那一串没动的叶；重来，抬 F 之前弄坏那片叶的头一份，`raise_rollback_floor_to_the_admission_ceiling`。期望（推的）：`reads>0`、`other=0`。之后再发一次改动看叶换没换（只打印）。

我只核了 `:1767`、`:1809`、`:2826` 这一条路。`:1407`（`rebuild_previous_version`）与 `:3128`（`rebuilt_allocator`）读码确在挂载这一串；`:1998`、`:2170`、`:2241`、`:2525` 没逐个核谁调，算「没核」。

四句同上一条；改法「抬 F 重算影子账那一读套数只坏一份的那一层」只改 `:5484` 这一路的读者（**推的，被攻过零轮**）：`isolate_…` 的 `devices` 参数同时给 `:3178` 挂载那一路用，要分开。

### 打中（轻）：接了线而没有变异行的六处

穷举表与派发提示用的是改之前的行号，下面写现行号。依据里「要在两读之间弄坏才分得开」只对不注入读故障的造法成立；盘包一层按第 n 次读弄坏（`DeviceFailingOnRequest` 同形）就分得开，所以都不算等价（`.claude/rules/mutation-sampling.md@第三类不许当等价变异留档`）。

| 处（提示里的号 → 现行号） | 能分辨的输入 | 量的还是推的 |
|---|---|---|
| `mount.rs:10901` → `:10910` 水位 | **不用注入**：目标与现行之间那一版独有的数据单元弄坏一份，回退做成，别的读那个数 1，只有这一读读到 | 模型 `only_the_watermark_read_of_a_rollback_sees_a_unit_of_a_root_between_the_target_and_the_current_version`，推的，待跑 |
| `mount.rs:10789` → `:10798` 读现行账 | **不用注入**：现行账一片没人动的叶弄坏一份，再弄坏目标一个复活单元一份，拒在 `:10846`（「every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;」`crates/singlefs-core/src/mount.rs:10846`），走不到水位那一读 | 推的，没写模型 |
| `mounted_session.rs:4115` → `:4490` 回退门 | 整理一趟中途发布写了新树表，入口那一读（`:3319`）没读过它；按读次序弄坏新树表一份 | 推的 |
| `mount.rs:5350` → `:5357` 抬 F 算生效值 | `:5340` 与它读同一条链，第二读之前弄坏 | 推的 |
| `instance_table_sweep.rs:743` → `:742` 清扫发之前 | 判与发之间弄坏 | 推的 |
| `instance_table_sweep.rs:921` → `:920` 挂载末尾 | 挂载写行之后那张新表读的时候头一份读坏 | 推的 |

## 攻击面 4：账的口径与 D4 已定项 4

**打中（轻）**：

- 条款字面：「那一批搬没搬成都只试这一次，结局随那次改动报出，之后的用户改动不再接着搬」（`.claude/kb/decisions/04-校验和位置.md:73`）。
- 代码现状：「同一组之后再被读到就再记、再试一次」（`crates/singlefs-core/src/mounted_session.rs:228`）。挂载读到的单元那一批被拒，之后经会话读文件再读到，下一次改动就接着搬，这正是条款写的「不再接着搬」。r2 判决要的是「文档收窄成写实的那一句」，代码文档收窄了，条款没动：`git diff --stat -- .claude/kb/decisions/04-校验和位置.md` 空（量的）。这一格要么改 D4 字面，要么改代码；r2 判决第五节写「kb 写回随这一批收尾」，这一批打进主工作区了，kb 还没动。
- 同一行另一句也过期：「它们在挂载这一串的发布里本来就换落点」（`.claude/kb/decisions/04-校验和位置.md:73`），r1 就判过打中。
- 去重范围：`.contains(&read_locations)`（`crates/singlefs-core/src/allocator.rs:2022`）按两条位置条目连校验和比，一组等下一次改动的这一程只算一次。D4 字面是「读到只坏一份时计数」，没说一程只算一次；这是实现的读法，文档写明了，kb 写回要带上。
- 口径统一那一句本身：数据单元与映射树叶现在同是 `(1,0,0)`（任务 11331 里 `the_other_reads_count_…` 过，量的）。没打中。

## 攻击面 5：未定项清单

r1、r2 判决与实现员「停下交主 agent」里，下面这些我没查到去向（查的是这三份报告、两份判决、`.claude/kb/checks-owed.md`、D4）。

| # | 条目 | 出处 | 现状 |
|---|---|---|---|
| 1 | 写入口轮换那一读不接 | `result.md` 第 100 行 | 实现员交主 agent，r3 正文列为攻击面，没定 |
| 2 | 抬 F 影子账那一读不接（`:1767` 一带） | `result.md` 第 101 行 | 同上；实现员的依据不成立 |
| 3 | 固定点走满 64 的结局 | `transaction.rs:13773` 下一段「条款没写这一格怎么办」 | 没有欠账号；回落这一格 E206 没量 |
| 4 | 攒批读不到待发写 | r2 判决第五节「登 `.claude/kb/checks-owed.md`」 | **没登**：`grep -n '读不到待发\|读自己待发\|读待发' .claude/kb/checks-owed.md` 0 命中（量的）。我的看法：进条款，至少登 |
| 5 | D4 第 71、73 行、C946 写回 | r1、r2 判决 | 没做：C946 还写「一份读错就读另一份，不计数」（`.claude/kb/checks-owed.md:589`）；D4 欠账还写「这两样未实现」（`.claude/kb/decisions/04-校验和位置.md:77`） |
| 6 | 「重试尾巴」（抬 F 的空发布已经换下记着的叶，结局仍报进 `units_exempt_from_the_mapping_replaced`） | `result.md` 第 245 行 | 没去向 |
| 7 | 多片实例表链的用例 | `result.md` 第 246 行；D4 第 73 行「多片实例表链里不被改写的那几片没核」 | 没入库 |
| 8 | 坏盘 × 部分落盘只判 `recover`，可写挂载不判 | `result.md` 第 57 行 | 「派发提示不碰那一路在定」，没查到定了没有 |
| 9 | `RefusedNotForSpace(InstanceTableOfTheVersionUnreadable)` 那一形是实现员定的 | `result.md` 第 35 行「这一处是我定的，主 agent 不同意就改」 | 没有判 |
| 10 | 写行那一份闭包没证红 | `result-reds-and-closure.md` 第 66 行 | 本报告攻击面 1 |
| 11 | `files_1000_units_100` 不再撞 64 没有用例，靠测量补丁上的数 | `result-reds-and-closure.md` 第 163 行 | 没去向 |
| 12 | 闭包每跳线性查、`q5_net_steps` 小点 `costs_more` | `result-reds-and-closure.md` 第 164 行 | 「要不要换归之后定」，没落点 |
| 13 | 水位、读现行账、判候选三读没有单独变异行；第 7112 行同一个变异 | `result-unkilled-rows.md` 第 81、82 行 | 没判 |
| 14 | 改了锚点的 4 行变异没证红 | 见攻击面 6 | 归整表复跑，整表任务 11017 取消了 |
| 15 | 全量域 GPU、不指定范围的门禁没跑 | r3 正文第二节 | 归提交时，正文写了 |
| 16 | 映射位置对换、key 集合相等 | 本报告 | 零轮，交用户表 |

## 攻击面 6：数字能不能核

| 正文或页上的数 | 核法与结果 | 判 |
|---|---|---|
| clippy 任务 11330 退 0 | `task-server/task status 11330`：「结局 succeeded、退 0」 | 对 |
| `unit_read_…` 37 条过、`crash_points_…` 4 条过 | 任务 11331 日志：`test result: ok. 37 passed`、`test result: ok. 4 passed` | 对 |
| `states_judged_by_the_gpu_here=182` | 任务 11331 日志同名行 `::states=182 … states_judged_by_the_gpu_here=182` | 对 |
| P2a 目标 1 条过 | 任务 11332 日志 `test result: ok. 1 passed`；没带 `--nocapture`，`SETTLING` 行 0 行，轮数读数在这一趟看不到 | 对，但只证绿 |
| 缩域 `states=177 … exhaustive=false violations=0` | 任务 11334 日志那一行逐字有。**任务结局 command-failed、退 1**，门禁判红「不带 exhaustive=true」；正文写「不算门禁通过」，意思对，没写退 1 | 对，少写一句 |
| 变异在副本上都证过红 | 四个任务的日志合起来 43 行、33 行追加全部末次「抓到」（脚本比，量的）。**替换的 17 行里 12 行不在这四个日志里**：8 行开关改名，4 行改了锚点（`实二二三 h`、`合入后验证一 Q4`、`RGall`、`C724`）。任务 11220、11288 本身退 1（有没红的行），由 11311 补齐 | **窄了 12 行** |
| 「释放处不走闭包」红在 `覆盖写 inode 1 的固定点走了 29 轮` | 任务 11220 日志只写「红了（日志 …/003.log）」；那份日志路径现在 `No such file`（量的）；只剩 `records/2026-10-13-里程碑四预开发.md` 第 1581 行主 agent 的转述 | **核不动** |
| E206 P2a 峰值至多 5、16 个点 | `grep 'name=q1 item=.*__p2a' research/results/e206-fixed-point-cost-report-2026-11-09-r2.out`：16 行，3 的 10 行、4 的 5 行、5 的 1 行（`files_30000__p2a`） | 对 |
| 第二段 33 项没跑 | E206 页现状一节写明，登记第五节数下来 33 | 对 |
| 代码注释「E206 全部取样点量到至多 5 轮」 | 登记的取样点还有第二段 12 个点没跑，「全部」只是第一段 16 个点；回落这一格一个点都没有 | **宽了** |
| 新目标注释用 P1 读数当依据（「不走闭包的那一臂量到 29 轮、重写 99 个角色、87 个节点」） | E206 页写 P1 逐轮读数作废、只作线索。99、87 是用例钉的值，用例在 P2a 上绿，这两个数本身量到了；注释的来历写的是作废的那一路 | 轻 |
| E206 页 `transaction.rs:13466` | 上界常量现行号 `:13953`：页上写的是当时的号，这批改动挪了行 | 回看时改 |

## 第四节检查单逐格

- **走不到的理由二次查实：成立。** `grep -nE '^\+.*(todo!|unreachable!|assert!|assert_eq!|panic!|\.expect\()' research/prompts/_m4-close-fixes-code-r3-diff.md` 0 行，`Undecided|Unsupported|OutsideTheFirstVersion` 加行 0 行（量的）。删的那一支：`publish_the_change_raising_the_floor_for_space` 各臂要么拒（走不到 `relocate_…`，那里是 `published?`），要么 `publish_the_change_once` 发成一次，每次发布换树表；这一批推抬 F 新读到的树表归下一次改动，下一次改动又换。读码，推的。
- **类型与名字：成立。** 老开关名、老函数名全 `crates/` 0 命中（`grep -rn` 量的）。新名字说得清。文档宽了一处，见攻击面 6。
- **不兼容旧代码与旧格式：成立。** `grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-fixes-code-r3-diff.md` 0 行（量的）。
- **不新建判定库：成立。** 产品代码不写判定；`violated_invariants_of_the_image` 走给定状态引擎。
- **所有任务多机可用：成立。** 新目标 `grep -nE 'env::|/home|/tmp|hostname'` 0 行（量的）。
- **测试优先多线程：成立。** 用例注释写明一个场景、次序是被测对象；我没找到能拆的部分。
- **带 --item 的命令与项：不成立（窄）。** 任务 11017 `结局 cancelled、退 239`（量的）；差的 12 行见攻击面 6。
- **新崩溃流多罩了什么：推不出，算打中（半格）。** GPU 缩域流：任务 11307「跑了 4 条，抓到 4 条」，查映射与下界两处真多罩（量的）。`crash_points_of_the_repair_…` 的 182 个态 × 不放回字节：真多罩（量的 182，判的只有 `recover`），用例头写 `crash-case-check:not-a-crash-case`，不进枚举用例；**可写挂载在坏盘 × 部分落盘上走不走得通，这条流答不了**。

## 几个改法各修哪一格

都**只在我读码上推过、被攻过零轮**，交用户时照写。

| 改法 | 修哪一格 | 标注 |
|---|---|---|
| 写入口加一份只坏一份的账（同 `:376` 那一份），会话发完并账 | `transaction.rs:679` 不计 | 推的，没实现没跑 |
| `:5484` 这一路的读者套数只坏一份的那一层，读完记账 | `mount.rs:1767` 会话那一路不计 | 推的，没实现没跑 |
| 回落时固定点走满：把错报成空间不够让会话推抬 F | 对 F = 2 这一格不起作用：抬 F 不改空闲形状，只多放些槽（推的） | 推的，没实现没跑 |
| 回落时取落点按「已在重写集里的叶优先」 | 取落点那一半不再碰新叶，轮数收敛，代价是落点政策改、D3 已定项 8 ② 字面要改 | 推的，没实现没跑；要走条款 |
| 上界照留，走满时登条款与欠账 | 不修，只让这一格有去向 | 推的 |

## 没打中的形状

- 闭包多写：查了丢节点（`rewritten` 掉出 `nodes_after`）、被搬过的分配记录树节点（指针是不是提示）、回退拼的上一版、树表 0 条那一版（`tree.version` 不经映射，指针就是落点）、复用窗口置 0 开关（只供测试）。没造历史，读码。
- 映射 key 合法盘：五种状态读码，模型十六组比重 × 两种盘宽 × 至多 24 个种子 × 60 步，待跑。
- 读路径错读：key 换成另一个单元的 key，只读了单元头核那一判，没造。
- 账的去重：同一组两份轮流坏、跨挂载、分配器拷贝共用账（抬 F 预演、回退换回）：账各份拷贝共用、回退换回不退账，文档写明了，没发现数错；可写挂载取号之前的预演也会经共用的账记数（`establish_instance` 里只进内存那一趟），一次挂载里至多把挂载那串的抬 F 读记一次，没造。
- 只坏一份的计数与 `read_at` 冷走读：不在这批 diff 里，没看。

## 这条腿自己的限度

- 只抽一次：「没打中」的格都不支撑结论（`.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`）。
- 模型全都没跑。forged 那一段的盘面不可达，它只量机制；natural 那一段可能根本走不进回落（打印会说）。严格 F = 2 的可达历史写了步骤、没实现：压低端要几遍重挂载，punch 要活快照，大约要 2.6 GiB × 2 的内存盘（推的）。
- 没有在产品代码上比 P1 与 P2a 的重写集，只在读码上说整理批那一形可能不同。
- 崩溃状态一个都没枚举：攻击面 2 的理由写在那一节，这是推的，全量域归提交时的门禁。
- 引文用现行号；派发提示与穷举表的旧号在表里写了对应。
- 没查 D3 已定项 8 ② 全文与 D16 全文，只读了代码注释转述的那一句。

## 要主 agent 跑的命令

1. 复跑模型（整份）
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-fixes-code-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r3-opus/copy --work /home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r3-opus/rerun-1`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。`--frozen` 那一份是 `minimal-copy.py --with-test-inputs` 建的（52M，量的），`.deliverables` 写了交接。要拷到别处复跑，模型目录整份拷、`--frozen` 仍指这份。`--work` 给新目录：`rerun_steps` 每次尝试写在 `rerun-1.attempts/attempt.*` 里、跑完才把日志与观测拷进 `rerun-1`，所以服务撞内存上限重起同一条命令时 `--work` 还是空的，模板不拒（账本二百零三节那一条；没改模板，这一形只靠 `rerun_steps`）。没跑完被停的看 `rerun-1.attempts/` 最新那一个。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=`；日志 `name=attack_fixes_r3_other_reads_target rc=… expected=0`、`name=attack_fixes_r3_legal_disks_target rc=…`、`name=attack_fixes_r3_settling_fallback_target rc=…`、`name=attack_fixes_r3_summary target_as_expected=…`；`attack-fixes-r3:` 观测行在 `rerun-1/observations.txt`；`rerun-1/*.log` 看 `error`、`panicked at`。复跑完 `rerun-1.attempts/` 留着日志，里面的 `frozen-tree` 已删。退 0 只说断言照期望，第三个目标只量：撞没撞看 `fallback_summary` 的 `did_not_settle` 与 `most_rounds`、`natural_summary` 的 `most_rounds`。
   - 期望观测（推的）：`rotation_read reads_of_the_damaged_copy>0 other=0 outcome=NotNeeded still_damaged=true`；`floor_raise … reads_of_the_damaged_copy>0 other=0`；`watermark_read rolled_back=true other=1`；`legal_disks … key_set_refusals=0`；`swapped_positions` 只观测；`fallback` 每片两个空槽那一行在前缀扫到差很小时 `outcome=did_not_settle_limit_64`，F ≥ 3 不撞；`natural` 不预测轮数。
   - 挂钟（推的，没量）：编验证包一次加三个目标，几分钟到十几分钟。`legal_disks` 先跑一个种子打估时（`estimate` 那一行），按 40 分钟预算定种子数；`fallback` 自带 20 分钟预算、`natural` 每段 15 分钟预算，到了就停并打印停在哪。内存（推的）：`fallback` 单元区 2¹⁸ 槽但几乎没写数据，每次探拷一份盘，几十 MiB；`natural` 三段各约 1 GiB。
   - 编译错、断言红都交回：新派同立场的腿，「承接：research/prompts/m4-close-fixes-code-r3-opus-output.md」带任务号。`floor_raise` 红在 `reads>0` 说明我读错了那条路，这一格撤回。

等这条结果才能往下做的：把「推的，待主 agent 跑」换成量的；`did_not_settle` 非 0 时写 natural 的形状离它多远；`key_set_refusals` 非 0 时逐条查那条根。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数；没往统一任务服务交任务，判决要引的数都没复跑（命令在上一节）。
- 没改产品代码；模型只在 `research/prompts/m4-close-fixes-code-r3-opus-model/`。
- 没跑门禁；没枚举崩溃状态；没走第二次抽样。
- 删编译目录：没建。没删 `/home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r3-opus/copy`：交接给上面那条命令当 `--frozen`。

## cite-check

`python3 scripts/agents/cite-check.py research/prompts/m4-close-fixes-code-r3-opus-output.md` 末行（量的）：

```text
  ✓ 核了 16 处引文、25 处裸行号，对上 16 处，没判 25 处（其中裸行号没有引文可比 19 处；逐处列在上面）
```
