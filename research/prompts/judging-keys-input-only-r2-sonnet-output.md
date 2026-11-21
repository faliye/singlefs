# judging-keys-input-only-r2 辩方腿报告（Sonnet，2026-11-20）

立场：辩方。复核 `research/prompts/judging-keys-input-only-r1-main-verification.md` 的七格挂起、三格用户已定、六条零轮改法，并估新键下的耗时与复用。没读禁读清单里的三份文件。

读的树：集成副本 `/home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree`（主工作区加两份补丁，不在 git 里，行号取读它那一刻的样子）；引它的行写「集成副本 <相对路径> 第 N 行」，引文用 `grep -nF` 逐句现查过。仓内文件与 kb 照 `路径:行号`。

没写模型：没有模型目录、没有 sha256、没有复跑脚本，探针内存与时间不适用。全部结论来自读源码、grep 与 awk 算术；数各自标「量的」（命令现数）或「推的」。要主 agent 跑的命令在第六节。

## 一、结论先说

| 项 | 结论 |
|---|---|
| 七格挂起 | 全部有真实的代码依据，但各有一处腿说宽了：格 1 的「只剩」、格 2 的「看不见」、格 5 的「不在任何键里」、格 6 的「02、03 删不掉」、格 7 的「定值版本」。逐格见第二节 |
| 同样打中所有替代方案的格 | 格 6 与格 7 的字面冲突那一支：任何改了待判行格式、或沿用三类代码表的设计都中，不分辨新键与旧键；这两支不该算作对「键只有输入数据」的打中，但各自要处理。其余五格都分辨得出（旧键下那一格不成立） |
| 用户已定三格 | 产品代码进键：关掉了大半，剩判器入口手登、WGSL 另写一份恢复。录入按屏障：只关掉「录制管线放不放进键」，「录入检测」没有落成任何一道检查。工具链与依赖：用户原话只到编译档与线程数，集成副本的键里工具链行与依赖行都还在 |
| 六条改法 | 强制现判按比例：修得了变严与对拍库，修不了漏判，对格 4 修不了、也没更糟（今天收尾那一遍读已经让旧块全红）。阳性样本集：两侧大半已经作为测试存在（CPU 一侧 114 条不变量各一张），新增的只是不靠键触发、每趟现判。人登记整族删：修得了，但「按那一种结论」做不到，只能按整张表。计数向量名不同答不复用：单独修不了（存库拒绝）。观察者修订号：只逼出一次决定，并撞已定项 17 手写版本常量那句。布局号升 7：修得了，有一个窗口 |
| 耗时与复用 | 新键对「非产品代码改动」的重判与重录归零；对产品代码改动与上一版相同。库里判过的态里换键造成的上限是 5,980,821,966 态（量的，基线）。第一趟不省。会把省下的花回去的：强制现判的比例乘上观察者重跑，与判器引到的产品代码那一组拒算 |

## 二、七格逐格（判决「二、判决」挂起的 1–7）

「替代」列：把「键只有输入数据」换成第一轮「三、改法」六条里任一条（或旧键），这一格还中不中。

### 格 1 判器改动的发现手段（`research/prompts/judging-keys-input-only-r1-main-verification.md:38`「判器改动的发现手段只剩每流每趟一块强制现判」）

判定：站得住，但「只剩」说窄了。

- 哨兵只验复用没坏：集成副本 crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs 第 604 行「复用失灵：哨兵块 {}（点 {} 的 [{}, {})，块键 {}）键相同、库里有不红的结论，这一趟却现判了」；比结论的只有第 644 行「判定不稳：强制现判的块 {}」那一支，每流一块（同文件第 430 行「强制现判照同一串数接着从剩下的里抽一块」），缺省哨兵 3 块（同文件第 16 行「pub const DEFAULT_REUSE_SENTINEL_COUNT: usize = 3;」）。读得出。
- 候选只取不红的块：同文件第 443 行「候选都是库里有不红结论的块：is_stored_for_reuse 刚答过」，谓词在集成副本 crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs 第 358–359 行（判过且不红，强制现判表里的答否）。读得出。
- 给定状态表零抽样：集成副本 crates/singlefs-verification/src/crash_amplification/given_states_engine.rs 第 972 行「crate::verdict_store::ForcedRejudging::NotForced,」。读得出。判表待判行本来有强制现判字节（集成副本 crates/singlefs-verification/src/verdict_store/packed_tables_rows.rs 第 134 行 `pub forced: ForcedRejudging,`），所以给定状态这一支不缺格式，缺调用方挑块与 GPU 服务判表池里对 `forced` 的处理（集成副本 gpu-verification-server/src/library_pending_pool/packed_tables_pending_rows.rs 里 grep `forced` 零命中，处理只在 image_judgement_pending_rows.rs 与 library_pending_pool.rs，量的）。
- 核对（第 ② 段）零抽样：核对待判行没有强制现判字段（集成副本 crates/singlefs-verification/src/verdict_store/facts_verifier_rows.rs 的 `FactsVerifierPendingRow` 五个字段里没有），文档却写「改核对代码不作废已核的块，核对对不对靠两边对拍与复用哨兵抓」（集成副本 crates/singlefs-verification/src/crash_amplification/facts_verification.rs 第 42 行）。这一份文件里没有哨兵逻辑（grep 命中只有这一句文档）。文档说的机制在这条路上不存在：站得住，是真缺口。
- 对拍按输入复用：键 = 输入摘要接判器引到的产品代码（集成副本 crates/singlefs-verification/src/crash_library/pairing_inputs_and_conclusions.rs 第 373 行 `pub fn pairing_conclusion_key_under_the_judged_product_code(`），两边一致就复用（同文件第 415、418 行）。对拍的三份文件里没有强制现判与哨兵的逻辑（grep 只命中集成副本 crates/singlefs-verification/src/verdict_store/pairing_rows.rs 第 7 行一句文档）。
- 哨兵抽样只在两处调：集成副本 crates/singlefs-verification/src/crash_state_enumeration/judging_stage.rs 第 112 行、集成副本 crates/singlefs-verification/src/crash_amplification/run_from_the_environment.rs 第 1317 行（`grep -rn plan_the_reuse_sentinels` 现数，量的）。

够不着的那半句：「只剩」。第 ③ 段每趟对每个块读判器那一份结论与核对表那一行逐状态比（集成副本 crates/singlefs-verification/src/crash_amplification/facts_verification.rs 第 276 行 `pub fn compare_the_conclusions_and_the_verifier_of_the_blocks(`，`is_compared` 选块）。判器改了之后新判的块与已核的核对行之间这一比是活的，缺的只是「复用的旧块」那一半。腿与判决没把这一比算进去。

替代：强制现判按比例（改法 1）配人登记整族删（改法 3）能修；其余四条碰不到。不是所有替代都中，格 1 分辨新键与旧键（旧键下判器单元进版本，判器一改旧块全不复用）。

推翻它的现象：把池级 checker 或 WGSL 改严一处，旧块里有受影响的块，跑两趟，第二趟没有任何「判定不稳」行、也没有第 ③ 段红。

### 格 2 漏判类（`research/prompts/judging-keys-input-only-r1-main-verification.md:39`「漏判类（红变绿）三样都看不见」）

判定：「三样（对拍、哨兵、强制现判）看不见」站得住；「没有东西看得见」够不着。

- 绿状态存空向量：集成副本 crates/singlefs-verification/src/crash_judge_conclusions.rs 第 36 行「绿的状态是空向量」。读得出。
- 红块现判成绿直接换上：集成副本 crates/singlefs-verification/src/verdict_store/conclusion_blocks.rs 第 321 行「两份里有一份红的：照这一份换上」，两份都不红而不同拒绝在同文件第 819 行。读得出。
- 腿与判决漏数的第四类，都在今天的树里：
  - 池级 checker 每条不变量各有一张坏镜像：集成副本 crates/singlefs-verification/tests/checker_known_bad_images/each_invariant_reddened_by_its_own_bad_image.rs 第 835 行「const INVARIANTS_WITHOUT_A_BAD_IMAGE: [&str; 0] = [];」，加第 1022–1030 行的断言「第一版每条不变量（`INVARIANTS_WITHOUT_A_BAD_IMAGE` 那几条除外）都至少有一份坏镜像，也没有清单之外的」；清单 114 条（集成副本 crates/singlefs-checker/src/image.rs 第 181 行「pub const IMPLEMENTED_INVARIANTS: [&str; 114] = [」）。这是测试代码，单测复用键里有 checker 的闭包，checker 改了会重跑。量的。
  - 变异表 `crates/mutations.tsv` 8608 行里，行上点名 `.wgsl` 的 604 行、点名 `singlefs-checker` 的 948 行（`grep -c` 现数，量的），表头写「原文在文件里必须恰好命中一次（不是的那一行记锚点失效、不跑，别的行照判，整道判红）」：改弱一行判法，若那一行有变异行，它的锚点失效，整道红，逼人重登；没有变异行的行不受这一条罩。
  - 第 ③ 段逐块比对（格 1）。
- 这几样挡不住的：弱化落在没有坏镜像分支的判法里、CPU 参照与 WGSL 与核对器三处同弱、记着红的块被改弱后的判器判成绿而静默换上（上面第 321 行那一支是有意的「记红、撤绿」）。腿的结论在这几处照样成立。
- GPU 一侧：验证包 tests 下 32 份文件引了 `known_bad_images`，其中名字带 gpu 的 9 份（`grep -rl known_bad_images tests` 现数，量的）；集成副本 crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/flows_breaking_one_field_per_invariant.rs 第 1 行写「每条不变量一条只改坏它管的那个字段的流，与逐条钉 GPU 与 CPU 逐状态对拍的用例」。114 条不变量在 GPU 一侧是否各有一张，没逐条核：推不出。

替代：强制现判按比例碰不到（候选只取不红的块，弱化后旧绿块现判仍绿）；阳性样本集（改法 2）两侧大半已经作为测试存在，新增的是「不靠单测键触发、每趟现判」；人登记整族删要靠人想起。格 2 分辨新键与旧键。

推翻它的现象：在没有坏镜像的分支上放宽一处 WGSL 条件，变异表对应行、对拍测试与第 ③ 段在新数据上都不红。

### 格 3 观察者（`research/prompts/judging-keys-input-only-r1-main-verification.md:40`「观察者：强制现判不重跑观察者函数」）

判定：站得住（读得出，没跑）。

- 不重跑：强制现判抽中的块，观察者录入块那一行在库里就算录过：集成副本 crates/singlefs-verification/src/crash_amplification/observer_recording.rs 第 1016–1018 行（`if is_forced_to_be_judged_again {` 接 `.read_observer_block(key, &kind, &observer_versions.observer)?` 再 `.is_some()`）。强制现判重拼的只有池级 checker 判观察者交回的盘那一半（同文件第 1121 行起的文档：判段重拼要读的那一行不在库里才「回录入段，只给这一块录」）。
- 标识不随意思变：版本里的判定项标识是 `observer_code:<观察者代码文件>:<观察者名>` 取 SHA-256（集成副本 crates/singlefs-verification/src/judging_units_entries.rs 第 1323 行 `format!("observer_code:{observer_code_file}:{observer_name}")`，集成副本 crates/singlefs-verification/src/crash_amplification/observer_definitions.rs 第 247 行 `pub fn observer_identity_digest(`）。观察者函数断言放宽、文件路径与名字不动，标识与版本都不变。
- 撞已定项 17：标识是路径与名字，`.claude/kb/decisions/13-验证路线.md:436`「手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时」一律不进键。这是字面冲突，不是推出来的。
- 腿与判决没提的现状：观察者登记 80 条、录入场景 150 条（集成副本 crates/singlefs-verification/src/judging_units_entries.rs 第 26 行与第 646 行两张表，`grep -c` 数到的条目，量的）；变异表里文件列带 observer 的有 84 行，其中点名共用模块 `common_crash_point_observers` 的 5 行（`awk` 现数，量的）。观察者断言有变异覆盖，每个观察者覆盖多厚没逐个核：推不出。

替代：观察者标识带修订号（改法 5）修「逼出决定」，不修「决定对不对」；旧键（观察者代码进键）不中。分辨新键与旧键。

推翻它的现象：把一个交回镜像的观察者的断言放宽，库里有它的录入块行时，强制现判那一块的日志里出现「回录入段重录」行且观察者那份结论变了。

### 格 4 判法换代没有合规的作废手段（`research/prompts/judging-keys-input-only-r1-main-verification.md:41`「判法换代没有合规的作废手段」）

判定：无杠杆那半站得住；「存不进、每趟红」那半是推的（读到存库那一步，没跑），但读到的比腿还硬一点。

- 无杠杆：集成副本里没有「全部不复用」的开关（`grep -rnE 'SINGLEFS_[A-Z_]*(REJUDGE|FORCE|NO_REUSE|WITHOUT_REUSE|REUSE)'` 在验证包源码与崩溃放量入口脚本里除哨兵变量外零命中，量的）；版本只由种类名与产品指纹定（集成副本 crates/singlefs-verification/src/crash_identity/conclusion_kinds_and_versions.rs 第 206 行 `pub fn of_the_judged_conclusion(`），不含判器的任何东西；`.claude/kb/decisions/13-验证路线.md:436`「按提交、HEAD 或树算的改动范围」一律不进键，也不许当复用依据。站得住。
- 存不进：强制现判那一块「判完照存块的规矩写回」（集成副本 crates/singlefs-verification/src/verdict_store.rs 第 1464 行），两份都不红而不同走 `DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`（集成副本 crates/singlefs-verification/src/verdict_store/conclusion_blocks.rs 第 819 行）；比的是块、违例、计数向量、恢复字（集成副本 crates/singlefs-verification/src/verdict_store/conclusion_block_rows.rs 第 243 行 `pub fn is_the_same_conclusion_as(&self, other: &Self) -> bool {`）。错误名自带「第一版不决定」：这条路是写库的人明说没定的，不是腿读岔了。
- 补一条腿没点的：计数向量里逐条不变量名与 `IMPLEMENTED_INVARIANTS` 不同，每趟收尾那一遍读就报「逐条不变量计数的行与池级 checker 的清单不同」（集成副本 crates/singlefs-verification/src/crash_amplification/conclusions_read_at_the_end.rs 第 132 行 `if invariant_names != expected_names {`），而复用谓词不看它（集成副本 crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs 第 358–359 行）。所以「增删一条不变量」这一种判器改动，今天的结局是每趟都红、红在收尾、没有自愈。这个红是响的，不是静默的。

替代：人登记整族删（改法 3）修；计数向量名不同答不复用（改法 4）单独修不了，见第三节；强制现判按比例（改法 1）修不了，也没更糟。分辨新键与旧键（旧键下版本带判器指纹，判器一改旧行就不在同一把键下）。

推翻它的现象：往池级 checker 加一条不变量，不改别的，跑两趟，第二趟收尾没有计数向量那句红，或红了而下一趟自己变绿。

### 格 5 枚举与状态物化（`research/prompts/judging-keys-input-only-r1-main-verification.md:42`「枚举与状态物化：块键里是态号」）

判定：对「号到持久集合的映射一起被改」站得住；对「只改一份实现」够不着。

- 块键里是态号：集成副本 crates/singlefs-verification/src/crash_amplification/crash_point_plan.rs 第 295 行「块键只由内容定：段写表与起步镜像（复用键）、计划哈希（段展开、判器另读的输入、态数）、段内序号；设点文件、点名、身份都不进。」；枚举算法本身不进。读得出。块宽改了有先例防（同文件第 315–316 行把块长拼进计划哈希，因为「块长不进键时」会复用没判过的状态），次序没有。
- 映射不止一份：序号到持久集合在三处各实现一遍：集成副本 crates/singlefs-verification/src/crash/crash_state_plan.rs 第 984 行、集成副本 crates/singlefs-verification/src/crash_facts.rs 第 750 与 1060 行、集成副本 crates/singlefs-verification/src/crash_judge_tables.rs 第 833 行；`crash_facts.rs` 的文档写明与 `CrashStatePlan` 是同一套数字次序（同文件第 1049 行）。多份测试互相对（例：集成副本 crates/singlefs-verification/tests/crash_judge_tables_versions_reproduce_every_crash_image.rs 第 172 行把判表持久集合重现成镜像）。这些是测试代码，在单测复用键里，枚举代码改了会重跑。
- 所以只改一份实现会红；同一次改一个共用的辅助（切段、段内序号拆法）让几份一起变，三者仍然互相一致，测试不红，块键不变。腿与判决点的是后一种，成立。
- 旧键下这一格不成立（主工作区的登记表里，判表建法、`crash_facts::verify_states_from_facts` 与 `crash_facts::extract_flow_facts_with_slices` 都挂在入口上，`grep -n` 在 `crates/singlefs-verification/src/judging_units_entries.rs` 第 75–78、106–107、123 行命中，量的）：分辨新键与旧键。六条改法没有一条点名这一格。

用户 100 行那句「这个过程是键值，但是他和录入的函数无关，他是录入的过程路径」，一种读法是键应取「遍历到的每个状态的持久集合」而不是「态号」；这是读法，不是定案，没当结论用。

推翻它的现象：改共用的段内序号拆法（序号 0 到 N 的态集合不变、次序变），库里已判块全复用、判表重现镜像的测试不红、三处实现仍一致。

### 格 6 删旧库（`research/prompts/judging-keys-input-only-r1-main-verification.md:43`「删旧库：旧格式待判行一行就让 GPU 服务整扫失败」）

判定：前三句站得住，第四句（02、03 删不掉）够不着；整格不分辨新键与旧键。

- 待判行：格式号已是 2（集成副本 crates/singlefs-verification/src/verdict_store/gpu_pending_rows.rs 第 32 行 `const GPU_PENDING_ROW_FORMAT_NUMBER: u8 = 2;`），读不出的一行让整扫失败（同文件第 340 行 `GpuPendingRow::from_value_bytes(&value_bytes).ok_or_else(not_in_the_layout)?;`）；服务一侧扫不了就点名一次、交空（集成副本 gpu-verification-server/src/library_pending_pool/image_judgement_pending_rows.rs 第 17 行「扫不了待判行（库里有一行不合验证包的布局）」）。是整道停转，不是假绿。
- 布局号没升：集成副本 crates/singlefs-verification/src/verdict_store/library_opening_and_layout.rs 第 7 行「const LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION: u32 = 6;」。没重编的二进制照旧开得了库。
- 02、03 删不掉：拒的是写批里的删除记号（集成副本 crates/singlefs-verification/src/verdict_store/prune_server_storage.rs 第 573 行 `if has_a_deletion_where_the_deletion_marker_reads_as_a_row {`），原因是空值在那两张表里读成一行。用户要的是经 prune-server 客户端删前缀，那一帧「不认表码、不认键的种类」（`prune-server/src/protocol.rs:30`），客户端也写明「不认表码、不为哪一张表写分支」（`scripts/prune-server/prune_server_client.py:143`），不经写批，不拒。所以「标保留」不是必须；留着无害是真的，删也删得掉（推的：没起服务试，只读了协议文件头与客户端用法）。
- 不分辨：任何改了行格式的设计（含旧键）都要处理这一格；它不该算作对「键只有输入数据」的打中，但要处理。

推翻它的现象：用旧二进制与新二进制先后开同一份库，新二进制写下的旧格式待判行之外还有别的读不出的行，或旧二进制开库被拒。

### 格 7 已定项 17 内部的字面冲突（`research/prompts/judging-keys-input-only-r1-main-verification.md:44`「已定项 17 内部的字面冲突」）

判定：拆成三支，结论不同。

| 支 | 判定 | 依据 |
|---|---|---|
| 判定项标识 对 名字与路径 | 站得住 | 集成副本 crates/singlefs-verification/src/judging_units_entries.rs 第 1323 行把观察者代码文件路径与名字拼成标识；`.claude/kb/decisions/13-验证路线.md:436`「路径（仓根也算）、名字」一律不进键 |
| 结论种类定值版本 对 手写的版本常量 | 站不住 | 集成副本里版本是种类名接判器引到的产品指纹取 SHA-256（集成副本 crates/singlefs-verification/src/crash_identity/conclusion_kinds_and_versions.rs 第 212 行 `"singlefs conclusion version\nkind={}\n{}",`），没有会被人手改的版本数。腿读的是第一轮的补丁草稿，那一版已经不在了 |
| 测试服务代码不进任何键 对 单测键照旧 | kb 字面冲突站得住；实现一侧推不出 | `.claude/kb/decisions/13-验证路线.md:443`「测试服务代码（task-server、gpu-verification-server 连同它跑的 GPU 判器与核对内核、prune-server）不进任何键」与 `.claude/kb/decisions/13-验证路线.md:461`「照旧：测试闭包引到的产品代码与测试代码」，三类代码表把核对（`crash_facts.rs`、`crash_verify_gpu.rs`）归测试服务代码（`.claude/kb/decisions/13-验证路线.md:452`「核对（CPU 参照」）。集成副本的脚本一侧没有按这条排文件：按角色排除的只有 `NOT_JUDGING_FILES` 那两份（`scripts/gate/lib_function_units.py` 的「按角色排除的文件」一节，我在 scripts 下的 .py、.sh 里 grep「测试服务代码」零命中，量的）。所以腿说的「改核对代码不让测它们的单测重跑」今天不成立，等这一条落到脚本里才成立 |

替代：不分辨。三类代码表是 2026-11-19 的第一道定案，不是「键只有输入数据」带来的；换成六条改法照样中。

## 三、用户已定的三格：定案真关掉了腿打中的哪一半

判决「二、判决」第一张表（`research/prompts/judging-keys-input-only-r1-main-verification.md:32`「崩溃判定的键 = 块的输入数据 + 判器引到的被检测产品代码的指纹」）把这三格标成不再挂起。逐格看用户原话与集成副本。

### 定案甲：产品代码进键（`records/2026-11-19-判定键只算产品与测试代码.md:99`「崩溃判定的键 = 块的输入数据 + 判器引到的被检测产品代码的指纹」）

关掉的：攻方腿 A2 说的「块键与版本里没有任何东西随产品恢复代码变」。集成副本里三种判器结论的版本是种类名接 `judged_product_code=<指纹>`（集成副本 crates/singlefs-verification/src/crash_identity/conclusion_kinds_and_versions.rs 第 206 行起），对拍结论的键同样拼它（集成副本 crates/singlefs-verification/src/crash_library/pairing_inputs_and_conclusions.rs 第 373 行），一组入口 6 个（集成副本 crates/singlefs-verification/src/judging_units_entries.rs 第 1269 行），测试钉了这一组算得出键、只含产品包单元、含 `recover`（集成副本 crates/singlefs-verification/src/crash_identity/compiled_judging_units_key_table/tests/judged_product_code.rs 第 6–11 行）。

没关掉的另一半（都是读出来的，没跑）：

- GPU 一侧恢复是 WGSL 另写的一份，不在闭包里（集成副本 crates/singlefs-verification/src/judging_units_entries.rs 第 1267 行起的文档「GPU 判器的 WGSL」「不在这几个入口的闭包里」）。产品恢复改了，版本换，GPU 路线的块用没改的 WGSL 重判，存在新版本下，像是新恢复判过的。发现它靠对拍测试重跑（测试闭包含 CPU 一侧 `recover`）与第 ③ 段逐块比对（核对器是 CPU，入口也在这一组里）；块里没有一个状态的结局因恢复改动而不同时，两处都不红。
- 6 个入口是手登的，没有「新加一条判器入口没登记」的检查：在验证包的 src、tests 与 build.rs 里 grep `JUDGED_PRODUCT_CODE_ENTRIES`，命中只有表本身、构建脚本、键表的单测与几处文档；上面那条测试只钉「含 `recover`」。入口登漏，漏的入口只引到的产品代码就静默不进版本。
- 闭包收不全就整组拒算、版本改拼占位、那组的结论「照判不入库」（集成副本 crates/singlefs-verification/src/crash_identity/conclusion_kinds_and_versions.rs 里的 `conclusion_versions_kept_out_of_the_library_in_this_process`）；拒算里说闭包收没收全的，不论落在哪份文件都留（集成副本 crates/singlefs-unit-graph/src/closure/units_kept_by_package.rs 里 `outcome_keeping_the_listed_packages_with_the_entry_target` 的函数文档）。判器入口先经验证包与 checker 的代码再到产品代码，那里有动态分派、宏拼名字就拒算整组，复用归零。今天这一组拒不拒算没量：写进第六节要主 agent 跑的命令。

### 定案乙：录入只是管道、按屏障（`records/2026-11-19-判定键只算产品与测试代码.md:100`「录入只是一种管道。录入是否完全录入是录入检测和设计的课题，不是判定的问题。」，`records/2026-11-19-判定键只算产品与测试代码.md:102`「按照屏障录入，能改变键值的就是屏障的编号。和录入代码无关。是产品代码的屏障决定键值」）

关掉的：录制管线放不放进键。不放：集成副本 crates/singlefs-verification/src/judging_units_entries.rs 第 12–16 行文档「录制管线、判器、观察者……都不进」。

没关掉的：

- 「录入检测」没有落成任何一道检查：`grep -rln 录入检测 crates scripts` 在集成副本零命中（量的）。用户把「录没录全」交给它，今天没有它。
- 攻方腿 B1 的机制没动：录入键命中就跳过 `record_into`（集成副本 crates/singlefs-verification/src/crash_state_enumeration/flow_through_the_library.rs 第 21 行算流字节、第 37 行算录入键、第 46 行 `let manifest = flow_to_record.record_into(library, gpu_judges, &recorded_bytes)?;`）。事实抽取、建判表不在键里，改了它们，写流字节与录入键不变，旧事实表与旧判表照用。这一半用户的话没碰：用户说的是「不要把管线放进键」，B1 要的是「命中时别信旧产物」。
- 「屏障的编号」按字面读不够：同样多屏障、字节不同的两条流会撞同一把键。代码里块键是写表内容的摘要（集成副本 crates/singlefs-verification/src/crash_amplification/crash_point_plan.rs 第 295 行「段写表与起步镜像（复用键）」），比用户的话强；不是照话做出来的，写回 kb 时别把「编号」写成键。
- 判决没列、腿 F4 提过的补法：命中录入键时仍从流字节现算事实表与判表、与清单里的摘要比（`research/prompts/judging-keys-input-only-r1-opus-output.md` 的 F4 行）。代价上界：基线里一条大用例 22,440 秒中「录流加算键加写流与判表约 2.7 秒」（基线报告第 118 行；这 2.7 秒含录流与写库，所以对抽取与打包是偏大的上界，推的），150 条录入场景（集成副本 crates/singlefs-verification/src/judging_units_entries.rs 第 26 行那张表，`grep -c` 数到，量的）按每条都是大用例算，上界约 405 秒一趟，对 84,710 秒的录入段是 0.5%（awk 算的，推的）。这条没进六条改法，建议第三轮看。

### 定案丙：编译档、线程数、工具链、依赖版本（`records/2026-11-19-判定键只算产品与测试代码.md:105`「经常变得都不该进」）

用户原话只到编译档与线程数：`records/2026-11-19-判定键只算产品与测试代码.md:105`「编译档与线程数原话」之后是引号里的「经常变得都不该进」，结论是「都不进键」。「工具链与依赖版本也不进」只出现在主 agent 自己的记录行里（`records/2026-11-19-判定键只算产品与测试代码.md:110`「编译档线程工具链依赖不进键」），那一行没有用户的引号。判决把它说成用户已定，原话出处第二轮材料要补；我在记录里没找到。

关掉的：编译档。集成副本 crates/singlefs-unit-graph/src/build_toolchain.rs 第 2 行「编译档（`[profile.<名>]` 的各项设置、增量编译开关）不拼进来，也不读：哪个档编的不进任何键。」线程数同理（只定一块切几段，不改结论，判决已引）。

没关掉的：工具链与依赖版本还在三组键里。集成副本 crates/singlefs-verification/build.rs 第 160 行起拼 `rustc -vV`、`cargo -V`、目标三元组与 RUSTFLAGS 的工具链行，第 168 行读 `Cargo.lock`，观察者组、录入组、判器引到的产品代码组都传这两样（同文件第 224、242、264 行）。所以现状是：升一次 rustc 或 `cargo update` 一次，所有判器结论的版本、所有观察者版本、所有录入键都换，全库作废。这正是腿问 3 问的那一格，没有被关掉。

两种出路，要用户定一句：
- 若工具链与依赖也不进：改点在上面那几处传参。注意工具链行里的目标三元组是闭包算平台条件用的（`crates/singlefs-unit-graph/src/bin/unit_graph.rs` 文件头「目标平台的 cfg……从工具链的目标三元组查表」），不能整行去掉，只能去 rustc 与 cargo 的版本行与 RUSTFLAGS 里不影响 cfg 的部分。
- 若留着：改写 `records/2026-11-19-判定键只算产品与测试代码.md:110`「编译档线程工具链依赖不进键」那句。

「不进」的代价（腿问 4 要的反例，推的）：编译器升级让不稳定排序的并列次序变，字节就变，键却不变。仓内候选：集成副本 crates/singlefs-core/src/transaction/transaction_output.rs 第 558 行 `.sort_unstable_by_key(|record| std::cmp::Reverse(record.rollback_txg))`，同一个 `rollback_txg` 两条记录并列时次序由标准库实现定；该函数文档自己写「同一个 T_rb 两条的谱系接不成一条父链，这里不另判」，说明并列只在坏盘上出现。没查这个函数进不进判器闭包、崩溃枚举会不会造出并列的谱系记录：所以只是候选，不是已坐实的反例。产品包里 `sort_unstable` 共 15 处（core 10、replay-prefix 2、background-gpu 3，`grep -rn` 现数，量的），`HashMap` 与 `HashSet` 13 处（core 7、replay-prefix 1、background-gpu 5），依赖只有 `libc = "0.2"`、`vstd` 与精确钉死的 `wgpu = "=30.0.1"`（各 `Cargo.toml` 现读）。

推翻定案丙「没关掉」的现象：用户另有原话把工具链与依赖也定了不进（找得到出处），或集成副本里这几处传参已经改掉。

## 四、六条零轮改法：在它点名的那一格上构造一次改动走一遍

判决「三、改法」表在 `research/prompts/judging-keys-input-only-r1-main-verification.md:52`「强制现判从一块改成按比例」起到 `research/prompts/judging-keys-input-only-r1-main-verification.md:57`「布局号升到 7」止。下面逐条写落点、构造、走完的结局。都是读代码推的，没编、没跑。

### 改法 1：强制现判按比例或按块键哈希轮转，同一个旋钮罩对拍库、给定状态表、核对行、观察者录入（`research/prompts/judging-keys-input-only-r1-main-verification.md:52`「强制现判从一块改成按比例或按块键哈希轮转取一批」）

点名格 1、4。判定：格 1 变严那半修得了；格 1 变弱那半与格 4 修不了；四个消费者里两个单包内改（对拍库、观察者录入），两个要跨包改（给定状态表、核对行）。

走一遍：
- 变严：池级 checker 与 WGSL 同时加一条条件，旧绿块里 k 块该红。落点是集成副本 crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs 第 430 行那一次抽（一块改一批）。抽中的受影响块现判成红，新红整份换上库里旧绿（集成副本 crates/singlefs-verification/src/verdict_store/conclusion_blocks.rs 第 321 行那一支），与库里不是同一结论，第 644 行那句「判定不稳」报红。抽中概率 1-(1-p)^k；按块键哈希轮转时 1/p 趟之内必抽中（确定）。修得了，p=1 是确定的。
- 变弱：旧绿块被弱化后的判器重判仍是绿，与库里逐字相同，不报。p=1 也不报。候选本来只取不红的块（集成副本 crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs 第 358–359 行）。修不了格 2。
- 格 4 加不变量：每个被抽的块现判出的计数向量名与库里不同，存库拒绝（集成副本 crates/singlefs-verification/src/verdict_store/conclusion_blocks.rs 第 819 行）。修不了。没更糟：今天收尾那一遍读已经让所有旧块红（集成副本 crates/singlefs-verification/src/crash_amplification/conclusions_read_at_the_end.rs 第 132 行），比例只改「现判几块」，不改「红几块」。
- 四个消费者（推的）：
  - 对拍库：集成副本 crates/singlefs-verification/src/crash_library/pairing_inputs_and_conclusions.rs 第 415 行 `if stored_pairing_conclusion_is_reused(&stored) {` 前加「选中则不复用、重判、与 `stored.conclusion` 逐字节比」。单包内改，修得了单边判器变动。
  - 给定状态表：集成副本 crates/singlefs-verification/src/crash_amplification/given_states_engine.rs 第 972 行把 `NotForced` 改成按选中取值；判表待判行已经有 `forced` 字段（集成副本 crates/singlefs-verification/src/verdict_store/packed_tables_rows.rs 第 134 行），GPU 服务的判表池还不处理它。要同时改验证包与 GPU 服务，跨包。
  - 核对行：待判行没有 `forced` 字段，服务一侧的核对池也没有强制现判的处理（集成副本 gpu-verification-server/src/library_pending_pool/facts_verifier_pending_rows.rs 里 grep `forced` 零命中，量的）。要同时改验证包的行格式与 GPU 服务。跨包。
  - 观察者录入：集成副本 crates/singlefs-verification/src/crash_amplification/observer_recording.rs 第 1016–1018 行，强制现判时不看 `.is_some()` 一律算没录过。单包内改；代价见第五节，是四个里最贵的。
- 新问题：旋钮罩四处，每处各一个抽法，「同一个旋钮」是口号，实现是四处各接一个。`p=1` 的「全量对照」今天没有对应开关（`crash-amplification.sh` 里没有）。

推翻它的现象：变严构造跑 `⌈1/p⌉` 趟（轮转）仍无一处报红。

### 改法 2：每趟两边现判一张阳性样本集，不入库复用（`research/prompts/judging-keys-input-only-r1-main-verification.md:53`「每趟两边现判一张阳性样本集（每条不变量至少一张违例镜像），不入库复用」）

点名格 2。判定：修得了「有坏镜像的分支」，修不了「没有坏镜像的分支」；改法的实质已大半以测试存在，新增的是触发方式。

走一遍：
- 构造一：只把 WGSL 里一条不变量的违例条件放宽、CPU 不动。现状：CPU 与 GPU 逐状态对拍的测试（集成副本 crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/flows_breaking_one_field_per_invariant.rs 第 1 行写「每条不变量一条只改坏它管的那个字段的流，与逐条钉 GPU 与 CPU 逐状态对拍的用例」）在今天的单测键里带着 WGSL（推的：没算过这条测试的闭包，命令见第六节），WGSL 改了就重跑、两边不一致、红。照 `.claude/kb/decisions/13-验证路线.md:443`「不进任何键」把测试服务代码移出键之后，这条测试 WGSL 改了不重跑，红不了：这正是改法 2 补的地方。修得了。
- 构造二：放宽的是同一条不变量的第二个违例分支，坏镜像只踩第一个分支。样本集每趟现判也绿。变异表里点名 `.wgsl` 的 604 行对 114 条不变量，平均每条约 5.3 行（awk 算的，推的：一行是一个变异点，不等于一个分支），说明一条不变量不止一个分支，单张样本罩不全。修不了。
- CPU 一侧已有：集成副本 crates/singlefs-verification/tests/checker_known_bad_images/each_invariant_reddened_by_its_own_bad_image.rs 第 835 行 `const INVARIANTS_WITHOUT_A_BAD_IMAGE: [&str; 0] = [];`，所以 114 条每条都有一张。
- 新问题：样本集在 GPU 上现判要走 GPU 服务（卡只归服务），每趟多一次服务往返；量级小（114 张乘一两个状态，推的）。

推翻它的现象：构造二跑出红，说明样本集或变异表已经罩到那个分支。

### 改法 3：判法换代由人登记、经 prune-server 客户端整族删那一种结论（`research/prompts/judging-keys-input-only-r1-main-verification.md:54`「判法换代由人登记、经 prune-server 客户端整族删那一种结论」）

点名格 4。判定：整表删修得了，「那一种结论」做不到。

走一遍：往池级 checker 加一条不变量，每趟红在收尾（格 4）。人登记「判法换代」，经客户端 `delete-prefix` 删：
- 键的布局是「结论块 `0A` ‖ 块键 ‖ 结论种类 ‖ 版本」（集成副本 crates/singlefs-verification/src/verdict_store.rs 第 39 行），种类在 72 字节块键之后，前缀删够不到种类；客户端的数据命令只有 `check`、`read`、`delete-prefix`、`delete-by-item-name`（`scripts/prune-server/prune_server_client.py:134`「命令行：」往下几行）；最后一个按值里 JSON 的 item 字段挑键，崩溃判定库的二进制值不是 JSON，用不上。所以只能整张表删，镜像判定与观察者两种结论一起没。「按种类删」要新写一个扫描加过滤的命令，或把种类挪到键头（布局号再升，与改法 6 合并做）。
- 要删的表不止结论块：结论违例 `0B`、恢复字 `0C`、计数向量 `0D`、块属性 `0E`、被抛弃根 `18`，还有核对表 `08`（键布局表里没列，行键由集成副本 crates/singlefs-verification/src/crash_amplification/facts_verification.rs 第 44 行 `pub fn facts_verifier_row_key(` 给）、对拍表 `19` 与观察者盘上的池级 checker `17`（后两张见集成副本 crates/singlefs-verification/src/verdict_store.rs 的键布局表）。漏一张，新判器就看到同一版本下的陈旧行，静默复用。这份清单要入库登记，不能每次靠人想。
- 要先停判段与 GPU 服务：服务写完结论之后才删，删了又被写回。`delete-prefix --dry-run` 能先数条数：`scripts/prune-server/prune_server_client.py:144`「--dry-run 只数不删，数的口径和删一样。」
- 删完以后收尾那一遍读与存库都没有旧行可比：格 4 的卡死解开。修得了（带上面三条）。
- 触发还是人：改法 3 不回答「判器改了谁发现」，只回答「发现之后怎么办」。与格 1 的强制现判配着用：强制现判报「判定不稳」，人看到再登记。

推翻它的现象：按上面的表清单删完、重起服务，收尾那一遍读仍有计数向量或库里同版本下的旧行。

### 改法 4：计数向量里不变量名与当前清单不同的块答不复用（`research/prompts/judging-keys-input-only-r1-main-verification.md:55`「计数向量里不变量名与当前清单不同的块答不复用」）

点名格 4（存不进那一支）。判定：单独修不了。

走一遍：加一条不变量。复用谓词（集成副本 crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs 第 349 行起的 `is_stored_for_reuse`）多读计数向量，名字清单不同的块答「不复用」，现判。现判出新绿，计数向量多一行名字，与库里同键同版本的旧绿比，`is_the_same_conclusion_as`（集成副本 crates/singlefs-verification/src/verdict_store/conclusion_block_rows.rs 第 243 行）解构里就有 `count_vector`，不同；两份都不红，拒绝（集成副本 crates/singlefs-verification/src/verdict_store/conclusion_blocks.rs 第 819 行）。改法 4 把「复用」放开了，「存」还是关着，红的位置从收尾那一遍读挪到判段写库。要加一个存库规则才够：存着的计数向量名与当前清单不同的绿，允许被新绿整份换上。这条规则要让库层知道 checker 的清单，库层今天不知道（谓词由调用方传进去才行）。

加上存库规则之后的边界：只修「名字清单变」这一种；判器改动让逐条计数的数变（判到、不适用的口径变）、恢复字变而名字不变的，仍卡。

推翻它的现象：加存库规则后，同一构造下判段写库不再报 `DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`，且收尾那一遍读转绿。

### 改法 5：观察者标识带修订号，钉表核观察者指纹（`research/prompts/judging-keys-input-only-r1-main-verification.md:56`「观察者标识带修订号，钉表核观察者指纹（指纹不进键）逼出换号」）

点名格 3。判定：修「逼出一次决定」，修不了「决定对不对」，并撞已定项 17。

走一遍：放宽 `common_crash_point_observers/mod.rs` 里一个观察者的断言。标识（路径加名字）不变，版本不变，旧录入块与旧结论照用（格 3）。加钉表：每个观察者一行，记入口单元闭包里非产品单元的指纹与修订号，检查按单元图现算与钉表比，不同则红。
- 红了出路两条：修订号加一（观察者那一把版本换，`16`、`17` 里它的行全不复用，重录全部块，是整个录入段里最贵的那种，见第五节），或重钉指纹并写「意思没变」。第二条是人的断言，检查不验。
- 钉的对象是观察者入口 fn 的闭包里非产品的单元：共用辅助函数（例 `common_crash_point_observers/mod.rs` 里被多个观察者调的计数函数）一改，登记的 80 个观察者里挂着它的全红。噪声大，但方向对。
- 修订号是手写的版本常量：`.claude/kb/decisions/13-验证路线.md:436`「手写的版本常量」一律禁，所以要先写一条例外才能落地。判决说的张力是「判器那几组入口登记整批删」，集成副本里观察者登记表还在（集成副本 crates/singlefs-verification/src/judging_units_entries.rs 第 646 行），那一条张力今天不成立；真正的冲突是 436 行这一句。
- 指纹的算法落点：入口单元在 `LeftOutWithTheEntryUnits` 下被排出键（集成副本 crates/singlefs-unit-graph/src/closure/units_kept_by_package.rs 里该枚举的文档），所以钉表要另算一遍不过滤的闭包，放在 `scripts/` 一侧，不进构建脚本。

推翻它的现象：把一个观察者里提取变量（意思不变）也让钉表红，说明钉的粒度比「意思」细；或放宽断言而钉表不红，说明钉错了闭包。

### 改法 6：布局号升到 7；删库次序写成「停判段 → 删待判行与崩溃判定各表 → GPU 服务照新代码重起」，`02`、`03` 标保留（`research/prompts/judging-keys-input-only-r1-main-verification.md:57`「布局号升到 7；删库次序写成」）

点名格 6。判定：修得了，有一个窗口；「02、03 标保留」不必要。

走一遍：
1. 停两台上所有判段与 GPU 服务。
2. `delete-prefix` 删待判行、结论各表、对拍表、核对表（清单同改法 3）。
3. 删布局行 `00`。
4. 新代码起、头一次开库，布局行不在，写下 7（集成副本 crates/singlefs-verification/src/verdict_store/library_opening_and_layout.rs 第 7 行的常量改成 7；该文件文档写「库只认这一个布局号」）。
- 窗口：第 3 步之后、第 4 步之前，任何一份没重编的旧二进制开库，布局行不在，会写下 6；之后新二进制读到 6 拒绝开库，要再删一次布局行。不升号时同一窗口的后果更糟：旧二进制开得了库，往里写旧格式待判行，GPU 服务整扫失败（格 6）。升到 7 把「静默毒化」变成「响亮拒绝」。
- 升号还有一条好处：库里布局行还是 6 时，新二进制一律拒绝开库，逼着第 2、3 步先做；不会有人忘了删就起新代码。
- 客户端没有「写一行」命令，只能删布局行等新代码来写；要一步到位须新写客户端命令，或在删完之后立刻起新代码。
- `02`、`03`：见格 6，`delete-prefix` 不看表码，删得掉；两张表要一起删（编号与内容互指），并且得先删完引它们的结论行。留着无害，所以「标保留」可以做也可以不做，不是必须。
- 同一次删还应把 `10`（录入清单）一并清掉：新键另立一组，旧清单读不到，留着只是占地（推的：没读 `10` 的读法）。

推翻它的现象：按上面四步做完，旧二进制开库仍被接受，或新二进制开库被拒而布局行是 7。

## 五、耗时与复用（正文第三节第 5 问）

基线数全部取自 `/home/fy5090/.local/share/singlefs/work/judging-keys-baseline-costs/result.md`（下称基线报告，引「基线报告第 N 行」），那份里的数是量的；下面各段的新键估值全是推的，算术用 awk 做。没假设「被采纳的改法」是哪条，以改法 1（强制现判按比例）为被采纳项估代价。

### 5.1 各段在新键下的变化

| 基线段（基线报告） | 非产品代码改动之后（判器、观察者、录制管线、核对器、服务、编译档、线程数） | 产品代码改动之后 | 没改动的第二趟 | 出处与量级 |
|---|---|---|---|---|
| 录入段 约 84,710 秒，占 60%（第 11 行） | 新键下不重录；上一版每次整段重录，每轮约 8 万秒（第 233 行） | 与上一版相同：观察者版本含观察者引到的产品代码（集成副本 crates/singlefs-verification/src/crash_amplification/observer_definitions.rs 第 247 行起的版本拼法），引到的产品函数一变，它的录入块全重录 | 命中，近 0 | 量的：9 项合计；推的：新键下的变化 |
| 等卡 约 24,784 秒，占 17%（第 11 行） | 不由键定，由 GPU 服务排队定；间接：现判态数少，卡上队列短 | 同 | 同 | 一项 17333 占约 22,620 秒（第 98 行）；变化推不出 |
| 判段 32,629 秒，占 23%，其中 GPU 判 28,498 秒（第 99–100 行） | 重判归零。上一版换判器键一次重判的量：约 4.4 亿态（事件 3）到 56.6 亿态（事件 1），按四卡合计约 6.5 万态每秒是 1.9 到 24 小时（第 183–185 行，推的） | 版本换，已判的块全部现判，与上一版相同 | 全复用 | 上界见 5.2 |
| 写库 852 秒、内核编译 53 秒（第 103–104 行） | 不变 | 不变 | 不变 | 量的 |
| 每片编验证包 17 到 24 分钟（第 117 行） | 不变；构建脚本要算的闭包组数变多，见 5.3 | 同 | 同 | 量的；新增部分没量 |
| 变异表、广谱变异、崩溃点重放按条（第 223–224 行） | 不变（键照旧） | 不变 | 不变 | 基线报告自己的判断 |

### 5.2 复用上限：库里判过的态有多少是换键造成的

基线报告第 153–157 行读真库：`recorded_states=83204785477 states_judged_under_the_current_key=35893860 states_judged_only_under_another_key=5980821966 states_never_judged=77188069651`（量的）。awk 算：

- 判过而键对不上的 5,980,821,966 态是录入态的 7.19%；从没判过的 77,188,069,651 态是 92.77%；当前键下的 35,893,860 态是 0.043%。
- 若这 59.8 亿态的换键全部由非产品代码引起，新键下它们都能复用，上界是 5,980,821,966 / 285,868,471（基线 9 项复用的态）≈ 20.9 倍；按 6.5 万态每秒是 92,013 秒，约 25.6 小时 GPU（awk 算的，推的）。
- 若全由产品代码引起，一态都不多。真实值在两者之间，基线报告第 254 行自己说没比组成清单，没有现成命令；本报告也没比。
- 要点：复用率拿不高的主因不是键。9 项现判 605,888,874 态、复用 285,868,471 态，复用 32.1%（awk）；库里 92.77% 的录入态从没判过，那部分新键救不了（用户定「一共跑100亿就好了」的缩范围才管它）。

第一趟不省：用户定旧库不能复用的都删（`records/2026-11-19-判定键只算产品与测试代码.md:94`「旧的数据和判定 如果无法复用的就都删掉好了」），内容寻址的几张表（`0F`、`11`、`12`、`14`、`1B`、`1C`）按内容存，新键下读得到，可以留，省的是「录流加算键加写流与判表约 2.7 秒」那一级（基线报告第 118 行），观察者录入块 `16` 的键含版本，要重录，那才是大头。

### 5.3 哪里会让耗时不降反升

| 位置 | 机制 | 量级 | 标 |
|---|---|---|---|
| 改法 1 对观察者录入的强制现判 | 强制现判块上观察者真重跑，不走见证去重（集成副本 crates/singlefs-verification/src/crash_amplification/observer_recording.rs 第 1016–1018 行改成不看 `.is_some()`）。每趟多重跑 p × 复用态数 | 每态成本有两个口径：基线每态 1646.26 微秒（基线报告第 112 行，是真跑观察者的态）；9 项录入段 84,710 秒除以 891,757,345 个态是 94.99 微秒每态（awk，含去重，是下限）。p=0.01：272 到 4,706 秒；p=0.1：2,715 到 47,061 秒；p=1：27,155 到 470,614 秒（130.7 小时）。后者是 9 项录入段的 5.6 倍 | 推的 |
| 改法 1 对 GPU 判的强制现判 | 多判 p × 复用态数 | 285,868,471 × p ÷ 6.5 万：p=0.01 是 44 秒，p=0.1 是 440 秒，p=1 是 4,398 秒，对判段 32,629 秒是 13% | 推的 |
| 改法 5 换观察者修订号 | 该观察者所有 `16`、`17` 行不复用，全块重录 | 同上一版「观察者代码变」一次的整段重录，量级每轮约 8 万秒（基线报告第 233 行） | 推的 |
| 改法 3、6 整库删 | 下一趟等于第一趟，全部现判 | 判段 891,757,345 ÷ 6.5 万 = 13,719 秒（9 项）；录入段 84,710 秒；两样都回到基线 | 推的 |
| 判器引到的产品代码那一组拒算 | 整组版本改拼占位、结论「照判不入库」（集成副本 crates/singlefs-verification/src/crash_identity/conclusion_kinds_and_versions.rs 里 `conclusion_versions_kept_out_of_the_library_in_this_process`）；观察者那 80 组同理各自拒 | 每趟都回到基线：判段与录入段都不复用 | 条件式：拒不拒算今天没量 |
| 构建脚本组数变多 | 每次编验证包算 231 组闭包（录入 150 加观察者 80 加判器 1，`grep -c` 数，量的），主工作区今天的构建脚本是 94 组（登记表里 `group_name:` 12 个加观察者 82 个，`grep -c` 数，量的；基线报告第 186 行说的是当时的 91 组）；每片各编一遍 | 单组闭包用时没量；乘 231 除以线程数再乘片数，才是新增的挂钟 | 没量 |
| 判器引到的产品代码取得太宽 | 入口 6 个先经验证包与 checker 的代码再到产品代码，闭包里产品函数越多，产品改动命中的概率越大，复用越少 | 没量闭包里产品单元数；命令见第六节 | 没量 |

### 5.4 要主 agent 量的数

- 库里判过的态按当前键与别的键的拆分，今天的真库：命令 1。
- 判器引到的产品代码那一组拒不拒算、闭包里有多少产品单元、算一次多久：命令 2、3。
- 构建脚本新增时间：命令 4、5。
- 强制现判的每态成本与 p 的取值：要改法 1 落地之后才量得到，今天没有装置。

## 六、要主 agent 跑的命令

没有模型，所以没有 `rerun.sh`。下面每条都不写脚本自己的上限与线程参数，被范围声明闸或项目代码闸拒了，照拒绝说明改写法再交。

1. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=崩溃判定库读库程序的 keys 子命令（只读）; 范围=1 个二进制; 为什么=把库里判过的态拆成当前键下的与别的键下的，估新键下的复用上限' task-server/task submit -- cargo run --release -p singlefs-verification --bin crash_library_reader -- keys`。起点：仓根。看：`TASK_SERVER_RESULT` 行 `type=succeeded exit=0`；`task-server/task log <号> --all` 里 `CRASH_LIBRARY_STATES` 行的三个数与 `CRASH_LIBRARY_STORED_VERSION` 各行；用法与行格式在集成副本 crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头。
2. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=验证包库里 1 条测试; 范围=1 个测试; 为什么=核判器引到的产品代码那一组在集成副本里算得出键' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo test -p singlefs-verification --lib -- crash_identity::compiled_judging_units_key_table::tests::judged_product_code::the_judged_product_code_group_is_keyed_with_product_units_only_and_holds_the_recovery --exact`。起点：集成副本根。看：`exit=0` 且日志里「1 passed」；「0 tests」不算通过（全名要先用 `--list` 核）。
3. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=单元图 closure 子命令算判器引到的产品代码那一组的闭包; 范围=6 个入口; 为什么=看闭包里有无拒算、产品单元有多少、算一次多久' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo run --release -p singlefs-unit-graph --bin unit-graph -- closure . singlefs_verification::crash::judge_crash_image singlefs_verification::crash_judge_cpu_blocks::judge_a_piece_of_a_block_on_the_cpu singlefs_verification::crash_given_states::judge_the_given_states_on_the_cpu singlefs_verification::crash_amplification::images_after_the_observers::judge_the_images_after_the_observers_on_the_cpu singlefs_verification::crash_facts::verify_states_from_facts singlefs_verification::crash_facts::verify_states_from_facts_by_the_segment_expansions --target lib --toolchain-of-the-build --lock-file Cargo.lock --judgment-kind product-behavior`。起点：集成副本根。看：退出码 0 是闭包没拒算，3 是有拒算（命令行没有按包过滤，所以 3 只说明未过滤的闭包里有拒算，要看拒算行落在哪个包、是不是「说闭包收没收全」那一种）；标准错误里「单元数、拒算数与用时」那一行；日志里单元行按路径前缀 `crates/singlefs-core`、`crates/singlefs-format` 等数产品单元个数。
4. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=验证包构建的计时; 范围=1 个包; 为什么=量新增的构建脚本时间' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo build --release -p singlefs-verification --timings`。起点：集成副本根。看：退出码 0；编译目录里 `cargo-timing` 的页面里 `singlefs-verification build-script (run)` 那一行的秒数。
5. 命令：同命令 4，起点换成仓根（上一版的构建脚本）。看：同一行秒数，两个相减就是 231 组与 94 组的差。
6. 命令（核格 7 与改法 2 的前提，单测键今天含不含 WGSL）：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=单元图 closure 子命令算一条 GPU 对拍测试的闭包; 范围=1 个测试入口; 为什么=看单测键今天含不含 WGSL 单元' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo run --release -p singlefs-unit-graph --bin unit-graph -- closure . singlefs_verification::the_gpu_judge_reddens_a_pending_entry_whose_epoch_is_raised_above_the_txg_of_the_version_like_the_cpu_judge --target test:crash_judge_gpu_agrees_with_the_cpu_judge_on_the_pending_release_bound --cfg test --toolchain-of-the-build --lock-file Cargo.lock`。起点：集成副本根。看：日志里 wgsl 单元行（`crash_judge_checker` 起头的文件）出现与否；出现就是今天的单测键带 WGSL，格 7 第三支与改法 2 的构造一要照「今天重跑、移出键之后不重跑」写。入口写法认不得时，照 `crates/singlefs-verification/src/judging_units_entries.rs` 的 `recording_scenario_entry_path` 那一格的写法改。

## 七、没做什么

- 没写模型、没编、没跑任何测试与命令：第二到五节每个结论都是读集成副本与主工作区得出，标「读得出」的是代码直接写着，标「推的」的是由代码推到运行结局。第六节的命令没有一条跑过，判决要引的数都得主 agent 复跑。
- 没读禁读清单里的三份文件；没读 `gpu-verification-server` 服务一侧补丁的全部，只读了待判行取活与核对池两处。
- 没比库里各把键的组成清单（基线报告也没有），所以第 5.2 节只是上界。
- 没判格 5 的「读法」那一句（用户 100 行的话是否要求键取持久集合）：写成读法，不当结论。
- 没查 `rollbacks_on_the_parent_chain` 是否进判器闭包、崩溃枚举是否造得出并列谱系记录：定案丙里的工具链反例只是候选。
- 没替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/judging-keys-input-only-r2-sonnet` 只有认领文件，没建仓副本、编译目录与库拷贝，没有要删的。

## 八、交回前核对

`python3 scripts/agents/cite-check.py research/prompts/judging-keys-input-only-r2-sonnet-output.md --background research/prompts/_judging-keys-input-only-r2-background.md` 的末行：「✓ 核了 36 处引文、0 处裸行号，对上 36 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）」。集成副本里的行（写成「集成副本 <相对路径> 第 N 行」的）不在这个脚本的核对范围，由我逐句 `grep -nF` 过。
