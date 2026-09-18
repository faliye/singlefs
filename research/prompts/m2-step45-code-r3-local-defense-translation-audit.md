# 核对表：m2-step45-code-r3-local-defense 英文提示逐句核转述

机械对照，逐行列：英文项 / 原文文件:行 / 首稿缺的 / 定稿。行号均现查（`grep -n` / `awk 'NR==...'`），不从背景材料的 diff 抽取行号抄。

| 英文项（提示里出现的句子，概括标注） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Instance table validity 判据（无行 ⇒ 可选；有行 (i,Ti,Wi) 且 T ≤ Ti ⇒ 可选） | `.claude/kb/decisions/23-journal的角色与格式.md:1209` | 无 | 见提示 BACKGROUND FACTS 第 2 段 |
| F_effective 定义（per-device max 再取 min，"生效" 行） | `.claude/kb/decisions/16-发布语义.md:375` | 无 | 见提示 BACKGROUND FACTS 第 3 段 |
| 回退候选集定义（有效 ∧ txg ≥ F_生效） | `.claude/kb/decisions/16-发布语义.md:376` | 无 | 见提示 BACKGROUND FACTS 第 3 段 |
| 可再分配谓词（已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)） | `.claude/kb/decisions/16-发布语义.md:371` | 无 | 见提示 BACKGROUND FACTS 第 3 段 |
| D23 已定项 14 主句逐字（"被抛弃时间线的根离开根环之前……只隔离其中只被被抛弃根引用的槽"） | `.claude/kb/decisions/23-journal的角色与格式.md:1209` | 无（"只被…引用"限定词保留） | 见提示 BACKGROUND FACTS 第 4 段与 ITEM 2 第 2 段 |
| 2026-09-16 窄读法定案逐字（"仍被有效根引用的槽不在其内"、有效只按实例表判） | 同上 | 首稿把这句连同其上下文一起译成一句英文时，末尾误加了一段中文原文括注（原文 "仍被有效根引用的槽不在其内"），违反「提示用英文写」——已删除该中文括注，只留英译 | 见提示 BACKGROUND FACTS 第 4 段末句 |
| G5 机制（豁免集 = 候选集∪当前账未释放，隔离被抛弃根账里未释放且不在豁免集里的槽） | `crates/singlefs-core/src/mount.rs:201-208`（doc）、`:213-258`（函数体） | 首稿把函数体行区间误写成 `213-253`（从背景材料的 diff 抽取行号抄的，未现查）；现查函数真实结束于第 258 行的 `}`（`grep -n "^fn isolate_slots_referenced_only_by_abandoned_roots\|^}"`） | 定稿改为 `213-258`，见提示 ITEM 1 第 1 段 |
| G5 对窄读法措辞的收严说明（doc 注释逐字："对那句措辞的收严……违反 D23 已定项 14 的主句"） | `crates/singlefs-core/src/mount.rs:202-205` | 无（现查行号与背景材料一致，未改） | 见提示 ITEM 1 第 2 段 |
| D28 已定项 1 第九项定义（上界 = 被抛弃根已分配 − R_old 已分配） | `.claude/kb/decisions/28-挂载期承诺量.md:30` | 无 | 见提示 ITEM 1 第 3 段 |
| T1 攻方问题整句（豁免集三个否定条件） | `research/prompts/_m2-step45-code-r3-background.md:34` | 首稿把中文原句直接引在英文译文前面（"只隔离被抛弃根账里未释放……" 后接 "-- translated:"），违反「提示用英文写」——已删除中文原句，只留 "this fix, translated):" 后的英译 | 见提示 ITEM 1「Attack question」段 |
| T2 机制（读不出跳过、计数、不拒挂） | `crates/singlefs-core/src/mount.rs:241-245`（跳过逻辑）、`:99-101`（`MountOutput::abandoned_roots_unreadable` 字段的 doc 注释） | 首稿两处行号都是从背景材料的 diff 抽取行号直接抄的，未现查：跳过逻辑误写 `238-244`（现查该 for 循环体是 `241-245`）；字段 doc 注释误写 `215-217`（现查该字段与其上方两行注释在真实源文件里是 `99-101`，因为 diff 附录按自己截取的片段重新编号，与源文件实际行号不同） | 定稿分别改为 `241-245`、`99-101`，见提示 ITEM 2 第 1 段 |
| "abandoned_roots_unreadable 今天没有消费者：只有用例读它" | `research/prompts/_m2-step45-code-r3-background.md:26` | 无 | 见提示 ITEM 2 第 1 段 |
| T2 攻方问题整句（跳过、计数、不拒挂；隔离罩不到） | `research/prompts/_m2-step45-code-r3-background.md:35` | 无 | 见提示 ITEM 2「Attack question」段 |
| reclaim_floor 函数签名与 doc 注释（第一参数名 effective_floor，门槛取 max(F_生效, 环里最旧有效根)） | `crates/singlefs-core/src/mount.rs:178`（签名）、`:175-176`（doc） | 无 | 见提示 ITEM 3 第 1 段 |
| 两处调用点：重建用真实 effective_floor，抬 F 用 new_floor | `crates/singlefs-core/src/mount.rs:306`（`reclaim_floor(effective_floor, oldest_valid_root)`）、`:466`（`reclaim_floor(new_floor, oldest_valid_root)`） | 无（`grep -n "reclaim_floor("` 现查两处行号与命名参数） | 见提示 ITEM 3 第 1 段 |
| raise_rollback_floor 里「记账先动、口径归 alloc-basis」注释逐字 | `crates/singlefs-core/src/mount.rs:433-436` | 无 | 见提示 ITEM 3 第 2 段 |
| T3 攻方问题整句（扣住到落满每块盘、记账先动） | `research/prompts/_m2-step45-code-r3-background.md:36` | 无 | 见提示 ITEM 3「Attack question」段 |
