//! K3 攻 I-3.5 出现规则在合法历史上误红：放开用户动作的随机历史（不注读故障）。一段历史 = 若干「命」：每一命一次可写挂载（新实例）
//! 加一串动作（覆盖写 O、空发布 E、准入抬 F、回退到随机候选 R），收尾是崩溃（层 0 的合法截点：前 k 段全落 + 第 k 段任一子集）、
//! 正常卸载 U 或直接退出；崩溃之后的盘面是下一命的起点。每个动作做完（盘面上那一刻全落）与每一命收尾的盘面跑池级 checker，
//! 记 I-3.5 与别的不变量的违例。崩溃截点是历史的一部分（下一命从哪里起），不是这一段的崩溃状态全量，这里不判截点之外的崩溃状态。
//! 用法：appearance_fuzz <起始种子> <S> <段数>
use std::collections::BTreeMap;

use e2_attack_proto::{build_plain, content, geometry, parameters, publish_empty, recording_of, rollback_targets, WRITE_TIME};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish, unmount, RollbackTarget, ShadowLedger};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 { if n == 0 { 0 } else { self.next() % n } }
    fn chance(&mut self, percent: u64) -> bool { self.below(100) < percent }
}

thread_local! {
    /// I-3.5 在判过的盘面上各报了什么：[成立, 不适用, 违例]；另记候选集（攻方按条款算的）相邻 txg 有洞的盘面数与其中 I-3.5 成立的数。
    static I35: std::cell::RefCell<[u64; 5]> = const { std::cell::RefCell::new([0; 5]) };
}

fn violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
    let verdicts = singlefs_checker::walk::check_pool_image(image);
    let mut txgs: Vec<u64> = rollback_targets(image).into_iter().map(|(_, t)| t).collect();
    txgs.sort_unstable();
    let has_hole = txgs.windows(2).any(|w| w[1] > w[0] + 1);
    for (name, verdict) in &verdicts {
        if *name == "I-3.5" {
            I35.with(|c| {
                let mut c = c.borrow_mut();
                let index = match verdict {
                    singlefs_checker::image::InvariantVerdict::Holds => 0,
                    singlefs_checker::image::InvariantVerdict::NotApplicable(_) => 1,
                    singlefs_checker::image::InvariantVerdict::Violated(_) => 2,
                };
                c[index] += 1;
                if has_hole {
                    c[3] += 1;
                    if index == 0 { c[4] += 1; }
                }
            });
        }
    }
    verdicts
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(text) => Some((name, text.chars().take(400).collect())),
            _ => None,
        })
        .collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed0: u64 = a[1].parse().expect("种子");
    let s: u64 = a[2].parse().expect("S");
    let count: u64 = a[3].parse().expect("段数");
    let acts: Vec<u64> = std::env::var("FUZZ_P_ACTS").ok().map_or(vec![50, 62, 74], |v| v.split(':').map(|x| x.parse().expect("百分比")).collect());
    let max_actions: u64 = std::env::var("FUZZ_MAX_ACTIONS").ok().and_then(|v| v.parse().ok()).unwrap_or(3 * s + 7);
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    for seed in seed0..seed0 + count {
        let mut rng = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ (s << 32) ^ 0x35);
        let start_txg = 1 + rng.below(3 * s + 6);
        let mut image = build_plain(s, start_txg);
        let mut log = format!("start{start_txg}");
        let mut reds: BTreeMap<&'static str, (u64, String)> = BTreeMap::new();
        let mut note = |image: &MemoryPool, at: &str, reds: &mut BTreeMap<&'static str, (u64, String)>| {
            for (name, text) in violations(image) {
                let entry = reds.entry(name).or_insert((0, format!("{at}: {text}")));
                entry.0 += 1;
            }
        };
        let mut states = 0u64;
        let mut aborted = String::new();
        let lives = 2 + rng.below(4);
        for life in 0..lives {
            let stream = SharedStream::retaining_contents();
            let mut devices = recording_of(&image, &stream);
            let mounted = match mount_writable(&parameters, &mut devices) {
                Ok(m) => m,
                Err(e) => { aborted = format!("life{life}:mount_err:{}", format!("{e:?}").chars().take(100).collect::<String>()); break; }
            };
            let instance: InstanceGeneration = mounted.output.instance;
            log.push_str(&format!(" |m{}@{}", instance.0, mounted.output.row_publish.root().checkpoint_txg.0));
            let mut allocator = mounted.allocator.clone();
            let Some(mut current) = mounted.current.clone().into_file_version() else { aborted = format!("life{life}:no_file"); break; };
            let snap = |stream: &SharedStream| { let mut i = image.clone(); i.apply(&stream.retained_operations()); i };
            let crash_in_mount = rng.chance(15);
            let actions = if crash_in_mount { 0 } else { rng.below(max_actions) };
            for step in 0..actions {
                let roll = rng.below(100);
                if roll < acts[0] {
                    let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                    let salt = seed * 1000 + life * 100 + step;
                    match publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + step }, instance) {
                        Ok(o) => { current = o; log.push('O'); }
                        Err(_) => log.push('o'),
                    }
                } else if roll < acts[1] {
                    let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                    match publish_empty(&mut pool, &mut allocator, &current, instance) {
                        Ok(o) => { current = o; log.push('E'); }
                        Err(_) => log.push('e'),
                    }
                } else if roll < acts[2] {
                    match raise_rollback_floor_to_the_admission_ceiling(&parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On) {
                        Ok(_) => log.push('F'),
                        Err(_) => log.push('f'),
                    }
                } else {
                    let targets = rollback_targets(&devices);
                    if targets.is_empty() { log.push('_'); continue; }
                    let (ti, tt) = targets[usize::try_from(rng.below(targets.len() as u64)).expect("小")];
                    let target_inside = e2_attack_proto::inside_the_window_of_the_newest(&devices, tt);
                    if !target_inside { println!("OUTSIDE_TARGET seed={seed} life={life} target=({ti},{tt}) current={}", current.root.checkpoint_txg.0); }
                    match roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator, &mut current, RollbackTarget { instance: InstanceGeneration(ti), checkpoint_txg: CheckpointTxg(tt) }) {
                        Ok(_) => {
                            log.push_str(&format!("R{tt}"));
                            if !target_inside { println!("ACCEPTED_OUTSIDE seed={seed} life={life} target=({ti},{tt})"); }
                        }
                        Err(e) => {
                            log.push_str(&format!("r{tt}"));
                            let text = format!("{e:?}");
                            let reason: String = text.chars().take(240).collect();
                            println!("REFUSED seed={seed} life={life} target=({ti},{tt}) current={} reason={reason}", current.root.checkpoint_txg.0);
                        }
                    }
                }
                states += 1;
                note(&snap(&stream), &format!("life{life}/step{step}/txg{}", current.root.checkpoint_txg.0), &mut reds);
            }
            let end = rng.below(100);
            let crash = crash_in_mount || end < 55;
            if !crash && end < 80 {
                let mut version = PoolVersion::WithFile(current.clone());
                match stream.record_entry(singlefs_harness::RecordedPublishEntry::Unmount, || unmount(&parameters, &mut devices, &mut allocator, &mut version, ShadowLedger::On)) {
                    Ok(_) => log.push('U'),
                    Err(_) => log.push('u'),
                }
            }
            drop(devices);
            let operations = stream.retained_operations();
            let (writes, segments, _) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries(&operations, &stream.entry_spans(), &geo);
            let mut next = image.clone();
            if crash && !segments.is_empty() {
                let k = usize::try_from(rng.below(segments.len() as u64 + 1)).expect("小");
                for seg in &segments[..k] { for i in seg { next.apply_writes(std::slice::from_ref(&writes[*i])); } }
                if k < segments.len() { for i in &segments[k] { if rng.chance(50) { next.apply_writes(std::slice::from_ref(&writes[*i])); } } }
                log.push_str(&format!("C{k}/{}", segments.len()));
            } else {
                next.apply_writes(&writes);
            }
            states += 1;
            note(&next, &format!("life{life}/end"), &mut reds);
            image = next;
        }
        let summary: Vec<String> = reds.iter().map(|(n, (c, _))| format!("{n}:{c}")).collect();
        let i35 = I35.with(|c| std::mem::take(&mut *c.borrow_mut()));
        println!("FUZZ seed={seed} s={s} lives={lives} states={states} i35_holds_na_red_holestates_holeholds={i35:?} reds={summary:?} aborted={aborted:?} log={log}");
        for (name, (_, first)) in &reds {
            println!("FIRST seed={seed} {name} {first}");
        }
    }
}
