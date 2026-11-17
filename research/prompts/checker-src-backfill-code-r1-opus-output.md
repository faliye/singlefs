# 池级 checker 在途改动代码轮第一轮：攻方腿

日期：2026-11-17。批次：`records/2026-11-16-三方豁免取消与存量补轮.md` 补轮 2。

开工核过五份被判文件：`sha256sum -c research/prompts/checker-src-backfill-code-r1-snapshot/crates-sha256.txt` 五行全 `OK`（image.rs、lib.rs、position_addressed.rs、rebuild.rs、walk.rs），下文行号都是这一版的。副本里同一条命令同样五行 `OK`（副本在补探针之前核的）。

没写模型目录：打中的那几格用库单测探针与候选变异行量，放在仓副本 `~/.local/share/singlefs/work/checker-src-backfill-code-r1-attack/copy` 里（`.deliverables` 写成交接），命令在「要主 agent 跑的命令」一节，读数都还没跑出、一律标「推的，待主 agent 跑」。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一、CRC 按内容复用（lib.rs） | 没打中 | 复用之前整段逐字节比，取到的值就是对同一串字节现算的值；不依赖调用次序与线程；`#[cfg(test)]` 的计数与上限覆盖不进非测试构建。只记一条记账口径的线索（内存上限只数内容字节） |
| 二、环长判据加「装进 4 字节」（image.rs） | 没打中 | 与 core 的 `JournalRingBytesOutsideTheSupportedRange` 同式同序；`3 << 44` 是精确边界，小 1 到 12287 字节都装得进；归 I-7.13 与条款字面一致 |
| 三、I-3.5「走到没有条款的形状」（walk.rs） | **打中**（C1、C2） | 注释与不变量状态格说「由 I-9.16 ④ 判红」，对「十二种齐全再多一条没登记种类」与「livelist / 稀疏旁表根指针非全零」两形不成立：I-9.16 四道都成立、走读不记失败，相邻两对改报部分没判，原本只红 I-3.5 的镜像整份不红（推的） |
| 四、新加 21 行变异罩不罩得住 | **打中**（C3） | 21 行原文都恰好命中一次（量的，见第四节）；有三处决定「这一对判不判」的分支没有行：`walk.rs:809` 根指针全零那一格（推的：现有测试一条都不红）、`walk.rs:2270` 折成 I-3.5 输入那一步、`walk.rs:1101` 那条 `continue` |
| 五、改名一批有没有夹带行为改动 | 没打中（代码）；打中一处文字残留（C4，低） | 代码里纯改名与重排版；`.claude/kb/checks-owed.md:961`（与 en、ja 两份同一行）还写旧成员名 |
| 检查单：走不到的理由二次查实 | 成立 | lib.rs 三处 `expect` 里真新加的只有 154 那一处，在 `#[cfg(test)]` 语句里；163、178 是原样挪进闭包的旧行 |
| 检查单：类型与名字 | 不成立 | `walk.rs:825` 的 `other_kind =>` 是对盘上 `u16` 的兜底臂，与 `_ =>` 同义 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 加行里没有认旧版本、旧布局、迁移的分支 |
| 检查单：不新建判定库 | 成立 | 线程局部记忆只存 CRC 的算术结果、不落盘、不存判定 |
| 检查单：所有任务多机可用 | 成立 | 不读本机状态；取到的值与机器、线程无关 |
| 检查单：测试优先多线程 | 成立 | 新测试是单场景小用例；`std::thread::spawn` 只为拿一份空的线程局部记录 |
| 检查单：带 --item 的命令与项 | 成立 | 没有新项；新行点名的测试都筛得到 |
| 检查单：新崩溃流多罩了什么 | 成立 | 没有新崩溃流 |

## 一、CRC 按内容复用（lib.rs）：没打中

**会不会返回与现算不同的值**：不会。命中那一支只在「候选与输入整段逐字节相同」时取值，「.position(|candidate| *candidate.content == *bytes);」（`crates/singlefs-checker/src/lib.rs:149`），取的是那一个候选自己记的值，「return candidates[position].checksum;」（`crates/singlefs-checker/src/lib.rs:157`）；那个值是同一串字节查表现算后记下的，内容是 `Box<[u8]>` 自己的一份拷贝，调用方之后改自己的缓冲碰不到它。攻过的形状（都按代码推）：

- 指纹撞上而内容不同：逐字节比不同，落到现算；单测里 8192 那一字节不取样的一对就是这一形（`node_sized_content_differing_only_in_the_unsampled_middle_byte`）。
- 到上限整份清空、同一指纹挤掉最早那段、内容比上限还长不记：三条都只删候选或不记，不改任何候选记的值，删掉的下次现算。
- 0 到 8 字节的输入：`word_start` 恒 0、16 个字都取整段内容，指纹就是内容本身加长度，不会撞。
- `HashMap` 的随机哈希种子：只用 `get` 与 `entry`，没有遍历，次序不进任何结果。
- 重入：闭包里只查表与现算，不再调 `crc32_castagnoli_table`，`borrow_mut` 不会二次借。

**判定依赖不依赖调用次序或线程**：不依赖。函数的返回值只是输入字节的函数（上一条），记录只影响走不走现算那一支；每条线程一份记录，别的线程看不见，单测 `the_same_content_computed_again_on_the_same_thread_is_reused_and_another_thread_computes_its_own` 钉住。

**`#[cfg(test)]` 漏不漏进非测试构建**：不漏。上限在非测试构建里取常量，「let remembered_content_bytes_limit = REMEMBERED_CONTENT_BYTES_LIMIT_PER_THREAD;」（`crates/singlefs-checker/src/lib.rs:125`），它上面一行是 `#[cfg(not(test))]`（124）；覆盖与两个计数住在 `#[cfg(test)]` 的 `thread_local!` 里（99），两处累加是 `#[cfg(test)]` 语句（150、182）。验证包链的是不带 `cfg(test)` 的池级 checker，那里既没有覆盖、也没有计数；它们是私有项，别的包引不到。

**线索（不算打中，推的）**：上限数的只是内容字节，「一台机器上全部线程的记录合计是这个数乘线程数，压在几百 MiB。」（`crates/singlefs-checker/src/lib.rs:60`）没算每条候选的额外开销。按类型大小推：指纹 136 字节（`usize` 加 16 个 `u64`）、表项里的 `Vec` 头 24 字节、第一次 `push` 起的 4 个候选的堆块 96 字节、内容那一块的分配头，合起来每条两三百字节。16 KiB 的单元上这是百分之一二；全是一百来字节的单元头这类短输入时，实际占用是记的内容字节的三四倍。判定不受影响，只是那句估算偏小；不改判、不需要改法。

**没攻到的**：线程局部值析构之后再调（在别的线程局部值的 `Drop` 里调 CRC）会让 `.with` panic，这是新出现的一条路径。调用方 11 份文件（`grep -rln crc32_castagnoli_table crates`），里面的 `impl Drop` 是 GPU 借卡与回读缓冲一类，名字上看不调 CRC，没逐个读 `drop` 的函数体。

## 二、环长判据加「装进 4 字节」一格（image.rs）：没打中

- **同一张表**：checker 的算式「let in_flight_record_limit = journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;」（`crates/singlefs-checker/src/image.rs:970`）与 core 的「journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR」（`crates/singlefs-core/src/system_configuration.rs:116`）同式；core 读者拒的三格同序，第二格是「|| !journal_in_flight_record_limit_fits_its_four_byte_field(sizes.journal_ring_bytes)」（`crates/singlefs-core/src/recovery.rs:1478`），checker 加的是「&& u32::try_from(in_flight_record_limit).is_ok()」（`crates/singlefs-checker/src/image.rs:975`）。两边都取格式常量 F，不读槽里那 4 字节。
- **`3 << 44` 附近**：环长写成 `3 << 44 − 12288 + k`（`0 ≤ k < 12288`）时 ⌊环长 ÷ 4096 ÷ 3⌋ = 2³² − 1，装得进；`3 << 44` 起是 2³²，装不进。边界是精确的，新单测取的两点正好跨在上面。两点的单元区起点都按 `div_ceil` 现算、相等，第三格（环末端）两点都成立，这一对只有新加的那一格分得开（推的，按算术）。
- **「装不进时 I-8.1 那一格必不成立」**：I-8.1 先判槽里记的 F 等不等于格式常量（`crates/singlefs-checker/src/image.rs:1064`）；F 记的是 3 时现算的上限是 2³²，不等于任何 4 字节值；F 不是 3 时第一格先红。所以这一格只是把判红从 I-8.1 挪到 I-7.13，与 core 整池拒同一个分类，没有放过原本会红的镜像。
- **归哪条不变量**：I-7.13 那一行已写「这个商装得进系统配置里它那 4 字节一格（偏移 345，装不进时不走到 I-8.1（环几何够大） 的等式）」（`crates/singlefs-checker/invariants/02-根环.md:20`），与注释一致。

**顺带看到的文字滞后（不在被判的五份里）**：D23 已定项 19 的射程只列了两格，「挂载读者一侧另判环长：在飞上限（环槽数 ÷ F，已定项 18）≥ 1，且环末端不越过系统配置偏移 417 记录的单元区起点」（`.claude/kb/decisions/23-journal的角色与格式.md:546`），没写装进 4 字节那一格；core 的「checker 不算在飞上限，不另造」（`crates/singlefs-core/src/system_configuration.rs:112`）与 `image.rs:970` 不符（这一条在 HEAD 上就已经不符，`in_flight_record_limit >= 1` 原来就在）。

## 三、I-3.5「走到没有条款的形状」那一路（walk.rs）

### C1（打中）：「由 I-9.16 ④ 判红」对「十二种齐全再多一条没登记种类」不成立，原本只红 I-3.5 的镜像整份不红

被打中的一句：「共享的版本不重走条目，引用同样没收全。不进 `walk_failures`：I-7.2、I-4.8、I-7.4 判的是走读断没断，这一条由 I-9.16 ④ 判红。」（`crates/singlefs-checker/src/walk.rs:3747`）；I-3.5 状态格同一句「I-9.16 ④ 判红」（`crates/singlefs-checker/invariants/05-空间记账.md:11`）。

**形状 S1**：一版的树表是十二种各一条，再多一条种类 16、根指针全零、头 ID 0 的条目，树 ID 取 65540（mkfs 那条流上中央映射树的号，它不进树表，`walk.rs:2860` 的注释写了这个号），插在记账树（65539）与 livelist（65541）之间。逐道看许可它的那一句：

- I-9.16 ①：树 ID 盘上仍严格升序。
- I-9.16 ③：条款字面就许没登记的种类出现一次，「③ 每一种种类码（登记过的与没登记的都算）至多一条」（`crates/singlefs-checker/invariants/09-inode树结构.md:28`），实现同一句「③ 同一张树表里每一种种类码（登记过的与没登记的都算）至多一条」（`crates/singlefs-checker/src/walk.rs:6024`）。
- I-9.16 ④：只查十二种在不在，「.find(|kind| !entries_of_each_kind.contains_key(kind));」（`crates/singlefs-checker/src/walk.rs:6140`），多出来的一条不查。
- I-9.16 ②：只按十二种的发号次序取条目，种类 16 不在那一串里，跳过。
- I-9.27：「_ => Self::NotJudged,」（`crates/singlefs-checker/src/walk.rs:5996`），种类 16 那一行不判。
- I-7.8：65540 比水位小（推的：mkfs 之后水位不小于 65548）。
- 走读：种类 16 在 `walk.rs:3359` 拿不到 key 形态、直接返回，根指针全零、底下本来就没有东西，引用其实收全了；不记走读失败。
- 新路：「other_kind => key_schema_for_tree_kind(other_kind).is_none(),」（`crates/singlefs-checker/src/walk.rs:825`）对 16 判真，这一版记下种类 16，它参与的每一对改报部分没判（`walk.rs:1090-1102`）。

**历史**：拿已有坏镜像 `rollback_whose_lineage_record_is_later_than_the_rollback_version_reddens_only_the_reference_interval_invariant`（`crates/singlefs-verification/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs:514`，那一份只有 I-3.5 与干净镜像不同），在回退那次发布 R 的树表里再插上 S1 那一条，重封树表单元并沿指针链补校验和（写法照同目录 `common_deletion_in_progress/mod.rs` 的 `change_the_unit_and_propagate_its_checksum`，与 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里给树表追加第十三条那一段同一套改法）。R 与回退目标共用那张树表时两版都记下种类，结果相同。推的结局：改动前 I-3.5 照判、仍红（种类 16 底下什么都没有，引用集合与原镜像相同）；改动后 (R⁻, R) 那一对报部分没判，I-9.16、I-9.27、走读那几条都成立，**整份镜像一条违例都没有**。这里由用户决定的动作只有回退目标与回退之后写不写文件，坏镜像那一份已经固定了，放开它们不改上面每一道的判法（每一道只看树表那一条条目）。整份镜像没有在副本上造，库单测探针只钉住上面的两半（第七节第 1 条命令）。

**四问**：
1. 分不分辨臂：这一题没有臂；它分得开改动前后——同一份镜像改动前红 I-3.5，改动后整份不红。
2. checker 当时看不看得到：看得到，那一条的种类与根指针全零都在手里；根指针全零时引用并没有少收。
3. 满足的是判据字面的哪一句：I-3.5 内容格「走到没有条款的形状：树表里种类没登记的条目算，用户 2026-11-09 定」（`crates/singlefs-checker/invariants/05-空间记账.md:11`）字面许可这一对不判，不判本身合条款；打中的是注释与状态格的那半句担保「I-9.16 ④ 判红」——④ 只在没登记的种类**顶掉**一种登记过的种类时红（已有用例 `an_intermediate_version_without_a_deleted_inodes_entry_…` 就是顶掉的那一形），多一条时不红，③ 的字面还明说多一条不算违例。
4. 改法在打中的那几格上还中不中（三条都是推的、没实现、被攻过零轮）：
   - 甲：I-9.16 加一道「每条条目的种类都在登记表里（码 1..4、6..15）」。S1 上红；要改 I-9.16 ③ 的字面，交用户定。对 C2 的 S2 不起作用（S2 的种类登记过）。
   - 乙：把「走到没有条款的形状」记进走读失败，与 core 冷走读拒成「树的种类没登记」对齐（I-7.2、I-4.8、I-7.4 红）。S1 上**不起作用**：core 冷走读先跳过根指针全零的条目，「if entry.root == NodePointer::empty_root() {」（`crates/singlefs-core/src/recovery.rs:7465`），再判种类，S1 在 core 那里也走得过去。对 S2 起作用。
   - 丙：根指针全零的没登记条目不记种类（引用收全了，照判）。S1 上 I-3.5 照红；与内容格「种类没登记的条目算」的字面冲突，交用户定。

### C2（打中）：livelist、稀疏旁表根指针非全零那一支，条款内容格没写，「I-9.16 ④ 判红」同样不成立，core 冷走读却拒

- 分法：「TREE_KIND_LIVELIST | TREE_KIND_SPARSE_SIDE_TABLE => !root_pointer_is_all_zero,」（`crates/singlefs-checker/src/walk.rs:824`）。I-3.5 内容格只把「树表里种类没登记的条目」点名进「没有条款的形状」，livelist、稀疏旁表根指针非全零这一支只写在状态格里；它是不是用户 2026-11-09 那一次定的射程之内，从内容格推不出，交主 agent 核用户原话。
- **形状 S2**：十二种各一条，livelist 那条的根指针换成一条非全零的指针（例如抄 extent 树那一条的 86 字节），别的不动。I-9.16 不读根指针，四道都成立；I-9.27 对 livelist 要的是头 ID 为 0，头 ID 没动；checker 走读在 `walk.rs:3359` 返回、不记失败；新路记下种类 6，它参与的每一对报部分没判。所以「I-9.16 ④ 判红」在 S2 上也不成立（推的，库单测探针钉住 I-9.16、I-9.27 与记下种类 6 那三半）。
- core 那一侧：冷走读对根指针非全零的条目往下取 key 宽，livelist 那一格是「TREE_KIND_LIVELIST | TREE_KIND_SPARSE_SIDE_TABLE | TREE_KIND_DEADLIST => None,」（`crates/singlefs-core/src/recovery.rs:5625`），接着报「what: "树的种类没登记",」（`crates/singlefs-core/src/recovery.rs:7583`）；`walk_to_file_in_the_unit_area_starting_at` 里 7460–7580 之间单独接住的种类没有 livelist 与稀疏旁表（我逐行扫过那一段的 `continue` 与种类判断）。也就是 core 冷走读读不下这一版，而 checker 说走读没断、注释说「I-7.2、I-4.8、I-7.4 不跟着红」——两边对这一版读不读得下判的不一样（推的：没在镜像上跑 core 的冷走读）。
- 四问：分得开改动前后（改动前 S2 上 I-3.5 照判；livelist 底下的单元不进任何一版的引用集合，判出来红不红取决于那些单元还被谁引用）；checker 看得到根指针非全零；打中的是同一句「I-9.16 ④ 判红」；改法乙对 S2 起作用，甲、丙不起作用。

### 共享树表的版本记的种类：没打中

共享那片树表的版本走 `judge_the_tree_table_entries_an_earlier_version_walked`，用先走那一版记下的同一份条目（`walk.rs:3776`），与先走那一版记的种类逐条相同；每一版开始时复位（`walk.rs:2183`），走完一版折进 I-3.5 的输入（`walk.rs:2221`、`walk.rs:2270`）。一版只有一张树表、走两条路里的一条，`note_the_first_tree_table_entry_without_a_clause` 一版至多调一次，后一次覆盖前一次的情形不会发生。`judge_the_appearance_rule_of_adjacent_candidate_versions` 里 `previous_versions` 只是相邻那一个 txg 上的几版（`walk.rs:1063`），一版的种类不会连坐到不相邻的对。

## 四、新加 21 行变异罩不罩得住

**量的**（草稿目录里一段只读的 python，读 `crates/mutations.tsv` 与五份源文件）：表里指到这五份的 938 行，原文在文件里都恰好命中一次；`mutation-rows-new.tsv` 的 21 行按（文件、原文、替换文）都在表里。与 `git show HEAD:crates/mutations.tsv` 比，指到这五份、键消失的 5 行（walk.rs 4 行、rebuild.rs 1 行：I-8.9 第一条不从 1 起、比 L 晚的根走不完的守卫、I-9.7 inode 树那一处、I-4.10 截断那一处、I-1.8 ② 码 3 报成码 1）同名行都还在，第五、六段（参数、必须红的测试）逐字相同，原文与替换文只差改掉的变量名或成员名：改名没丢变异。

逐处对（「有行」指 21 行里有一行改坏它）：

| 改动 | 决定结局的分支 | 有没有行 | 现有测试抓不抓得到 |
|---|---|---|---|
| image.rs:975 | 装进 4 字节那一格 | 有 | 库单测抓（行里点名的） |
| lib.rs:149 / 157 | 逐字节比、取哪一个候选的值 | 有（两行） | 库单测抓 |
| lib.rs 清空、记下、候选上限、`checksum_field_holds` 走这一份 | 记录的形状 | 有（四行） | 库单测抓 |
| lib.rs:184 超上限不记、挤掉哪一段、指纹取样 | 只影响走不走现算 | 没有 | 不改返回值（第一节），不算漏判红分支 |
| walk.rs:821–827 三臂 | 哪些条目算 | 有（三行） | 库单测抓 |
| **walk.rs:809** 「root_pointer_is_all_zero: root_pointer.all_zero,」（`crates/singlefs-checker/src/walk.rs:809`） | livelist、稀疏旁表根指针非全零那一支从盘上字节接到判式 | **没有** | **推的：改成 `true` 时现有测试一条都不红**。库单测 `only_unregistered_kinds_…` 直接拿字面的真假喂判式；`version_sharing_…` 手造 `TreeTableEntryOfTheWalk`、根指针写死全零；验证包那条用例用的是种类 16；GPU 对拍那段断言稀疏旁表的根指针「本来全零」。没有一份镜像让 livelist 或稀疏旁表的根指针非全零、再经 `tree_table_entry_of_the_walk` 走到判式 |
| walk.rs:2221 | 走完一版折进 VersionPlacements | 有 | 验证包用例抓 |
| **walk.rs:2270** 「kind_of_a_tree_table_entry_without_a_clause: version」（`crates/singlefs-checker/src/walk.rs:2270`） | VersionPlacements 再折成 VersionReferences | **没有** | 推的：验证包 `an_intermediate_version_without_a_deleted_inodes_entry_…` 抓（它走整条 `check_pool_image`）；库单测抓不到（直接造 VersionReferences） |
| walk.rs:2939 / 3776 / 2183 | 记、共享记、复位 | 有（三行） | 抓 |
| walk.rs:1090–1093 前一版、后一版各算不算 | 有（两行） | 库单测抓 |
| **walk.rs:1101** `continue;` | 记了部分没判之后不再判这一对 | **没有** | 推的：删掉它时库单测 `later_version_with_…` 抓（照判那一对红在谱系那一半，判定变成违例） |

C3（打中，中）：809 那一格是 C2 那一支唯一的入口，没有行、也没有测试；2270 与 1101 两格测试抓得到、只是缺行（缺行时以后测试被改弱，门禁看不见）。改法：809 那一格加一条库单测，从带根指针非全零的 livelist 条目字节经 `tree_table_entry_of_the_walk` 走到判式（第七节第 1 条命令里的探针就是这样写的），再挂一行变异；2270、1101 各挂一行。四行候选写进了副本的 `crates/mutations.tsv` 末尾（以「攻方探针」起头），证红命令在第七节第 2、3 条。只在副本上写过、被攻过零轮，期望的结局都是推的。

## 五、改名一批：代码里没有夹带行为改动；kb 里残留旧成员名（C4，低）

- 逐块读了 `changes.patch` 的每一处改名：rebuild.rs 三个成员（声明、`reason()` 三臂、两处构造、三处测试断言）只换名字，`reason()` 的三句话一字不变；`crash_judge_conclusions.rs` 认的是 `fn reason` 里的句子、不认成员名，不受影响。walk.rs 三个局部变量与 `truncation_intent_names_the_record` 只换名字；`judge_publish_ordinals_and_last_record_flags` 那一处是同一个表达式重排版；`walk.rs:7273` 那一处 `///` 改 `//` 写在函数体里的局部 `enum` 上，不进文档、不改代码；70 个改名的测试函数函数体一行没动（patch 里那些块只有签名行与 `{` 换行）。
- 旧名残留：把 patch 里删掉的 70 个函数名与 `let a_…` 变量名逐个 `git grep -w`（排除 `research/`），只有 `a_later_root_is_incomplete` 在 `crates/singlefs-verification/src/crash_judge_checker_scan.wgsl` 里出现 3 次，那是 GPU 内核自己的变量，与改名无关。旧成员名 `DataUnitVersionsTiedOnTheTotalOrderKeyOfI18` 还在 `.claude/kb/checks-owed.md:961`（C464 那一行），同一行的 en、ja 两份同样（`.claude/kb/checks-owed.en.md:961`、`.claude/kb/checks-owed.ja.md:961`）；正文「全仓 grep … crates scripts」零命中是真的，但范围只到 `crates`、`scripts`。那一行写的是当时改名这件事，要不要跟着改由主 agent 判（是现状句还是经过句）。
- 不一致（不算打中）：改名一批去掉 `a_` 前缀，新加的测试却叫 `a_fingerprint_keeps_at_most_the_candidate_limit_and_evicts_the_earliest_remembered_content`（`lib.rs:1577`）；同文件之外还有不少 `an_` 起头的测试没改。命名检查 N1 不判打头的冠词（`.claude/singlefs-ai-sop/scripts/naming-lint.sh` 文件头 N1 那一条），所以这只是口径不齐。

## 六、检查单八格

**走不到的理由二次查实：成立。** 自己在 `changes.patch` 里按新文件行号把加行切到各文件 `mod tests` 那一行之前再数（草稿目录里一段只读的 python，量的）：image.rs、position_addressed.rs、rebuild.rs、walk.rs 生产段加行里 `unreachable!`、`panic!`、`.expect(`、`.unwrap(`、`assert` 一处都没有；lib.rs 三处。
- `lib.rs:154`「.set(comparisons.get() + u64::try_from(compared).expect("候选数装得进 u64"));」（`crates/singlefs-checker/src/lib.rs:154`）在 150 那一行 `#[cfg(test)]` 管的语句里，只进测试构建；`compared` 不超过一个指纹之下的候选数，`usize` 到 `u64` 在 64 位目标上不会失败。
- `lib.rs:163`、`lib.rs:178` 不是新加的：patch 里删掉的旧行（`index` 小于 256、`remainder & 0xFF` 装进 `u8`）原样挪进了闭包，只多了缩进；两处的前提照旧成立（下标来自 256 项数组的 `enumerate`，与 0xFF 按位与）。

**类型与名字：不成立（打中，低）。** 「other_kind => key_schema_for_tree_kind(other_kind).is_none(),」（`crates/singlefs-checker/src/walk.rs:825`）是对盘上读来的 `u16` 的兜底臂，绑了名字、与 `_ =>` 同义；检查单那一格的判法写「不写 `_ =>`」（`.claude/skills/three-way/SKILL.md:83`），`.claude/singlefs-ai-sop/rules/code-discipline.md@分支：穷举写全，不写通配臂` 对「盘上读到的」值要的是把未知做成显式成员。clippy 的 `wildcard_enum_match_arm` 只管枚举，判不到 `u16` 上的这一臂。后果：以后登记一个没有 key 形态的新种类，这一臂静默判真（那一版的每一对改报部分没判），编译器不逼人来看。同文件里 `_ => Self::NotJudged`（5996）与 lib.rs 的 `key_schema_for_tree_kind` 是同一种写法的旧例。改法（推的、没实现、零轮）：按区间写满，`1..=5 | 8..=13 => false`、`0 | 16..=u16::MAX => true`，整数区间的穷尽性编译器照样判。新加的类型与常量名字说得出内容，没有缩写；新加的 `expect` 写了依赖什么。

**不兼容旧代码与旧格式：成立。** 加行里按（旧版、旧布局、旧格式、legacy、migrat、迁移、兼容旧、backward）`grep -ciE` 是 0（量的）；按代码读也没有认旧版本的分支。

**不新建判定库：成立。** 按内容复用存的是 CRC 的算术结果（内容到 32 位值），进程内、每条线程一份、不落盘，不是 `.claude/rules/verification.md@判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB` 说的「判定」（剪枝按单元键复用要读的那一类）；加行里按 rocksdb、sqlite、prune-server、`File::create`、`std::fs` 计数是 0（量的）。

**所有任务多机可用：成立。** 加行里按 hostname、`/proc/`、`/sys/`、`env::var`、`std::env`、`available_parallelism`、`test-threads` 计数是 0（量的）；记录在哪台、哪条线程上都只影响走不走现算，返回值不变（第一节）。

**测试优先多线程：成立。** 新测试是单场景小用例；lib.rs 的 `std::thread::spawn`（`lib.rs:1359`）只为拿一份空的线程局部记录，不是切片并行；没有写死或覆盖线程数。

**带 --item 的命令与项：成立。** 没有新项；21 行与改名后那几行第五段 `--` 之后的筛选词都筛得到点名的测试（库单测在 lib.rs、image.rs、walk.rs、rebuild.rs 的测试模块里按名字找得到；验证包那条在 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs:659`），由门禁 crates-mutation-replay 按行点名。

**新崩溃流多罩了什么：成立。** 加行里按 `enumerate_crash_states`、`crash_state`、`CrashImage`、`every_crash` 计数是 0（量的），没有新崩溃流或崩溃枚举用例。

## 事实表写错的地方

1. lib.rs 第 3 样「生产段新加 3 处」：163、178 是旧行原样挪进闭包（patch 里同一句先删后加），真新加的只有 154，而且只在测试构建里。
2. lib.rs 第 1 样「6 个 #[test]」（正文第 2 件「新测试 6 条」同样）：patch 里 lib.rs 加的 `#[test]` 是 5 个（`awk` 按 `+    #[test]` 数，量的），五条是 `table_crc32c_reusing_…`、`clearing_the_remembered_content_…`、`the_same_content_computed_again_…`、`a_fingerprint_keeps_…`、`checksum_field_holds_on_the_same_slot_twice_…`。
3. walk.rs 第 3 样「生产段新加的条目名 3 个」把 `truncation_intent_names_the_record` 算成新加，它是 `a_truncation_intent_names_the_record` 改名。
4. walk.rs 第 4 样「两处都没引到（grep 名字）的 7 个」里有 `note_the_first_tree_table_entry_without_a_clause` 与 `tree_table_entry_of_the_walk`：按名字没 grep 到，但前者经 `judge_the_tree_table_entries_an_earlier_version_walked` 被库单测 `version_sharing_…` 调到，后者经 `check_pool_image` 被验证包用例走到；「没引到」不等于没测。
5. 正文（不是事实表）写「`walk.rs:823` 的臂」，那一臂在 825；正文第 3 件说「由 I-9.16 ④ 判红」照抄注释，就是 C1 打中的那一句。

## 七、要主 agent 跑的命令

副本：`/home/fy5090/.local/share/singlefs/work/checker-src-backfill-code-r1-attack/copy`（`minimal-copy.py --with-test-inputs --with-gate-tables --with scripts=…` 建，`.deliverables` 写成交接）。副本相对主工作区只改了两份：`crates/singlefs-checker/src/walk.rs` 末尾测试模块里加两条探针与两个辅助函数（前 29220 行与主工作区逐行相同，量的：`diff` 退 0；副本 sha256 `f10bf745f57d078ba67b54eab1b1fee2fed556da4a651810efb54f292638433c`），`crates/mutations.tsv` 末尾加四行「攻方探针」起头的候选变异（前面各行与主工作区逐字相同；副本 sha256 `e6c3d7dc11bb2ff8bf9eeff2f9c7584f2d2d17dd85d8df9e8bceb1c32d2611f7`）。探针原文另存在 `~/.local/share/singlefs/work/checker-src-backfill-code-r1-attack/probe/probe_tests.rs`（sha256 `b0715b90b93ec6302a4aa9816203d4eef0fb081c409520c8f2136b38bfee8028`，`cargo fmt` 之前的那一份）。

1. 探针两条（核 C1、C2 里 I-9.16、I-9.27 成立、记下的种类，与同一对从违例变成部分没判）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker 的测试目标 lib 里以 attack_probe 起头的两条探针; 范围=1 个测试目标; 为什么=核攻方报告第三节 C1 C2 的树表判定与 I-3.5 那一对的判定' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/checker-src-backfill-code-r1-attack/copy -- cargo test -p singlefs-checker --lib attack_probe -- --nocapture`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/checker-src-backfill-code-r1-attack/copy`
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 `test result: ok. 2 passed`；三行 `ATTACK_PROBE table=…`，干净那张 `kind_without_a_clause=None`、种类 16 那张 `Some(16)`、livelist 那张 `Some(6)`，三张都是 `i_9_16=Holds i_9_27=Holds`；`ATTACK_PROBE pair_without_kind=Violated(…)` 与 `ATTACK_PROBE pair_with_kind_16=PartiallyJudged(…)`。哪一条对不上，C1 或 C2 里对应那一半撤回。
2. 池级 checker 那三行候选变异的证红（核 C3）
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/checker-src-backfill-code-r1-attack/copy -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/checker-src-backfill-code-r1-attack/copy singlefs-checker '攻方探针 I-3.5 走到没有条款的形状：树表条目的根指针一律读成全零（库单测那一格）' '攻方探针 I-3.5 走到没有条款的形状：树表条目的根指针一律读成全零（探针那一格）' '攻方探针 I-3.5 走到没有条款的形状：记了部分没判之后照判那一对'`
   - 在哪起：同上
   - 看哪几行：每行一条「<名>\t<档>\t…」。期望（推的）：「库单测那一格」是「没红」（809 那一格现有的库单测抓不到），「探针那一格」与「照判那一对」是「抓到」；因为有一行没红，整次 `exit` 不是 0，这是期望内的。「库单测那一格」也抓到的话，C3 里 809 那一半撤回。
3. 验证包那一行候选变异的证红（核 C3 的 2270 那一格）
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/checker-src-backfill-code-r1-attack/copy -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/checker-src-backfill-code-r1-attack/copy singlefs-verification '攻方探针 I-3.5 走到没有条款的形状：折成 I-3.5 输入的那一步丢掉种类'`
   - 在哪起：同上
   - 看哪几行：那一行是「抓到」、末行 `exit=0`（推的）。范围声明闸若判这个目标会走崩溃枚举而拒，照拒绝说明加缩域变量或全域理由再交。

跑完由主 agent 删副本（交接那一行）。

## 没打中的形状

- CRC：指纹撞上而内容不同、清空、挤掉、超上限、0 到 8 字节、调用方之后改缓冲、哈希种子、重入、跨线程（第一节）；按代码推，没跑。
- 环长：`3 << 44 − 12288 + k`（`k` 取 0 与 12287 两端）、`3 << 44`、槽里 F 写 3 与不写 3、环末端越界与不越界（第二节）；按算术推。
- I-3.5：共享树表的版本、同一 txg 几版、前一版与后一版各自带种类、复位、快照两种条目、种类 5 进树表（`key_schema_for_tree_kind` 给它映射树的 key 形态，判式判假、走读照往下走，与「不进树表」的登记不符但不是这次改动的事）。
- 改名：70 个旧函数名与 `let a_…` 变量名全仓 `git grep -w`（排除 `research/`）、五行换锚点的变异行逐段比、rebuild 三个成员的 `reason()` 句子。

## 这条腿自己的限度

- C1、C2 的整份镜像没有造：「原本只红 I-3.5 的镜像整份不红」是按每一道判法推的，探针只钉住其中三半（I-9.16、I-9.27、记下的种类）与「同一对从违例变成部分没判」；I-7.8 水位、别的不变量会不会因为多一条条目而红，没在镜像上量。
- C2 里 core 冷走读拒成「树的种类没登记」是读 `walk_to_file_in_the_unit_area_starting_at` 推的，没跑 core；也没核 core 的挂载路径（`rebuild_version`）对 livelist 根指针非全零怎么办。
- C3 里「现有测试一条都不红」只按名字与写法排查了库单测、验证包里点到 livelist 与稀疏旁表的几处，没把整包测试在那条变异下跑一遍。
- 用户 2026-11-09 那一次定的原话没读（不在被判的文件里），C2 的「条款射程」只能写推不出。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数；没往统一任务服务交任务，第七节三条命令都没跑，正文里对它们结局的预期都是推的。
- 没读禁读清单里的另一条腿报告。
- 草稿目录：`copy`（仓副本，191 MiB，交接，未编译、里面没有 `target`）、`probe/`、几份只读输出（`i35-row.txt`、`i713-row.txt`、`mutations-head.tsv`、`old-fn-names.txt`、`minimal-copy*.txt`、`fmt-*.txt`）。
