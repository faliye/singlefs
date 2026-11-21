use super::*;

#[test]
fn the_digest_kept_for_a_content_seen_before_equals_the_digest_computed_afresh() {
    let kept = DigestsOfContentsSeenBefore::empty();
    let first = vec![7_u8; 3000];
    let mut differs_in_the_last_byte = first.clone();
    *differs_in_the_last_byte.last_mut().expect("3000 个字节") = 8;
    let mut differs_in_the_first_byte = first.clone();
    differs_in_the_first_byte[0] = 8;
    for content in [
        &first[..],
        &first[..],
        &differs_in_the_last_byte[..],
        &differs_in_the_first_byte[..],
        &differs_in_the_last_byte[..],
        &[][..],
        &first[..2999],
    ] {
        assert_eq!(
            kept.digest_of(content),
            sha256_digest(content),
            "长 {} 的内容：表交回的摘要要等于现算的",
            content.len()
        );
    }
    assert_eq!(
        kept.contents_kept(),
        5,
        "五份不同的内容各记一份，再来的不重记"
    );
}

/// 表满了之后没见过的内容照样交回现算的摘要、不再记；记下的那几份照旧取得到。
#[test]
fn a_content_not_seen_before_is_digested_afresh_once_the_kept_contents_are_full() {
    let kept = DigestsOfContentsSeenBefore::empty();
    let content_numbered = |number: usize| -> Vec<u8> {
        u64::try_from(number)
            .expect("装得进 u64")
            .to_le_bytes()
            .to_vec()
    };
    for number in 0..DigestsOfContentsSeenBefore::MAXIMUM_CONTENTS_KEPT + 3 {
        let content = content_numbered(number);
        assert_eq!(kept.digest_of(&content), sha256_digest(&content));
    }
    assert_eq!(
        kept.contents_kept(),
        DigestsOfContentsSeenBefore::MAXIMUM_CONTENTS_KEPT,
        "记下的份数停在上限"
    );
    for number in [0, DigestsOfContentsSeenBefore::MAXIMUM_CONTENTS_KEPT + 2] {
        let content = content_numbered(number);
        assert_eq!(kept.digest_of(&content), sha256_digest(&content));
    }
}

/// 读集见证的摘要带「写没写过」：清零把基础镜像写过的扇区删掉，读出来仍是 0，journal 记录槽提示却不再报它；读集外的改动不进摘要。
#[test]
fn a_read_set_witness_tells_a_zeroed_sector_from_the_written_one_and_ignores_sectors_outside_the_set(
) {
    let device = singlefs_core::address::DeviceIdentity(1);
    let sector_bytes = crate::memory_pool::SECTOR_BYTES;
    let mut base = MemoryPool::with_devices(&[device], 1 << 20);
    base.apply_writes(&[RetainedWrite {
        device,
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: singlefs_core::address::DeviceOffsetInBytes(16 * sector_bytes),
        contents: WrittenContents::Bytes(vec![0; 512]),
    }]);
    let zeroing = RetainedWrite {
        device,
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: singlefs_core::address::DeviceOffsetInBytes(16 * sector_bytes),
        contents: WrittenContents::Zeros {
            length: sector_bytes,
        },
    };
    let digester = ReadSetWitnessDigester::of(&base, &[zeroing]);
    // 每个状态各从空的前缀叠起（这一条只比两个状态，不看前缀共用）
    let overlay_of = |persisted: &[bool]| {
        digester.overlay_of_the_state(persisted, &std::cell::RefCell::new(None))
    };
    let the_sector = [ReadRangeOnTheCrashImage {
        device: 1,
        first_sector: 16,
        end_sector: 17,
    }];
    assert_ne!(
        digester.witness_digest(&overlay_of(&[false]), &the_sector),
        digester.witness_digest(&overlay_of(&[true]), &the_sector),
        "基础镜像里写过的全零扇区与清零之后没写过的扇区读出来相同、提示不同，摘要要分得开"
    );
    let elsewhere = [ReadRangeOnTheCrashImage {
        device: 1,
        first_sector: 32,
        end_sector: 33,
    }];
    assert_eq!(
        digester.witness_digest(&overlay_of(&[false]), &elsewhere),
        digester.witness_digest(&overlay_of(&[true]), &elsewhere),
        "读集外的改动不进摘要"
    );
}

/// 一条两块盘的小流的基础镜像与六次写（普通写与清零交错，清零盖住基础镜像写过的与前面写过的扇区）：前缀共用那几条用例用。
fn base_and_writes_where_zeroing_covers_earlier_writes() -> (MemoryPool, Vec<RetainedWrite>) {
    let device_where_a_zeroing_covers_earlier_writes = singlefs_core::address::DeviceIdentity(1);
    let device_where_a_zeroing_covers_the_base = singlefs_core::address::DeviceIdentity(2);
    let sector_bytes = crate::memory_pool::SECTOR_BYTES;
    let write = |device, first_sector: u64, contents| RetainedWrite {
        device,
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: singlefs_core::address::DeviceOffsetInBytes(first_sector * sector_bytes),
        contents,
    };
    let bytes = |value: u8, sectors: u64| {
        WrittenContents::Bytes(vec![
            value;
            usize::try_from(sectors * sector_bytes)
                .expect("几个扇区装得进 usize")
        ])
    };
    let mut base = MemoryPool::with_devices(
        &[
            device_where_a_zeroing_covers_earlier_writes,
            device_where_a_zeroing_covers_the_base,
        ],
        1 << 20,
    );
    base.apply_writes(&[
        write(
            device_where_a_zeroing_covers_earlier_writes,
            16,
            bytes(0xAA, 1),
        ),
        write(
            device_where_a_zeroing_covers_earlier_writes,
            20,
            bytes(0x11, 1),
        ),
        write(device_where_a_zeroing_covers_the_base, 3, bytes(0x22, 1)),
    ]);
    let writes = vec![
        write(
            device_where_a_zeroing_covers_earlier_writes,
            16,
            bytes(0x01, 2),
        ),
        write(
            device_where_a_zeroing_covers_earlier_writes,
            15,
            WrittenContents::Zeros {
                length: 6 * sector_bytes,
            },
        ),
        write(device_where_a_zeroing_covers_the_base, 3, bytes(0x03, 1)),
        write(
            device_where_a_zeroing_covers_earlier_writes,
            17,
            bytes(0x04, 1),
        ),
        write(
            device_where_a_zeroing_covers_the_base,
            2,
            WrittenContents::Zeros {
                length: 3 * sector_bytes,
            },
        ),
        write(
            device_where_a_zeroing_covers_earlier_writes,
            30,
            bytes(0x05, 1),
        ),
    ];
    (base, writes)
}

/// 场景（改动表的前缀相邻状态共用一份）：两块盘、六次写（`base_and_writes_where_zeroing_covers_earlier_writes`），六次写的每一种持久集合（64 种），
/// 照三种次序各走一遍、每种次序共用一份留着的前缀：先走「前缀短、下一个状态在它后面清零」的那几个（留着的前缀比这个状态的短，清零要盖住前缀里的写），
/// 再按持久集合升序、降序各一遍。
/// 预期：每个状态两截合起来的改动与从头逐次叠的那一份（[`ReadSetWitnessDigester::overlay_of`]）逐项相同，几组读集上的见证摘要与照整份改动算的
/// （[`ReadSetWitnessDigester::witness_digest_of_the_whole_overlay`]）逐字节相同；头两次写持久、别的都不持久的那个状态，盘 1 上的改动恰好是
/// 扇区 16、17、20 三个都没写过（绝对值另钉）。
/// 判别力：`crates/mutations.tsv` 里「观察者录入段改动表留着的前缀比这个状态的长也照用」「观察者录入段清零只盖这个状态自己那一截叠过的扇区」两行
/// 各让这一条红。
#[test]
fn image_overlays_sharing_a_prefix_equal_laying_every_persisted_write_from_the_start() {
    let (base, writes) = base_and_writes_where_zeroing_covers_earlier_writes();
    let digester = ReadSetWitnessDigester::of(&base, &writes);
    let persisted_of = |mask: u32| -> Vec<bool> {
        (0..writes.len())
            .map(|position| mask & (1 << position) != 0)
            .collect()
    };
    let read_sets: Vec<Vec<ReadRangeOnTheCrashImage>> = vec![
        ReadRangeOnTheCrashImage::whole_pool(&base),
        vec![ReadRangeOnTheCrashImage {
            device: 1,
            first_sector: 16,
            end_sector: 18,
        }],
        vec![
            ReadRangeOnTheCrashImage {
                device: 1,
                first_sector: 29,
                end_sector: 31,
            },
            ReadRangeOnTheCrashImage {
                device: 2,
                first_sector: 0,
                end_sector: 8,
            },
        ],
    ];
    let every_mask: Vec<u32> = (0..1u32 << writes.len()).collect();
    let orders: [Vec<u32>; 3] = [
        vec![
            0b000001, 0b000011, 0b000111, 0b000101, 0b111111, 0b011111, 0b000000, 0b010011,
        ],
        every_mask.clone(),
        every_mask.iter().rev().copied().collect(),
    ];
    for order in &orders {
        let prefix_kept = std::cell::RefCell::new(None);
        for mask in order {
            let persisted = persisted_of(*mask);
            let laid_from_the_start = digester.overlay_of(&persisted);
            let over_a_prefix = digester.overlay_of_the_state(&persisted, &prefix_kept);
            for device in [1u32, 2] {
                assert_eq!(
                    over_a_prefix
                        .changes_in(device, 0..u64::MAX)
                        .collect::<Vec<_>>(),
                    laid_from_the_start
                        .range((device, 0)..(device, u64::MAX))
                        .map(|(position, change)| (*position, *change))
                        .collect::<Vec<_>>(),
                    "持久集合 {mask:#08b}、盘 {device}：两截合起来的改动与从头叠的逐项相同"
                );
            }
            for ranges in &read_sets {
                assert_eq!(
                    digester.witness_digest(&over_a_prefix, ranges),
                    digester.witness_digest_of_the_whole_overlay(&laid_from_the_start, ranges),
                    "持久集合 {mask:#08b}、读集 {ranges:?}：见证摘要逐字节相同"
                );
            }
        }
    }
    assert_eq!(
        digester
            .overlay_of_the_state(&persisted_of(0b000011), &std::cell::RefCell::new(None))
            .changes_in(1, 0..u64::MAX)
            .collect::<Vec<_>>(),
        vec![
            ((1, 16), SectorChange::Unwritten),
            ((1, 17), SectorChange::Unwritten),
            ((1, 20), SectorChange::Unwritten),
        ],
        "头两次写持久：清零盖住了基础镜像写过的 16、20 与头一次写的 16、17"
    );
}

/// 一组读集写成字节、按它的摘要读回来逐段相同；字节少一个、或摘要对不上（库里那一行被改过、或键引错了行）都不认。
#[test]
fn an_observed_read_set_round_trips_through_its_bytes_and_refuses_bytes_whose_digest_differs() {
    let read_set = ObservedReadSet::of_the_ranges(vec![
        ReadRangeOnTheCrashImage {
            device: 0,
            first_sector: 0,
            end_sector: 8,
        },
        ReadRangeOnTheCrashImage {
            device: 1,
            first_sector: 1000,
            end_sector: 1004,
        },
    ]);
    let bytes = observed_read_set_bytes(&read_set.ranges);
    assert_eq!(
        read_set.digest,
        sha256_digest(&bytes),
        "读集摘要就是它的字节的 SHA-256"
    );
    assert_eq!(
        ObservedReadSet::of_the_stored_bytes(&read_set.digest, &bytes),
        Some(read_set.clone())
    );
    assert_eq!(
        ObservedReadSet::of_the_stored_bytes(
            &sha256_digest(&bytes[..bytes.len() - 1]),
            &bytes[..bytes.len() - 1]
        ),
        None,
        "少一个字节解不出（摘要对得上也不认）"
    );
    let mut changed = bytes.clone();
    *changed.last_mut().expect("读集字节不空") ^= 1;
    assert_eq!(
        ObservedReadSet::of_the_stored_bytes(&read_set.digest, &changed),
        None,
        "字节与摘要对不上不认"
    );
    assert_eq!(
        ObservedReadSet::of_the_ranges(Vec::new()),
        ObservedReadSet::of_the_stored_bytes(
            &sha256_digest(&observed_read_set_bytes(&[])),
            &observed_read_set_bytes(&[])
        )
        .expect("空读集也读得回"),
        "空读集读回来相同"
    );
}

/// 读集见证索引的用例用的见证摘要与镜像键：按编号各不相同，彼此也不相同（摘要与键的前缀不同）。
fn witness_digest_numbered(number: usize) -> [u8; DIGEST_BYTES] {
    sha256_digest(format!("witness digest {number}").as_bytes())
}

fn observed_image_key_numbered(number: usize) -> [u8; DIGEST_BYTES] {
    sha256_digest(format!("observed image key {number}").as_bytes())
}

/// 一组读集：盘 `device` 上从 `first_sector` 起的两段。
fn read_set_starting_at(device: u32, first_sector: u64) -> ObservedReadSet {
    ObservedReadSet::of_the_ranges(vec![
        ReadRangeOnTheCrashImage {
            device,
            first_sector,
            end_sector: first_sector + 8,
        },
        ReadRangeOnTheCrashImage {
            device,
            first_sector: first_sector + 100,
            end_sector: first_sector + 101,
        },
    ])
}

/// 读这组段的第 `number` 号见证：摘要与镜像键按编号造（[`witness_digest_numbered`]、[`observed_image_key_numbered`]），段是这组的共享引用。
fn witness_numbered(read_set: &ObservedReadSet, number: usize) -> ReadSetWitness {
    ReadSetWitness {
        read_set: read_set.clone(),
        digest: witness_digest_numbered(number),
        observed_image_key: observed_image_key_numbered(number),
    }
}

/// 桶里 300 条见证读的是同一组段、只有摘要不同：匹配一个状态只算一次这组段上的摘要，交回摘要相同那一条的镜像键；对不上的也只算一次。
#[test]
fn matching_a_state_against_witnesses_sharing_one_read_set_digests_that_read_set_once() {
    let read_set = read_set_starting_at(0, 64);
    let witnesses: Vec<ReadSetWitness> = (0..300)
        .map(|number| witness_numbered(&read_set, number))
        .collect();
    let index = ReadSetWitnessIndex::of_the_stored_witnesses(witnesses);
    assert_eq!(index.read_set_group_count(), 1, "300 条见证共用一组读集");
    let mut digest_calls = 0_usize;
    let matched = index.find_the_matching_witness(None, |asked_ranges| {
        digest_calls += 1;
        assert_eq!(asked_ranges, read_set.ranges.as_ref(), "问的是那一组读集");
        witness_digest_numbered(237)
    });
    assert_eq!(
        digest_calls, 1,
        "同一组读集只算一次摘要，不是逐条见证各算一次"
    );
    assert_eq!(
        matched.map(|found| found.observed_image_key),
        Some(observed_image_key_numbered(237)),
        "交回摘要相同那一条见证看出的镜像键"
    );
    let mut digest_calls_on_a_miss = 0_usize;
    let missed = index.find_the_matching_witness(matched.map(|found| found.group), |_| {
        digest_calls_on_a_miss += 1;
        witness_digest_numbered(300)
    });
    assert_eq!(missed, None, "摘要不在桶里的状态对不上");
    assert_eq!(digest_calls_on_a_miss, 1, "对不上也只算一次");
}

/// 两组读集各五条见证（偶数号见证一组、奇数号见证一组，偶数号那一组先建）：命中奇数号那一组时摘要次数不超过组数（2），先查上一次命中的那一组时
/// 只算一次；一条都不中时每组恰好一次（2）、交回 None。
#[test]
fn matching_against_two_read_sets_digests_each_read_set_at_most_once() {
    let read_set_of_the_even_numbered_witnesses = read_set_starting_at(0, 0);
    let read_set_of_the_odd_numbered_witnesses = read_set_starting_at(1, 4096);
    let witnesses: Vec<ReadSetWitness> = (0..10)
        .map(|number| {
            witness_numbered(
                if number % 2 == 0 {
                    &read_set_of_the_even_numbered_witnesses
                } else {
                    &read_set_of_the_odd_numbered_witnesses
                },
                number,
            )
        })
        .collect();
    let index = ReadSetWitnessIndex::of_the_stored_witnesses(witnesses);
    assert_eq!(index.read_set_group_count(), 2, "两组读集");
    let state_matching_witness_seven = |asked_ranges: &[ReadRangeOnTheCrashImage]| {
        if asked_ranges == read_set_of_the_odd_numbered_witnesses.ranges.as_ref() {
            witness_digest_numbered(7)
        } else {
            witness_digest_numbered(1000)
        }
    };
    let mut digest_calls = 0_usize;
    let matched = index.find_the_matching_witness(None, |asked_ranges| {
        digest_calls += 1;
        state_matching_witness_seven(asked_ranges)
    });
    assert_eq!(
        matched.map(|found| found.observed_image_key),
        Some(observed_image_key_numbered(7)),
        "对上奇数号那一组里的七号见证"
    );
    assert!(digest_calls <= 2, "摘要次数不超过组数，实为 {digest_calls}");
    let mut digest_calls_trying_the_last_group_first = 0_usize;
    let matched_again =
        index.find_the_matching_witness(matched.map(|found| found.group), |asked_ranges| {
            digest_calls_trying_the_last_group_first += 1;
            state_matching_witness_seven(asked_ranges)
        });
    assert_eq!(matched_again, matched, "先查哪一组都对上同一条");
    assert_eq!(
        digest_calls_trying_the_last_group_first, 1,
        "上一次命中的那一组先查，命中就停"
    );
    let mut asked_read_sets: Vec<Vec<ReadRangeOnTheCrashImage>> = Vec::new();
    let missed =
        index.find_the_matching_witness(matched.map(|found| found.group), |asked_ranges| {
            asked_read_sets.push(asked_ranges.to_vec());
            witness_digest_numbered(1000)
        });
    assert_eq!(missed, None, "一条都不中");
    assert_eq!(
        asked_read_sets,
        vec![
            read_set_of_the_odd_numbered_witnesses.ranges.to_vec(),
            read_set_of_the_even_numbered_witnesses.ranges.to_vec()
        ],
        "一条都不中时每组恰好算一次：上一次命中的奇数号那一组先算，偶数号那一组后算"
    );
}

/// 往索引里添一条桶里已有摘要的见证：不添、见证条数与分组都不变（与观察者另一个线程刚添了同一摘要时的判一致）；摘要新的才添。
#[test]
fn adding_a_witness_whose_digest_is_already_in_the_bucket_leaves_the_index_unchanged() {
    let read_set = read_set_starting_at(0, 64);
    let mut index = ReadSetWitnessIndex::empty();
    assert_eq!(
        index.add(witness_numbered(&read_set, 1)),
        ReadSetWitnessAddition::AddedWithANewReadSet,
        "空桶里添一条：读集头一回出现"
    );
    let before_the_duplicate = index.clone();
    assert_eq!(
        index.add(ReadSetWitness {
            read_set: read_set_starting_at(1, 0),
            digest: witness_digest_numbered(1),
            observed_image_key: observed_image_key_numbered(2),
        }),
        ReadSetWitnessAddition::AlreadyInTheBucket,
        "同一摘要（读集不同也一样）不再添"
    );
    assert_eq!(index, before_the_duplicate, "没添的那一次索引一点不变");
    assert_eq!(index.witness_count(), 1);
    assert_eq!(
        index.add(witness_numbered(&read_set, 2)),
        ReadSetWitnessAddition::AddedToAKnownReadSet,
        "摘要新的照添，读集已在桶里"
    );
    assert_eq!(index.witness_count(), 2);
    assert_eq!(index.read_set_group_count(), 1, "读集相同的进同一组");
}

/// 几十条见证随机分进四组读集（摘要各不相同），一批状态逐个按索引匹配（带着上一个状态命中的那一组），交回的镜像键与按添的次序逐条扫的
/// 逐个相同；一半状态各对上一条随机挑的见证、一半一条都不中。逐条添建出的索引与按同一次序从读回的见证建出的相同。
#[test]
fn the_read_set_index_finds_the_same_witness_as_scanning_the_stored_witnesses_in_order() {
    let mut random_state: u64 = 0x5eed_1d0b_43bd_9a3f;
    let mut next_random = move || {
        random_state = random_state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = random_state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    };
    let read_sets: Vec<ObservedReadSet> = (0..4)
        .map(|group_number| read_set_starting_at(group_number % 2, u64::from(group_number) * 512))
        .collect();
    let witness_count = 48_usize;
    let witnesses: Vec<ReadSetWitness> = (0..witness_count)
        .map(|number| {
            let group_number = usize::try_from(next_random() % 4).expect("小于 4");
            witness_numbered(&read_sets[group_number], number)
        })
        .collect();
    let index = ReadSetWitnessIndex::of_the_stored_witnesses(witnesses.clone());
    let mut built_by_adding = ReadSetWitnessIndex::empty();
    for witness in &witnesses {
        assert_ne!(
            built_by_adding.add(witness.clone()),
            ReadSetWitnessAddition::AlreadyInTheBucket,
            "摘要各不相同，每条都添"
        );
    }
    assert_eq!(
        built_by_adding, index,
        "逐条添建出的与按同一次序从读回的见证建出的相同"
    );
    assert_eq!(index.witness_count(), witness_count);
    let mut group_matched_last: Option<ReadSetGroupPosition> = None;
    let mut hits = 0_usize;
    let mut misses = 0_usize;
    for state_number in 0..60_usize {
        // 对上的状态：在挑中的那条见证的读集上摘要与它相同，别的读集上对不上；对不上的状态：哪组读集上都对不上
        let chosen_witness = (state_number % 2 == 0).then(|| {
            &witnesses[usize::try_from(next_random()).expect("u64 装得进 usize") % witness_count]
        });
        let witness_digest_of_the_state =
            |asked_ranges: &[ReadRangeOnTheCrashImage]| match chosen_witness {
                Some(chosen) if chosen.read_set.ranges.as_ref() == asked_ranges => chosen.digest,
                Some(_) | None => witness_digest_numbered(10_000 + state_number),
            };
        let scanned_in_order = witnesses
            .iter()
            .find(|witness| witness_digest_of_the_state(&witness.read_set.ranges) == witness.digest)
            .map(|witness| witness.observed_image_key);
        let matched =
            index.find_the_matching_witness(group_matched_last, witness_digest_of_the_state);
        assert_eq!(
            matched.map(|found| found.observed_image_key),
            scanned_in_order,
            "状态 {state_number}：索引与按添的次序逐条扫交回同一个镜像键"
        );
        match matched {
            Some(matched) => {
                hits += 1;
                group_matched_last = Some(matched.group);
            }
            None => misses += 1,
        }
    }
    assert_eq!(hits, 30, "偶数号状态各对上一条");
    assert_eq!(misses, 30, "奇数号状态一条都不中");
}

/// 不看崩溃镜像的那几条用例借给录入段上下文的一条空流：没有盘、没有写、没有段。
static EMPTY_POOL_OF_THE_EMPTY_FLOW: std::sync::LazyLock<MemoryPool> =
    std::sync::LazyLock::new(MemoryPool::default);
static ENUMERATION_TABLES_OF_THE_EMPTY_FLOW: std::sync::LazyLock<
    crate::crash::TornImageTablesOfTheReadBackFlow,
> = std::sync::LazyLock::new(|| {
    crate::crash::TornImageTablesOfTheReadBackFlow::of_the_read_back_flow(
        &EMPTY_POOL_OF_THE_EMPTY_FLOW,
        &[],
        &[],
    )
});

/// 录入段那几张共享表（桶的见证索引、桶的真跑锁、这一块看过几个状态，连同一条空流排好的状态），用例里建一份、借给 [`ObserverRecordingContext`]。
struct ObserverRecordingTablesForATest {
    crash_states_of_the_empty_flow: crate::crash::CrashStatesOfTheReadBackFlow<'static>,
    witnesses_of_each_bucket: WitnessIndexesOfEachBucket,
    run_locks_of_each_bucket: RunLocksOfEachBucket,
    states_looked_at_in_the_block: std::sync::atomic::AtomicU64,
}

impl ObserverRecordingTablesForATest {
    fn empty() -> Self {
        let not_expanded = |_segment_index: usize, _segment: &[usize]| {
            CrashPointReplaySegmentExpansion::NotExpanded
        };
        Self {
            crash_states_of_the_empty_flow: ENUMERATION_TABLES_OF_THE_EMPTY_FLOW
                .crash_states_planned_by(
                    &EMPTY_POOL_OF_THE_EMPTY_FLOW,
                    &[],
                    &[],
                    &not_expanded,
                    &[],
                ),
            witnesses_of_each_bucket: std::sync::Mutex::new(std::collections::HashMap::new()),
            run_locks_of_each_bucket: std::sync::Mutex::new(std::collections::HashMap::new()),
            states_looked_at_in_the_block: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// 只存见证与镜像用的录入段上下文：不建摘要器（这几条用例不看崩溃镜像），基底摘要随便一个。
    fn context<'context, 'store>(
        &'context self,
        store: &'context LibraryTakenAtEachReadOrWrite<'store>,
    ) -> ObserverRecordingContext<'context, 'store> {
        ObserverRecordingContext {
            crash_states_of_the_flow: &self.crash_states_of_the_empty_flow,
            digester: None,
            foundation_digest: [0; DIGEST_BYTES],
            witnesses_of_each_bucket: &self.witnesses_of_each_bucket,
            run_locks_of_each_bucket: &self.run_locks_of_each_bucket,
            store,
            states_looked_at_in_the_block: &self.states_looked_at_in_the_block,
        }
    }
}

/// 第 `group_number` 组读集：`range_count` 段互不相邻的单扇区段，各组的起扇区错开，段数大时读集行就大。
fn read_set_of_separate_sectors(group_number: usize, range_count: u64) -> ObservedReadSet {
    let first_sector_of_the_group =
        u64::try_from(group_number).expect("组号装得进 u64") * 4 * range_count;
    ObservedReadSet::of_the_ranges(
        (0..range_count)
            .map(|range_number| ReadRangeOnTheCrashImage {
                device: 0,
                first_sector: first_sector_of_the_group + 2 * range_number,
                end_sector: first_sector_of_the_group + 2 * range_number + 1,
            })
            .collect(),
    )
}

/// 一批里一行写占几个字节（`crate::verdict_store` 的批的字节数：键与值的字节之和）。
fn library_put_bytes(key_bytes: usize, value_bytes: usize) -> u64 {
    u64::try_from(key_bytes + value_bytes).expect("一行的字节数装得进 u64")
}

/// 观察者交回的镜像记录在这几条用例里的字节：前缀接见证摘要，见证不同记录就不同。
fn image_record_of(witness: &ReadSetWitness) -> Vec<u8> {
    [b"image observed under witness ".as_slice(), &witness.digest].concat()
}

/// 改法乙丙：往一个桶里添 2000 条见证（16 组读集、每组 1 万段，第几号见证进第「号 % 16」组）。每添一条写进库的字节不超过
/// 「一条见证行 + 一条镜像」，这组读集在这个桶里头一回出现的那一条再加一行读集行；添完的总字节不超过
/// 2000 ×（一条见证行 + 一条镜像）+ 16 × 一行读集行，与见证条数成线性，16 行读集行也确实都写了。
/// 每添一条都把整份见证清单（每条各带一份读集）重写一遍的话，第二条见证就超出这一条的界。
#[test]
fn adding_two_thousand_witnesses_to_one_bucket_writes_bytes_linear_in_the_witness_count() {
    const WITNESS_COUNT: usize = 2000;
    const READ_SET_COUNT: usize = 16;
    const RANGES_PER_READ_SET: u64 = 10_000;
    // 一次性库留着：见证存法的自证（写多少字节），要空库
    let directory = crate::crash_library::DisposableLibrary::new("observer-witness-bytes")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let mut store = directory
        .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
        .expect("开得了一次性库");
    let read_sets: Vec<ObservedReadSet> = (0..READ_SET_COUNT)
        .map(|group_number| read_set_of_separate_sectors(group_number, RANGES_PER_READ_SET))
        .collect();
    let read_set_row_bytes = observed_read_set_bytes(&read_sets[0].ranges).len();
    let image_record_bytes = image_record_of(&witness_numbered(&read_sets[0], 0)).len();
    let witness_batch_bytes_at_most = library_put_bytes(1 + 2 * DIGEST_BYTES, 2 * DIGEST_BYTES)
        + library_put_bytes(1 + DIGEST_BYTES, image_record_bytes);
    let read_set_row_bytes_at_most = library_put_bytes(1 + DIGEST_BYTES, read_set_row_bytes);
    let bucket = [0x5A; DIGEST_BYTES];
    let tables = ObserverRecordingTablesForATest::empty();
    let shared_store = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let context = tables.context(&shared_store);
    let bytes_written_so_far = || context.store.take().synced_write_tally().bytes_written;
    let bytes_before_the_witnesses = bytes_written_so_far();
    let mut read_sets_already_in_the_bucket: std::collections::HashSet<usize> =
        std::collections::HashSet::new();
    for number in 0..WITNESS_COUNT {
        let group_number = number % READ_SET_COUNT;
        let witness = witness_numbered(&read_sets[group_number], number);
        let image_record = image_record_of(&witness);
        let bytes_before_this_witness = bytes_written_so_far();
        context
            .store_the_image_with_its_witness(&bucket, witness, &image_record)
            .expect("存得下见证与镜像");
        let bytes_of_this_witness = bytes_written_so_far() - bytes_before_this_witness;
        let read_set_is_new_in_the_bucket = read_sets_already_in_the_bucket.insert(group_number);
        let bound_of_this_witness = witness_batch_bytes_at_most
            + if read_set_is_new_in_the_bucket {
                read_set_row_bytes_at_most
            } else {
                0
            };
        assert!(
                bytes_of_this_witness <= bound_of_this_witness,
                "第 {number} 条见证写了 {bytes_of_this_witness} 字节，超过这一条的界 {bound_of_this_witness}（读集{}在桶里头一回出现）：写的不止这一条",
                if read_set_is_new_in_the_bucket { "是" } else { "不是" }
            );
    }
    let bytes_of_the_witnesses = bytes_written_so_far() - bytes_before_the_witnesses;
    let witness_count_in_bytes = u64::try_from(WITNESS_COUNT).expect("条数装得进 u64");
    let read_set_count_in_bytes = u64::try_from(READ_SET_COUNT).expect("组数装得进 u64");
    let linear_bound = witness_count_in_bytes * witness_batch_bytes_at_most
        + read_set_count_in_bytes * read_set_row_bytes_at_most;
    assert!(
        bytes_of_the_witnesses <= linear_bound,
        "2000 条见证共写了 {bytes_of_the_witnesses} 字节，超过线性的界 {linear_bound}"
    );
    assert!(
        bytes_of_the_witnesses
            >= read_set_count_in_bytes
                * u64::try_from(read_set_row_bytes).expect("一行读集行的字节数装得进 u64"),
        "16 组读集各写了一行读集行（共写 {bytes_of_the_witnesses} 字节）"
    );
}

/// 改法甲：添见证是原地添进这个桶唯一那一份索引、不拷：添之前拿到的那一份引用（别的线程正拿着查的那一份）添完 2000 条照样全看得到，
/// 与此刻桶表里的是同一份，引用只有桶表与这里两份（没有添的时候拷出来的快照）。段只按组存：16 组 × 1 万段，与见证条数无关。
/// 量法：索引常驻按「各组存的段数 × 一段的字节 + 见证条数 × 两份摘要」算，2000 条时不到 50 MB（若每条见证各带一份段，
/// 2000 条就是 2000 × 1 万 × 24 字节 ≈ 458 MiB，再乘上添的时候活着的快照份数）。
#[test]
fn adding_witnesses_grows_the_one_index_of_the_bucket_in_place_without_copying_the_others() {
    const WITNESS_COUNT: usize = 2000;
    const READ_SET_COUNT: usize = 16;
    const RANGES_PER_READ_SET: u64 = 10_000;
    const RESIDENT_BYTES_LIMIT: usize = 50 << 20;
    // 一次性库留着：见证索引的自证，要空库
    let directory = crate::crash_library::DisposableLibrary::new("observer-witness-in-place")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let mut store = directory
        .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
        .expect("开得了一次性库");
    let read_sets: Vec<ObservedReadSet> = (0..READ_SET_COUNT)
        .map(|group_number| read_set_of_separate_sectors(group_number, RANGES_PER_READ_SET))
        .collect();
    let bucket = [0x5B; DIGEST_BYTES];
    let tables = ObserverRecordingTablesForATest::empty();
    let shared_store = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let context = tables.context(&shared_store);
    let held_before_adding = context
        .witnesses_of_the_bucket(&bucket)
        .expect("空库里这个桶读得出（空桶）");
    for number in 0..WITNESS_COUNT {
        let witness = witness_numbered(&read_sets[number % READ_SET_COUNT], number);
        let image_record = image_record_of(&witness);
        context
            .store_the_image_with_its_witness(&bucket, witness, &image_record)
            .expect("存得下见证与镜像");
    }
    let held_now = context
        .witnesses_of_the_bucket(&bucket)
        .expect("这个桶这一趟已经在桶表里");
    assert!(
        std::sync::Arc::ptr_eq(&held_before_adding, &held_now),
        "此刻桶表里的与添之前拿到的是同一份索引，读者拿到的不是拷贝"
    );
    drop(held_now);
    assert_eq!(
        std::sync::Arc::strong_count(&held_before_adding),
        2,
        "只有桶表与这里各拿一份引用：添的时候没有拷出别的快照"
    );
    let index = held_before_adding
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(
        index.witness_count(),
        WITNESS_COUNT,
        "添之前拿到的那一份看得到添进去的全部见证"
    );
    assert_eq!(index.read_set_group_count(), READ_SET_COUNT);
    let ranges_of_the_read_sets =
        READ_SET_COUNT * usize::try_from(RANGES_PER_READ_SET).expect("一组的段数装得进 usize");
    assert_eq!(
        index.ranges_held(),
        ranges_of_the_read_sets,
        "段只按组存一份，与见证条数无关"
    );
    let resident_bytes = index.ranges_held() * std::mem::size_of::<ReadRangeOnTheCrashImage>()
        + index.witness_count() * 2 * DIGEST_BYTES;
    assert!(
        resident_bytes < RESIDENT_BYTES_LIMIT,
        "2000 条见证的索引常驻 {resident_bytes} 字节，不到 50 MB"
    );
}

/// 被打断后重跑：添到第 6 条见证时那一批落盘之前进程被杀（测试开关拒了，一行都没写）。重开库，按桶读回的见证恰好是落了盘的那 5 条，
/// 每条引的读集读得回（不然读回就报错）、它看出的镜像也在：见证行、读集行与镜像同一批写，见证在则读集与镜像必在；被杀的那一条一样都没留下。
#[test]
fn a_witness_batch_cut_off_before_it_reached_the_disk_leaves_every_stored_witness_with_its_image() {
    const BATCHES_WRITTEN_BEFORE_THE_CRASH: usize = 5;
    // 一次性库留着：自证被杀之后读回的见证，要空库
    let library = crate::crash_library::DisposableLibrary::new("observer-witness-cut-off")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let read_sets = [read_set_starting_at(0, 64), read_set_starting_at(1, 4096)];
    let witnesses: Vec<ReadSetWitness> = (0..8)
        .map(|number| witness_numbered(&read_sets[number % 2], number))
        .collect();
    let bucket = [0x5C; DIGEST_BYTES];
    let mut witnesses_stored = 0_usize;
    {
        let mut store = library
            .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
            .expect("开得了一次性库");
        store.refuse_the_observer_witness_batches_after(
            u64::try_from(BATCHES_WRITTEN_BEFORE_THE_CRASH).expect("批数装得进 u64"),
        );
        let tables = ObserverRecordingTablesForATest::empty();
        let shared_store = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
        let context = tables.context(&shared_store);
        for witness in &witnesses {
            match context.store_the_image_with_its_witness(
                &bucket,
                witness.clone(),
                &image_record_of(witness),
            ) {
                Ok(()) => witnesses_stored += 1,
                Err(refusal) => {
                    assert!(
                        refusal.contains("ObserverWitnessBatchRefusedByTheTestSwitch"),
                        "停在测试开关拒的那一批：{refusal}"
                    );
                    break;
                }
            }
        }
    }
    assert_eq!(
        witnesses_stored, BATCHES_WRITTEN_BEFORE_THE_CRASH,
        "开关放过 5 批，一条见证一批，之后那一条被杀"
    );
    let store = library
        .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
        .expect("重开一次性库");
    let index = read_the_witness_index_of_the_bucket(&store, &bucket)
        .expect("重开库读得回这个桶的见证（每条引的读集都在）");
    assert_eq!(
        index.witness_count(),
        witnesses_stored,
        "读回的恰好是落了盘的那几条"
    );
    for witness in &witnesses[..witnesses_stored] {
        assert!(
            index.holds_the_witness_digest(&witness.digest),
            "落了盘的见证读得回"
        );
        assert_eq!(
            store
                .read_observed_image(&witness.observed_image_key)
                .expect("读得了库"),
            Some(image_record_of(witness)),
            "见证在，它看出的镜像必在"
        );
    }
    assert!(
        !index.holds_the_witness_digest(&witnesses[witnesses_stored].digest),
        "被杀的那一条见证没留下"
    );
}

/// 见证按行存、按桶读回：一个桶里 40 条见证分在三组读集上、另一个桶 3 条；重开库按桶读回的索引恰好是这个桶的那几条、那几组读集，
/// 每条见证在自己那组段上的摘要对得上、交回它看出的镜像键。
#[test]
fn witnesses_stored_row_by_row_read_back_by_their_bucket_after_reopening() {
    // 一次性库留着：自证见证按桶读回，要空库
    let library = crate::crash_library::DisposableLibrary::new("observer-witness-read-back")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let read_sets = [
        read_set_starting_at(0, 0),
        read_set_starting_at(0, 512),
        read_set_starting_at(1, 0),
    ];
    let bucket = [0x5D; DIGEST_BYTES];
    let other_bucket = [0x5E; DIGEST_BYTES];
    let witnesses: Vec<ReadSetWitness> = (0..40)
        .map(|number| witness_numbered(&read_sets[number % 3], number))
        .collect();
    let witnesses_of_the_other_bucket: Vec<ReadSetWitness> = (40..43)
        .map(|number| witness_numbered(&read_sets[0], number))
        .collect();
    {
        let mut store = library
            .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
            .expect("开得了一次性库");
        let tables = ObserverRecordingTablesForATest::empty();
        let shared_store = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
        let context = tables.context(&shared_store);
        for (bucket_of_the_witness, witness) in
            witnesses.iter().map(|witness| (&bucket, witness)).chain(
                witnesses_of_the_other_bucket
                    .iter()
                    .map(|witness| (&other_bucket, witness)),
            )
        {
            context
                .store_the_image_with_its_witness(
                    bucket_of_the_witness,
                    witness.clone(),
                    &image_record_of(witness),
                )
                .expect("存得下见证与镜像");
        }
    }
    let store = library
        .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
        .expect("重开一次性库");
    let index = read_the_witness_index_of_the_bucket(&store, &bucket).expect("读得回见证");
    assert_eq!(
        index.witness_count(),
        40,
        "这个桶的 40 条一条一行，都读得回"
    );
    assert_eq!(index.read_set_group_count(), 3, "三组读集");
    for witness in witnesses.iter().chain(&witnesses_of_the_other_bucket) {
        let matched = index.find_the_matching_witness(None, |asked_ranges| {
            if asked_ranges == witness.read_set.ranges.as_ref() {
                witness.digest
            } else {
                [0; DIGEST_BYTES]
            }
        });
        let is_in_this_bucket = witnesses.contains(witness);
        assert_eq!(
            matched.map(|found| found.observed_image_key),
            is_in_this_bucket.then_some(witness.observed_image_key),
            "这个桶的见证对得上、交回它的镜像键；另一个桶的见证不在这个桶里"
        );
    }
    let other_index =
        read_the_witness_index_of_the_bucket(&store, &other_bucket).expect("读得回见证");
    assert_eq!(other_index.witness_count(), 3);
    assert_eq!(other_index.read_set_group_count(), 1);
}
