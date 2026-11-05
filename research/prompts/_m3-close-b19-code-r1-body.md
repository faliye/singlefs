# m3-close-b19-code-r1：批 19 验证段代码轮第一轮正文（2026-10-29）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 P1 P2 P3 P4 -->

## 一、题面

这一轮攻批 19（调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md` 批 19）合进主工作区的产品代码改动，连同判决里被攻过零轮的几处。改动全文 `research/prompts/_m3-close-b19-code-r1-diff.md`（`git diff HEAD` 六份文件：`crates/singlefs-core/src/admission.rs`、`allocator.rs`、`instance_table_sweep.rs`、`mounted_session.rs`、`mount.rs`、`root_record.rs`）。这份 diff 是 HEAD 到此刻主工作区的上界：`mounted_session.rs` 里 `CompactionRun`、`BackgroundCompactionCall`、`CompactionStep`、`run_compaction_on*`、`advance_compaction_by_one_publish`，`mount.rs` 里 `mount_writable_with_compaction_switches`、`WritableMountWithBackgroundComputeRefused`、`mount_writable_with_background_compute_settings` 那几处是别的会话的改动，不在这一轮射程里；其余 hunk 出自批 19 的补丁（core 判决实现、P2 第一波、P3、两份验证段修补、改名一份）。

被判的条款：判决 `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R1–R4 与第四节；`research/prompts/m3-close-b19-design-r2-main-verification.md` 第二节 Q3、Q5、Q9、Q10 与第四节；`.claude/kb/checks-owed.md` 的 C773、C724、C928、C874、C929、C690、C934、C937 各行；D16（发布语义） 已定项 1、D23（journal 的角色与格式） 已定项 14。

## 二、实现今天的样子（主 agent 的观测，2026-10-29）

方案按 `crates/` 今天的实现来谈。批 19 的补丁都已打在主工作区，合入后验证在第二台整工作区编过、测试全绿（除 checker 档包；`~/.local/share/singlefs/work/merge-and-verify/2026-10-29-1405859/`）；fmt 与 clippy 的红都在别的会话的文件里。下列落点按此刻主工作区现查：

- `crates/singlefs-core/src/allocator.rs`：`ReclaimFloorWhenTheRootWasRecorded`、`oldest_txg_in_the_ring_rereading_slots_bad_at_mount`、`PoolAllocator` 里写根时带（钉值, 槽）的那几处（C773 写者，core 判决 R1 采乙：第 0 代根所在槽的钉值取 0、平局按（钉值, 槽）破）。
- `crates/singlefs-core/src/root_record.rs`：`PinnedRingSlot`、`UnmountMarker` 与测试模块。
- `crates/singlefs-core/src/mount.rs`：`raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk`、`raise_the_floor_through`、`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`（C724：第二处改 `expect` 并写理由）；`RewriteGateNotMet`、`MountOutput`、`devices_rewritten_by_the_acquisition_after_checking_each_device`、`SystemConfigurationUnreadableAfterTheReadStage`、`DevicesCheckedBeforeTheAcquisition`（C874 可观测：补写闸没过的原因进错误载荷、成功一支在挂载输出里报补写过的盘）；`txgs_ring_slots_bad_at_mount_can_hold`、`ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes`、`ring_slots_recorded_as_holding_an_abandoned_root_by`、`establish_instance`、`devices_without_the_selected_version`。
- `crates/singlefs-core/src/mounted_session.rs`：`discard_the_segments_reusable_as_a_whole` 与 `DiscardsAfterThePublishes`（C928：discard 先判冻结）。
- `crates/singlefs-core/src/instance_table_sweep.rs`：清扫阈值默认 365（C690，常量 `DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE`）。
- `crates/singlefs-core/src/admission.rs`：只动 `#[cfg(test)]` 模块（用例改名与 P3 的用例）。
- 池级 checker（测试代码，不在代码轮射程，P1–P4 格是推论）：`crates/singlefs-checker/src/walk.rs` 的 I-7.9 按 journal 重建（`root_in_the_ring_slot_at_the_raise`）、I-3.1 按 I-3.18 改记槽数取宽、I-3.20 缺 `deleted_inodes` 报部分没判；GPU 扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 同判。

## 三、格

### T1　C773 写者带（钉值, 槽）到写根
`allocator.rs` 与 `root_record.rs` 对 core 判决 R1（采乙）与 D16（发布语义） 已定项 1：三个槽 0 的钉值取 0、按（钉值, 槽）取最小带到写根、平局按（钉值, 槽）破，p 照存公式槽 `3·槽 + 区域`。问：所有写根路径是不是都带上了这一对；读坏、读回全零、读故障三形下 (t, p) 是不是条款说的那一个；零轮的那部分（R1 的 GPU 走读内核改法不在产品代码，但写者与池级 checker 两边对 p 的还原是不是同一个函数关系）。

### T2　C724 两处 NoValidRoot
`mount.rs` 两处对 core 判决 R2（写明走不到）：第二处 `expect` 的理由句与结构上被第一处先拦的论证成立不成立；第一处在本进程准入路径上走不到的两条前提（前提 b「根带的 F 不高于它自己的 txg」是推的，判决要求补一条断言钉住）有没有钉住；共享函数保留这个错之后外部调用方拿到它的语义。

### T3　C928 冻结着时整理不发 discard
`mounted_session.rs` 对 core 判决 R3 的 K3-1 与 C928：discard 开头判冻结之后，冻结着的整理那一趟还有没有别的路径把冻结那次已落盘的根引用的单元所在段交出去（入口、批间钩子、`DiscardsAfterThePublishes` 的累积）；冻结解除之后积下的 discard 什么时候发、会不会丢。

### T4　C874 补写闸的原因与补写过的盘
`mount.rs` 对 core 判决 R4（新的公开封闭枚举：闸前读错加五件闸，只报第一件、第一块盘）：枚举成员是不是覆盖了 `devices_rewritten_by_the_acquisition_after_checking_each_device` 里每一处拒绝；「只报第一件、第一块盘」的取法在多盘多件同时不过时是不是确定的；成功一支报的补写过的盘与真写了的盘是不是同一集合。C929（补写盘回卷之后不再过补写闸）条款没写，这一格顺带答：回卷之后那块盘该不该再过闸，今天的代码做的是哪一种。

### T5　C690 清扫阈值 365
`instance_table_sweep.rs` 与 `mount.rs` 对 C690（用户定 365）：阈值只改了清扫触发、没改每片 369 行，两者的关系（365 行之后清扫、369 行写满一片）在第 366–369 行之间有没有一形会让行写不进也不清扫。

### T6　其余 mount.rs 改动
`txgs_ring_slots_bad_at_mount_can_hold`、`ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes`、`ring_slots_recorded_as_holding_an_abandoned_root_by`、`establish_instance`、`devices_without_the_selected_version`：逐处判它压着哪一条已定条款（C773 或 C874），有没有压不着条款的行为变化。

### P1　C934 认 u 落了盘的前提
`walk.rs` 的 `root_in_the_ring_slot_at_the_raise` 认「u 那一版落了盘」只靠「u 与 r 同一实例，或 u 那个实例的行 T > u」，前提是实例中间的一版根槽一定落了盘（推的）。攻：写者在根槽写失败时实例停不停（对着 `allocator.rs` 记根与冻结的路径），恢复会不会把 T 推过 u，钉槽 p 能不能认出这一形；造得出就给盘面。

### P2　C937 I-3.1 照 I-3.18 改记取宽
两个方向（记录比单元宽、比单元窄）都放过、(L)(P)(U) 三判都取宽，会不会漏一块没有别的不变量红的畸形；只放「记录比单元窄」一个方向够不够。

### P3　RCW 拿掉 F_生效 例外
design-r2 判决 Q5 拿掉「F_生效 高于根上 F 时不判」那条例外（攻方推的）：今天的写者上有没有一段合法历史在 RCW 下因此误红。

### P4　Q9 条件的通用写法
「某处配不上那块畸形若有别的不变量判红，那一处报部分没判；没有就判红」落到 I-3.1（I-9.16 ④、I-3.18 两形）与 I-3.20（缺 `deleted_inodes`）三处：认「别的不变量判红」的判法是不是与那条不变量自己的判法同一份（不是各写一份近似）；有没有一形两边认法不一致，导致既不红也不报部分没判。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿（Sonnet） | T1–T6 | 逐格核「代码做的是不是条款说的」，给出每格压着的条款原文与代码行 |
| 攻方腿（Opus） | T1–T6、P1–P4 | 攻哪一格会错；P1–P4 是推论格，攻得出就造盘面或小模型 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码写开工快照里的路径与行号。
- 方案按 `crates/` 今天的实现来谈；要建小模型核的，模型放 `research/prompts/m3-close-b19-code-r1-<模型>-model/`，草稿放腿自己的目录（`~/.local/share/singlefs/work/<腿名>/`），不在主工作区改。
- 不跑重型测试、不起 checker 档的 cargo、不碰 GPU；小模型与 harness 探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限。
- 报告写进 `research/prompts/m3-close-b19-code-r1-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
