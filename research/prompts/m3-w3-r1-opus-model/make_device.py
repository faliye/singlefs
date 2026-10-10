#!/usr/bin/env python3
"""从 E175 装置源码原样（device-source.rs）生成攻方腿的原型装置 w3_attack_candidates.rs。

每处替换旧串必须恰好命中一次，否则退出 3。用法：make_device.py <device-source.rs> <输出.rs>
加进去的是「W3 候选的影子记录」：按每条根的父版本链算出各候选在盘上本该留下的发号历史，再在每一步对最新那条根判红。
影子记录不在盘上，只是把候选的写规则套到真代码走出的根链上；判的时候只读最新那条根的影子与它的实例表（单镜像）。
"""
import sys

src_path, out_path = sys.argv[1], sys.argv[2]
text = open(src_path, encoding="utf-8").read()

def once(old, new):
    global text
    n = text.count(old)
    if n != 1:
        sys.stderr.write(f"旧串命中 {n} 次：{old[:80]!r}\n")
        sys.exit(3)
    text = text.replace(old, new)

# 1. ReissueEvent 带 W3 判定
once("""    /// 同一步 S2 对 I-9.29 的判定（四栏之一）。
    one_birth_per_number_at_this_step: VerdictClass,
}""", """    /// 同一步 S2 对 I-9.29 的判定（四栏之一）。
    one_birth_per_number_at_this_step: VerdictClass,
    /// 同一步 W3 各候选对最新那条根的判定。
    w3: W3Verdicts,
}""")
once("""                            one_birth_per_number_at_this_step: VerdictClass::Missing,
                        });""", """                            one_birth_per_number_at_this_step: VerdictClass::Missing,
                            w3: W3Verdicts::default(),
                        });""")
# 2. HistoryAccount 带 W3 账
once("""    /// 每次做成的回退：目标那一版的身份与 S1 读到的水位（D 族 A5 k ≥ 2 那几格按它钉）。
    rollback_targets: Vec<(RootIdentity, Option<u64>)>,
}""", """    /// 每次做成的回退：目标那一版的身份与 S1 读到的水位（D 族 A5 k ≥ 2 那几格按它钉）。
    rollback_targets: Vec<(RootIdentity, Option<u64>)>,
    /// W3 攻方腿：各候选的影子记录与计数。
    w3: W3Account,
}""")
once("""            current_trajectory: Vec::new(),
            rollback_targets: Vec::new(),
        }""", """            current_trajectory: Vec::new(),
            rollback_targets: Vec::new(),
            w3: W3Account::default(),
        }""")
# 3. observe 里在 checker 判过之后调 W3
once("""        self.judge_with_the_checker(observation.image, &position, &mut events_of_this_step);
""", """        self.judge_with_the_checker(observation.image, &position, &mut events_of_this_step);
        let operation_text = observation.operation.map_or_else(|| "start".to_string(), operation_code);
        let rolled_back = matches!(applied_effect, Some(AppliedEffect::RolledBack { .. }));
        let i9_29_now = self.step_verdicts.last().map_or(VerdictClass::Missing, |verdicts| verdicts.one_birth_per_number);
        let i9_6_now = self.step_verdicts.last().map_or(VerdictClass::Missing, |verdicts| verdicts.watermark_above_both_highest_numbers);
        self.w3.step(&read.readable, table, rolled_back, &position, &operation_text, i9_29_now, i9_6_now, &mut events_of_this_step);
""")
# 4. 结果行与事件行
once("""first_violation={} first_s3_disagreement={} first_q11={} operations=[{}]{} line_end=ok\",""",
     """first_violation={} first_s3_disagreement={} first_q11={} {} operations=[{}]{} line_end=ok\",""")
once("""            self.account.first_q11_detail.as_deref().unwrap_or("none"),
            operations.join(","),""", """            self.account.first_q11_detail.as_deref().unwrap_or("none"),
            self.account.w3.render(),
            operations.join(","),""")
once("""                    "E7RESULT name=event family={} arm={} index={} kind={} position={} inode={} new_birth={} older_births=[{}] q9={:?} i9_29_at_step={}",""",
     """                    "E7RESULT name=event family={} arm={} index={} kind={} position={} inode={} new_birth={} older_births=[{}] q9={:?} i9_29_at_step={} {}",""")
once("""                    event.one_birth_per_number_at_this_step.label(),
                )
            })
            .collect()
    }""", """                    event.one_birth_per_number_at_this_step.label(),
                    event.w3.render(),
                )
            })
            .chain(self.account.w3.trace.iter().map(|line| {
                format!("E7W3TRACE family={} arm={} index={} {line}", self.family.label(), arm.label(), self.index)
            }))
            .collect()
    }""")
# 5. 每段历史开头决定要不要逐步打印
once("""    let started = Instant::now();
    let mut account = HistoryAccount::new(breakage);""", """    let started = Instant::now();
    let mut account = HistoryAccount::new(breakage);
    account.w3.trace_enabled = std::env::var("W3_TRACE_INDICES").ok().is_some_and(|list| {
        list.split(',').any(|index| index.trim().parse::<u64>().ok() == Some(item.index))
    });""")
# 6. 定制族：W3 攻方腿自己造的历史（K1 同实例回退、K3 等），并入 history_item
once("""        Family::Geometry(GeometryPoint::LongerBroad) => {
            random_history_item(family, index, &GenerationWeights::BROAD)
        }
    }""", """        Family::Geometry(GeometryPoint::LongerBroad) => {
            random_history_item(family, index, &GenerationWeights::BROAD)
        }
    }
}

/// 原型自己的族：前缀 `[Unlink, CreateInode, CreateInode]`、中段取 [`w3_middle_alphabet`] 上长 0..=3 的序列、后缀 `[CreateInode, CreateInode]`。
/// 与 F3a 同形，字母表把抬 F 的步数放开（1..=12，由用户定的那一步），并加挂载。只给原型用，不是 E175 登记的族。
fn w3_middle_alphabet() -> Vec<HistoryOperation> {
    let mut alphabet = vec![HistoryOperation::CreateInode, ring_root(1), ring_root(2), ring_root(3)];
    for steps in 1..=12 {
        alphabet.push(HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: steps }));
    }
    alphabet.push(HistoryOperation::CloseAndMountWritable);
    alphabet
}

fn w3_crafted_item(index: u64) -> HistoryItem {
    let alphabet = w3_middle_alphabet();
    let size = u64::try_from(alphabet.len()).expect("字母表很小");
    let mut remaining = index;
    let mut length = 0_u32;
    loop {
        let count = size.pow(length);
        if remaining < count {
            break;
        }
        remaining -= count;
        length += 1;
    }
    let mut letters = Vec::new();
    for position in (0..length).rev() {
        let place = size.pow(position);
        letters.push(remaining / place);
        remaining %= place;
    }
    let mut operations = vec![HistoryOperation::Unlink, HistoryOperation::CreateInode, HistoryOperation::CreateInode];
    for letter in &letters {
        operations.push(alphabet[usize::try_from(*letter).expect("小")]);
    }
    operations.push(HistoryOperation::CreateInode);
    operations.push(HistoryOperation::CreateInode);
    let names: Vec<String> = letters.iter().map(u64::to_string).collect();
    HistoryItem {
        family: Family::Directed,
        index,
        label: format!("w3middle[{}]", names.join(",")),
        history: GeneratedHistory {
            seed: HistorySeed(SEED_BASE_COPIED_FROM_THE_HISTORY_MODULE),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations,
        },
    }
}

fn w3_crafted_count(longest: u32) -> u64 {
    let size = u64::try_from(w3_middle_alphabet().len()).expect("小");
    (0..=longest).map(|length| size.pow(length)).sum()
}

#[test]
#[ignore = "原型：跑 W3 定制族的一批序号（W3_INDICES 逗号分隔），结果追加进 W3_RESULT_FILE；臂由 E175_ARM 给"]
fn w3_run_crafted_indices_into_the_result_file() {
    let arm = Arm::from_environment();
    let path = std::path::PathBuf::from(required_environment("W3_RESULT_FILE"));
    let worker_threads = usize::try_from(positive_number_from_environment("E175_WORKER_THREADS")).expect("线程数");
    let total = w3_crafted_count(3);
    let wanted: Vec<u64> = required_environment("W3_INDICES")
        .split(',')
        .map(|index| index.trim().parse().expect("W3_INDICES 是逗号分隔的序号"))
        .collect();
    assert!(wanted.iter().all(|index| *index < total), "序号越过 {total}");
    let items: Vec<HistoryItem> = wanted.iter().map(|index| w3_crafted_item(*index)).collect();
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&path).expect("开结果文件");
    let started = Instant::now();
    evaluate_items(&items, worker_threads, None, Breakage::NONE, |evaluation| {
        let mut block = String::new();
        for line in evaluation.event_lines(arm) {
            block.push_str(&line.replacen("family=D ", "family=W3c ", 1));
            block.push('\\n');
        }
        block.push_str(&evaluation.history_line(arm).replacen("family=D ", "family=W3c ", 1));
        block.push('\\n');
        file.write_all(block.as_bytes()).expect("写结果文件");
        file.flush().expect("刷");
    });
    println!("W3 crafted arm={} evaluated={} total_registered={total} elapsed_ns={}", arm.label(), items.len(), started.elapsed().as_nanos());
}

#[test]
fn w3_crafted_family_has_the_expected_size() {
    assert_eq!(w3_middle_alphabet().len(), 17);
    assert_eq!(w3_crafted_count(3), 1 + 17 + 289 + 4913);
    assert_eq!(w3_crafted_item(0).history.operations.len(), 5);
}

#[test]
fn w3_shadow_records_judge_a_hand_built_regression() {
    // 自证：手搭三条根 A(wm 5) → B(wm 7) → 回退根 C(wm 5，V1 那样取目标的)，C 的影子：C2c 峰值 7 > 5 红、C4 谱系 7 > 5 红、
    // C2b 同一实例那一行被 5 覆盖、不红；C3 号 6 在 B 有出生代 9、C 里没有，号最大 6 ≥ 5 红。
    let facts = |txg: u64, wm: u64, births: Vec<(u64, u64)>| RootFacts {
        identity: (1, txg),
        rollback_floor: 0,
        version: VersionFacts::WithTrees { watermark: wm, inode_births: births },
    };
    let mut account = W3Account::default();
    let mut events: Vec<ReissueEvent> = Vec::new();
    let a = facts(4, 5, vec![(1, 3), (4, 4)]);
    let b = facts(9, 7, vec![(1, 3), (4, 4), (5, 8), (6, 9)]);
    account.step(&[a.clone()], None, false, "op0", "x", VerdictClass::Holds, VerdictClass::Holds, &mut events);
    account.step(&[a.clone(), b.clone()], None, false, "op1", "x", VerdictClass::Holds, VerdictClass::Holds, &mut events);
    let c = facts(10, 5, vec![(1, 3), (4, 4)]);
    let verdicts = account.step(&[a, b, c], None, true, "op2", "rollback", VerdictClass::Holds, VerdictClass::Holds, &mut events);
    assert!(!verdicts.c1 && verdicts.c2a && !verdicts.c2b && verdicts.c2c && verdicts.c2d && verdicts.c3 && verdicts.c4, "{verdicts:?}");
""")
import os
module = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "w3_module.rs"), encoding="utf-8").read()
open(out_path, "w", encoding="utf-8").write(text + "\n" + module)
