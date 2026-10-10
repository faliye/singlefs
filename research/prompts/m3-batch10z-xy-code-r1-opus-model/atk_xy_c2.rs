
// ---------- K1-2：推环入口照收次序对调的盘表（核盘表那一套不看次序） ----------

/// 这一刻的一份拷贝，盘外面包上录写流的那一层（几块盘共一条流，保留内容）。
fn atk_recording_pool(snap: &AtkSnap, stream: &SharedStream) -> PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>> {
    PoolUnderTest {
        parameters: snap.parameters.clone(),
        device_bytes: snap.device_bytes,
        devices: plain_devices_on(&snap.image)
            .into_iter()
            .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
            .collect(),
        session: snap.session.as_ref().map(copy_of_the_session),
        write_time_seconds: snap.write_time_seconds,
        content_of_the_current_version: snap.content_now.clone(),
        content_of_each_root: snap.contents.clone(),
    }
}

/// 一条流按屏障与 FUA 切段（层 0 的切法，`writes_and_segments_with_stream_indexes`），每段里的写取 `{:?}` 的散列、排好：
/// 两条流段段相同，崩溃状态（段内写的整写子集）就是同一批。
fn atk_segment_multisets(stream: &SharedStream) -> (usize, Vec<Vec<u64>>, u64) {
    use std::hash::{Hash, Hasher};
    let operations = stream.retained_operations();
    let (writes, segments, _indexes) = writes_and_segments_with_stream_indexes(&operations, &fixed_geometry_of_the_384_slot_small_devices());
    let multisets = segments
        .iter()
        .map(|segment| {
            let mut hashes: Vec<u64> = segment
                .iter()
                .map(|index| {
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    format!("{:?}", writes[*index]).hash(&mut hasher);
                    hasher.finish()
                })
                .collect();
            hashes.sort_unstable();
            hashes
        })
        .collect();
    // 阳性对照：整条流按原样次序（不切段、不排序）的散列——次序对调真的改了写流的先后，这一个就不同。
    let mut ordered = std::collections::hash_map::DefaultHasher::new();
    for write in &writes {
        format!("{write:?}").hash(&mut ordered);
    }
    (operations.len(), multisets, ordered.finish())
}

/// 照 `table`（mount：挂载时那一份；reversed：次序倒过来、身份与盘不拆）推环一次，再照挂载时那一份盘表发 `then`，再崩、可写挂载一次。
/// 交回（推环结局原样、推完的写流段、推完的盘面、推完的会话、再发结局短名、再发之后盘面、再挂的结局原样）。
fn atk_push_with_table_and_follow(
    snap: &AtkSnap,
    table: &str,
    then: &AtkChange,
) -> (String, (usize, Vec<Vec<u64>>, u64), MemoryPool, String, String, MemoryPool, String) {
    let stream = SharedStream::retaining_contents();
    let mut pool = atk_recording_pool(snap, &stream);
    let mut session = pool.session.take().expect("挂着的会话");
    if table == "reversed" {
        pool.devices.reverse();
    }
    let pushed = session.push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(&mut pool.devices);
    if table == "reversed" {
        pool.devices.reverse();
    }
    pool.session = Some(session);
    if let Ok(done) = &pushed {
        for version in &done.versions_published {
            pool.note_root_of_the_current_content(&version.root);
        }
    }
    let segments = atk_segment_multisets(&stream);
    let image_after_push = pool.image();
    let session_after_push = format!("{:?} {:?}", pool.session().allocator, pool.session().current);
    let then_kind = atk_kind(&atk_apply(&mut pool, then));
    let image_after_then = pool.image();
    let remount = match pool.crash_and_mount_writable() {
        Ok(output) => format!("remount_ok {:?} {:?} {:?}", output.instance, output.chosen_root, output.effective_root),
        Err(error) => format!("remount_refused {error:?}"),
    };
    (format!("{pushed:?}"), segments, image_after_push, session_after_push, then_kind, image_after_then, remount)
}

/// K1-2：K1 九形（`ATK_PRES` × `ATK_FILLS`）每个报「被宽护卡住」的格，在被拒那一刻的拷贝上推环两次：照挂载时那一份盘表、照次序对调的那一份；
/// 逐样比推环结局、写流（按段比、段内不论先后）、推完的盘面与会话，再照挂载时那一份盘表发同一个改动、崩、再挂，逐样比。
/// 推环入口改成先比次序的那一臂（`mutants.py` 的 `ARM-push-checks-the-table-order`）下，次序对调的那一次在任何写之前拒，写流 0 条。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k12_push_with_the_device_table_in_reversed_order() {
    let pres: Vec<u64> = atk_env_list("ATK_PRES", "2,4,8");
    let fills: Vec<usize> = atk_env_list("ATK_FILLS", "6,11,16,21");
    let extra: usize = atk_env_or("ATK_EXTRA", 8);
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for pre in &pres {
        for fill in &fills {
            let Some((base, crashed, contents, write_time_seconds)) = atk_k1_record_on(384, *pre, *fill, 5) else { continue };
            let (probe, _) = atk_mounted_pool_from(&parameters, device_bytes, &crashed, &contents, write_time_seconds).expect("挂");
            let size_now = probe.content_of_the_current_version.len().div_ceil(unit_bytes);
            let _ = base;
            let mounted = AtkSnap::of(&probe);
            for grow in 1..=(40 + extra) {
                let mut pool = mounted.pool();
                let change = atk_seq_of(&mut pool, size_now + grow);
                let outcome = atk_apply(&mut pool, &change);
                if atk_stuck_of(&outcome).is_none() {
                    continue;
                }
                let after = AtkSnap::of(&pool);
                let (push_m, seg_m, img_m, ses_m, then_m, img_tm, re_m) = atk_push_with_table_and_follow(&after, "mount", &change);
                let (push_r, seg_r, img_r, ses_r, then_r, img_tr, re_r) = atk_push_with_table_and_follow(&after, "reversed", &change);
                let same = [
                    ("result", push_m == push_r),
                    ("segments", seg_m.1 == seg_r.1),
                    ("image", img_m == img_r),
                    ("session", ses_m == ses_r),
                    ("then", then_m == then_r),
                    ("image_after_then", img_tm == img_tr),
                    ("remount", re_m == re_r),
                ];
                let differing: Vec<&str> = same.iter().filter(|(_, s)| !*s).map(|(n, _)| *n).collect();
                let key = format!("push_m={} push_r={} differing={differing:?} same_order_unsplit={}", atk_head(push_m.clone()), atk_head(push_r.clone()), seg_m.2 == seg_r.2);
                *tally.entry(key).or_insert(0) += 1;
                println!(
                    "ATK k12 pre={pre} fill={fill} g={grow} push_mount={} push_reversed={} ops_mount={} ops_reversed={} segs={}/{} same_order_unsplit={} then_mount={then_m} then_after_reversed={then_r} differing={differing:?}",
                    push_m.chars().take(40).collect::<String>(),
                    push_r.chars().take(120).collect::<String>(),
                    seg_m.0, seg_r.0, seg_m.1.len(), seg_r.1.len(), seg_m.2 == seg_r.2
                );
            }
        }
    }
    println!("ATK k12 totals {tally:?}");
}
