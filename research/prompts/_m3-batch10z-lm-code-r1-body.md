# 根环实现第 10z 批第十二、十三块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 G1 G2 G4 E1 E2 E3 E7 Y1 Y2 Y3 Y4 H2 R C P Q -->

## 一、题面

这一轮攻两块，碰的文件不交：

| 块 | 照什么改 | 落点 |
|---|---|---|
| 第十二块（core） | 第十、十一块代码轮判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第二节 K2-3、K5 与第五节用户定案「认不出时宽护 tri」：认回退拆成读盘一层（R 的记录点名的每个打包单元读成读不出 / 读得出解不开 / 解开了三态）与纯函数一层（是回退 / 肯定不是 / 不知道）；「不知道」或「是回退而根环里没有 R_old、R_old 的账读不出」，R 那一槽并上现行账里全部已释放的落点；K5 补 E1、E2、E3、E7 的用例与变异 | `crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` |
| 第十三块（GPU） | 同一份判决第二节 K4-2、K5：借之前那一步挪进 harness 档 `GpuMemoryLedger::take_in_the_devices_kept_before_booking`，留着的设备丢了不 poll、当场照重起那一套记账（只在它还是这张卡上在用的那一台时）；K5 补 G1、G2、G4、Y1–Y4 与下游六层叫停的用例与变异 | `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` |

条款与用户原话：
- `.claude/kb/decisions/16-发布语义.md` 已定项 1 第 37 行「根槽这一次读坏」那一行（H2 丙、「丙继续补洞」）；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 第 390 行；`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第 27 行第九项；`.claude/kb/invariants.md` 第 58 行 I-7.4（近 K 代块未被复用）；回退那次发布追加谱系记录见 `.claude/kb/decisions/18-块里携带什么信息.md` 已定项 24；欠账 `.claude/kb/checks-owed.md` C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）、C762（抬 F 给根环表没记着的被抛弃根补的隔离一直不放）。
- 用户 2026-10-08 定 tri（弹窗里交代了：384 槽盘挂载 116 槽对补法 42 槽、1536 槽与回退目标深时 s2 相近一形 610–634 槽、删除发布的墓碑打包单元读不出时也会触发多护、被攻过零轮）；同日定 C762 记欠账、留原样。
- G2 岔路单 `research/prompts/m3-batch10z-a-code-r1-forks.md` G2 行；用户原话「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」；GPU 总口径 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。

改动全文：`research/prompts/_m3-batch10z-lm-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-l/patch/report.md`（第十二块，第三节量法表、第四节第九项读数、第五节 C762 复现、第六节停下交主 agent）、`…/batch10z-m/patch/report.md`（第十三块）。第十二块的量法副本留在 `~/.local/share/singlefs/work/m3-r1/batch10z-l/measure/`（带编译目录，可拷一份复跑）。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree/`，取在两块都合入之后；被判文件 sha256 在 `research/prompts/m3-batch10z-lm-code-r1-snapshot/crates-sha256.txt`。
- 第十二块关键位置（冻结副本 `crates/singlefs-core/src/mount.rs`）：`placements_in_the_accounts_of_newer_readable_roots` 第 1931 行；`placements_a_rollback_publish_revived_released_in_the_current_account` 第 2125 行；`PackedUnitNamedByAPublishAsRead` 第 2186 行；`packed_units_named_by_the_records_of_a_publish_as_read` 第 2203 行；`RollbackRecognizedAmongThePackedUnitsNamedByAPublish` 第 2229 行；`rollback_recognized_among_the_packed_units_named_by_a_publish` 第 2240 行。新用例在测试文件第 2130、2152、2167、2188、2240、2308 行。
- 第十三块关键位置（冻结副本）：`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 866 行 `take_back_the_driver_share_of_a_lost_device`、第 1443 行 `take_in_the_devices_kept_before_booking`、第 1673 行 `take_in_that_the_device_was_found_lost`；`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 第 1339 行 `take_in_the_devices_found_idle`；`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 4849 行 `every_layer_handed_the_stop_request_hands_it_on_or_asks_it`。
- 合后验证：
  - 第十三块（`~/.local/share/singlefs/work/m3-r1/merge-10z-m/summary.txt`）：merge-and-verify rc=0；mutation-tables 格 0 行筛不到，static-check `CHECKED 3446 322 10`；checker 档带 gpu lib 与 12 份挂 gpu 的测试全过，`NOT_RUN_NO_GPU_ADAPTER` 0 行；合后副本上 checker 档 9 行变异 prove-red `✓ 点名 9 条：跑了 9 条，跳过 0 条，跑的都抓到了`。
  - 第十二块（`~/.local/share/singlefs/work/m3-r1/merge-10z-l/summary.txt`）：merge-and-verify rc=0；mutation-tables 格 0 行筛不到，static-check `CHECKED 3464 322 10`；e158 `95 passed`；E171 装置 sha 前 12 位 ab35105f861d；harness 档慢档三份（19 / 23 / 16 passed）与 grep 有 `isolated_slots()` 或「第九项」的 10 份测试二进制 `--include-ignored` 全绿；checker 档快档 61 份结果 0 失败。
- 实现员自定、被攻过零轮的：
  - 第十二块：谱系片里有一条记录解不开算「不知道」；R_old 候选有一条账读不出就退到宽护；认出回退就不再看其余几片；
  - 第十二块量到的代价（量法副本）：认不出各形与正推对照臂逐格同（384 槽盘挂载 116–164 槽，1536 槽盘回退目标倒数第 23、24 版 634、整段 666），0 拒；R 不是回退而墓碑片读不出 26 → 142、R 写的全部单元读不出 18 → 116；认得出与 R 不是回退且读得出时与改前逐格同；
  - 第十三块：早记「丢了」只在有没等成借据的卡上做；早记之后到旧设备放掉之前这张卡少借多久没量；真卡上丢失标记何时变真没量，还没变真时照旧 poll，wgpu panic 被接住、算没空。
- 已知没做、不在这一轮被判范围的：C762 那一形（用户定记欠账）；Q 的记录全读不出那一形（C554）；新用例不枚举崩溃状态；第十三块真卡上的丢设备没造。
- 用户已定、报出来不算打中的：两进程同读一份空闲的竞态、一份 `nvidia-smi -q` 报告里约 1.3 毫秒的窗口；G2 乙本身；H2 丙本身；tri 的判法本身与它在「不知道」那一支上多护的那一截（只攻它的实现与没写到的边角）；C762。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　第十二块 tri 做到没有

1. 三态读盘与三种结局：有没有一形该判「不知道」却判成「肯定不是」（让复活的单元照旧发出去），或该判「肯定不是」却判成「不知道」（白多护）？R 的记录点名的打包单元有几片、几片读不出、几片解得开而类型不是谱系时呢？
2. 宽护那一截罩不罩得住：认不出时并的「现行账里全部已释放的落点」，是不是罩住了 R 引用而 C 里别处都没有的每个单元？现行账自己读不出、或 R 之后还有较新的根也走丙时呢？
3. 放的时机与准入：宽护并进 R 那一槽的落点什么时候放？有没有一形一直不放、把空间占满而准入白拒？D28 第九项计数与卸载有没有一形判错？

### K2　第十二块 K5 判别力

E1、E2、E3、E7 补的用例与变异，能不能拦住改回去？三态与三种结局的每一支，改回去有没有照绿的？

### K3　第十三块 借之前早记「丢了」

1. 早记丢了之后，账给的量会不会大于卡上真能再给的（多给）？旧设备还有克隆没丢、缓冲还占着卡时呢？
2. 只在那台设备还是在用的那一台时才改账：卡池取到的是重起之前留着的那一台、重起正在进行、两条线程同时借同一张卡时，有没有一形把新设备记成丢了，或丢了的永远记不上？
3. 早记只在有没等成借据的卡上做：没有没等成借据、但设备丢了的卡，借的时候会不会走到别的放不下或判错？

### K4　第十三块 K5 判别力

G1、G2、G4、Y1–Y4、下游六层叫停的用例与变异，能不能拦住改回去？有没有一形「改回去」照绿？checker 档那几行在合后副本上真跑过（第二节），扫源码用例能不能被一种不改行为的写法骗过？

### K5　只防 OOM、不压并发两头（第十三块新代码引进来的）

1. 有没有一形账给的量大于此刻真能用的？
2. 除用户已认的之外，有没有一形卡放得下却报本次未跑、判红、整趟弃卡，或并发被压住？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
