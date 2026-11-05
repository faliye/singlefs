# m3-close-b19-code-r2：批 19 验证段代码轮第二轮正文（2026-10-29）

<!-- doc-lint:not-numbers U1 U2 U3 U4 U5 U6 P1 P2 P3 P4 -->

## 一、题面

第一轮判决 `research/prompts/m3-close-b19-code-r1-main-verification.md`。这一轮攻第一轮的修补本身（改动 `research/prompts/_m3-close-b19-code-r2-diff.md`：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/instance_table_sweep.rs`；修补报告 `~/.local/share/singlefs/work/m3-close-b19-code-fix1/result.md`），产生第一轮结论的方法，与第一轮没看到的地方。第一轮改动全文仍在 `research/prompts/_m3-close-b19-code-r1-diff.md`，那一轮的六份产品代码照旧在射程里。第二轮分工照 `.claude/rules/three-way-inference.md`「多轮」一节：第一轮的修补本身、产生结论的方法、从未被看过的地方；辩方腿复核第一轮判决够不够得着；攻方腿换攻击面，不重复第一轮攻过的角度。

## 二、实现今天的样子（主 agent 的观测，2026-10-29）

方案按 `crates/` 今天的实现来谈。修补已打在主工作区（没验证；修补员在第二台跑过 harness 29 个目标 208 过、`singlefs-core --lib` 371 过）：

- `crates/singlefs-core/src/mount.rs` 的 `establish_instance`：逐盘核 `devices_rewritten_by_the_acquisition_after_checking_each_device` 挪到续做预演 `refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range` 之前，预演与真做都用逐盘核交回的补写盘表；`PublishesPlannedAfterAcquisition` 字段文档照新次序改。改法二（补写盘两槽读坏不记账）没加。
- `mount.rs`：公开入口 `raise_rollback_floor_to_the_admission_ceiling` 与只供测试的 `raise_rollback_floor` 交回 `NoValidRoot`；断言挪进 `push_one_floor_raise_within_the_admission_budget`（理由句照旧）。
- `crates/singlefs-core/src/instance_table_sweep.rs`：`DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE` 文档改说清扫触发阈值、每片仍 369 行。
- 新用例：`crates/singlefs-harness/tests/writable_mount_rehearsal_takes_the_devices_rewritten_by_the_acquisition_from_the_check_of_every_device.rs`、`admission_floor_ceiling_entry_returns_no_valid_root_for_an_allocator_without_the_root_ring_table.rs`、`discard_after_the_frozen_publish_is_resent_reaches_the_segments_reclaimed_before_the_freeze.rs`。
- 池级 checker（测试代码，P 格是推论）：`crates/singlefs-checker/src/walk.rs` 的 I-7.9 按 journal 重建（`root_in_the_ring_slot_at_the_raise`）、I-3.1 按 I-3.18 取宽、I-3.20 缺 `deleted_inodes` 报部分没判。

## 三、格

### U1　逐盘核挪到预演之前
新次序下：逐盘核本身读盘（读坏、读错）会不会让预演那一趟看到的盘面与真做不同；逐盘核拒与预演拒同时成立时今天报哪一个（修补员说从预演那一拒变成逐盘核那一拒），条款（D22 那一行与 C874）对先后有没有约束；钉读序数的用例一个没改，是不是因为它们没跑到新次序那一支。

### U2　改法二要不要
修补员推「这一形预演只走一趟、读坏账没被看」。攻：预演停在别的非 C841 错上、又有补写盘的那一形造不造得出；造得出就看补写盘两槽的读坏账会不会让它重走到上限后拒，给盘面。

### U3　T2 断言的放处
断言挪进 `push_one_floor_raise_within_the_admission_budget` 之后：本进程准入路径上还有没有一处在没有有效根时 panic 而不是交回错误；`raise_rollback_floor`（只供测试）一并改是不是越出了判决 T2。

### U4　C928 解冻用例钉的是不是条款
新用例钉「重发冻结那一次成功之后，积下的 discard 在下一次进 discard 调用点时发出」：它覆盖的是不是三个调用点里会走到的那一个，冻结期间再冻结、重发失败后回卷那两形有没有丢 discard。

### U5　C929 回卷之后过不过补写闸（设计判断）
第一轮两腿一致读出：回卷之后那块盘有自证槽、走落后支，补写闸三、四、五不再判。判：该不该再过闸；给两个候选（照今天不判 / 回卷后的下一次挂载仍判三、四、五）各自在哪些盘面上会错，推荐哪一个。

### U6　第一轮的方法
第一轮 T1、T3、T5、T6 判「相符」靠读码，没编没跑：挑其中最可能被读漏的一格，用会红的用例或探针核一次。

### P1–P4　第二次抽样
第一轮 P1（C934 认 u 落了盘的前提）、P2（C937 取宽条件）、P3（RCW 拿掉 F_生效 例外）、P4（Q9 条件在 I-3.1 / I-3.18 / I-3.20 三处的认法）都没打中、各一次抽样。换攻击面再攻一次，P4 把第一轮没读完的 I-9.16 ④、I-3.20 两处读完。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 辩方腿（Sonnet） | U1–U6 | 复核第一轮判决：采了的 T4 改法与 T2 改法够不够得着，判「相符」的那几格是不是被读码冤枉放过；U5 替两个候选各辩一次 |
| 攻方腿（Opus） | U1–U6、P1–P4 | 换攻击面：修补本身、第一轮的方法、没看过的地方；不重复第一轮攻过的角度 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码写开工快照里的路径与行号。
- 方案按 `crates/` 今天的实现来谈；要建小模型或探针的，模型放 `research/prompts/m3-close-b19-code-r2-<模型>-model/`（带 `tools/rerun.sh`），草稿放腿自己的目录（`~/.local/share/singlefs/work/<腿名>/`），不在主工作区改。
- 不跑重型测试、不起 checker 档的 cargo、不碰 GPU；小模型与 harness 探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限，本机排不上就经 `scripts/hosts/multi-host-run.sh --peer` 起。
- 报告写进 `research/prompts/m3-close-b19-code-r2-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
