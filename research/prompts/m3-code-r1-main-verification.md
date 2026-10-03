# 里程碑三第一批实现：代码轮第一轮判决（2026-10-01）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 H1 H2 H3 H4 H5 H6 H7 H8 W1 W3 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_m3-code-r1-body.md` | — |
| 正推腿（Sonnet，Z1） | `research/prompts/m3-code-r1-sonnet-output.md`（187 行） | 7ec03379 |
| 攻方腿（Opus，Z2–Z4） | `research/prompts/m3-code-r1-opus-output.md`（334 行），模型目录 `m3-code-r1-opus-model/` | 482bc1bd |
| 被判文件快照 | `research/prompts/m3-code-r1-snapshot/crates-sha256.txt`（冻结副本 `/tmp/claude-1000/m3-code-r1/tree/`） | — |

- 写判决前，冻结副本 12 份被判文件 `sha256sum -c` 全部 OK。
- `cite-check.py`：以冻结副本为根「核了 88 处引文，对上 42 处，没判 46 处」，没判的是 kb 与规则（不在冻结副本里）；以主仓为根「核了 88 处引文，对上 88 处，没判 0 处」。
- 攻方模型目录里 `SHA256SUMS-logs.txt` 逐份核对 OK；判决引的产物行（`MUT A_only_device_zero	GREEN	rc=0	failed=[]	results=94`、`MUT D_no_reread	GREEN	rc=0	failed=[]	results=94`、`MUT E_any_self_verified_root	GREEN	rc=0	failed=[]	results=94`、`MUT BASE	GREEN	rc=0	failed=[]	results=94`、`SwappedForTheCutAfterTheRootSlotWrite session_publish fail_read=1 outcome=DevicesBehindTheCurrentVersionAndMissingItsUnits …`）都在模型目录的日志里逐字找到。
- 攻方的量都在冻结副本的拷贝上做，副本上的数不进 kb。

## 二、逐条判决

| # | 格 | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|---|
| 1 | Z2-④ / Z4-① D 重读一次 | 去掉重读，一次瞬时读错就把健康的盘拒成落后支（会话发布第 1 次读、可写挂载第 8 次读）；harness 全包加 core 单测 94 个二进制照样全绿。正推腿独立报同一缺口（「重读一次没有用例，去掉它不会让任何新用例变红」） | 产物行逐字找到（第一节）；两条腿用各自的方法得到同一结论 | **打中**（测试缺口）。原型用例搬进 entries，补变异行（H1） |
| 2 | Z4-① A「只查盘 0 的根槽」 | 新用例与全包都绿；在攻方的旧快照表上 168 格 checker 红 | 产物行逐字找到 | **打中**（测试缺口），补一版根在盘 1 的用例（H2） |
| 3 | Z4-① E「根槽里自证得过就算是它」 | 新用例与全包都绿；在转圈表上 226 格红。正推腿独立报「`AnotherRoot` 读数没有用例」 | 同上 | **打中**（测试缺口），补根环转过一圈、断言 `AnotherRoot` 的用例（H2） |
| 4 | Z3-a 长度 0 的写零 | 文件后端报 `InputOutput(EINVAL)`，`O_DIRECT` 后端与稀疏盘交回 `Ok`；今天的调用方够不着。正推腿在 ext4 上用 ctypes 独立复现 | 主 agent 现查：`check_aligned_and_in_range`（冻结副本 `block_device.rs` 第 300 行起）放过长度 0；本机 ext4 上 ctypes 调 `fallocate(PUNCH_HOLE\|KEEP_SIZE)` 长度 0「返回 -1 errno 22」 | **打中**（三个后端对同一请求结局不同），改（H5） |
| 5 | Z3-d 用例红时留下镜像 | mkfs 打洞用例红了不删镜像，每次在 `/tmp` 留约 1.5 GiB | 现查：用例只在末尾第 128 行 `remove_file`，中途 panic 走不到；`/tmp` 下现成两份 9 月 30 日留下的 `singlefs-step67-punch-zeroed-ranges-*`（各实占 769M），主 agent 看清后删了 | **打中**，违反 `.claude/singlefs-ai-sop/rules/command-safety.md`「测试镜像一律放临时目录」那一节「成功、失败、panic 都删」，加 Drop 守卫（H6） |
| 6 | Z3-f SAFETY 里一句 | 「即使越界内核也只返回 errno」不成立：越过文件末尾打洞交回 0 | 现查：本机 ext4 上越过文件末尾、跨文件末尾两种「返回 0」；内存安全不受影响，越界请求本来就被 `check_request` 挡住 | **打中一句**，改注释（H7），改后仍要过 unsafe-allowances 格的描述符 / 偏移 / 长度三类词 |
| 7 | Z3-b `EINTR` | 打洞遇到 `EINTR` 不重试，落进「别的错」照报 | 现查：`classify_hole_punch_failure` 只把 `EOPNOTSUPP` / `ENOSYS` 当不支持 | 采：`EINTR` 重试打洞，不算退回写零（用户定的「只在不支持时退」不受影响）（H8） |
| 8 | Z1 ① 可写挂载上的缺根一半 | 正推腿：条款字面只到四个入口，可写挂载这一半是「代码做了、条款没说」；拒绝成员今天没有用例造得出来。攻方腿：17 条流 364 911 个合法崩溃态加可写挂载，零误拒 | 用户 2026-10-01 弹窗定「留着，条款补句、补用例」 | 留；补造得出这个拒绝的用例（两次连续读错）（H3）；D18（块里携带什么信息） 已定项 11 补句，并把正推腿报的同一分项第 314 行与第 323 行的内部不一致一起理顺（主 agent 写回 kb，随这一批提交） |
| 9 | Z1 ② 成员名 | `DevicesBehindTheCurrentVersionAndMissingItsUnits` 现在也覆盖缺根，名字只说了一半；8 个文件 30 处 | 现查与实现员报告一致 | 采，照 `.claude/singlefs-ai-sop/rules/code-discipline.md`「名字」那一节改名，连同 D18 第 314 行三处（H4） |
| 10 | Z1 ③ 打洞成功那一支没有强制开关 | 五条硬要求第 2 条 | 打洞成功是文件后端在支持打洞的文件系统上的默认路，`mkfs_on_image_files_punches_the_zeroed_ranges` 断言打洞次数等于写零步数，不支持时这条用例判红 | 不改：那条用例本身就证明这一支被进入了；环境前提写在用例文件头 |
| 11 | Z2-①②③、Z3-b 其余、Z3-e、Z4-②③④ | 没打中 | — | 只抽了一次样，不支撑「没有错」 |
| 12 | Z2-⑤ C586 的 W1 / W3 | 放得过，今天的可达历史分不出（推的：树号只在第一个文件那一版发） | — | 记进 C586 的「怎么拦」，等多文件建树之后补一格；这一批不改 |
| 13 | Z2-③ 撕裂系统配置槽那几态 | 被拒成 `NewerStateStillUnreadableAfterOneReread`，是改动之前就有的拒因 | — | 不属于这一批，记线索，进下一轮清单 |
| 14 | Z3-c | 真「不支持打洞」的文件系统没造出来（用户命名空间被拒） | — | 照实记；不支持那一支只由测试开关强制进入 |

## 三、改法（交实现员，一批做完；基于 `/tmp/claude-1000/m3-r1/merge-wt` 今天的样子）

- **H1**：把攻方原型 `research/prompts/m3-code-r1-opus-model/proto/tests/z2_reread_one_transient_root_slot_read_error.rs` 的甲那一形搬进 `entries_after_mount_refuse_swapped_or_behind_devices.rs`，会话发布 n = 1、可写挂载 n = 8 各一格；变异行替换文 `let reread = first_reading;`，证红。
- **H2**：C585 用例加两版：P 的 txg ≡ 1 (mod 3)、根在盘 1；根环转过一圈、根槽里是自证得过的另一条根，断言 `RingSlotReadingOtherThanTheVersionRoot::AnotherRoot`。各配一行变异（A「只查盘 0」、E「自证得过就算是它」），证红。
- **H3**：造一条可写挂载被 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot` 拒的用例（所选那一版就是择到的根，那一槽连着两次读错），断言拒在任何写之前、盘面不变。
- **H4**：`DevicesBehindTheCurrentVersionAndMissingItsUnits` 三处（`MountError`、`UserChangeRefused`、`CallerInputsDisagreeingWithTheDisk`）改成说出「缺单元或缺根」的名字（名字由实现员起，照 `.claude/singlefs-ai-sop/rules/code-discipline.md`「名字」那一节）；连带 `crates/singlefs-harness/src/history.rs`、`model_comparison.rs`、`tests/common_admission/mod.rs`、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`new_pool_file_creation_on_device.rs` 与 `crates/mutations.tsv` 里的锚点。
- **H5**：文件后端 `write_zeroes_at` 长度 0 不调系统调用、直接交回 `Ok`，与另两个后端一致；单测钉三个后端对长度 0 结局相同，配一行变异。
- **H6**：`mkfs_on_image_files_punches_the_zeroed_ranges.rs` 的镜像改由 Drop 守卫删，成功、失败、panic 都删。
- **H7**：SAFETY 注释那一句改成实情（越界请求由 `check_request` 挡在前面；越过文件末尾的打洞内核交回 0，不碰内存）。改完在 merge-wt 上跑 `code-source-discipline.sh --check unsafe-allowances` 仍绿，由主 agent 跑。
- **H8**：`fallocate` 返回 `EINTR` 时重试打洞，不退回写零、不报错；分路表单测加这一行。

H4 动的文件多于这一批原来的清单；`crates/singlefs-checker-tier/src/bin/` 下两份只改名字，不改行为。改完由主 agent 在 merge-wt 跑合入后验证，再开 m3-code-r2，只攻 H1–H8 与这一轮没攻到的面。

## 四、交用户

用户 2026-10-01 已定第二节第 8 条（留着，条款补句、补用例）。这一轮没有别的交用户项。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D18（块里携带什么信息） 已定项 11 | 支撑 | 2026-10-01 改了：要补「可写挂载上，所选那一版就是择到的那条根才核缺根，施加过记录的不核」，并理顺同一分项内部两处不一致（第二节第 8 条），随这一批写回 |
| D16（发布语义） 已定项 1 | 支撑 | 2026-10-01 不受影响：缺根那一半借用它的「根槽这一次读坏」重读一次，参数未改；H1 补的是用例 |
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-10-01 不受影响：C586 只补用例 |
| D17（实现分层与第三方管道） 已定项 5 | 不影响 | 2026-10-01 不受影响：两个后端不变，打洞只改文件后端的落地方式 |
| D22（单元原子性怎么合成） 已定项 8 | 不影响 | 2026-10-01 不受影响：mkfs 的写零动作次数、偏移、长度不变 |
