# m3-batch59-code-r1 判决：删除线第 59 批代码轮第一轮（2026-10-10）

被判的改动（按路径点名）：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs`、`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs`、`crates/mutations.tsv`；`crates/*/src/` 一行没动。全文 `research/prompts/_m3-batch59-code-r1-diff.md`，正文 `research/prompts/_m3-batch59-code-r1-body.md`。

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch59-code-r1/repo/` 与主工作区各 `sha256sum -c research/prompts/m3-batch59-code-r1-snapshot/crates-sha256.txt`；仓根 `kb-sha256.txt` | crates 5 份（含变异表）两处都对；kb 全对。根环线 V…E8 合入会整行换掉、删掉变异表里的行（它 2026-10-10 告知），合入前它拷的那份 `~/.local/share/singlefs/work/m3-merge-impl/batch59-snapshot-copy/mutations.tsv` 与快照、冻结副本三处 sha256 都是 fe458a20…；判决按冻结副本 |
| 正推腿 `research/prompts/m3-batch59-code-r1-sonnet-output.md`（175 行，sha256 7f694b306a44…）、攻方 `research/prompts/m3-batch59-code-r1-opus-output.md`（278 行，sha256 3e82a320cf77…） | `cite-check.py` 两份合判，`--root` 仓根，`--background` 背景材料 | 「✓ 核了 40 处引文，对上 40 处，没判 0 处（逐处列在上面）」 |
| 攻方模型 | 拷到 `~/.local/share/singlefs/work/m3-batch59-code-r1-verify/model/`，`rerun.sh run all` | 见第六节 |
| 合入后验证（主 agent 在主工作区跑） | `merge-and-verify.sh`、checker 档 CPU 快档、变异轻型（核心部分加点名第 3861、3865–3868 行）、崩溃轻型、带 gpu 那一趟；甲三条在干净构建上单跑 | harness「✓ 测试全绿（194 个测试二进制，共 2015 条测试）」；CPU 快档「61 targets passed 668 failed 0 ignored 16」；变异轻型点名 5 行全抓到，核心部分 13 行红（基线不绿 5、无效 2、没红 5、超时 1），都是旧行、不在第 59 批改动里，派分诊（第四节）；甲三条：谱系「states=489 … 393 / 393 … mismatches=0」、墓碑「states=561 … 429 / 429 … mismatches=0」，退役「gpu_red=24 … recovery_mismatches=24」（第 1 字 3，判不了），根环线打上 E6 后「mismatches=0」「1 passed」（`~/.local/share/singlefs/work/m3-merge-impl/ve6c-check/r-retirement.log`）；墓碑、谱系两行 GPU 变异「点名 2 条：跑了 2 条，抓到 2 条」。崩溃轻型（8 项退 252，内存包装自己的结局）与带 gpu 那一趟（「could not compile `singlefs-checker-tier`」15 处导入找不到）撞上根环线 V…E8 往主工作区合入的当口，都不算结果，合入完成后重跑，结果记进删除线调度记录、并进第 60 批合入后验证 |
| 攻方 M1-4 的 GPU 那一半 | 冻结副本重拷到 `m3-batch59-code-r1-verify/m14-repo/`、编译目录全新，`crash_judge_checker_scan.wgsl` 第 3357 行换成 `container_index + 3u >= d_container_count && …`，跑过滤串 `knob_two_agrees_with_the_cpu_on` 的 9 条（`m3-batch59-code-r1-verify/m14.log`） | 改坏下「test result: FAILED. 8 passed; 1 failed」，红的那一条是退役 L1（基线就红、正文第二节已知），墓碑、谱系的 J3 与 L1 四条都「ok」：坐实 M1-4 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| M1-1 只因旋钮支红 | 一致；附：断言只钉「CPU 红了旋钮那条」与逐位一致，不钉「只红这一条」，CPU 复现 52 个取样点上只红那一条 | 没打中，量过：CPU 替身状态数 561 / 489 / 621、违例态 429 / 393 与主 agent 卡上那一趟逐个相同，每个状态上只红旋钮那一条 | 两条腿同向，过。正推附的那一半断言采（N1） |
| M1-2 候选集与分得开的版本 | 一致：「只判倒数第二片」在转根第 22 / 23 次之后分得开，余量 7–8 次 | 没打中，量过：分得开的落在最后 8–9 版（谱系 txg 33..=40、墓碑 38..=46、退役 44..=51） | 一致，过 |
| M1-3 30 次空 inode 不改容器树 | 一致 | 没打中，量过 | 一致，过 |
| M1-4 GPU 只判最右之前两片 | 没报 | **打中（弱）**：GPU 内核改成 k = 2 时，J3 与 L1 六条 GPU 流替身摘要逐个等于基线，别的 GPU 旋钮流至多三片；CPU 那一侧 L2 已钉，GPU 那一侧没有对应的流 | 采。主 agent 在卡上坐实（第一节最后一行）：GPU 对拍分不开 k = 2；与「有限组关不死」的已知欠账同族，但 CPU 一侧窗口已到 3、GPU 一侧还在 2，补齐到同一档（N1） |
| M1 变异表第 3862–3864 行 | 一致：原文命中 1 次，三行同一刀；附：甲补丁注释说 harness 那三条是「慢档」，乙已去掉 `#[ignore]`，两处不一致 | — | 注释那一处采（N1） |
| M2-1 [2,1,2,2,1] 三张盘 | 一致：只红 I-9.22 / I-9.37，k = 2 下三条全红 | 没打中，量过 | 一致，过 |
| M2-2 k = 3、4 | 现有盘没有一张分得开 | 已知欠账，不当打中报 | 一致。有限组排布对足够大的 k 关不死，照旧记欠账（第四节） |
| M2-3 去掉 `#[ignore]` 后 | 结果一致；附：判决 L5「各单跑一次量用时」与 `.claude/rules/verification.md` 第 26 行「单条用时看平常跑的时候量到的，不另外单独量」冲突，规则没说什么负载算平常 | 没打中，量过：debug 快档整份 265 passed、挂钟 111 秒，用例之间没有共享状态 | 主 agent 判：L5 的写法错在判决（照规则不该另外单独量），结论照规则也成立——平常跑每条 CPU 约 2–3 秒，远在 60 秒之下；忙时几十上百秒是 nice 19 被抢 CPU。不改 |
| M3-1 J1 ② 可写挂载 | 一致；附：「分配记录判在条目宽之前」这半句没有盘钉，造的「挪到条目宽之后、逐条判之前」一形整份 41 条全绿 | 没打中，量过 | 一致，过。附的那一半是第 58 批起记的已知欠账；照用户 2026-10-10 定的「测试要钉住」，不再挂着，进第 60 批（N3） |
| M3-2 第 3861 行 | 一致：只红 J1 ① 可写挂载 | 没打中，量过 | 一致，过 |
| M3-3 L4 新造法 | 一致：两种读者改法下新用例绿、旧用例红；第 3668 行变异下新用例红；附：将来重建在水位判之前逐行判 I-3.13 时红在 `core_review_…:627` | 没打中，量过 | 一致，过 |
| M3-4 读者逐条判 I-3.13 | 同上一行附的那句（规则没说） | **打中**：逐条判 I-3.13 时 L4 那条是 56 条用例加 71 条合法历史里唯一红的，红在 `core_review_tree_table_duplicates_and_slot_one_search.rs:632`，重建报 I-3.13；用例注释第 543–544 行「14 个 key 都已经有行」不成立（统计量 7、13、14 都登记过，这一版没写）；换成补 (7, 0, 0xFFFF_FFFF, txg) 三种读者改法下都照报水位行缺、第 3668 行下照红。另一半不分辨：按今天的数条目规则，任何造法都要靠读者少判一条 | 主 agent 现查：`.claude/kb/decisions/05-快照-空间记账机制.md` 第 124 行「7 \| 扩展点配额已用量」、第 130、131 行 13、14 都在登记表里，注释那句确实不成立。采攻方的造法并改注释（N2）。不分辨的那一半照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「打中不分辨臂」另记，不拿它判造法；也不为它改 `src`：读者收严时这条红在别的成员上、看得见，不是悄悄绿 |

## 三、采纳的改法（第 60 批）

| 编号 | 改法 | 验收 | 来源 |
|---|---|---|---|
| N1 | GPU 三棵树各加一条 [2,1,2,2,1]、转一圈根环的旋钮对拍；L1 那三条与新三条加断言「CPU 违例集恰好是这一棵那一条」；文件里说 harness [2,1,2,1] 三条是「慢档」的注释改掉 | GPU 内核 k = 2 那一形（第 3357 行换成 `container_index + 3u >= d_container_count && …`）各一行变异，红在新加的三条上（主 agent 在卡上证）；新断言在正确判法下绿 | 攻方 M1-4，正推 M1-1、M1-d |
| N2 | L4 那条换成补 (7, 0, 0xFFFF_FFFF, txg) 让条目数对上，注释第 543–544 行的推理改成实情 | 第 57 批两种读者改法与「逐条判 I-3.13」三种分开与合起来都照报水位行缺；第 3668 行照红 | 攻方 M3-4 |
| N4 | 变异轻型核心部分红的 13 行（变异分诊 `~/.local/share/singlefs/work/m3-batch59/light-triage/report.md`，都不是删除线第 52–59 批带进来的）：照它的 `proposed-rows.tsv` 整行替换 6 行（增补 1 漏计数据单元的写、步 6 失配即中止、C490 offset 段、实二七、C673 ⑤、E171R1 U12），C490 共享内生块那一行换一处真改行为的改坏或删行，实审A1 第19条那一行写一条「回卷后再取号」的敏感用例（草稿 `light-triage/r858-sensitive-sample-test.diff`），第10g批影子账那一行补抬 F 那一路的取样点（造不出就按等价重判、写进报告）；5 行超时低负载重跑 | 低负载下 `--quick` 核心部分「无效 0、没红 0、超时 0、基线不绿 0」；改了名的行，轻型登记表 `.claude/gate.d/crates-mutation-light-rows.tsv` 由主 agent 跟着换 | 变异分诊；用户 2026-10-10 定「并进第 60 批」 |
| N3 | 钉「分配记录判在条目宽之前」：一张分配记录代晚于根、同时一条记账条目窄于字段表的盘，冷走读与可写挂载都报分配记录 | 正推造的「分配记录判挪到条目宽之后、逐条判之前」（冷走读与重建各一形）各一行变异红在新用例上 | 正推 M3-1 |

## 四、零轮与欠账

- N1–N3 被攻过零轮，第 60 批合入后开它的代码轮。
- 单次没打中、不作结论的格：M1-1–M1-3、M2-1、M2-3、M3-1–M3-3 两腿同向、各抽一次。
- 「只判最右之前 k 片」：CPU 一侧窗口推到 3（L2），GPU 一侧 N1 推到 3；对足够大的 k 有限组排布关不死，照旧记欠账。
- 水位行那一判在完整的读者里永远轮不到先报（M3-4 不分辨的那一半）：记欠账，题面是「`InodeNumberWatermarkRowMissingFromTheAccountingTree` 只在读者少判某一条时可达」；要关死得改判法（例如水位行缺并进数条目那一判），那是 `crates/*/src` 与 I-1.10 判定顺序的改动，不在删除线这几批里做。
- 变异轻型核心部分 13 行红（不在第 59 批改动里）：派分诊 `~/.local/share/singlefs/work/m3-batch59/light-triage/`，按它的结论归到改它的那一方。
- 上一轮记下的欠账与线索照旧（第 56、57、58 批判决第四节）。

## 五、交用户表

无。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D18（块里携带什么信息） 已定项 24 | 不影响 | 2026-10-10 不受影响：第 3 条「满了才关」照写，第 59 批只改测试 |
| D5（快照 / 空间记账机制） 已定项 2 | 不影响 | 2026-10-10 不受影响：N2 换造法，不改「每条根的记账树只装这条根那一代的行」 |
| D5（快照 / 空间记账机制） 已定项 4 | 不影响 | 2026-10-10 不受影响：统计量 7、13、14 的登记照表 |

## 六、复跑核对（攻方模型）

攻方模型拷到 `~/.local/share/singlefs/work/m3-batch59-code-r1-verify/model/`，在空工作目录 `run/` 从头跑 `SINGLEFS_SCRIPTS=scripts OPUS_THREADS=10 OPUS_MEMORY_CAP=8G bash model/rerun.sh run all`（冻结副本取缺省路径），rc=0，compare 原样末行「compare：46 份，不一致 0 份」。第二节引的攻方数（CPU 替身状态数 561 / 489 / 621、违例态 429 / 393；分得开的根 txg 区间；k = 2 下 J3、L1 六条摘要等于基线、[2,1,2,2,1] 三条违例态 417→324、477→372、513→420；M2 debug 快档 265 passed；M3 第 3868、3861 行各只红点名一条；逐条判 I-3.13 时 L4 那条红在第 632 行、统计量 7 那一造法照报水位行缺）都在这 46 份里，主 agent 复跑与腿交的一致。
