# 里程碑三第一批实现：代码轮第一轮正文（2026-10-01）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 -->

## 一、题面

里程碑三开工第一轮交实现的三笔已经在 worktree 里合入、过了合入后验证（harness 档 98 个测试二进制全绿、clippy 过；fmt 只红在 5 个这批没动过的原有文件；日志 `/tmp/claude-1000/merge-and-verify/2026-10-01-628739/`）。这一轮攻实现与测试：代码做的是不是条款说的、哪一格会错、用例放不放得过错的实现。

| # | 改动 | 条款落点 | 实现员报告 |
|---|---|---|---|
| 1 | C585（会话发布放进发布途中取下的旧快照） 用户选的改法 C：挂着之后收盘表的四个入口（会话发布、正常卸载、准入抬 F、管理员回退）与可写挂载取号之前，落后的盘「缺现行那一版的单元，或现行那一版的根写在这块盘上而那一槽里不是它」才拒 | D18（块里携带什么信息） 已定项 11「取号之前逐盘核带不带所选那一版」与挂着之后入口的落后支；重读次数取 D16（发布语义） 已定项 1「根槽这一次读坏」的一次重读；改法被攻过零轮 | `/tmp/claude-1000/m3-r1/writer-c585-c586/report.md` |
| 2 | C586（回退取树 ID 水位的那一半没有用例）：补会红的用例与变异行；litmus 注释一处过期路径 | D23（journal 的角色与格式） 已定项 14「水位」 | 同上 |
| 3 | 文件后端写零改成打洞（`fallocate(2)` 的 `PUNCH_HOLE \| KEEP_SIZE`），只在 `EOPNOTSUPP` / `ENOSYS` 时退回逐块写零，别的错（含 `ENOSPC`）照报；core 根从 `forbid(unsafe_code)` 降成 `deny`，全 crate 只在打洞那一个函数放开；`O_DIRECT` 后端不动 | C603（测试与验证的资源浪费没有自检）、C486（整环清零的 55 秒与冷重开的 17 秒都没人管）；D17（实现分层与第三方管道） 已定项 5（两个后端）；D22（单元原子性怎么合成） 已定项 8（mkfs 经写零动作清根环与 journal 环）；用户 2026-09-30、2026-10-01 弹窗定（打洞、只在不支持时退、core 一处 unsafe、ENOSPC 照报），原话在 `records/2026-09-30-里程碑三开工第一轮.md` 第三节与第五节 | `/tmp/claude-1000/m3-r1/sparse-zero-fill/report.md`（第一版）、`/tmp/claude-1000/m3-r1/sparse-zero-fill-2/report.md`（第二版，被判的是这一版） |

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

腿读代码一律读冻结副本 `/tmp/claude-1000/m3-code-r1/tree/`（基于提交 fcc4d340 打了三份补丁；被判文件的 sha256 在 `research/prompts/m3-code-r1-snapshot/crates-sha256.txt`，名单在同目录 `judged-files.txt`）。完整 diff 在 `research/prompts/m3-code-r1-diff.patch`。行号以冻结副本为准。

- `crates/singlefs-core/src/mount.rs`：新类型 `VersionLackingOnADeviceBehindIt`（第 635 行，`UnitsMissing` / `RootNotInItsRingSlot`）、`RootOfTheVersionInTheRing`（第 664 行，`Written{..}` / `NotWritten`）；`SelectedVersionLackingOnDevice` 加成员 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot`（第 707 行）；共用判法 `what_a_device_behind_the_version_lacks`（第 3350 行：先调原有的缺单元判法，单元都在又落后的再调 `root_of_the_version_not_in_its_ring_slot_on_the_device`，第 3399 行：那一版的根写进了根环、`target_for_publish(txg)` 那一槽归这块盘时读那一槽，不是它（读坏，或自证过而 (实例, txg) 不同）就重读一次，还不是它才算缺根）。四个入口经 `devices_behind_the_current_version`（第 3464 行）恒传 `Written`；可写挂载经 `devices_without_the_selected_version`（第 3234 行），在第 3997–4002 行按 `effective_root` 与 `chosen_root` 的 (实例, txg) 相同传 `Written`、不同（施加过 journal 记录）传 `NotWritten`。`DeviceBehindTheCurrentVersion` 的 `units_missing` 字段换成 `lacking`。成员名 `DevicesBehindTheCurrentVersionAndMissingItsUnits` 没改名（现在也覆盖缺根，改名要动清单外五个文件）。回退水位 `.fold(current.root.tree_identifier_watermark, u64::max)` 在第 5386 行（这一批没改它，只补了用例）。
- `crates/singlefs-core/src/mounted_session.rs`：只改文档注释。
- `crates/singlefs-core/src/block_device.rs`：文件头第 10–16 行；`ZeroFillLanding`（第 149 行，正式成员 `PunchHoleOrWriteZeroBytesIfUnsupported`，两个只供测试的开关 `ActAsIfPunchingWereUnsupported`（造 `EOPNOTSUPP`）、`ActAsIfPunchingFailedWithAnInputOutputError`（造 `EIO`））；模块私有 `HolePunchFailure`（第 162 行）与纯函数 `classify_hole_punch_failure`（第 171 行，按 `raw_os_error()` 穷举）；计数 `ZeroFillLandingCounts`（第 180 行）与访问器 `zero_fill_landing_counts`（第 219 行）、`with_test_only_zero_fill_landing`（第 208 行），都是 `pub`；唯一的 `unsafe` 在 `punch_hole_keeping_the_file_length`（第 365 行，`#[allow(unsafe_code, reason = …)]` 挂在它上面，`// SAFETY:` 写了描述符、偏移、长度三样前提）；文件后端 `write_zeroes_at`（第 423 行）先 `check_request`，再打洞，失败按分路判定退回或报 `InputOutput`；`O_DIRECT` 后端的 `write_zeroes_at`（第 665 行）没动。新单测在第 823、891、971、1029 行。
- `crates/singlefs-core/src/lib.rs`：`#![deny(unsafe_code)]` 与三行文档注释；`crates/singlefs-core/Cargo.toml` 加 `libc = "0.2"`；`Cargo.lock` 只多 core 依赖表里一行 `"libc"`（0.2.189 早已在锁里）。
- `.claude/gate.d/unsafe-allowances.tsv`：门禁 code-source-discipline 的 unsafe-allowances 格读的登记表（core 一行 deny 加那一个函数、理由、出处；其余四个 crate forbid）。主 agent 2026-10-01 在冻结副本上跑那一格退 0，副本里把 SAFETY 退回只写描述符的旧版退 1。
- 用例：`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 新增 6 条（四个入口各一条「发布途中取下的旧快照换进来、在任何写之前拒」，「切在根槽 FUA 之后不误拒」，「切在根槽之前可写挂载施加记录照常挂上」），这个文件整份 rustfmt 过、diff 里含原有段落重排；`unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 新增 1 条（C586 树 ID 水位）；新文件 `mkfs_on_image_files_punches_the_zeroed_ranges.rs`（mkfs 在两块镜像文件上：经 write 交出的字节恰好等于录制层记的写入字节、每盘退回写零 0 次、打洞次数等于写零步数、镜像长度 4 GiB、`st_blocks` 远小于清零长度；量本线程 `/proc/thread-self/io` 的 `wchar`；前提是 `std::env::temp_dir()` 所在文件系统支持打洞）。
- `crates/mutations.tsv`：追加 14 行（C585 改法 C 6 行、C586 1 行、打洞 7 行）；实现员用 `research/scripts/prove-red.sh` 证过 C585 / C586 的 7 行、打洞的第 2、3、5 行，打洞第 1、4、6、7 行没证（留给提交时的 checker-tier-crates-mutation-replay）。
- `litmus/publish-returns-after-the-rotation-barrier.litmus` 第 14 行改路径。
- 实现员交主 agent、还没定的：① 成员名要不要改；② D18（块里携带什么信息） 已定项 11 要不要补「可写挂载上，所选那一版就是择到的那条根才算写进了根环，施加过记录的不核缺根」一句；③ 缺根那一半的重读一次没有用例钉。实现员列的「被攻过零轮」的六个候选（可写挂载缺根那一半几乎打不到；重读没钉；旧快照取在更早一次发布途中而现行版根不在这块盘；P 的根住在别的盘而被换的盘是 P 途中取下的；根环被盖掉一圈的前提在满环 / 洞 / 补洞之后要重核；「是它」只比 (实例, txg) 不比整条根记录）在 `writer-c585-c586/report.md`「被攻过零轮」那一节。打洞那边：持久性只查了 ext4 源码（`ext4_punch_hole` 调 `ext4_update_inode_fsync_trans`）；xfs、btrfs、tmpfs 上真实 errno 没量。

## 三、格

### Z1　条款对实现、条款对测试（云端正推）
逐条核题面三笔的条款原文（kb 文件自己的行号）与「实现今天的样子」列的每一项：代码做的是不是条款说的，有没有条款说了而代码没做、代码做了而条款没说的；新写的 8 条用例与 core 里 4 条新单测测的是不是条款要的那件事、断言钉的是条款里哪一句，条款里有而没有用例钉的逐句列出。另答三问：① 可写挂载按「`effective_root` 与 `chosen_root` 是不是同一条」决定传 `Written` 还是 `NotWritten`，这个读法与 D18（块里携带什么信息） 已定项 11 的字面合不合，条款要不要补句；② `DevicesBehindTheCurrentVersionAndMissingItsUnits` 这个名字现在覆盖缺根，按 `.claude/singlefs-ai-sop/rules/code-discipline.md`「名字」那一节判要不要改；③ 打洞失败分路（只有 `EOPNOTSUPP` / `ENOSYS` 退）与 `.claude/rules/fs-design.md`「五条硬要求」逐条对：判据写下来没有、两条分支能不能被测试强制进入、可观测、分支变量不中途变。

### Z2　C585 / C586 哪一格会错（云端攻方）
在冻结副本的拷贝上造反例，至少攻：① 实现员列的六个零轮候选逐个造盘面量，尤其「被换的盘是发布 P 途中取下的、P 的根住在另一块盘上、单元都在」与「旧快照取在更早一次发布途中、现行那一版复用了那块盘上全部旧单元」两种漏拒；② 可写挂载传 `NotWritten` 的那一支：有没有「根槽已写、择根没择到它」之外的形会因此放过一块真缺根的盘；③ 误拒：合法历史（含崩在根槽 FUA 前后、撕裂轮换、只剩一槽自证、一次瞬时读错）里有没有被新判法拒成落后支的；④ 重读一次：造一次瞬时读错，看去掉重读的变异会不会红，不红就写出钉它的最短用例；⑤ C586 那条用例：换一种「算水位那一遍读不出」的造法，看 `.fold(0, u64::max)` 之外的错实现（例如只取内存里现行版、不并环里）放不放得过。只跑 harness 档。

### Z3　打洞哪一格会错（云端攻方，与 Z2 同一条腿、另成一节报告）
① `fallocate` 的偏移、长度在什么输入上会与 `write_zeroes_at` 的语义不同（不对齐、跨文件末尾、长度 0、超过 `off_t`），`check_request` 是不是都先挡住了；② 读回全 0 之外，打洞与真写零在录制层、checker、崩溃枚举、checker 档 `fold_declared_zero_fills`（只走 `O_DIRECT` 后端）上有没有可观测的差别；③ 退路：强制「不支持」与「EIO」两个开关之外，真 errno 下（例如在 tmpfs、或用 `unshare` / 别的手段造一个不支持打洞的文件系统，造得出就造）分路对不对；`ENOSPC` 照报的后果写清（造得出就量）；④ `mkfs_on_image_files_punches_the_zeroed_ranges` 依赖 `temp_dir()` 支持打洞：在不支持的环境里它红成什么样，是不是该红；`wchar` 按本线程量，测试线程里还有别的写会不会让它误红或误绿；⑤ `unsafe` 那一处：`// SAFETY:` 写的前提逐条成不成立（描述符借用期间不会被关、`try_from` 拒负与溢出、越界只返回 errno）。只跑 harness 档与 core 的 lib 单测。

### Z4　测试（云端攻方，同一条腿、再另成一节报告）
① 对每条新用例与新单测，写至少一种「看起来对、其实错」的实现（不是变异表里已有的 14 行），看放不放得过：例如落后支只查盘 0 的根槽、根槽比对只比 txg 不比实例、可写挂载一律传 `Written`；打洞一律当成功不调系统调用但仍计数、退路只写第一块。放得过的写出最短的那一种。② 造盘面的造法（`root_device_of_the_publish_taken_mid_publish` 取「P 之前的盘面 + P 的写流取到它根槽 FUA 写之前那一截」）是不是一段真崩溃留得下的盘面；不是的，它钉住的行为会不会因此是假的。③ entries 文件整份 rustfmt 带来的原有段落重排有没有改掉原有用例的语义。④ 变异表 14 行：各自红的是不是点名那条用例、替换文是不是真把行为退回了旧的一种；没证的打洞第 1、4、6、7 行在副本上证一次。

## 四、分工

| 腿 | 格 | 立场 | 理由 |
|---|---|---|---|
| 云端正推（Sonnet） | Z1 | 条款对实现、对测试 | 代码轮第一轮，没有前一轮代码判决可复核，不派辩方 |
| 云端攻方（Opus） | Z2、Z3、Z4 | 在副本上真跑：实现的反例与测试放得过的错实现 | 要真跑；三节分开报 |

## 五、跑前写死的判据

- Z2 造出一条合法历史上的漏拒（换进来的旧快照被放过、之后池级 checker 红或丢写）或误拒（合法历史被拒成落后支、或被拒成永远只读） ⇒ 打中，改法并进下一批，这一轮判决写明；零轮候选只在一种盘面上造出的，同样记打中，判决写明盘面。
- Z2 ④ 去掉重读的变异在瞬时读错的盘面上不红 ⇒ 补用例与变异行，并进下一批。
- Z3 造出打洞与真写零在任何可观测量上不同（读回、录制流、checker 判定、崩溃枚举结果） ⇒ 打中；只在不支持打洞的环境里才出现的，判决写明环境。
- Z3 ⑤ SAFETY 的某条前提不成立 ⇒ 打中，改代码或改注释并进下一批。
- Z1 判出条款与代码对不上 ⇒ 以条款为准改代码；条款自己没写清的（Z1 ① 那一句），交用户。
- Z4 ① 写出一种错实现、新用例全绿 ⇒ 打中，补用例（或收紧断言）。
- Z4 ② 判出造法不是真崩溃留得下的盘面、而它钉住的行为只在那种盘面上成立 ⇒ 打中，改造法。
- 一条腿报「没打中」的格只抽了一次样，不支撑否定结论（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」）。

交付：腿的报告写 `research/prompts/m3-code-r1-<模型>-output.md`（`-sonnet-` / `-opus-`），分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；攻方的副本改法与用例放 `research/prompts/m3-code-r1-opus-model/`。草稿与临时文件放腿自己的目录 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。方案按 `crates/` 今天的实现来谈（冻结副本 `/tmp/claude-1000/m3-code-r1/tree/crates/`）。
