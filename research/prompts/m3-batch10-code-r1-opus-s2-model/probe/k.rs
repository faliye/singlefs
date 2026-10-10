//! m3-batch10-code-r1 攻方第二次抽样（Opus）：MODE=k。底子是 m3-batch17-code-r1 攻方原型的 MODE=fx（选槽法、按读序号开窗、写到哪一槽那一槽就不再坏），
//! 改三处：① 读坏计划可以用 `+` 连几段（每段各自的选槽、读序号窗口、读回全 0 / 报读错、挂载交回之后还坏不坏），造「重建那一遍全坏、重读那一遍只坏一部分」；
//! ② 起点可以不经卸载（K_SOURCE=now：起点会话那一刻就是崩溃镜像）；③ 每个前缀刀除了丢盘判，还在两块盘都在的镜像上跑池级 checker，
//! I-3.1 按「记账多 / 记账少」分边（记账的已分配 > 遍历 ⇒ more），别的不变量按名字记，另记读坏的槽被这条写流写掉的位置（判「红到那一槽被写掉为止」）。
use super::*;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
struct Sub {
    filters: [SlotFilter; 2],
    from: u64,
    to: u64,
    err: bool,
    /// 读回不是全零、自证不过的乱字节（「g」）：Q3 新并进上限的那一类。
    garbage: bool,
    persist: bool,
}

fn parse_sub(text: &str, boundaries: &[u64]) -> Option<Sub> {
    let parts: Vec<&str> = text.split(':').collect();
    let (from, to) = if parts[0] == "w" {
        (0, u64::MAX)
    } else if let Some(i) = parts[0].strip_prefix('P') {
        let i: usize = i.parse().expect("遍号");
        if i + 1 >= boundaries.len() {
            return None;
        }
        (boundaries[i], boundaries[i + 1])
    } else if let Some(r) = parts[0].strip_prefix('r') {
        let (a, b) = r.split_once('-').expect("rA-B");
        (a.parse().expect("A"), if b == "end" { u64::MAX } else { b.parse().expect("B") })
    } else {
        panic!("窗口 {}", parts[0]);
    };
    let f: Vec<&str> = parts[1].split('/').collect();
    Some(Sub { filters: [SlotFilter::parse(f[0]), SlotFilter::parse(f[1])], from, to, err: parts[2] == "e", garbage: parts[2] == "g", persist: parts.get(3) == Some(&"p") })
}

struct KShared {
    counter: u64,
    subs: Vec<Sub>,
    log: Vec<(u32, usize)>,
    in_mount: bool,
    faulted_reads: u64,
    trace: Vec<String>,
}

#[derive(Clone)]
struct KEntry {
    offset: u64,
    len: u64,
    ordinal: usize,
    sel: Vec<bool>,
}

struct KDevice {
    inner: SparseBlockDevice,
    device: u32,
    ring: Vec<KEntry>,
    shared: Arc<Mutex<KShared>>,
}

impl singlefs_core::block_device::BlockDevice for KDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let length = buffer.len() as u64;
        let hits: Vec<&KEntry> = self.ring.iter().filter(|e| e.offset < offset.0 + length && offset.0 < e.offset + e.len).collect();
        if hits.is_empty() {
            return self.inner.read_at(offset, buffer);
        }
        let fault = {
            let mut s = self.shared.lock().expect("锁");
            let index = s.counter;
            s.counter += 1;
            let starts_a_pass = s.log.last().is_none_or(|last| hits[0].ordinal <= last.1);
            s.log.push((self.device, hits[0].ordinal));
            if starts_a_pass && env_str("K_TRACE", "0") == "1" {
                let trace = format!("{}", std::backtrace::Backtrace::force_capture());
                let names: BTreeSet<String> = trace
                    .lines()
                    .filter_map(|l| l.split("singlefs_core::").nth(1))
                    .filter(|l| l.starts_with("mount::") || l.starts_with("recovery::") || l.starts_with("allocator::") || l.starts_with("transaction::"))
                    .map(|l| l.split(|c: char| c == '<' || c == ' ' || c == '{').next().unwrap_or("").trim_end_matches("::").to_string())
                    .filter(|n| !n.ends_with("::read") && !n.contains("read_root_ring_slot") && !n.starts_with("mount::mount_writable") && !n.contains("ReadStageCache") && n != "mount::establish_instance")
                    .collect();
                let line = format!("pass_start={index} dev={} {}", self.device, names.into_iter().collect::<Vec<_>>().join(","));
                s.trace.push(line);
            }
            let mut fault = None;
            for (i, sub) in s.subs.iter().enumerate() {
                if (s.in_mount || sub.persist) && index >= sub.from && index < sub.to && hits.iter().any(|e| e.sel[i]) {
                    fault = Some((i, sub.err, sub.garbage));
                    break;
                }
            }
            if fault.is_some() {
                s.faulted_reads += 1;
            }
            fault
        };
        if let Some((_, true, _)) = fault {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(std::io::Error::other("注入的读错")));
        }
        self.inner.read_at(offset, buffer)?;
        if let Some((i, false, garbage)) = fault {
            for e in hits.iter().filter(|e| e.sel[i]) {
                let a = e.offset.saturating_sub(offset.0) as usize;
                let b = ((e.offset + e.len - offset.0).min(length)) as usize;
                buffer[a..b].fill(if garbage { 0xA5 } else { 0 });
            }
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: singlefs_core::block_device::WriteDurability) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let length = bytes.len() as u64;
        for e in self.ring.iter_mut() {
            if e.offset < offset.0 + length && offset.0 < e.offset + e.len {
                e.sel.iter_mut().for_each(|s| *s = false);
            }
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        for e in self.ring.iter_mut() {
            if e.offset < offset.0 + length && offset.0 < e.offset + e.len {
                e.sel.iter_mut().for_each(|s| *s = false);
            }
        }
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

impl DeviceOverASparseImage for RecordingBlockDevice<KDevice> {
    fn sparse_image(&self) -> &SparseDevice {
        &self.wrapped_device().inner.image
    }
}

struct KRun {
    run: Result<R2Run, String>,
    partial: Option<R2Run>,
    log_in_mount: Vec<(u32, usize)>,
    faulted_reads: u64,
    trace: Vec<String>,
    faulted_entries: Vec<(u32, u64, u64)>,
}

/// 两块盘都在、按读坏计划（几段）再可写挂载，做成之后覆盖写 `after_j` 次（窄盘族在没有文件版本时改成建 inode）；挂载与覆盖写录在同一条写流里。
fn k_remount(crashed: &MemoryPool, start: &Start, subs: &[Sub], ring: &BTreeMap<u32, Vec<RingEntry>>, after_j: u64) -> KRun {
    k_remount_clearing(crashed, start, subs, ring, after_j, &BTreeSet::new())
}

/// 同 [`k_remount`]，`cleared` 里的槽（盘, 偏移）在前几次挂载里已被写掉，这一次不再坏（跨挂载记「写到哪一槽那一槽就不再坏」）。
fn k_remount_clearing(crashed: &MemoryPool, start: &Start, subs: &[Sub], ring: &BTreeMap<u32, Vec<RingEntry>>, after_j: u64, cleared: &BTreeSet<(u32, u64)>) -> KRun {
    let stream = SharedStream::retaining_contents();
    let shared = Arc::new(Mutex::new(KShared { counter: 0, subs: subs.to_vec(), log: Vec::new(), in_mount: true, faulted_reads: 0, trace: Vec::new() }));
    let mut faulted_entries = Vec::new();
    let devices: Vec<(DeviceIdentity, RecordingBlockDevice<KDevice>)> = plain_devices_on(crashed)
        .into_iter()
        .map(|(id, d)| {
            let base = ring.get(&id.0).cloned().unwrap_or_default();
            let per_sub: Vec<Vec<RingEntry>> = subs.iter().map(|s| select(&base, s.filters[id.0 as usize])).collect();
            let entries: Vec<KEntry> = base
                .iter()
                .enumerate()
                .map(|(j, e)| KEntry { offset: e.offset, len: e.len, ordinal: e.ordinal, sel: per_sub.iter().map(|v| v[j].selected && !cleared.contains(&(id.0, e.offset))).collect() })
                .collect();
            for e in &entries {
                if e.sel.iter().any(|s| *s) {
                    faulted_entries.push((id.0, e.offset, e.len));
                }
            }
            (id, RecordingBlockDevice::with_shared_stream(id, KDevice { inner: d, device: id.0, ring: entries, shared: shared.clone() }, stream.clone()))
        })
        .collect();
    let mut pool = PoolUnderTest {
        parameters: start.pool.parameters.clone(),
        device_bytes: start.pool.device_bytes,
        devices,
        session: None,
        write_time_seconds: start.pool.write_time_seconds,
        content_of_the_current_version: start.pool.content_of_the_current_version.clone(),
        content_of_each_root: start.pool.content_of_each_root.clone(),
    };
    let mounted = if start.label.starts_with("narrow") {
        mount_writable_with_test_only_switches_and_deletion_switches(
            &pool.parameters, &mut pool.devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On,
            deletion_switches_with_batches_of_one(),
        )
    } else {
        mount_writable_with_space_admission(&pool.parameters, &mut pool.devices, SpaceAdmission::JudgedByTheFormula)
    };
    let (log_in_mount, faulted_reads, trace) = {
        let mut s = shared.lock().expect("锁");
        s.in_mount = false;
        (s.log.clone(), s.faulted_reads, s.trace.clone())
    };
    let mounted = match mounted {
        Ok(m) => m,
        Err(e) => {
            let (writes, segments, _) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&stream.retained_operations(), &start.geometry);
            let text = format!("writes_before_err={} {}", writes.len(), format!("{e:?}").chars().take(220).collect::<String>());
            let mount_writes = writes.len();
            let partial = (!writes.is_empty()).then(|| R2Run { writes, segments, summary: format!("REFUSED {text}"), acceptable: BTreeSet::new(), versions: pool.published_versions(), mount_writes });
            return KRun { run: Err(text), partial, log_in_mount, faulted_reads, trace, faulted_entries };
        }
    };
    let (output, session) = MountedSession::of_the_mount(mounted);
    if let Some(content) = pool.content_of_each_root.get(&root_key(&output.effective_root)) {
        pool.content_of_the_current_version = content.clone();
    }
    let roots: Vec<singlefs_core::root_record::RootRecord> = std::iter::once(&output.row_publish).chain(&output.warm_up_publishes).map(|version| *version.root()).collect();
    for root in &roots {
        pool.note_root_of_the_current_content(root);
    }
    pool.note_floor_raises(output.space_admission.floor_raises());
    let rereads = format!("ring:{:?}/table:{:?}", output.rereads.root_ring_of_the_rebuilt_allocator, output.rereads.instance_table_of_the_newest_root);
    let mut summary = format!(
        "ok chosen={} effective={} floor_in_row={} row_txg={} warm_up={} floor_raises={} rereads={rereads}",
        output.chosen_root.checkpoint_txg.0,
        output.effective_root.checkpoint_txg.0,
        output.row_publish.root().rollback_floor.0,
        output.row_publish.root().checkpoint_txg.0,
        output.warm_up_publishes.len(),
        output.space_admission.floor_raises().len(),
    );
    pool.session = Some(session);
    let mount_writes = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&stream.retained_operations(), &start.geometry).0.len();
    let mut acceptable = BTreeSet::new();
    if let Ok((_, hash)) = read_everything(&pool.image()) {
        acceptable.insert(hash);
    }
    let mut done = 0;
    let mut pattern = String::new();
    let mut refused_first: Option<u64> = None;
    for i in 0..after_j {
        let outcome = match pool.overwrite(env_u64("K_OVERWRITE_BYTES", OVERWRITE_BYTES as u64) as usize) {
            Err(singlefs_core::mounted_session::UserChangeRefused::NoFileVersionToChange { .. }) => pool.create_one_inode().map(|_| ()),
            other => other.map(|_| ()),
        };
        match outcome {
            Ok(_) => {
                done += 1;
                if let Ok((_, hash)) = read_everything(&pool.image()) {
                    acceptable.insert(hash);
                }
            }
            Err(e) => {
                pattern.push('x');
                if refused_first.is_none() {
                    refused_first = Some(i);
                    summary.push_str(&format!(" overwrite{i}_refused={}", format!("{e:?}").chars().take(env_u64("K_ERRLEN", 80) as usize).collect::<String>()));
                }
                if env_u64("K_CONTINUE", 0) == 0 {
                    break;
                }
                continue;
            }
        }
        pattern.push('o');
    }
    summary.push_str(&format!(" overwrites_done={done} pattern={pattern}"));
    if env_u64("K_UNMOUNT_AFTER", 0) == 1 {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().expect("会话");
        let mut allocator = session.allocator.clone();
        let mut current = session.current.clone();
        let shadow = session.shadow_ledger;
        let outcome = match stream.record_entry(RecordedPublishEntry::Unmount, || unmount(parameters, devices, &mut allocator, &mut current, shadow)) {
            Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => format!("raised:{}", raised.publishes.len()),
            Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "nothing".to_string(),
            Err(e) => format!("err:{}", format!("{e:?}").chars().take(120).collect::<String>()),
        };
        summary.push_str(&format!(" unmount={outcome}"));
    }
    let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(&stream.retained_operations(), &stream.entry_spans(), &start.geometry);
    KRun { run: Ok(R2Run { writes, segments, summary, acceptable, versions: pool.published_versions(), mount_writes }), partial: None, log_in_mount, faulted_reads, trace, faulted_entries }
}

/// 池级 checker 判一张两块盘都在的镜像：红的不变量名（I-3.1 按边记成 I-3.1:more / I-3.1:less / I-3.1:both）与 I-3.1 的说明原文。
fn k_checker(image: &MemoryPool) -> (Vec<String>, String) {
    let mut names = Vec::new();
    let mut i31_text = String::new();
    for (name, verdict) in check_pool_image(image) {
        if let InvariantVerdict::Violated(text) = verdict {
            if name == "I-3.1" {
                let (mut more, mut less) = (false, false);
                for piece in text.split("记账的已分配 ").skip(1) {
                    let a: Option<u64> = piece.strip_prefix("Some(").and_then(|r| r.split(')').next()).and_then(|x| x.parse().ok());
                    let b: Option<u64> = piece.split("遍历全部有效根得到 ").nth(1).and_then(|r| r.split(|c: char| !c.is_ascii_digit()).next()).and_then(|x| x.parse().ok());
                    match (a, b) {
                        (Some(a), Some(b)) if a > b => more = true,
                        (Some(a), Some(b)) if a < b => less = true,
                        _ => {}
                    }
                }
                names.push(match (more, less) { (true, false) => "I-3.1:more", (false, true) => "I-3.1:less", (true, true) => "I-3.1:both", _ => "I-3.1:unparsed" }.to_string());
                i31_text = text.chars().take(400).collect();
            } else {
                if env_str("K_DETAIL", "") == name {
                    i31_text = format!("{name}: {}", text.chars().take(600).collect::<String>());
                }
                names.push(name.to_string());
            }
        }
    }
    (names, i31_text)
}

/// 一条写流每个前缀刀上：两块盘都在的镜像跑池级 checker，丢盘 0 / 丢盘 1 各判一遍剩下那块盘（K_SURVIVOR=0 时不判）。
fn k_screen(label: &str, n: &str, j: u64, plan_name: &str, crashed: &MemoryPool, run: &R2Run, judge: &R2Judge, faulted: &[(u32, u64, u64)], tally: &mut BTreeMap<String, u64>) -> String {
    let w2 = &run.writes;
    let survivor = env_u64("K_SURVIVOR", 0) == 1;
    let print_limit = env_u64("K_PRINT", 4);
    // 读坏的每一槽被这条写流第几个写盖掉（没盖掉记 None）。
    let overwritten_at: Vec<Option<usize>> = faulted
        .iter()
        .map(|(d, off, len)| w2.iter().position(|w| w.device.0 == *d && w.offset.0 < off + len && *off < w.offset.0 + w.length_in_bytes()))
        .collect();
    let all_overwritten_at: Option<usize> = if overwritten_at.iter().all(|x| x.is_some()) { overwritten_at.iter().map(|x| x.expect("有")).max() } else { None };
    let mut red_by_name: BTreeMap<String, (u64, usize, usize)> = BTreeMap::new();
    let mut printed = 0;
    let mut survivor_bad = 0u64;
    let at = env_str("K_SCREEN_AT", "all");
    let ms: Vec<usize> = (0..=w2.len())
        .filter(|m| at == "all" || *m == 0 || *m == w2.len() || w2[m - 1].kind == StepKind::RootRecordFua || (at == "mountroots" && *m <= run.mount_writes))
        .collect();
    let threads = env_u64("JUDGE_THREADS", 14).max(1) as usize;
    let chunk = ms.len().div_ceil(threads).max(1);
    // 每个前缀刀各自现做镜像、各自判：(m, 前缀里落了几条根, checker 红的名字, I-3.1 原文, 丢盘判 [(盘, 择到的根, 结局)])。
    let mut results: Vec<(usize, usize, Vec<String>, String, Vec<(u32, Option<u64>, String)>)> = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = ms
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|&m| {
                            let p2: Vec<bool> = (0..w2.len()).map(|i| i < m).collect();
                            let image = materialize(crashed, w2, &p2);
                            let (names, i31) = k_checker(&image);
                            let roots_so_far = w2[..m].iter().filter(|w| w.kind == StepKind::RootRecordFua).count();
                            let lone: Vec<(u32, Option<u64>, String)> = if survivor {
                                [0u32, 1].into_iter().map(|lost| { let (c, r) = fx_judge(judge, &image, lost, &run.acceptable); (lost, c, r) }).collect()
                            } else {
                                Vec::new()
                            };
                            (m, roots_so_far, names, i31, lone)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for handle in handles {
            results.extend(handle.join().expect("判定线程"));
        }
    });
    results.sort_by_key(|r| r.0);
    let mut i79_excess: Option<(u64, String)> = None;
    for (m, roots_so_far, names, i31, lone) in results {
        if let Some(rest) = i31.strip_prefix("I-7.9: ") {
            // 「把回退下界 F 从 A 抬到 B，高于它之前的根算出的抬 F 上限 C」：记 B − C 最大的那一句。
            let num_after = |key: &str| rest.split(key).nth(1).and_then(|t| t.split(|c: char| !c.is_ascii_digit()).next()).and_then(|x| x.parse::<u64>().ok());
            if let (Some(b), Some(c)) = (num_after("抬到 "), num_after("抬 F 上限 ")) {
                if i79_excess.as_ref().is_none_or(|(e, _)| b.saturating_sub(c) > *e) {
                    i79_excess = Some((b.saturating_sub(c), rest.chars().take(300).collect()));
                }
            }
        }
        let phase = if m <= run.mount_writes { "in_mount" } else { "after_mount" };
        for name in &names {
            let e = red_by_name.entry(name.clone()).or_insert((0, usize::MAX, 0));
            e.0 += 1;
            e.1 = e.1.min(m);
            e.2 = e.2.max(m);
            *tally.entry(format!("checker j={j} {phase} {name}")).or_default() += 1;
            let past_overwrite = all_overwritten_at.is_some_and(|a| m > a);
            if printed < print_limit && (name != "I-3.1:more" || past_overwrite || printed == 0) {
                printed += 1;
                println!("KRED {label} n={n} j={j} plan={plan_name} m={m} roots_in_prefix={roots_so_far} phase={phase} past_all_faulted_overwritten={past_overwrite} {name} {}", if name.starts_with("I-3.1") || i31.starts_with(name.as_str()) { i31.clone() } else { String::new() });
            }
        }
        for (lost, chosen, read) in lone {
            if read.starts_with("BAD") {
                survivor_bad += 1;
                if printed < print_limit {
                    printed += 1;
                    println!("KBAD {label} n={n} j={j} plan={plan_name} m={m} lost=dev{lost} chosen={chosen:?} {read}");
                }
            }
        }
    }
    let ms_len = ms.len();
    let reds: Vec<String> = red_by_name.iter().map(|(k, (c, a, b))| format!("{k}x{c}[m{a}-{b}]")).collect();
    if let Some((e, t)) = &i79_excess {
        println!("KI79 {label} n={n} j={j} plan={plan_name} max_excess={e} {t}");
    }
    format!("i79_max_excess={} screened={} prefixes={} faulted_slots={} all_faulted_overwritten_at={all_overwritten_at:?} survivor_bad={survivor_bad} checker_red={}", i79_excess.as_ref().map_or(0, |(e, _)| *e), ms_len, w2.len() + 1, faulted.len(), if reds.is_empty() { "none".to_string() } else { reds.join(",") })
}

/// 层 0 枚举域（每一段都展开，`enumerate_layer0_selecting_versions_observing_each_state` 的 expand 恒真）：枚举函数自己对每个状态跑 checker 与恢复判官；
/// 观察者记下每个状态哪几个写持久，枚举完再多线程逐状态跑池级 checker 分 I-3.1 的边。
fn k_enumerate(label: &str, n: &str, plan_name: &str, j: u64, crashed: &MemoryPool, run: &R2Run) {
    let seg_from = env_u64("K_SEG_FROM", 0) as usize;
    let seg_to = env_u64("K_SEG_TO", u64::MAX) as usize;
    let part: Vec<Vec<usize>> = run.segments.iter().enumerate().filter(|(i, _)| *i >= seg_from && *i < seg_to).map(|(_, s)| s.clone()).collect();
    let states = closed_form_state_count(&part);
    let max_states = env_u64("MAX_STATES", 1_000_000);
    let w2 = &run.writes;
    let Some(judged_root_index) = w2.iter().position(|w| w.kind == StepKind::RootRecordFua) else {
        println!("KENUM_SKIPPED {label} n={n} plan={plan_name} j={j} no_root_write");
        return;
    };
    if states > max_states {
        println!("KENUM_SKIPPED {label} n={n} plan={plan_name} j={j} closed_form_states={states} > {max_states}");
        return;
    }
    let t0 = Instant::now();
    let mut persisted_of_each: Vec<Vec<bool>> = Vec::new();
    let te = enumerate_layer0_selecting_versions_observing_each_state(
        crashed, w2, &run.segments, judged_root_index, &run.versions, &|i, _| i >= seg_from && i < seg_to,
        &mut |image, _report| persisted_of_each.push(image.persisted.clone()),
    );
    let threads = env_u64("JUDGE_THREADS", 14).max(1) as usize;
    if env_u64("K_RECHECK", 0) == 0 {
        persisted_of_each.clear();
    }
    let chunk = persisted_of_each.len().div_ceil(threads).max(1);
    let mut histogram: BTreeMap<String, u64> = BTreeMap::new();
    let mut sample: Option<String> = None;
    std::thread::scope(|scope| {
        let handles: Vec<_> = persisted_of_each
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    let mut h: BTreeMap<String, u64> = BTreeMap::new();
                    let mut s: Option<String> = None;
                    for persisted in part {
                        let image = materialize(crashed, w2, persisted);
                        let (names, i31) = k_checker(&image);
                        for name in &names {
                            *h.entry(name.clone()).or_default() += 1;
                            if name == "I-3.1:less" && s.is_none() {
                                s = Some(i31.clone());
                            }
                        }
                    }
                    (h, s)
                })
            })
            .collect();
        for handle in handles {
            let (h, s) = handle.join().expect("判定线程");
            for (k, c) in h {
                *histogram.entry(k).or_default() += c;
            }
            if sample.is_none() {
                sample = s;
            }
        }
    });
    println!(
        "KENUM {label} n={n} plan={plan_name} j={j} segments={seg_from}..{}/{} closed_form_states={states} states={} observed={} violations={} failed_states={} checker_violated={:?} recheck={:?} elapsed={:.1}s",
        seg_to.min(run.segments.len()), run.segments.len(), te.states, te.observed_states, te.violations, te.failed_states, te.checker_violated_states, histogram, t0.elapsed().as_secs_f64()
    );
    for (name, v) in &te.checker_first_violation {
        println!("KENUM_CHECKER_FIRST {label} n={n} plan={plan_name} j={j} {name} {}", v.chars().take(400).collect::<String>());
    }
    if let Some(v) = te.first_violation {
        println!("KENUM_FIRST_VIOLATION {label} n={n} plan={plan_name} j={j} {}", v.chars().take(300).collect::<String>());
    }
    if let Some(s) = sample {
        println!("KENUM_LESS_SAMPLE {label} n={n} plan={plan_name} j={j} {s}");
    }
}

/// 一个起点上：崩溃镜像（K_SOURCE=unmount：录一次卸载，CUT_LIST 里每个前缀刀；K_SOURCE=now：起点那一刻）× AFTER_J × K_PLANS（`;` 分计划、`+` 连段）。
pub(crate) fn run_k_on(start: Start) {
    let t0 = Instant::now();
    let session = start.pool.session.as_ref().expect("会话");
    let current_txg = session.current.root().checkpoint_txg.0;
    let pre = start.pool.image();
    let baseline: BTreeMap<u32, (Option<u64>, String, Option<u64>)> = [0u32, 1].into_iter().map(|d| (d, lone_outcome(&pre, d))).collect();
    let mut acceptable = BTreeSet::new();
    if let Ok((_, h)) = read_everything(&pre) {
        acceptable.insert(h);
    }
    for (_, (_, _, h)) in &baseline {
        if let Some(h) = h {
            acceptable.insert(*h);
        }
    }
    let judge = R2Judge { pre: &pre, records: Vec::new(), current_txg, baseline, acceptable };
    let source = env_str("K_SOURCE", "unmount");
    let mut crash_images: Vec<(String, MemoryPool)> = Vec::new();
    if source == "now" {
        crash_images.push(("now".to_string(), pre.clone()));
        println!("KSTART {} | current_txg={current_txg} source=now", start.label);
    } else {
        let (result, operations, spans) = recorded_unmount(&start);
        let (writes, _segments, _) = writes_and_segments_with_stream_indexes_and_entries(&operations, &spans, &start.geometry);
        println!("KSTART {} | current_txg={current_txg} source=unmount unmount_ok={} writes={}", start.label, result.is_ok(), writes.len());
        if result.is_err() {
            return;
        }
        for n in env_list("CUT_LIST", "0") {
            if n as usize > writes.len() {
                continue;
            }
            let persisted: Vec<bool> = (0..writes.len()).map(|i| (i as u64) < n).collect();
            crash_images.push((n.to_string(), materialize(&pre, &writes, &persisted)));
        }
    }
    let renum: Vec<(String, String)> = env_str("K_ENUM", "")
        .split(';')
        .filter(|x| !x.is_empty())
        .map(|x| {
            let (n, p) = x.split_once('@').expect("n@计划");
            (n.to_string(), p.to_string())
        })
        .collect();
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    for (n, crashed) in &crash_images {
        let ring = ring_entries_of(crashed);
        let ring_txgs: Vec<String> = ring.iter().map(|(d, e)| {
            let mut t: Vec<u64> = e.iter().filter_map(|x| x.txg).collect();
            t.sort_unstable();
            t.reverse();
            t.truncate(5);
            format!("dev{d}:{t:?}")
        }).collect();
        for j in env_list("AFTER_J", "0") {
            let clean = k_remount(crashed, &start, &[], &ring, j);
            let boundaries = pass_boundaries(&clean.log_in_mount);
            println!("KPASSES {} n={n} j={j} ring_reads={} boundaries={boundaries:?} ring_newest={ring_txgs:?}", start.label, clean.log_in_mount.len());
            if env_str("K_TRACE", "0") == "1" {
                for (i, t) in clean.trace.iter().enumerate() {
                    println!("KTRACE {} n={n} plan=clean pass={i} {t}", start.label);
                }
            }
            let plan_names: Vec<String> = std::iter::once("clean".to_string())
                .chain(env_str("K_PLANS", "").split(';').filter(|x| !x.is_empty()).map(str::to_string))
                .collect();
            for plan_name in plan_names {
                let subs: Vec<Sub> = if plan_name == "clean" { Vec::new() } else {
                    let parsed: Vec<Option<Sub>> = plan_name.split('+').map(|t| parse_sub(t, &boundaries)).collect();
                    if parsed.iter().any(|s| s.is_none()) {
                        println!("KPLAN_SKIPPED {} n={n} j={j} plan={plan_name} pass_out_of_range", start.label);
                        continue;
                    }
                    parsed.into_iter().map(|s| s.expect("有")).collect()
                };
                let cycles = env_u64("K_CYCLES", 1);
                if cycles > 1 {
                    // 连挂几次：每次挂载做成之后覆盖写 j 次（K_CONTINUE=1 时被拒也接着试），整条写流都落盘，下一次在它上面再挂；
                    // 读坏的槽按第一次的选法固定在那几个物理槽上，被哪一次写掉就从此不再坏。
                    let mut image = crashed.clone();
                    let mut cleared: BTreeSet<(u32, u64)> = BTreeSet::new();
                    for c in 0..cycles {
                        let kr = k_remount_clearing(&image, &start, &subs, &ring, j, &cleared);
                        match &kr.run {
                            Err(e) => {
                                println!("KCYCLE {} n={n} j={j} plan={plan_name} cycle={c} REFUSED {e}", start.label);
                                break;
                            }
                            Ok(run) => {
                                let all: Vec<bool> = vec![true; run.writes.len()];
                                image = materialize(&image, &run.writes, &all);
                                for (d, off, len) in &kr.faulted_entries {
                                    if run.writes.iter().any(|w| w.device.0 == *d && w.offset.0 < off + len && *off < w.offset.0 + w.length_in_bytes()) {
                                        cleared.insert((*d, *off));
                                    }
                                }
                                let (names, _) = k_checker(&image);
                                let pat = run.summary.split_whitespace().find(|t| t.starts_with("pattern=")).unwrap_or("pattern=?").to_string();
                                let err = run.summary.split(" overwrite").nth(1).map(|t| t.chars().take(env_u64("K_ERRLEN", 80) as usize).collect::<String>()).unwrap_or_default();
                                println!("KCYCLE {} n={n} j={j} plan={plan_name} cycle={c} {pat} faulted_now={} cleared_total={} checker_end={names:?} first_refusal=overwrite{err}", start.label, kr.faulted_entries.len(), cleared.len());
                            }
                        }
                    }
                    continue;
                }
                let kr = if subs.is_empty() { None } else { Some(k_remount(crashed, &start, &subs, &ring, j)) };
                let kr_ref = kr.as_ref().unwrap_or(&clean);
                if env_str("K_TRACE", "0") == "1" && kr.is_some() {
                    for (i, t) in kr_ref.trace.iter().enumerate() {
                        println!("KTRACE {} n={n} plan={plan_name} pass={i} {t}", start.label);
                    }
                }
                let run = match (&kr_ref.run, &kr_ref.partial) {
                    (Err(e), partial) => {
                        let kind = e.split_whitespace().nth(1).unwrap_or("").split(|c: char| c == '(' || c == '{').next().unwrap_or("").to_string();
                        *tally.entry(format!("refused j={j} {kind} after_writes={}", partial.is_some())).or_default() += 1;
                        println!("KREFUSED {} n={n} j={j} plan={plan_name} faulted_reads={} {e}", start.label, kr_ref.faulted_reads);
                        match partial {
                            Some(p) => p,
                            None => continue,
                        }
                    }
                    (Ok(run), _) => {
                        *tally.entry(format!("mounted j={j}")).or_default() += 1;
                        run
                    }
                };
                let screened = if env_u64("SCREEN", 0) == 1 {
                    k_screen(&start.label, n, j, &plan_name, crashed, run, &judge, &kr_ref.faulted_entries, &mut tally)
                } else {
                    "unscreened".to_string()
                };
                println!("KPLAN {} n={n} j={j} plan={plan_name} faulted_reads={} {} writes2={} mount_writes={} closed_form_states={} {screened}",
                    start.label, kr_ref.faulted_reads, run.summary, run.writes.len(), run.mount_writes, closed_form_state_count(&run.segments));
                if renum.iter().any(|(rn, rp)| (rn == n || rn == "*") && (*rp == plan_name || rp == "*")) {
                    k_enumerate(&start.label, n, &plan_name, j, crashed, run);
                }
            }
        }
    }
    for (k, c) in &tally {
        println!("KTALLY {} {k} count={c}", start.label);
    }
    println!("KDONE {} elapsed={:.1}s", start.label, t0.elapsed().as_secs_f64());
}

/// MODE=k4：随机历史里掉电接缝落在不落在段首（崩溃注入第一截只把不晚于崩溃段段首的接缝交给记录核对器，
/// `crates/singlefs-checker-tier/src/crash_injection.rs` 的 `first_write_after_each_power_loss_in_the_write_table` 与它下面那一判；
/// 这里照抄那一换算，逐道接缝看它在写表里的下标是不是某一段的第一个写）。不跑 checker、不跑崩溃枚举：只数接缝。
pub(crate) fn main_k4() {
    use singlefs_harness::history::{execute_history_with, generate_history_with_weights, GenerationWeights, HistoryExecution, HistorySeed, PerStepChecker};
    let weights = match env_str("K4_WEIGHTS", "orphans").as_str() {
        "orphans" => &GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP,
        "broad" => &GenerationWeights::BROAD,
        other => panic!("K4_WEIGHTS {other}"),
    };
    let width = width_of(env_u64("K4_WIDTH", 4));
    let execution = HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: width, space_admission: SpaceAdmission::JudgedByTheFormula };
    let steps = env_u64("K4_STEPS", 40) as usize;
    let (from, to) = (env_u64("K4_SEED_FROM", 1), env_u64("K4_SEED_TO", 50));
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    for seed in from..=to {
        let history = generate_history_with_weights(HistorySeed(seed), steps, weights);
        let stream = SharedStream::retaining_contents();
        let run = execute_history_with(&history, execution, &stream, &mut |_| {});
        let operations = stream.retained_operations();
        let (writes, segments, stream_indexes) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&operations, &width.fixed_geometry());
        let seams: Vec<usize> = run
            .first_stream_index_after_each_power_loss
            .iter()
            .map(|first| stream_indexes.iter().position(|i| *i >= *first).unwrap_or(stream_indexes.len()))
            .collect();
        *tally.entry("histories".into()).or_default() += 1;
        *tally.entry("seams".into()).or_default() += seams.len() as u64;
        for s in &seams {
            let at_start = *s == writes.len() || segments.iter().any(|seg| seg.first() == Some(s));
            let key = if at_start { "seam_at_segment_start" } else { "seam_inside_a_segment" };
            *tally.entry(key.into()).or_default() += 1;
            if !at_start {
                let seg = segments.iter().position(|seg| seg.contains(s)).map(|i| (i, segments[i].len(), segments[i].iter().position(|w| w == s)));
                println!("K4INSIDE seed={seed} seam_write={s} segment(index,len,offset_in_segment)={seg:?} writes={}", writes.len());
            }
        }
    }
    for (k, v) in &tally {
        println!("K4TALLY {k}={v}");
    }
}
