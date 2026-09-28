# E158 第 4 次跑：装置交回（执行员，2026-09-27）

## 一、结论

- 这一趟只写装置、不跑产物。第一、三、四、五段与跨臂的 `r4-compare` 已写完，另有模式 `r4-bases`（今天那一臂造的起始镜像）。臂表的 `r4` 行（18 条臂）已写，单测与变异表也写完并证红。**第二段没写**：实七-甲、实七-乙、实八、H-随 30 段、H-随全 68 段、PC-随、M22、S5a/S5b、S9。原因见第六节。
- C0 快照：先取 manifest-1、再取 manifest-2，两份逐字节相同；之后 `cp -a`。**快照指纹** `bb5ca9bef0b09eb957906317adf7d2b47567d901ef341e85c14b6fe23a37cf4b`。四个 core 文件的 sha256 与底座认定逐个相同。S0 七件都没变（行号在登记修订第 3 条），不停。
- 主工作区里已写（cp 之后 cmp 相同）：
  - `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`：19513 行，sha256 `37a8a312cf9c2d3bf6bf1a835530e78fa6d9f3a4b36607d3ecb8fd5e5cf1445b`，已 fmt、clippy 清零
  - `research/prompts/e158-r4-prereg.md`：只在第十二节加「执行员修订」17 条，原判据没动
  - `research/mutations/e158_arms.tsv`：末尾加第 52 行注释与第 53–70 行共 18 条 `r4` 行
  - `research/mutations/e158_r4_arm_mutations.tsv`：新文件
- **`crates/mutations.tsv` 我没写**（它不在主 agent 放行的路径里），交补丁 `/tmp/claude-1000/e158-r4-device/patch/crates-mutations.patch`（sha256 `344a00ad8b583c7eac8116cb01219b8f775ff222fdbdd8e3ffeb5e610dea647c`）。补丁做两件事：第 953 行（第 3 次跑 M14）的原文与替换文按 fmt 之后的排版改，语义不变；末尾追加 11 行第 4 次跑的装置变异。`git apply --check -p1` 在主工作区通过。**补丁打进去之前，门禁 33 号在第 953 行红，59 号的预扫整张表退出。** 打上之后 69 行 E158 变异原文在新 bin 里各恰好命中一次（脚本数的）。
- 推翻条件：
  - 打补丁之后 33 号仍点名 E158 的行；
  - 在新 bin 上重跑单测不是全绿；
  - 在新 bin 上重跑装置变异有一条没红（最终一轮的结果见第三节末）。

## 二、要主 agent 定的（都在跑产物之前）

登记修订第 9–11 条原文在登记里。三条都是 [臂] 句或读法的落法，登记没改。

1. **PC-多读 (ii) 的「-槽 +3S」按臂行推不出**（读代码推的，没量过）。-槽 臂的读阶段每槽读一次，替换择根那一遍，与今天那一串读相同，推出来是 +0。打不中就走 F18，那几臂的 Q5 按 V1 作废。
2. **乙-窄读 在 PC-N-盖、`reads_zeros`、T(δ) 那一格照拒**（草稿副本上试跑看到的，不是产物）。被藏那次发布的记录读回全零，窄读把它们当 journal 空槽收进缓存，E 那次发布的末条计数器找不到，N-配置 判不出、按真处置。这是认定第 1 项定义的直接后果；要不要改窄读的收法，交主 agent 定。
3. **PC-多读 (ii) 丙 那一句里的 p 取的是 `crates/` 的沿链读**。装置与 checker 都没有公开的实例表沿链读，所以这个数不是「装置自己解出」的。
4. 乙-窄读 的 R = 3 是补的臂（登记 8.2 只写了乙 三臂），只在第五段跑。

## 三、单测与变异

- 单测：`grep -c '#\[test\]' crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 数出 82 条，第 4 次跑新加 12 条（模块 `fourth_run_tests`）。在 today 副本上 `cargo test --release --bin e158_root_choice_repair` 的结果为 `test result: ok. 82 passed; 0 failed`（格式化前 `trial/unit-all.log`；最终一轮 `unit-final.log`，末行贴在第八节）。
- 装置变异 11 条（草稿目录 `device-mutation-rows.tsv`，即补丁里追加的那 11 行）：M14（第 4 次跑那一份）、M16、M18、M19、M20、M21，另补 M24–M28。在 today 副本的拷贝里逐条改坏，跑点名的单测。格式化之前那一轮：**抓到 11 / 无效 0 / 没红 0**，收尾 `restored baseline rc=0 test result: ok. 82 passed`（`check_device_mutations_final.log`）。最终 bin 上的那一轮见第八节。
- 臂表变异 8 条（`research/mutations/e158_r4_arm_mutations.tsv`），**抓到 8 / 无效 0 / 没红 0**（`check_arm_mutations.log`，逐条改坏之后编译、跑点名的模式）：
  - M17、M23、M7、M15、M3 在登记的取样点上红：
    - M17：PC-N-环与 PC-N-盖 的 `reads_zeros`、T1 两格 fail
    - M23：`bing_minus_jia_positive` 四句 fail
    - M7：PC-22 丙 那一句 fail
    - M15：PC-N0 8 格 fail
    - M3：H1g 在 a=1 的 8 格上，基线 jia-cfg 是 K3，变异之后是 K0
  - **M2、M4、M11 在登记的取样点之前就红了**：开跑检查的臂探针 `r4_arm_code_check` 判 fail、V3 整轮停，登记的取样点没跑到。按红记，形态与登记不同：
    - M2、M11：探针那次挂载被拒（K3，或第一次可写挂载就报 `R3UnreadableRootRingSlot`）
    - M4：臂观测到的 R = 0，与登记的 1 不等
- 登记第九节里这一趟没写的：M22（H-随 判红就截断，归第二段）、M9（L1(a) 的确认账，需要装置另开一个「根槽 FUA 时就记确认」的入口，这一趟没做）。M1、M5、M6、M8、M10、M12、M13 是第 3 次跑的装置变异；它们的函数这一次没改，复用 `crates/mutations.tsv` 里那几行，格式化之后原文仍恰好命中一次（第 953 行除外，已在补丁里）。

## 四、fmt 与 clippy（主 agent 消息要的）

- `cargo fmt --all -- --check`：从 259 处 `Diff in` 降到 0（在草稿副本 `lint/` 上跑的；整份文件都格式化了，第 2、3 次跑那几段也在内，只动排版）。
- `cargo clippy -p singlefs-harness --bin e158_root_choice_repair --all-features --keep-going`，lint 集照 `.claude/singlefs-ai-sop/scripts/check.sh` 第 72–78 行（`-D warnings` 加编码纪律七条），`--profile dev` 与 `--profile test` 各一次，都是 `rc=0 errors=0 warnings=0`。加 `--all-targets` 再跑一次，E158 文件命中 0 处；这一次仍红，红在 `tests/checker_narrow_invariants_and_abandoned_roots.rs` 与 bin `e156_allocation_basis_counts`，两处都不是我的文件，是快照上的状态。
- 清零的改法：
  - `shadow_unrelated` 各处改名，含第 2 次跑的 `ranges` / `run`、第 3 次跑的闭包参数 `geometry` 与两条旧单测里的第二个 `run`
  - 一处 `too_many_arguments` 加了带 reason 的 `#[allow]`
  - `VoidedQuantities` 类型别名
  - 改名不改语义
- naming-lint：E158 的 13 处单字母与字母加数字的名字都改了（`l5`→`cell_22`、`q6`→`cell_22_*`、`l_*`→`legal_or_fault_state_*` 这类；产物行名 `r4_q6_*` 没动）。之后 `bash .claude/scripts/naming-lint.sh` 在 E158 上 0 处，全仓仍是 rc=1，红在别的文件。

## 五、每一段的派发提示要给什么

五段共用的部分：
- 臂副本已编好（release）：`/tmp/claude-1000/e158-r4-device/arms/<臂>/`。主族 12 条是 today、jia-cfg、jia-cfg-carry、jia-slot、yi-cfg、yi-cfg-carry、yi-slot、yi-narrow-cfg、yi-narrow-cfg-carry、yi-narrow-slot、bing-cfg、bing-cfg-carry；第五段另有 6 条 `-three-rounds`。
- 装置只读这几份副本；几段并行跑同一份副本，只执行不写，可以。
- 每条命令：进 `arms/<臂>`，带环境变量 `SINGLEFS_E158_ARM=<臂> SINGLEFS_E158_SNAPSHOT_SHA256=bb5ca9bef0b09eb957906317adf7d2b47567d901ef341e85c14b6fe23a37cf4b SINGLEFS_E158_TODAY_BASES=<目录>`，跑 `bash research/scripts/capped.sh N bash research/scripts/run-with-memory-cap.sh 16G ./target/release/e158_root_choice_repair <模式> > <产物>`。
- 先在 today 副本上跑一次 `r4-bases`，把 231 份起始镜像写进那个目录（试跑用时约 1 秒，`r4_today_bases … saved=231 failures=0 verdict=pass`）。第三、四、五段读这个目录。
- 产物名：`research/results/e158-root-choice-repair-2026-09-27-r4-seg<N>-<臂>.out`。跨臂量：`… r4-compare research/results/e158-root-choice-repair-2026-09-27-r4-seg<N>` 写成 `…-r4-seg<N>-compare.out`；它可以一次给几个前缀，最后可以对全部段一起跑一次，求跨段的 V1。
- 每份产物第一行是 `E7INPUT … sha256=<快照指纹>`。开跑检查 `r4_local_constant_check`、`r4_arm_code_check`、`r4_section_seven_two_anchor` 任一不过，就打 `name=stop reason=V3`、整轮停。

| 段 | 模式（每臂一份） | 跑哪些臂 | 判据在登记哪一节 |
|---|---|---|---|
| 第一段 | `r4-seg1`，另跑 `r4-compare` | 主族 12 条 | PC-N-环、PC-N-盖、PC-O、PC-丢写、PC-只读、PC-N0、PC-多读 在 5.4 与 11.1 V1；Q0–Q3、Q5 在第六节；F4、F6、F13、F15、F17 在第十节；7.3 第二至四行（在 compare 里）|
| 第二段 | **没有装置**（第六节） | — | 5.3 H-随、5.4 PC-随、S5a、S5b、S9、M22 |
| 第三段 | `r4-seg3`（等于 `r4-h1f` 加 `r4-h1g`），另跑 `r4-compare` | 主族 12 条 | 第六节：H1f、H1g 两族的丢写格；7.2 H1g 32 格；M3 的臂那一半（`research/mutations/e158_r4_arm_mutations.tsv`）|
| 第四段 | `r4-seg4`，另跑 `r4-compare` | 主族 12 条 | 第六节 Q4、Q5、Q6；5.4 的 PC-多拒、PC-22、PC-554（跨臂那几句在 compare 里）；F1、F2、F5、F14；7.2 的 L 族格数 |
| 第五段 | `r4-seg5` | 主族 12 条跑几何点与 k = 2（G_S4、G_S16、G_默认、G_小环）；6 条 `-three-rounds` 跑 R = 3。已按岔路单出局的臂不跑，记「够判后未跑」 | 8.2（判决格、翻面、判别力自证）|

- 试跑的量（草稿副本上，不是产物）：today 的 `r4-seg4` 约 5 秒、663 格 L 族加 72 格 Q6；`r4-pc` 在两条臂上各几秒。第一段 H1d 有全部断点、第三段 H1f 也有全部断点，这两段最长，没有实测过。
- 第四段的 L1(b) 读 `r4-bases` 存的 205 个崩溃状态。L5 与 PC-554 读 L5 那 6 份起始镜像。第五段在 S4、S16 上的 L5 同样读 `r4-bases` 存的那几份。

## 六、没开的：第二段的装置

`execute_history_with_faults` 在判红或模型对不上时就停，逐步施加用的 `apply_operation` 又是私有的（`crates/singlefs-harness/src/history.rs:3628`），所以第二段装置得有自己的一套执行器：八种操作照 harness 的语义各写一遍（`history.rs:2482`–`3660`），另把 harness 在今天那一臂上抽的注入点与崩溃状态映到各臂自己的写序列上（按「同一步、段内同一序号」映，要写进修订）。这一段做了的只有读 harness 的 API（`GeneratedHistory`、`HistoryOperation`、`CrashPoint`、`DrawnFault`）。要另派一个执行员写：
- 执行器，接到装置的确认账（`ThirdRunHistory` 那一套）
- 实七 / 实八 / H-随 / H-随全 四族
- PC-随
- 5.6 新加的 H-随 常量回比与 S9 的逐项比
- S5a、S5b
- M22

写好的第四段 Q4 / Q5 汇总（`tested_mount_lines`）已经收 `r4_random_mount` 行，交回汇总收 `r4_random_summary` 行，第二段照这两个行名打就能进 compare。

## 七、岔路表（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行）

| 那一样 | 状态 | 还差什么 | 剩下的量能不能让它翻面 | 算它的命令 |
|---|---|---|---|---|
| 丢写（每个候选：H1d、实七-甲、实七-乙、实八、H-随；-配置 各臂另有 H1f、H1g） | 未够判：这一趟不量 | 第一段（H1d、H1e）、第二段（没有装置）、第三段（H1f、H1g） | 能 | 产物出来后看 `r4-compare` 的 `name=r4_loss` 行 |
| 多拒（Q4） | 未够判 | 第四段；第二段的 H-随全 | 能 | `r4-compare` 的 `name=r4_q4` 行 |
| 多读（Q5-同成） | 未够判 | 第一、四段（第二段补） | 能 | `r4-compare` 的 `name=r4_q5 … column=both_mounted` 行 |
| 第 22 条那一格（Q6） | 未够判 | 第四段 | 能 | `r4-compare` 的 `name=r4_q6` 行，与 `r4_cross_arm_sentence … control=PC-22` |
| 够判条件第二半（判据那一样今天有没有） | 已在快照上核过：S0 七件与后两件都没变（修订第 3 条） | — | 不能 | 修订第 3 条里的行号（快照 `snapshot-crates/` 上 `grep -n` 现查） |

## 八、最终一轮（格式化、改名之后的 bin）
这一轮在主工作区那份 bin 上跑（sha256 `37a8a312…445b`），18 份臂副本里的 bin 都与它 cmp 相同：
- 18 份副本重编：`grep -c 'build rc=0' build_arms_final2.log` 数出 18，warning 与 error 都是 0
- today 副本上的单测：`test result: ok. 82 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s`
- 装置变异 11 条逐条 `caught`，收尾 `restored baseline rc=0 test result: ok. 82 passed; 0 failed; 0 ignored`（`check_device_mutations_final2.log`）：**抓到 11 / 无效 0 / 没红 0**
- 臂表变异的结果（第三节）是在格式化前那份 bin 上得到的；臂代码与装置语义都没变，这一轮没重做

## 九、门禁（登记给 experiment-runner 的阶段，主工作区，bin 与三份 research 文件已搬回、`crates/mutations.tsv` 补丁没打）

| 阶段 | 退出码 | 末行 / 与 E158 的关系 |
|---|---|---|
| 33 | 1 | 红在 `crates/mutations.tsv:953`（E158 第 3 次跑 M14，fmt 拆了原文那一行）；补丁打上就清 |
| 52 | 1 | 末行「再让 research/scripts/replay.sh 里 E142 那一行的入库产物名字跟上」，与 E158 无关 |
| 80 | 0 | 末行列文件名 `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`（通过） |
| 96 | 0 | 通过 |
| 27 | 1 | 日志里没有 E158 |
| 34 | 0 | 「实验索引行与正文标题一致（索引 159 行、正文 159 份）」 |
| 40 | 1 | E158 那几行点名的是第 3 次跑与 `r2-all` 那几份旧产物（不是这一趟写的），另有 E160 等 |
| 69 | 1 | 点名 `research/mutations/e158_arms.tsv` 与 `e158_r4_arm_mutations.tsv` 改了、`research/results/` 里没有更新的 E158 产物：这一趟按派发不跑产物，各段跑完就清 |
| 75 | 1 | 日志里没有 E158 |
| 84、85、86、99 | 0 | 通过 |
| 88 | 77 | 本次未跑（无对象） |

各阶段原样日志在草稿目录 `gate-*.log`。

## 十、草稿目录（`/tmp/claude-1000/e158-r4-device/`）

- 留着：
  - `arms/` 下 18 份臂副本，各带 `target/`：五段的执行员要在上面跑产物（全部从 C0 那一次 `cp -a` 派生，S3）
  - `snapshot-crates/`：C0 快照；第二段的执行员派生新副本要用
  - `repo/`：搬回主工作区的那几份文件的底本
  - `patch/`：`crates/mutations.tsv` 的补丁
  - `src/`：格式化之前的分段源码，只作参考；以 `repo/` 那份 bin 为准
  - `trial/`：草稿副本上的试跑输出，不是产物，不入库；其中 `trial/bases` 是试跑用的起始镜像
  - 各脚本：`make_arms.sh`、`build_arms.sh`、`apply_arm.py`、`check_*_mutations.sh`、`make_device_mutations.py`
- 删了：`mut-device/`、`mutarm/`、`lint/`（大小在交回正文里）。

## 十一、没做什么

- 第二段的装置（第六节）、M9、M22。
- 没跑任何一段的产物，没写 `replay.sh` 登记行，没写实验页、索引行与 `experiments-history.md`。门禁 69 号因此红。
- `crates/mutations.tsv` 没写，交补丁。
- 没判任何候选出局或胜出，没跑重型测试，没提交。
- 要登记进 `.claude/preflight-exclude` 的路径（我写不了，交主 agent 转门禁审核会话）：`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 这一趟改了。它原本在不在排除表里，我没查。
- 登记第 5.6 节新加的 H-随 常量（快档首种子、段数、步数、28 个种子偏移、第 43 行那条用例的 8 步）的现查值没写进第十二节：它们归第二段。

## 十二、草稿目录没删的逐个理由

- `arms/` 下 18 份臂副本（today、jia-cfg、jia-cfg-carry、jia-slot、yi-cfg、yi-cfg-carry、yi-slot、yi-narrow-cfg、yi-narrow-cfg-carry、yi-narrow-slot、bing-cfg、bing-cfg-carry 与六条 `-three-rounds`）：五段执行员跑产物要用；today 那一份另要先跑 `r4-bases`。
- `snapshot-crates/`：C0 快照，第二段派生新副本要用。
- 删了：`mut-device`（476M）、`mutarm`（2.8G）、`lint`（370M）。
