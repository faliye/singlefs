# 删除批第 14 批合入的代码：代码轮第一轮判决（2026-10-04）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 GA GB GC U8 K1 K2 K3 FX MQ1 MQ3 MG1 MG2 MG4 IMM HELD -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-batch14-code-r1-body.md`、`_m3-batch14-code-r1-diff.md`、`_m3-batch14-code-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-batch14-code-r1-snapshot/`（crates 21 份、kb 2 份）；冻结副本 `/tmp/claude-1000/m3-batch14-code-r1/tree/` | `bash scripts/devices/snapshot-check.sh m3-batch14-code-r1`：23 份里 6 份对不上，倒推见 `m3-batch14-code-r1-snapshot/reversal-notes.md`。D16 反做本线第三次写回 2 处、根环第 9 批写回乙 4 处，D8 反做本线第三次写回 1 处，两份 sha256 都回到快照；四份 crates 文件的改动出自根环第 9 批补丁与 2026-10-03 那一次提交，腿引的代码行都指冻结副本，不受影响 |
| 云端正推（Sonnet） | `research/prompts/m3-batch14-code-r1-sonnet-output.md`（244 行，sha256 0924656f…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-batch14-code-r1-opus-output.md`（240 行，sha256 ed2c5ce5…），模型 `…-opus-model/` | 报告开头列的 31 份文件 sha256 全部对上 |
| 引文 | 两份腿报告 | `cite-check.py --root <冻结副本> --root . --background <背景材料>`：「✓ 核了 81 处引文，对上 81 处，没判 0 处」。腿引的 D16 第 37、38 行被根环写回乙改过，引到的那一段（第 37 行准入式子、第 38 行前半句）在快照原样里逐字相同 |
| 主 agent 复跑 | `research/prompts/m3-batch14-code-r1-main-checks/`（README 写明命令与比对） | 攻方模型目录只读、工作目录放会话草稿目录。h2 那一份逐行复出；层 0 枚举 n = 1 那一刀 20511 个状态逐行复出；窄盘族覆盖写 0 那一档 GA 48 个起点里 5 个读不出，FX 0 个，两臂再挂被拒都是 0 |
| 读代码 | 冻结副本 `crates/singlefs-core/src/mount.rs` 第 1834–1847 行、第 1336–1341 行 | 可写挂载重建分配器时，按 `reclaim_floor(effective_floor, oldest_valid_root_in_the_window)` 当场回收；`reclaim_floor` 取两者较大，F_生效 含系统配置里的 F，形 R 的机理照攻方说的成立 |

被判的 21 份文件（K3），每份一格判到（正推腿第二节逐份判：一致 21，冲突 0；本判决第二节按格采纳）：
- `crates/mutations.tsv`
- `crates/singlefs-core/src/allocator.rs`
- `crates/singlefs-core/src/mounted_session.rs`
- `crates/singlefs-core/src/mount.rs`
- `crates/singlefs-core/src/recovery.rs`
- `crates/singlefs-core/src/transaction.rs`
- `crates/singlefs-harness/src/history.rs`
- `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs`
- `crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs`
- `crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs`
- `crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs`
- `crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs`
- `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs`
- `crates/singlefs-harness/tests/overwrite_in_one_instance.rs`
- `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs`
- `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs`
- `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs`
- `crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs`
- `crates/singlefs-harness/tests/unlink_single_file_publishes.rs`
- `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`
- `crates/singlefs-harness/tests/write_accounting_per_structure.rs`

同一批合进主仓、只改注释与消息的第 15 批四份（C307 的说明、「不到十二个号」、`mounted_read_by_offset` 的注释），行为不变，由主 agent 读 diff 判：
- `crates/singlefs-checker/src/walk.rs`：注释，一致
- `crates/singlefs-core/src/records.rs`：注释，一致
- `crates/singlefs-harness/src/model_comparison.rs`：一处注释，一致
- `crates/singlefs-harness/tests/mounted_read_by_offset.rs`：注释，一致

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| N1 GA 卸载那一串自己 | 一致：用户定案 → D16 第 42 行 → 冻结副本 `mount.rs` 第 2346、2380–2391、2947–2967 行；GC 两条与变异 2273、2274 钉住；准入那一格有钉 | 上一轮四族在入库代码上重扫：窄盘 1900 个起点、两族循环 400 + 186 个起点，交集 0，读不出 0，卸载被拒 0；同一份原型复出上一轮丙1 的 174 / 37 / 1 与改之前那一版的 252 | **一致**；「没打中」只有一次抽样，不记站住，第二次抽样并进第 16 批代码轮的攻方 |
| N1 形 R：卸载半途崩溃之后的下一次可写挂载 | — | 打中：卸载写了系统配置里的新 F = C、带新 F 的根没落满每块盘时崩溃 → 两盘再可写挂载，重建分配器按 F_生效 = C 当场回收 → 写行那次的单元写进另一块盘上最新的根（< C）引用的槽 → 暖机落满之前再崩、丢住着现行那一版的那块盘 → 剩下那块盘只读挂载失败或打开活文件失败。GA 窄盘 13 / 186、普通盘 9 / 95（含 4 GiB）；丙1 14、9；改之前那一版 9、8；FX 0、0 | **坐实**（主 agent 复跑与读代码，见第一节）。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」四句核：不分辨臂，病根在三臂共用的「可写挂载按含系统配置 F 的 F_生效 当场回收」；系统当时看得到判别量；满足的是正文第五节结局判据（丢一块盘之后只读挂载与整读），不是 D16 已定项 1「正常卸载」那一行的字面，那一行只管「这一串」写的单元，没写下一次挂载；跑前条款没给改法，上一轮岔路单三个候选在这一格上都中，只有 FX 不中。交用户，见第四节 |
| N1 形 R 的层 0 枚举域 | — | GA h2 那一份两刀各 20511 个状态，丢盘 0 读不出 12302、8195 个；两盘都在时池级 checker 0 违例 | **坐实**（n = 1 那一刀主 agent 逐行复出）；两盘都在时 checker 看不出，与上一轮判决「池级 checker 没有掉了一块盘的读法」那一笔同一笔欠账 |
| N1 单盘恢复择中被抛弃的根 | — | 9027 个判定择中 0 次：被抛弃的根出现之后那次挂载的暖机已在每块盘上落了新实例的根 | **没打中**（一次抽样）。冻结副本 `mount.rs` 第 2364 行那句理由「被抛弃的那条时间线接着那一版走，也不引用它」与 C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）那一形对不上（推的），GA 的结论不靠它：改写那句理由，进下一批 |
| N1 根环里读不出的槽、有洞 | 一致：读不出与自证不过的槽当没有根，条款没写（「规则没说」） | 没打中（读代码） | **一致**；「某块盘一条有效根都读不出时整截扣到生效」「读不出的槽当没有根」两格是代码取的保守读法，条款里补写，进第四次写回 |
| N1 卸载—再挂循环 | — | 没打中（一次抽样）；形 R 是它加一次崩溃的变形 | **没打中**，第二次抽样同 N1 第一行 |
| N2 GC | 一致：两条判的是只读挂载与整读，用户看得到的结局；h2 那一份活文件为空 | MG1（取最大）被两条抓到 | **一致**；它们罩不到形 R。h2 整读的活文件为空：第 16 批补一形活文件不为空的 |
| N3 Q-F 乙 意图与各批 | 一致：意图写调用方的时间、三格同值、各批照抄；变异 2275、2276 红；①b 用例绿 | ①b：288 份崩溃镜像两次续做逐字节相同；续做里还带这条记录的 1275 步写入时间都等于给的值 | **一致**；①b「没打中」一次抽样，第二次抽样同 N1 第一行 |
| N3 模型不比时间的缝 | 挂载续做那一支的时间拷贝仓里没有用例钉（腿的探针量过） | 打中缝：MQ1（会话那一层丢掉调用方的时间、意图写 0）四个测试目标 80 条全绿；MQ3 只有那一条专门用例红 | **坐实**（腿的副本上量过，主 agent 没复跑这两刀；读代码：专门用例直接调 `publish_unlink_intent`，不经会话）。进下一批：经会话发 unlink、读盘上三个时间秒的用例（MQ1 要红），挂载续做那几批照抄意图时间的用例 |
| N4 U8 与 N2 注错 | 一致：五行变异改坏的是名字说的那一处，复跑全红 | — | **一致** |
| N5 GB 与文档 | 一致：「三样」与代码三处不同对上；「没有 ctime」零命中 | — | **一致** |
| N6 回扫旧树值 | 一致：只改注释、消息、名字；`7`→`11` 不改结果；`recovery.rs` 那条 TODO 没指欠账表 | — | **一致**；TODO 指到 C692，进下一批 |
| 变异覆盖 | 被抛弃根的过滤、「某块盘没有有效根」两处没有变异；`admission_raises…` 钉 GA 结局的断言不分辨丙1（量过） | MG4 被四条抓到；MG2（不滤被抛弃的根）在可达状态上与原样等价（推的） | 进下一批：「没有有效根整截扣」补一行变异并点名会红的用例；被抛弃根的过滤先照 `.claude/rules/mutation-sampling.md` 找敏感取样点，找不到再按等价变异留档（等价性写成测试） |

顺手看到、不在这一轮判（攻方报告「顺手看到」一节，主 agent 没复现）：
- 被抛弃的根的账读不出时卸载被拒：78 个起点里 11 个，三臂相同，出在 C314 那一形之后。记欠账，派调查员复现之前不改。
- 普通盘族覆盖写 0 次、没崩了再挂的 8 个起点：卸载之前丢盘 0 剩下那块盘就读不出（树表 0 条那一版在单盘上 `UnitUnreadable { slot: SlotNumber(0) }`），四臂相同，与卸载无关。记欠账，同上。

## 三、进下一批的改法（第 16 批；各自验收标准在派发时写）

1. 形 R：用户 2026-10-04 选甲（岔路单第 1 行），照 FX 写进 `mount.rs` 重建分配器那一处，补形 R 的读回用例：GA 下红、甲下绿；D16 已定项 1 补写下一次可写挂载那一句，进第四次写回。
2. 冻结副本 `mount.rs` 第 2364 行那句理由改写。
3. 「某块盘没有有效根整截扣」补一行变异；被抛弃根的过滤按变异覆盖那一格办。
4. GC 补一形活文件不为空的。
5. Q-F：经会话的 unlink 读三个时间秒（MQ1 要红）；挂载续做那几批照抄意图时间。
6. `recovery.rs` 第 4265 行那条 TODO 指到 C692；crates 里「四个固定点单元」那一族旧说法（`write_accounting_per_structure.rs`、`model.rs`、`transaction.rs`、`writable_mount_takes_a_second_instance.rs`）改成现值。
7. 第 16 批的代码轮攻方，攻击面里带上这一轮「没打中」各格的第二次抽样（GA 卸载那一串的四族、被抛弃的根、①b）。

## 四、交用户

岔路单 `research/prompts/m3-batch14-code-r1-forks.md`：形 R 怎么处置（甲 再挂也只放开一截；乙 不改代码，条款写明不承诺）。两个候选都是这一轮新冒出来的零轮形态；甲的数只在攻方腿的副本上量过（窄盘 186 个、普通盘 95 个起点，读不出 0、再挂被拒 0，相关 7 个 harness 测试目标全绿），主 agent 复跑了覆盖写 0 那一档。

记欠账（第四次写回落表）：
- 形 R 两盘都在时池级 checker 看不出，并进「池级 checker 没有掉了一块盘的读法」那一笔。
- 被抛弃的根的账读不出时卸载被拒；树表 0 条那一版在单盘上读不出。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-04 不受影响：「正常卸载」那一行 GA 的实现与条款对上；形 R 落在条款没写的「下一次可写挂载」，等用户选了再写回；「没有有效根整截扣」「读不出的槽当没有根」两格进第四次写回 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-04 不受影响：偏移 64 起三个时间秒照用户 Q-F 乙的定案写，实现与条款对上 |
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-04 不受影响：第 7 条 ①b 在 288 份崩溃镜像上成立（一次抽样） |
