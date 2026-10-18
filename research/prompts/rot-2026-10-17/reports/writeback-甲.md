# 写回甲（欠账表）报告 2026-10-17

没建副本，没起编译，没跑门禁与 doc-lint，没提交。只改了 `.claude/kb/checks-owed.md` 一个文件。

## 一、结论

- 改了 89 处，两条命令：`replace-batch.py` 一次 88 处；写完后回读发现 C532 行新写的 `grep -c '|exact$'` 里的竖线没转义、把那行撑成 7 格，再用 `replace-once.py` 补了一处（转成 `\|`）。回读脚本核过：欠的表每行 6 格、已还清表每行 4 格，没有欠的表里以「已还」「销账」「作废」开头的格。
- fixes-甲.tsv 93 行：改了 90 行（同一载体行被几段各判一次的合成一处，`git diff -U0` 数得 69 个已有表行被改、新增 9 行：欠的表 5、已还清表 4），没改并交主 agent 3 行：C605、C709、C714（见第四节）。主 agent 交的 10 件：全部有去向，其中 C709 按指示不改、列进交用户清单。
- 编号核过：欠的表最大号 C846，但 `/home/fy5090/.local/share/singlefs/work/c834-writeback-spec/spec.json` 第 8 条已占 C847–C851（挂在 C846 行后），所以新立的欠账从 C852 起，不是 C847。

## 二、规格与命令原样输出

- 规格：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-wb-甲/spec.json`（sha256 `c1429a25eec2f6c3ff7e92784cea17f399d9f4021206c83d5c4f48c71ef7cbba`），同内容 `spec.md`（`8889baa50c6ae14e98f67bcea2eb7ba192e83f9aad5da9424a9da90193267998`）；生成脚本同目录 `build-spec.py`。
- 写之前：`python3 scripts/kb/kb-spec-check.py spec.json` 与 `spec.md` 末行都是 `✓ 规格 88 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 88 条）`，退 0。写之后再跑会红（旧串已换掉），属正常，不作数。
- 写范围闸：`write-guard.sh` 对 `.claude/kb/checks-owed.md` 退 0。
- `replace-batch.py --dry-run`：`✓ 88 处都命中得对（1 个文件），--dry-run 没写`。
- `replace-batch.py`：`✓ 88 处替换，写了 1 个文件（改名换上新 inode），回读一致`，退 0。
- `replace-once.py`（C532 竖线）：`✓ .claude/kb/checks-owed.md：命中 1 次，已替换（改名换上新 inode）并回读确认`，退 0。规格文件里 C532 那一条仍是改前的写法（没转义），以文件为准。
- 没跑 `doc-decisions.sh --write`（并行写回，主 agent 合入后跑）；本次没动决策与状态，不需要。

## 三、主 agent 交的 10 件

1. 已还清表 8 笔占位句：C626、C627、C601、C607、C608、C620、C674、C676 都换成「证据：…」，全部找到，没有交回的。证据与现查：
   - C626：门禁 code-source-discipline 的 gpu-exemptions 格，样本 `.claude/gate.d/fixtures/code-source-discipline.sh/gpu-exemptions-red`
   - C627：门禁 checker-tier-formal-verification 的 pinned-red-examples-complete、pinned-red-examples-replay 两格，样本 `…/checker-tier-formal-verification.sh/pinned-red-examples-complete-whole-row-red`
   - C601：`crates/mutations.tsv`「GPU 判器 查版本只比组合掩码低 32 位」，用例 `the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write`
   - C607：mutations.tsv 以 C607 为题的 16 行（`grep -c '^C607 ' crates/mutations.tsv` 现数 16）
   - C608：mutations.tsv「GPU 判器 每次写都当持久」「GPU 判器 走读判不变量一条都不记违例」「GPU 核对 根指针的落点不比整单元校验和」三行
   - C620：mutations.tsv「GPU 取卡 PCI 上有 NVIDIA 显示控制器而 Vulkan 枚举不出也当没卡」，用例 `an_nvidia_display_controller_on_pci_that_vulkan_does_not_enumerate_is_red`
   - C674：mutations.tsv 带「GPU 判器不判最新那条根自己的水位」的行，用例 `gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_like_the_cpu_checker`
   - C676：mutations.tsv「第 10 批 10b GPU 判器：走读内核 …只按发号次序数到种类 8」一行（恢复内核那一行成对），用例 `the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order`
   - 所有点名的 fn 名都用 `grep -rlw --include=*.rs "fn <名>" crates` 核过存在（门禁 doc-registries 的 paid-cited-tests 格要的）。
   - 注意：2026-10-11 治理记录写这几笔的变异证红被用户停掉（乙组 14 行没出结论）；这里按用户 10-11 定的「现成变异行即证据」写，没有复跑证红。
2. C169、C285、C421、C505 四行补进已还清表末尾（C548 之后），「作废：对象已删（`scripts/kb/oov-check.py` / `scripts/crash/corruption-check.py`，随本地模型腿一起删，提交 1b6f9865）」，日期 2026-10-14；简称、旧行原文取自 `git show ce1457c3`。
3. C635 去向改成「含并入自 C20（罕见布局无人跑） 的按套布局标记」。
4. C102、C182、C216、C224、C271 开头统一成「已有」「仍欠」，内容不变（C224 的仍欠在同行「要拦什么」一格已写）。
5. 数：C532 230→237（命令用 `grep -c '\|exact$' scripts/experiments/replay.sh`，237；派发写的 `grep -c exact` 会数到注释与自证行，出 256；「产物不在树里 101 行」现数仍 101）；C56 101→100；C722 三个展开数改 334 651 / 395 742 / 345 991，补余量 65 349 / 4 258 / 54 009（取自 `crash_judge_dispatch.rs` 钉值，该文件在暂存区）；C97 61→60（命令加 `[A-Z_0-9][A-Z_0-9]*`，现数 60，原写法出 62）。
6. C692 尾巴删了；另立 C852（见第五节）。
7. 另立欠账：C852–C856，见第五节。
8. C650 括注 C657 已还清（2026-10-03）；C484 行实为 C723（第 484 行），C705 括注「已作废」；C609 改「E165 钉表已整行抄（C627 2026-10-11 已还清）」；C143 前置改「曾是同一个前置」并说明 C331 已还清、C332 已作废；C370 的 C355 括号改「该笔取丁已定、2026-10-11 已还清」；C564：13→14，① 断点续跑改成「已有」（现查 `crash_amplification_pipeline_records_checks_and_reuses.rs` 有用例 `a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest`：叫停后重跑只判剩下的块），仍欠两样，补法列删了断点续跑一句。
9. C603 去向改「里程碑三「待优化」表第 39 行」；现查 milestone/03 第 39 行「自检的性能浪费」已在。
10. C618 去向补「里程碑十（真盘上的 O_DIRECT 会话）…提前到十的出口 2 与一·3」；C765、C767 去向改「里程碑十四一·1 多头」，补「克隆头 2026-10-13 起在里程碑十四」；C709 不改。

## 四、没改的行与理由

| 行 | 理由 |
|---|---|
| C605（原第 402 行，F34 要人看） | pending-main.md 没有 S03 一节，没有去向；它问的是「先定版本表进不进键」是否被已定项 17 覆盖，要对 `crash_identity.rs` 的键现查，我没查，不改 |
| C709（原第 472 行，F507 要人看） | 派发指示：前置「不挡里程碑三」与去向「里程碑三二·5」矛盾，不改，列交用户（见第七节） |
| C714（原第 475 行，F633 要人看） | pending-main S24：引的 layout/03 第 59 行现文要在改 layout/03 那一行时同改，layout/03 不在我的写范围，没改 |
| C568、C750、C572 是否挪已还清 | fixes 与 pending-main S26 都写「请主 agent 定」，我只改了「进代码批」「键只取起点槽」这类过时句，没挪行 |
| C572、C61、C402、C403、C420、C508、C554 里的「待提交落地」 | pending-main S19：等别的会话销账后再改；C554 的那一处已在 fixes 里判 HEAD 已有，改了，其余没动 |
| C376「收口表第 16 行」（pending-main S15） | 现查 milestone/02 第 16 行讲 C334，第 6 行有 C376 但去向列不是「要挂载内实例切换」，改指第 6 行有没有道理我判不了，没改 |
| C308「偏移列现已相同」（pending-main S11） | 现查 layout/01 只有第 200 行那张表头写「字段（偏移）」，另两张只有宽度；原句一半对不上，改成「宽度与行序」要主 agent 判 |

## 五、新立欠账（C852–C856）与简称

派发写「从 C847 起」；现查 C847–C851 已被 c834 写回规格占用（该规格第 8 条在 C846 行后插 C847–C851），故顺排 C852 起。新行插在「**状态**」段之前（C846 行之后），不动 C846 行，与那份规格互不挤占：它的旧串是 C846 整行，仍恰好命中一次，替换后它的行排在我的行前面。

| 编号 | 简称 | 要拦什么（一句） | 去向 |
|---|---|---|---|
| C852 | GPU 恢复内核每盘落点下限写死 10 | `crash_judge_common.wgsl` 仍写 `first_device_count < 10u`，CPU 一侧已按几何现算（默认几何每盘 14、1024 MiB 环那套 16）；主 agent 写的题面「盘数写死 10」不准（写死的是每盘落点下限，不是盘数），我改了题面 | 未排进里程碑 |
| C853 | 并行提交的共享状态分法装置没追平退役树 | `scripts/experiments/e166_parallel_commit_shared_state_partition.rs` 水位 65548 / 12 个号，`crates/singlefs-format/src/lib.rs` 现 65549 / 13 个号；E166 页读数是当时的，不改 | 未排进里程碑 |
| C854 | 分项引用状态的扫描范围漏了实验源码 | 路径在 `.claude/gate.d/lib-item-ref-status.py` 第 76 行（派发写的 `scripts/kb/lib-item-ref-status.py` 第 75–76 行不存在，现查是 gate.d 下，范围只剩第 76 行）；relabel-item.py 与分项引用状态格都不看 `scripts/experiments/` | 未排进里程碑，归 tooling-writer |
| C855 | 过期候选脚本的载体模式写死 | `scripts/kb/stale-candidates.py` 的 `CARRIER_PATTERNS` 不读 `.claude/kb-directories` | 未排进里程碑，归 tooling-writer |
| C856 | 删除写出字节的段串还标推的 | `.claude/kb/layout/03-third-txn.md` 里 7 处「推的，待干跑核」（`grep -c` 现数），E172 第 3 次跑之后回看 | E172 第 3 次跑之后回看 |

没写去向的里程碑：五笔都没有现成的里程碑落点，我没编，写的是「未排进里程碑，等主 agent 排」。

要主 agent 跑 `relabel-item.py` 的简称清单（这一轮我没跑）：

- C774：简称「E142 装置没追平退役树」现在只剩第 26 次登记，建议改「E142 装置追平退役树之后剩第 26 次登记」（pending-main S10）。其余没有。
- 另提醒：C564 简称「另外 6 条崩溃点重放全量没人跑」与现登记 14 条、仍欠两样是否仍对，要主 agent 对着 `stage-inputs.tsv` 核（fixes F942 也这样写）。

## 六、要主 agent 判的点

1. C793 现状：真跑的自证已加（`crash-point-replay-shard-run.sh` 文件头 `real-two-hosts-run`，`selftest-registry.tsv` 登记 heavy-if-set），两套假用例的 `CRASH_LIBRARY_RECORDED` 格也在；我只改了「层 0」旧词与「真跑的自证仍欠」一句（改成「后来加了，仍欠设了变量真跑一次，是否跑过没核」）。这一笔能不能挪已还清、归谁跑，主 agent 判。
2. C789：fixes F908 要核「池级 checker 那一遍改走两段」落没落地；现查 `crash_state_enumeration.rs` 的 `pool_checker_of_the_images_on_the_route` 已在观察者判段里按选路判（配了 GPU 上卡），按此写了。
3. C750：我写成「随代码欠账批甲簇的 B3 已改成按单元占的每一槽」，日期没写（fixes 写 2026-10-14，我没核到）。
4. 登记表头注文字 `.claude/gate.d/stage-inputs.tsv` 第 52 行「不留进度文件（没有断点续跑）」与 `.claude/gate.d/crash-point-replay-items.tsv` 第 2 行头注「13 条 crash-case:」仍是旧说法（F910、pending-main S34），不是我的写范围，列给主 agent。
5. C842 开头写「已还这一半」：`doc-registries.sh` table-shape 格的正则放行「一半 / 这一半 / 大半」（该格注释与代码 `已还(?!一半|这一半|大半)`），读代码判不会红；没跑这一格，主 agent 跑。
6. 已还清表八笔占位句换成现成变异行为证据，没复跑变异（2026-10-11 治理记录写乙组变异证红被用户停掉）；用户 10-11 定「现成变异行即证据」，我照此写。

## 七、交用户的清单

- C709（`.claude/kb/checks-owed.md` 现第 472 行）：前置写「不挡里程碑三」，去向写「里程碑三二·5」，`milestone/03-third-txn.md` 没点名它，到底挡不挡出口要用户定一边。
- C605：「先定版本表进不进键」是仍开着还是已被已定项 17 覆盖（要对 `crash_identity.rs` 现查）。
- 其余 S19（C566 的「欠检查清单里的条目全部未实现」，等 C61、C402、C403、C420、C508、C572 销账后再改）、S26（C568、C750、C572 是否挪已还清）是收尾提醒。

## 八、没做什么

- 没判定案对不对，没写规格外的句子（新立五行与四行作废行是主 agent 交的内容，句子是我按现查起草的）。
- 没跑 `relabel-item.py`、`doc-decisions.sh --write`、doc-lint、门禁；没碰 `decisions.md`、`decisions-history.md`、实验页、里程碑；没提交。
- 没建副本，没起编译。

## 九、改过的文件与 sha256

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| `.claude/kb/checks-owed.md` | `b386453e188425e6784c06ef28295ed5df4452ae435237c36710b0de556d878f` | `78882f834da3251fed503f397312400355c3fe1f540de792b4239ce70613abd1` |
| `.claude/kb/decisions.md`（没改） | `c89fc61328076ec93f5e8f51bf0f99fc22cc01616dad5a7ada3bbb76e6ab57be` | `c89fc61328076ec93f5e8f51bf0f99fc22cc01616dad5a7ada3bbb76e6ab57be` |
| `.claude/kb/decisions-history.md`（没改，别的会话在改） | `c8d81ebc8c1112b1aee8ed045898fdc88ca4dc7a35e46d18e4bf19458b91a1b5` | `62570728893035c86e213b0c18c22f1071aebfaf8da6382058bf5efc68cddbc9` |

`checks-owed.md` 开工时 `git status` 是 `M `（只有暂存区改动，工作区无改动）。
草稿目录 `/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-wb-甲/`：`spec.json`、`spec.md`、`build-spec.py`、`row.py`、`ctx.py`、`name.py`（查行与登记简称的辅助脚本）、`replace-batch-output.txt`、`sha-start.txt`；没有仓副本与编译目录。
