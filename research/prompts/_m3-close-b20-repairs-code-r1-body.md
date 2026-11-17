# m3-close-b20-repairs-code-r1 正文：第 20 批修补的代码轮（2026-11-09）

批次：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 20 批。方案按 `crates/` 今天的实现来谈。上一轮判决 `research/prompts/m3-close-b20-code-r2-main-verification.md`，它第五节写「被攻过零轮的」与「下一次代码轮攻这一批修补」，这一轮攻的就是这一批。

## 被判的产品代码

| 文件 | 范围 | 来历 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs` | 第 9871 行 `first_txg_of_new_instance` 调用点，第 1478 行起它的定义；删掉的 `FirstTxgOfTheNewInstance` 一族 | E205 退场（用户定案，判决 `m3-close-b20-code-r2` 第四节），补丁 `~/.local/share/singlefs/work/m3-close-b20-e205-retire/patch-without-manifest/`，只供测试的开关连同入口删除，改动 +2 −586 |

## 一并攻的非产品改动（判决第五节点名的零轮形态）

| 文件 | 范围 | 来历 |
|---|---|---|
| `crates/singlefs-verification/src/mount_transparency.rs` | 第 3293 行 `judge_writable_mount_refusal` 带 `filesystem_identifier_of_the_pool`；第 2630 行 `image_after_the_refusal_into_the_undecidable_witness` | 判官加本池 fsid，补丁 `m3-close-b20-judge-fsid/patch`；崩溃观察者的共用造法，补丁 `m3-close-b20-crash-observers/patch` |
| `crates/singlefs-verification/src/verification_read.rs` | 第 194 行 `choose_system_configuration_of_the_pool` | 同判官加 fsid |
| `crates/singlefs-verification/tests/common_unwitnessed_publish/mod.rs` | 第 667 行 `image_with_the_newest_rotation_of_each_device_not_written` | 换造法，补丁 `m3-close-b20-tests-after-yi/patch` |

## 实现今天的样子（主 agent 的观测）

- 读过：`crates/singlefs-core/src/mount.rs` 第 1478 行、第 9871 行与删开关的 diff；`crates/singlefs-verification/src/mount_transparency.rs` 第 2630、3293 行。
- 第 9871 行：`let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records)?;`，没有档位，全仓 grep `FirstTxgOfTheNewInstance` 在 `crates/singlefs-core/src/` 下 0 命中。
- 同时在主工作区里的还有别的会话对 `mount.rs` 等文件的改动，不归这一轮；腿只判上表这几处。

## 压着的条款

- 判决 `research/prompts/m3-close-b20-code-r2-main-verification.md` 第四、五节。
- `.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 8（第 177 行起）射程；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 第 3 条；`.claude/kb/checks-owed.md` C971、C973、C974、C975 四行；`.claude/rules/fs-design.md` 五条硬要求第 2、4 条。

## 要两条腿回答的问题

1. 辩方腿：复核 r2 判决第五节的零轮形态。判官加 fsid 修没修到 r2 打中的 A1、A2；B1、B2、C973 射程句的条款写法与代码对不对得上；取号这一形的 D22 新写法是不是用户定的「拒，留两轮」。
2. 攻方腿，不重复 r1、r2 已攻的角度，攻这几面：
   - 删开关之后产品路径还是不是 D23 已定项 14 第 3 条那一档；删的 586 行里有没有被产品路径或别的测试入口依赖的逻辑（`every_root_ring_slot_read_once` 第 3270 行留着，核它的调用点）；`MountOutput` 少了一个字段有没有让断言变空。
   - C971 在 E205 退场之后的处置：不再量甲那两档，C971 那一形（多处读坏时写出倒挂的新实例根）今天靠什么拦、拦不住的那一半登记全没有。
   - 判官按本池 fsid 筛之后，判官与产品会不会在别的盘面上反过来判错（判官多拒、产品放行，或相反）；模型与实现是否仍共用错误前提。
   - 新造法 `image_with_the_newest_rotation_of_each_device_not_written` 造的盘面是不是用例想要的那一形，换造法后原来那批用例还罩不罩原来的场景。
   - 崩溃观察者 `image_after_the_refusal_into_the_undecidable_witness` 共用这一处，会不会把 18 个观察者一起判错。
3. 两条腿都对下面八格逐格回「成立 / 不成立（文件:行）/ 推不出」。

### 检查单：走不到的理由二次查实

本轮改动没有这一类：实现员报告 `~/.local/share/singlefs/work/m3-close-b20-e205-retire/result.md` 第 56 行写「新测试：无，故没有证红命令，也没有「走不到的理由」…（没动 checker，没加断言与错误成员）」。腿照调用点核这句话；另外三份报告在各自草稿目录的 `result.md`，核同一句。

### 检查单：类型与名字

删开关没有新加类型、错误成员或 `expect`（新加两行：`mount.rs` 文档注释一行与第 9871 行）。判官新参数 `filesystem_identifier_of_the_pool`、新函数 `choose_system_configuration_of_the_pool`、`image_with_the_newest_rotation_of_each_device_not_written`、`image_after_the_refusal_into_the_undecidable_witness`。腿核 fsid 是不是 newtype，有没有 `_ =>`、缩写。

### 检查单：不兼容旧代码与旧格式

这一批删东西，没有新认旧版本的分支。腿核删完有没有留下只为旧开关存在的分支或注释。

### 检查单：不新建判定库

本轮改动没有这一类（查过四份补丁的 `crates.patch` 改动行）。

### 检查单：所有任务多机可用

新用例用内存盘，不读本机状态。另知一事：`the_gpu_recovery_kernel_reads_a_unit_through_a_central_mapping_of_more_than_a_thousand_entries_like_the_cpu_recovery` 在 5060 Ti 上首次派活 Xid 109 超时（任务 11635），在 32G 卡上绿（任务 11588），不是这一批改的代码，正在查归属；腿判它算不算多机缺陷。

### 检查单：测试优先多线程

新用例都是单盘面。崩溃观察者新目标 `crash_points_observers_take_only_the_refusal_into_the_undecidable_witness` 12 条，各一个场景。腿判该不该并行。

### 检查单：带 --item 的命令与项

四份补丁改到的变异行 41 行，主 agent 交 11592、11593、11594（排队，等结局）；观察者 13 行在副本上证红过（任务 11062）。没有新崩溃枚举项，新目标由 `--list-items` 列得出（任务 11066）。

### 检查单：新崩溃流多罩了什么

本轮没有新加崩溃流。

## 已知的验证读数（主 agent 量的）

合入后验证 11336：测试 3375 条全绿、clippy、build 过；缩到展开 5 的 98 条崩溃用例判法修好后全绿（11507 判绿 55、11621 判绿 3，其余 40 条上一趟就绿），缩过域、不算门禁通过；挂特性主体通过 654、失败 1（上面那条 Xid 109）；`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 撞四小时硬上限没结局，留给提交门禁。
