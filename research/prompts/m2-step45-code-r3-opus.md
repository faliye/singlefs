你是里程碑「第二个事务」步 4 / 步 5 代码三方对抗**第三轮**（轮名 m2-step45-code-r3）的**云端攻方腿**。第二轮打中之后主 agent 改了两处（判决 `research/prompts/m2-step45-code-r2-main-verification.md` 第五节 1、2）：T1 影子账改成 G5（豁免只给候选集里的根、每次挂载算、抬 F 之后按新 F 在回收之前再算一次）；T2 被抛弃根的账读不出时跳过并计数、不拒挂。alloc-basis 第二轮的云端攻方腿（`research/prompts/alloc-basis-r2-opus-output.md` 2.1 节）又打中抬 F 自己的空发布把刚回收的槽发了出去，改了第三处：T3 抬 F 回收的槽扣住到带新 F 的根落满每块盘才放开（`ReclaimedReuse::HeldUntilFloorTakesEffect`、`DeviceFreeMap::hold_until_floor_takes_effect`、`PoolAllocator::release_reclaim_holds`），记账仍在第一条带新 F 的根之前动；没做过可写挂载的进程抬 F 报错不 panic。这一轮**只攻这三处改法本身**——问「改法有没有改对、有没有引进新的错」，不许重复前两轮攻过的角度（前两轮攻方报告 `m2-step45-code-r1-opus-output.md`、`m2-step45-code-r2-opus-output.md` 可读，用来避开）。

## 先读

1. 背景材料 `research/prompts/_m2-step45-code-r3-background.md`（正文 T1 / T2 / T3 改法表、X1–X8 判据、分工、禁读清单；第二轮判决全文；diff；附录一条款整段）。引条款从附录抄整行，行号写 kb 文件自己的（去 kb 文件里现查）。
2. 自己去读代码（原仓只读）：`crates/singlefs-core/src/mount.rs`（`isolate_slots_referenced_only_by_abandoned_roots`、`rebuilt_allocator`、`raise_rollback_floor`、`mount_rollback`、`establish_instance`）、`allocator.rs`（`isolate`、`mark_reclaimed`、`reclaim_released_up_to`、`is_free`、`lowest_empty_segment`）、`recovery.rs`（`allocation_records_under_root`、`readable_roots`、`choose_root`、`effective_rollback_floor`）、用例 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`second_transaction_step_five_reuse.rs`、`second_transaction_step_zero_layer0.rs`（固定脚本到 E）。
3. 要改代码跑用例，把仓拷到你自己的目录 `/tmp/claude-1000/m2-step45-code-r3-opus/copy/`（`rsync -a --exclude target --exclude .git`），在副本上改、在副本上 `cargo test`（副本里 `nice -n 19`）；副本上的数照实报、注明是副本。草稿也放 `/tmp/claude-1000/m2-step45-code-r3-opus/`。

## 分到的攻击面（只攻这几格；X4、X5 是本地攻方的，不碰）

- **X8 T3 扣住**：扣住只住内存、只在抬 F 那个进程里。崩在两条带新 F 的根之间再挂载：重建用 F_生效（旧 F）回收，那些槽回到「已释放、未回收」，新实例的根写 F_生效——这时 checker 的 I-3.1 按最新根（带新 F、只落一块盘）自己的 F 取并集，而那条根的记账行已经把回收算成空闲，两边说的是同一件事吗（在副本上崩了跑 checker）；扣住之后一次空发布分配不到固定点（开放段满、没有别的全空段——盘快满时）会怎样：`allocate_commit_generated` 返回 `None` ⇒ 发布失败 ⇒ 抬 F 半途而废、F 只在一块盘上、扣住的槽没放开；`release_reclaim_holds` 放开的时机是循环之后——循环因为 `ROOT_RING_REGIONS` 上限退出而没落满每块盘时也放开，够不够得着；扣住位与隔离位、分配位三者在 `is_free`、`lowest_empty_segment`、`lowest_user_data_slot`、`mark_reclaimed`、`record` 上的每一种组合。

- **X1 T1 的隔离集**：豁免集 = 当前账未释放的记录 ∪ 候选根（可读 ∧ 按实例表有效 ∧ txg ≥ F）账里未释放的记录。找一段可达历史让一个只被被抛弃根引用的槽被 `is_free` 判空闲（豁免算宽了）：例如一个槽在当前账里未释放、但当前账继承自 R_old 而 R_old 之后被抛弃的根释放过它又……；或让一个候选根引用的槽被隔离（豁免算窄了）：候选根的账读不出「当它什么都不豁免」、或候选根是最新根自己施加记录之后 effective_root 的账与盘上根的账不同。释放代与 F 的关系：一个槽的释放代 ≤ F 被回收、同时它被一条 txg ≥ F 的被抛弃根未释放地引用——G5 隔离它吗、回收之后 `mark_reclaimed` 不看隔离位、`isolated_slots` 还算它吗、D28 第九项的定义算它吗。
- **X2 T1 的时机**：抬 F 的重算在回收之前、在带新 F 的空发布之前，两次空发布之间可以崩。崩在第一次空发布之后（盘 0 有带 11 的根、盘 1 没有）再可写挂载：F_生效 = 0，重建的隔离集按候选集 txg ≥ 0 算（A 又是候选、50176–50177 豁免），而抬 F 那个进程已经把释放代 ≤ 11 的落点回收过——回收只在内存里，重建从 R 的分配记录重来，那些槽在新账里是「已释放、未回收」还是别的；新实例的根写 F_生效 = 0；之后再抬一次 F 到 11 会不会再回收一遍、隔离集怎么变。`raise_rollback_floor` 里 `oldest_valid_root` 与 `abandoned_by_table` 按 `current` 的表判、重建按最新根的表判——两张表恒同一份吗（`current` 是这个进程最后一次发布的版本；有没有别的进程写过盘）。
- **X3 T2 跳过的根**：一条被抛弃根的树表两盘都坏、根槽自证通过、还在根环里：`choose_root` 会不会选中它（最新根是它的时候：回退之后暖机的根都坏了而它还在）；候选集按实例表判它被抛弃、但退回它之前的实例 1 旧根时它的记录会不会被接上（第二轮 X1-A 那条路）；被跳过之后它引用的槽被复用，之后恢复选中它会读到什么；`abandoned_roots_unreadable` 没有消费者是不是一格空条款——什么东西该读它；候选根的账读不出「当它什么都不豁免」方向对不对（隔离多了会不会让准入少报可用、让一次本该成功的发布报空间不够）。

## 规矩

- 每个打中给出具体的历史或输入，每一步指到被判对象里许可它的那一句（代码路径与行号，或附录原文）。
- 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
- 自己提的改法写明「只在我的模型上量过、被攻过零轮」。
- 没打中的写试过哪些形状、取样范围多大。
- 禁读：`research/prompts/m2-step45-code-r3-*-output*.md` 里别的腿的输出。

## 交付

- **分段写**：报告写进 `research/prompts/m2-step45-code-r3-opus-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。写了模型或用例的放 `research/prompts/m2-step45-code-r3-opus-model/`，报告开头给复跑命令与每个文件的 sha256。
- 报告开头一张「各格判定一览」（X1 / X2 / X3 / X8 各一行），然后按格分节，末尾「没打中的形状」「这条腿自己的限度」。
- 引条款整行抄、行号写 kb 文件的；引代码写路径与行号。不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 60 分钟内交出报告。最后回复只写一句指向报告。
