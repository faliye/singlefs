    // ── 攻方用例（m3-batch63-code-r1 opus）：gen.py 接在单测模块末尾，用模块里的构造函数与私有方法 ──

    /// 第二份对方库 C 的同序版：内容同 build_the_second_other_library_c_with_the_same_new_verdict_vectors，只是两种新向量照本库的次序登记
    /// （先 F2 那一种、再 F3 那一种），C 里的编号与本库相同。
    fn opus_build_c_in_the_local_order(directory: &Path) -> VerdictStore {
        let mut store = build_the_local_library_a(directory);
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red_judged_on_the_other_host = store
            .register_verdict_vector(&verdict_vector(b"red-f-judged-on-the-other-host"))
            .expect("登记");
        let red_of_block_three = store
            .register_verdict_vector(&verdict_vector(b"red-f-block-three-on-the-other-host"))
            .expect("登记");
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_f_block(3),
            &ConclusionKind::CpuJudge,
            &CPU_CONCLUSION_VERSION,
            &conclusion_record(
                block_of(&[red_of_block_three, red_judged_on_the_other_host, clean]),
                &[0, 1],
            ),
        );
        store
    }

    fn opus_text(store: &VerdictStore, number: u8) -> String {
        store
            .verdict_vector_of_number(VerdictVectorNumber(number))
            .map_or_else(|| "None".to_string(), |vector| String::from_utf8_lossy(&vector.0).into_owned())
    }

    fn opus_numbers(store: &VerdictStore) -> String {
        (0..7u8).map(|number| opus_text(store, number)).collect::<Vec<_>>().join("|")
    }

    /// 键值行的差：新键按表（键的头一个字节）数，改了值的键按表数。
    fn opus_diff(before: &[StoredEntry], after: &[StoredEntry]) -> String {
        let before: BTreeMap<&[u8], &[u8]> = before.iter().map(|(k, v)| (&k[..], &v[..])).collect();
        let mut new_by_tag: BTreeMap<u8, u64> = BTreeMap::new();
        let mut changed_by_tag: BTreeMap<u8, u64> = BTreeMap::new();
        for (key, value) in after {
            match before.get(&key[..]) {
                None => *new_by_tag.entry(key[0]).or_default() += 1,
                Some(old) if *old != &value[..] => *changed_by_tag.entry(key[0]).or_default() += 1,
                Some(_) => {}
            }
        }
        format!("new_by_tag={new_by_tag:02x?} changed_by_tag={changed_by_tag:02x?}")
    }

    /// T1 账：照被判用例的前几步（A 照产品批长从 B 导、再登一种新向量），再从 C 只点结论导；导入前后整库键值行的差按表数，
    /// 与账里的 rows_written 两条路数同一件事。只打读数、不断言，变异下照跑。
    #[test]
    fn opus_t1_rows_of_the_c_import_by_table() {
        let other_directory = TemporaryLibraryDirectory::new("opus-t1-rows-other");
        let other = build_the_other_library_b_with_a_second_new_verdict_vector(&other_directory.path);
        let local_directory = TemporaryLibraryDirectory::new("opus-t1-rows-local");
        let mut local = build_the_local_library_a(&local_directory.path);
        let planned = planned_keys_of_flow_f_beyond_this_host_s_share();
        let b_tally = local.import_the_planned_rows_from(&other, &planned);
        eprintln!("OPUS_T1 b_tally={b_tally:?}");
        eprintln!("OPUS_T1 numbers_after_b={}", opus_numbers(&local));
        let fresh = local.register_verdict_vector(&verdict_vector(b"registered-after-the-import"));
        eprintln!("OPUS_T1 fresh={fresh:?}");
        let c_directory = TemporaryLibraryDirectory::new("opus-t1-rows-c");
        let c = build_the_second_other_library_c_with_the_same_new_verdict_vectors(&c_directory.path);
        let before = local.every_stored_entry();
        let conclusions_only = PlannedImportKeys {
            conclusions: planned.conclusions.clone(),
            paths: Vec::new(),
            verifier_blocks: Vec::new(),
        };
        let c_tally = local.import_the_planned_rows_from(&c, &conclusions_only);
        let after = local.every_stored_entry();
        eprintln!("OPUS_T1 c_tally={c_tally:?}");
        eprintln!("OPUS_T1 c_disk {} rows_before={} rows_after={}", opus_diff(&before, &after), before.len(), after.len());
        eprintln!("OPUS_T1 numbers_after_c={}", opus_numbers(&local));
        let full_tally = local.import_the_planned_rows_from(&c, &planned);
        eprintln!("OPUS_T1 c_again_full_plan_tally={full_tally:?} path_f0_f1={:?}", local.identity_of_path("f.0/f.1").ok().flatten());
    }

    /// T1：一批里登记三种新向量（产品批长），导完不重开读编号与内容、再登记、再从一份三种都用、编号与本库不同的对方库导，重开再比。
    /// 关的是只在一批三种以上才看得出的错法（take2 一类）；被判用例一批最多两种。
    #[test]
    fn opus_t1_three_new_vectors_in_one_product_batch() {
        let other_directory = TemporaryLibraryDirectory::new("opus-t1-three-other");
        let mut other = build_the_other_library_b_with_a_second_new_verdict_vector(&other_directory.path);
        let clean = other.register_verdict_vector(&verdict_vector(b"")).expect("登记");
        let third = other
            .register_verdict_vector(&verdict_vector(b"red-f-third-in-the-same-batch"))
            .expect("登记");
        store_conclusion_in_the_test_library(
            &mut other,
            &flow_f_block(3),
            &observer_kind(MOUNT_OBSERVER_NAME),
            &OBSERVER_CONCLUSION_VERSION,
            &observer_record(block_of(&[third, clean]), &[0]),
        );
        let local_directory = TemporaryLibraryDirectory::new("opus-t1-three-local");
        let mut local = build_the_local_library_a(&local_directory.path);
        let planned = planned_keys_of_flow_f_beyond_this_host_s_share();
        let tally = local.import_the_planned_rows_from(&other, &planned).expect("照产品批长导");
        eprintln!("OPUS_T1_THREE tally={tally:?}");
        eprintln!("OPUS_T1_THREE numbers={}", opus_numbers(&local));
        assert_eq!(
            (opus_text(&local, 2), opus_text(&local, 3), opus_text(&local, 4)),
            (
                "red-f-judged-on-the-other-host".to_string(),
                "red-f-block-three-on-the-other-host".to_string(),
                "red-f-third-in-the-same-batch".to_string()
            ),
            "OPUS 一批三种新向量：导完不重开，内存表里编号 2、3、4 照登记次序"
        );
        let kinds = [
            (ConclusionKind::GpuJudge, GPU_CONCLUSION_VERSION),
            (ConclusionKind::CpuJudge, CPU_CONCLUSION_VERSION),
            (observer_kind(MOUNT_OBSERVER_NAME), OBSERVER_CONCLUSION_VERSION),
        ];
        for block_index in 1..4 {
            for (kind, version) in &kinds {
                assert_eq!(
                    conclusion_read_back_by_content(&local, &flow_f_block(block_index), kind, version),
                    conclusion_read_back_by_content(&other, &flow_f_block(block_index), kind, version),
                    "OPUS 一批三种新向量：F{block_index} 的 {kind:?} 导完不重开按内存表读回与对方库不同"
                );
            }
        }
        let fresh = verdict_vector(b"registered-after-the-import");
        assert_eq!(
            local.register_verdict_vector(&fresh).expect("再登记"),
            VerdictVectorNumber(5),
            "OPUS 一批三种新向量：再登记一种，编号 5"
        );
        let c_directory = TemporaryLibraryDirectory::new("opus-t1-three-c");
        let mut c = build_the_local_library_a(&c_directory.path);
        let c_clean = c.register_verdict_vector(&verdict_vector(b"")).expect("登记");
        let c_third = c.register_verdict_vector(&verdict_vector(b"red-f-third-in-the-same-batch")).expect("登记");
        let c_three = c.register_verdict_vector(&verdict_vector(b"red-f-block-three-on-the-other-host")).expect("登记");
        let c_judged = c.register_verdict_vector(&verdict_vector(b"red-f-judged-on-the-other-host")).expect("登记");
        store_conclusion_in_the_test_library(
            &mut c,
            &flow_f_block(3),
            &ConclusionKind::CpuJudge,
            &CPU_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[c_third, c_three, c_judged, c_clean]), &[0, 1, 2]),
        );
        let conclusions_only = PlannedImportKeys {
            conclusions: planned.conclusions.clone(),
            paths: Vec::new(),
            verifier_blocks: Vec::new(),
        };
        let c_tally = local.import_the_planned_rows_from(&c, &conclusions_only).expect("接着从 C 导");
        eprintln!("OPUS_T1_THREE c_tally={c_tally:?}");
        assert_eq!(
            (c_tally.conclusion_blocks_written, c_tally.rows_written, opus_text(&local, 6)),
            (1, 7, "None".to_string()),
            "OPUS 一批三种新向量：接着从 C 导，只写 F3 的 CPU 7 行，三种向量按内容找到本库的编号，不另起编号 6"
        );
        drop(local);
        let (reopened, _) = VerdictStore::open_existing(&local_directory.path).expect("重开");
        for block_index in 1..4 {
            for (kind, version) in &kinds {
                let from = if block_index == 3 && *kind == ConclusionKind::CpuJudge { &c } else { &other };
                assert_eq!(
                    conclusion_read_back_by_content(&reopened, &flow_f_block(block_index), kind, version),
                    conclusion_read_back_by_content(from, &flow_f_block(block_index), kind, version),
                    "OPUS 一批三种新向量：重开之后 F{block_index} 的 {kind:?} 按内容读回与导自的库不同"
                );
            }
        }
        assert_eq!(
            (opus_text(&reopened, 5), opus_text(&reopened, 6)),
            ("registered-after-the-import".to_string(), "None".to_string()),
            "OPUS 一批三种新向量：重开之后编号 5 是再登记的那一种，没有 6"
        );
    }

    /// T4：同一份本库、同一份对方库、同一份计划导两次，三种口径（点名的、写下的、写下的加已有且相同的）各是几；
    /// 再照产品逐份导入的样子（同一个打开着的库、每份都用整份计划，连路径与核对行）接着导 C。只打读数、不断言。
    #[test]
    fn opus_t4_reimport_counts_and_the_product_loop_over_two_libraries() {
        let other_directory = TemporaryLibraryDirectory::new("opus-t4-other");
        let other = build_the_other_library_b_with_a_second_new_verdict_vector(&other_directory.path);
        let c_directory = TemporaryLibraryDirectory::new("opus-t4-c");
        let c = build_the_second_other_library_c_with_the_same_new_verdict_vectors(&c_directory.path);
        let local_directory = TemporaryLibraryDirectory::new("opus-t4-local");
        let mut local = build_the_local_library_a(&local_directory.path);
        let planned = planned_keys_of_flow_f_beyond_this_host_s_share();
        for round in ["first", "again"] {
            let tally = local.import_the_planned_rows_from(&other, &planned).expect("导");
            eprintln!(
                "OPUS_T4 round={round} named={} written={} identical={} missing={} pipeline_imported_blocks={} rows={} synced={}",
                planned.conclusions.len(),
                tally.conclusion_blocks_written,
                tally.conclusion_blocks_already_identical,
                tally.conclusion_blocks_missing_in_the_other_library,
                tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical,
                tally.rows_written,
                tally.synced_writes
            );
        }
        eprintln!("OPUS_T4 path_before_c={:?}", local.identity_of_path("f.0/f.1").expect("读路径"));
        let c_tally = local.import_the_planned_rows_from(&c, &planned).expect("照产品接着导 C");
        eprintln!("OPUS_T4 product_loop_c_tally={c_tally:?}");
        eprintln!("OPUS_T4 path_after_c={:?}", local.identity_of_path("f.0/f.1").expect("读路径"));
    }

    /// T1：一字节一批导完（每组一批，收尾那一批是空的）不重开，读内存表的编号 2、3，再登记一种新向量看编号；
    /// 被判用例只在照产品批长那一份上读内存表，一字节一批那一份只在重开之后读盘。
    #[test]
    fn opus_t1_in_memory_table_after_one_byte_batches() {
        let other_directory = TemporaryLibraryDirectory::new("opus-t1-one-byte-other");
        let other = build_the_other_library_b_with_a_second_new_verdict_vector(&other_directory.path);
        let local_directory = TemporaryLibraryDirectory::new("opus-t1-one-byte-local");
        let mut local = build_the_local_library_a(&local_directory.path);
        let planned = planned_keys_of_flow_f_beyond_this_host_s_share();
        let tally = local
            .import_the_planned_rows_in_batches_of(&other, &planned, ONE_BYTE_BATCH_WRITES_AFTER_EVERY_PUT_GROUP)
            .expect("一字节一批导入");
        eprintln!("OPUS_T1_ONE_BYTE tally={tally:?} numbers={}", opus_numbers(&local));
        assert_eq!(
            (opus_text(&local, 2), opus_text(&local, 3)),
            (
                "red-f-judged-on-the-other-host".to_string(),
                "red-f-block-three-on-the-other-host".to_string()
            ),
            "OPUS 一字节一批导完、不重开：编号 2、3 在内存表里"
        );
        assert_eq!(
            local
                .register_verdict_vector(&verdict_vector(b"registered-after-the-import"))
                .expect("再登记"),
            VerdictVectorNumber(4),
            "OPUS 一字节一批导完、不重开：再登记一种，编号 4"
        );
    }

    /// T1：一字节一批导完之后接着登记一种新向量（流水线里导入之后第 ① 段判本机的块就这样登记），关库重开：只打读数、不断言，
    /// 看「只在最后一批更新」一类错法在产品上会怎样（编号撞上盘上已有的、库重开不了）。
    #[test]
    fn opus_t1_one_byte_batches_then_register_then_reopen() {
        let other_directory = TemporaryLibraryDirectory::new("opus-t1-reopen-other");
        let other = build_the_other_library_b_with_a_second_new_verdict_vector(&other_directory.path);
        let local_directory = TemporaryLibraryDirectory::new("opus-t1-reopen-local");
        let mut local = build_the_local_library_a(&local_directory.path);
        let planned = planned_keys_of_flow_f_beyond_this_host_s_share();
        let _ = local.import_the_planned_rows_in_batches_of(&other, &planned, ONE_BYTE_BATCH_WRITES_AFTER_EVERY_PUT_GROUP);
        let fresh = local.register_verdict_vector(&verdict_vector(b"registered-after-the-import"));
        drop(local);
        let reopened = VerdictStore::open_existing(&local_directory.path).map(|(store, _)| opus_numbers(&store));
        eprintln!("OPUS_T1_REOPEN fresh={fresh:?} reopened={reopened:?}");
    }
