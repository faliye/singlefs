# 核对表：m2-step6-checker-r1-local-defense 英文提示逐句核转述

机械对照，逐行列：英文项 / 原文文件:行 / 首稿缺的 / 定稿。行号均现查（`grep -n` / `awk 'NR==...'`），不从背景材料的行号或诊断偏移直接抄。

| 英文项（提示里出现的句子，概括标注） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| valid_roots 函数范围（只核根槽自证校验和，不碰 I-2.1） | `crates/singlefs-checker/src/image.rs:216-227` | 首稿写成 `216-226`（闭合的 `}` 实际在 227 行，首稿数漏了一行）；现查 `awk 'NR==216,NR==227'` 确认闭合括号位置后改 | 已用 `replace-once.py` 改为 `216 to 227`，见提示「Root ring and self-certifying roots」段 |
| 实例表行 (i, Ti, Wi) 与可选判据 T ≤ Ti | `.claude/kb/decisions/23-journal的角色与格式.md:1209` | 无 | 见提示「Instance table validity」段 |
| 候选集定义主句（D23 已定项14） | `.claude/kb/decisions/23-journal的角色与格式.md:1209` | 无 | 见提示「The rollback candidate set」段第一处引用 |
| 候选集定义复述（D16 已定项1 表行「回退候选集」） | `.claude/kb/decisions/16-发布语义.md:376` | 无 | 见提示「The rollback candidate set」段第二处引用 |
| 用户定案逐字「选 1 最近四个状态……后来放弃了」 | `.claude/kb/decisions/16-发布语义.md:360-361` | 无（译文补了「root」一词以便读者判断量词所指，未改变原意，登记为受控意译） | 见提示「By user decision」段 |
| 抬 F 上限公式（min(每盘最新持久有效根, 第4新非空有效根)；不足4个取最旧） | `.claude/kb/decisions/16-发布语义.md:374` | 无（未译入同句「一次处置的目标 = min(...)」，与本轮辩护主题无关，按摘录处理，未标 verbatim） | 见提示「By user decision」段末句 |
| I-7.4 定义主句 | `.claude/kb/invariants.md:50` | 无 | 见提示「I-7.4, full decided text」段 |
| I-4.8 定义主句 + 判别力句 | `.claude/kb/invariants.md:153` | 首稿把候选集的完整定义子句原样重复译出；核对时改为「the same candidate set as I-7.4」以避免与上文重复（内容不变，仅去重，已在本表登记为受控压缩，未改代码或事实） | 见提示「I-4.8, full decided text」段 |
| I-2.1 定义主句 + 2026-09-17 状态列修订 | `.claude/kb/invariants.md:107` | 未译入状态列末尾关于 C22 必红的例句（与本轮三项辩护无关，按摘录处理） | 见提示「I-2.1, decided text」段 |
| 独立性硬约束（checker 不依赖实现 crate、不许链接实现记账与校验代码） | `.claude/kb/verification-build.md:111-112` | 无 | 见提示「Project-wide independence rule」段 |
| 共享常量模块那句（未加引号，按摘录处理，非逐字引用） | `.claude/kb/verification-build.md:112` 附近同段 | 无（未单独现查精确行号，因未标 verbatim，按同段落摘录处理） | 见提示「Project-wide independence rule」段第二句 |
| crates 里没有的三项（抹头独立判定、对象身份判重新分配、被抛弃根单元判定） | `research/prompts/_m2-step6-checker-r1-background.md:18` | 无 | 见提示「Project-wide independence rule」段引文 |
| 传统遍历规则表行（checker / 审计：必须遍历） | `.claude/rules/fs-design.md:24` | 无 | 见提示「Project-wide traversal rule」段 |
| 干净镜像三根事实（txg 1/2 暖机根、txg 3 是 A、50178 只被 txg1/2 引用） | `research/prompts/_m2-step6-checker-r1-background.md:15` | 无 | 见提示「The clean baseline pool image」段 |
| Judgements 结构体字段定义范围 | `crates/singlefs-checker/src/image.rs:44-49` | 首稿写成「struct fields at line 44」（只给了开括号那一行，未覆盖字段范围）；现查 `awk 'NR==43,NR==50'` 确认字段止于 49 行的 `}` 后改 | 已用 `replace-once.py` 改为「struct at lines 44 to 49」，见提示「The Judgements accumulator」段 |
| judge 方法整段 | `crates/singlefs-checker/src/image.rs:53-63` | 无 | 见提示「Method judge」句 |
| violation_count 方法整段 | `crates/singlefs-checker/src/image.rs:67-69` | 无 | 见提示「Method violation_count」句 |
| not_applicable 方法整段 | `crates/singlefs-checker/src/image.rs:71-77` | 无 | 见提示「Method not_applicable」句 |
| into_report 方法整段 | `crates/singlefs-checker/src/image.rs:79-98` | 无 | 见提示「Method into_report」句 |
| read_referenced_unit 文档注释 + 函数体 | `crates/singlefs-checker/src/image.rs:300-325` | 无 | 见提示「Function read_referenced_unit」段 |
| read_index_node 函数整体范围 | `crates/singlefs-checker/src/walk.rs:143-194` | 无 | 见提示「Function read_index_node」段 |
| read_index_node 内 read_referenced_unit 调用与读失败分支 | `crates/singlefs-checker/src/walk.rs:158-164` | 无 | 见提示「Function read_index_node」段第二句 |
| read_index_node 内 visited_units 去重分支 | `crates/singlefs-checker/src/walk.rs:169-174` | 无 | 见提示「Function read_index_node」段第三句 |
| Walk 结构体定义范围 | `crates/singlefs-checker/src/walk.rs:47-65` | 首稿写成 `47-77`（把结构体定义的范围与紧接着的 `impl Walk` 块开头几行混在一起数，实际结构体在 65 行闭合、67 行才是 `impl Walk<'_> {`）；现查 `grep -n "struct Walk"` 与 `awk 'NR==47,NR==67'` 确认闭合括号在 65 行后改 | 已用 `replace-once.py` 改为「lines 47 to 65」，见提示「The Walk struct」段 |
| check_pool_image 函数整体范围 | `crates/singlefs-checker/src/walk.rs:690-904` | 无 | 见提示「The Walk struct」段第二句 |
| 最新根 walk_root 调用 | `crates/singlefs-checker/src/walk.rs:777` | 无 | 见提示「The Walk struct」段第二句、「The newest root's own I-4.8 cell」段 |
| 候选根循环体范围（含 older_candidates 初始化） | `crates/singlefs-checker/src/walk.rs:798-826` | 无 | 见提示「The Walk struct」「The older-candidate loop body」两段 |
| 候选根循环准入条件（index != newest_index && !abandoned && !below_floor） | `crates/singlefs-checker/src/walk.rs:804` | 首稿写成「line 799 to 802」（把 for 循环起始行与 abandoned/below_floor 两个局部变量的计算行误当成条件本身所在行，未现查条件表达式真正落在哪一行）；现查 `grep -n "if index != newest_index"` 确认条件在 804 行后改 | 已用 `replace-once.py` 改为「line 804」，见提示 ITEM 2 攻方问题段与「The newest root's own I-4.8 cell」段 |
| mismatches_before / failures_before 取值 | `crates/singlefs-checker/src/walk.rs:806-807` | 无 | 见提示「The older-candidate loop body」段 |
| 候选根 walk_root 调用 | `crates/singlefs-checker/src/walk.rs:808` | 无 | 见提示「The older-candidate loop body」段 |
| walked_into_reused_or_erased_unit 差值判据 | `crates/singlefs-checker/src/walk.rs:811-813` | 无 | 见提示「The older-candidate loop body」段、ITEM 1 攻方问题段 |
| 候选根判定前的机制注释（校验和对不上/头用不了即复用或抹头） | `crates/singlefs-checker/src/walk.rs:809-810` | 无 | 见提示「The older-candidate loop body」段引文、ITEM 1 攻方问题段引文 |
| I-7.4 候选根判定调用 | `crates/singlefs-checker/src/walk.rs:815-820` | 无 | 见提示「The older-candidate loop body」段 |
| I-4.8 候选根判定调用 | `crates/singlefs-checker/src/walk.rs:821-824` | 无 | 见提示「The older-candidate loop body」段 |
| I-7.4 违例消息文本 | `crates/singlefs-checker/src/walk.rs:818` | 无 | 见提示「The older-candidate loop body」段引文、ITEM 1 攻方问题段引文 |
| older_candidates 自增 | `crates/singlefs-checker/src/walk.rs:805` | 无 | 见提示「The older-candidate loop body」段 |
| older_candidates == 0 时的 not_applicable 调用与理由串 | `crates/singlefs-checker/src/walk.rs:827-829`（理由串本身在 829 行） | 无 | 见提示「The older-candidate loop body」段末句、ITEM 3 攻方问题段引文 |
| 最新根 I-4.8 单格判据（newest_failures.is_empty() && violation_count(I-2.1) == 0） | `crates/singlefs-checker/src/walk.rs:776-785` | 无 | 见提示「The newest root's own I-4.8 cell」段 |
| I-7.2 复用 newest_failures.is_empty() | `crates/singlefs-checker/src/walk.rs:832` | 无 | 见提示「The newest root's own I-4.8 cell」段 |
| 「先走最新根」注释 | `crates/singlefs-checker/src/walk.rs:776` | 无 | 见提示「The newest root's own I-4.8 cell」段末句 |
| 干净镜像逐条判 Holds 的断言循环 | `crates/singlefs-harness/tests/checker_known_bad_images.rs:544-549` | 无 | 见提示 ITEM 3 攻方问题段 |
| IMPLEMENTED_INVARIANTS 共 26 条、known_bad_images 逐条对应 | `crates/singlefs-checker/src/image.rs:36-40`（清单本身）；`crates/singlefs-harness/tests/checker_known_bad_images.rs:551-561`（assert_eq!(targets, listed) 那段，本轮未在提示中单独引用行号，只用「twenty six」这个数） | 无（本轮未展开引用该段代码，只借用了已在别处核实过的「26」这个数） | 见提示 ITEM 2 与 ITEM 3 攻方问题段「twenty six」措辞 |

## 修正过程说明

首稿由主 agent（本地辩方腿）依据背景材料第一节「实现今天的样子」与本轮 diff 直接撰写，行号在撰写前已逐条用 `grep -n` / `awk 'NR==A,NR==B'` 现查一遍；写完之后又做了一轮独立复核（同样现查），发现四处行号偏差（`valid_roots` 少算一行、`Judgements` 结构体只给了开括号行未给字段范围、`Walk` 结构体误把 `impl` 块开头几行并入、候选根循环准入条件所在行数错），均用 `research/scripts/replace-once.py` 定点改正，改完逐次回读确认命中。上表「首稿缺的」列即这四处以及若干处内容层面的受控压缩（去重复述、略去与本轮辩护主题无关的从句），压缩处均未改变被引用条款的核心断言。

## 历史版本

（无）
