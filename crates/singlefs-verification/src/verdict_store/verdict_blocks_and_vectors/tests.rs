//! 判定块与判定向量的测试：存了读回、关了重开、编号由服务给、同一把键下换块、编号到 u32 的头、按判法单元指纹分行。

use super::*;

#[test]
fn stored_block_reads_back_byte_for_byte_identical() {
    let library = TestLibrary::new("read-back");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let torn = store
        .register_verdict_vector(&verdict_vector(b"torn"))
        .expect("登记");
    let lost = store
        .register_verdict_vector(&verdict_vector(b"lost"))
        .expect("登记");
    let block_key = block_key_named("stream", 65536);
    let block = block_of(&[clean, torn, clean, lost, lost]);
    assert_eq!(
        store
            .store_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT, &block, &[])
            .expect("存块"),
        VerdictBlockStoring::WrittenAndSynced
    );
    assert_eq!(
        store
            .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读块"),
        Some(block),
        "读回来的块要与存进去的逐个编号相同"
    );
    let stored_value_bytes = library
        .raw_value(
            &VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .to_key_bytes()
                .0,
        )
        .expect("块在库里");
    assert_eq!(
        stored_value_bytes,
        vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2],
        "库里的块值要是每个状态一个 4 字节大端的编号"
    );
}

/// 登记两种判定向量、存一块之后关库，新开一个 store 重开同一份库：读任何一块之前，两个编号就查得到原来那两种判定向量
/// （开库时整张读回按编号的判定向量表；开库之后从服务取不认得的编号只在读块时走，这里还没走过）；读回块与违例，
/// 再登记同一种拿回原来的编号，新登记的接着往下给。
/// 判别力（`crates/mutations.tsv`）：重开时不读判定向量表（装载的两张表都是空的），读块之前按编号查不到，这条红。
#[test]
fn blocks_and_verdict_vector_numbers_survive_closing_and_reopening() {
    let library = TestLibrary::new("reopen");
    let block_key = block_key_named("stream", 0);
    let (clean, torn, block) = {
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let torn = store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记");
        let block = block_of(&[torn, clean, torn]);
        store
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block,
                &[StateOffsetInBlock(2)],
            )
            .expect("存块");
        (clean, torn, block)
    };
    let mut store = library.open();
    assert_eq!(
        (
            store.verdict_vector_of_number(clean),
            store.verdict_vector_of_number(torn)
        ),
        (
            Some(&verdict_vector(b"clean")),
            Some(&verdict_vector(b"torn"))
        ),
        "重开之后、读任何一块之前，按编号就查得到：开库时读回了按编号的判定向量表"
    );
    assert_eq!(
        store.registered_verdict_vectors(),
        [verdict_vector(b"clean"), verdict_vector(b"torn")].as_slice(),
        "开库读回的按编号表依次是关库前登记的那两种"
    );
    assert_eq!(
        store
            .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读块"),
        Some(block)
    );
    assert_eq!(
        store
            .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读违例"),
        vec![StoredViolation {
            offset_in_block: StateOffsetInBlock(2),
            verdict_vector_number: torn,
        }]
    );
    assert_eq!(
        store.verdict_vector_of_number(clean),
        Some(&verdict_vector(b"clean"))
    );
    assert_eq!(
        store.verdict_vector_of_number(torn),
        Some(&verdict_vector(b"torn"))
    );
    assert_eq!(
        store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记"),
        torn,
        "重开之后再登记同一种判定向量要拿到原来的编号"
    );
    assert_eq!(
        store
            .register_verdict_vector(&verdict_vector(b"lost"))
            .expect("登记")
            .as_u32(),
        2,
        "重开之后新登记的判定向量接着原来的编号往下给"
    );
}

/// 绿的判定向量（一个红项都没有）是空字节，登记在 0 号时按编号那张表里那一行的值为空（`03 ‖ 0 → 空值`）：登记过绿与一种红之后关库，
/// 新开一个 store 重开同一份库不报错，绿仍是 0 号、红仍是 1 号，再登记两种拿回原来的编号。
/// 判别力（`crates/mutations.tsv`）：前缀扫把判定向量表里的空值也当删除记号跳过（重开时看不到 0 号，判连号报编号不是从 0 起），这条红。
#[test]
fn a_green_verdict_vector_registered_before_closing_is_still_number_zero_after_reopening() {
    let library = TestLibrary::new("green-survives-reopening");
    let green = crate::crash_judge_conclusions::encoded_verdict_of_the_red_items(&[]);
    let red = verdict_vector(b"I-1.1 seeded red");
    {
        let mut store = library.open();
        assert_eq!(
            store
                .register_verdict_vector(&green)
                .expect("登记绿")
                .as_u32(),
            0,
            "空库里头一种登记的绿拿到 0 号"
        );
        assert_eq!(
            store
                .register_verdict_vector(&red)
                .expect("登记红")
                .as_u32(),
            1
        );
    }
    assert_eq!(
        library.raw_value(
            &VerdictStoreKey::VerdictVectorContentByNumber(VerdictVectorNumber(0))
                .to_key_bytes()
                .0
        ),
        Some(Vec::new()),
        "按编号那张表里 0 号那一行在、值是绿的空字节"
    );
    let mut reopened = library.open();
    assert_eq!(
        reopened.registered_verdict_vectors(),
        [green.clone(), red.clone()].as_slice(),
        "重开读回两种：0 号是绿、1 号是红"
    );
    assert_eq!(
        reopened
            .register_verdict_vector(&green)
            .expect("重开之后再登记绿")
            .as_u32(),
        0,
        "重开之后再登记绿拿回 0 号"
    );
    assert_eq!(
        reopened
            .register_verdict_vector(&red)
            .expect("重开之后再登记红")
            .as_u32(),
        1,
        "重开之后再登记红拿回 1 号"
    );
}

/// 两个 store 都在空库上开着，一个登记了绿（0 号、值为空），另一个再登记绿：它认领 0 号时服务答已经登记过，读回那一行的空值就是绿，
/// 拿到 0 号、不报错；再登记一种红接着拿 1 号。
/// 判别力（`crates/mutations.tsv`）：成批点读把判定向量表里的空值也读成不在（报「答已经登记过，读回来却没有」），这条红。
#[test]
fn another_store_registering_green_after_it_was_registered_gets_number_zero() {
    let library = TestLibrary::new("green-registered-by-another-store");
    let green = crate::crash_judge_conclusions::encoded_verdict_of_the_red_items(&[]);
    let mut store_registering_green = library.open();
    let mut store_registering_green_again = library.open();
    assert_eq!(
        store_registering_green
            .register_verdict_vector(&green)
            .expect("先登记绿")
            .as_u32(),
        0
    );
    assert_eq!(
        store_registering_green_again
            .register_verdict_vector(&green)
            .expect("另一个 store 再登记绿")
            .as_u32(),
        0,
        "另一个 store 登记同一种绿拿到同一个 0 号"
    );
    assert_eq!(
        store_registering_green_again
            .register_verdict_vector(&verdict_vector(b"I-1.1 seeded red"))
            .expect("再登记红")
            .as_u32(),
        1,
        "认得了 0 号之后新登记的接着拿 1 号"
    );
}

/// 一个 store 在空库上开着，另一个登记了绿与一种红、存了一块引这两个编号：先开的那个读这一块时从服务取来不认得的 0、1 两号，
/// 0 号那一行的空值读成绿，读回的块与存的相同。
/// 判别力（`crates/mutations.tsv`）：成批点读把判定向量表里的空值也读成不在（0 号取不来，读块报「存着的判定向量编号在这个库里没登记过」），这条红。
#[test]
fn a_block_referring_to_green_stored_by_another_store_reads_back_with_green_fetched_from_the_service(
) {
    let library = TestLibrary::new("green-fetched-from-the-service");
    let green = crate::crash_judge_conclusions::encoded_verdict_of_the_red_items(&[]);
    let block_key = block_key_named("stream", 0);
    let opened_before = library.open();
    let block = {
        let mut storing = library.open();
        let green_number = storing.register_verdict_vector(&green).expect("登记绿");
        let red_number = storing
            .register_verdict_vector(&verdict_vector(b"I-1.1 seeded red"))
            .expect("登记红");
        let block = block_of(&[green_number, red_number, green_number]);
        storing
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block,
                &[StateOffsetInBlock(1)],
            )
            .expect("存块");
        block
    };
    assert_eq!(
        opened_before
            .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读块时从服务取来不认得的编号"),
        Some(block),
        "先开的 store 读回别的 store 存的块"
    );
    assert_eq!(
        opened_before.verdict_vector_of_number(VerdictVectorNumber(0)),
        Some(&green),
        "从服务取来的 0 号是绿的空字节"
    );
}

#[test]
fn registering_the_same_verdict_vector_twice_returns_the_same_number() {
    let library = TestLibrary::new("register-twice");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let torn = store
        .register_verdict_vector(&verdict_vector(b"torn"))
        .expect("登记");
    let entries_after_two_registrations = store.every_stored_entry();
    assert_eq!(
        store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记"),
        clean,
        "第二次登记同一种判定向量要拿到第一次的编号"
    );
    assert_ne!(clean, torn);
    assert_eq!(
        store.every_stored_entry(),
        entries_after_two_registrations,
        "第二次登记同一种判定向量不写库"
    );
}

#[test]
fn two_different_block_keys_do_not_overwrite_each_other() {
    let library = TestLibrary::new("distinct-keys");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let torn = store
        .register_verdict_vector(&verdict_vector(b"torn"))
        .expect("登记");
    let first_key = block_key_of(FINGERPRINT_BYTE, "stream", PLAN_HASH_BYTE, 0);
    let other_fingerprint = block_key_of(0x33, "stream", PLAN_HASH_BYTE, 0);
    let other_name = block_key_of(FINGERPRINT_BYTE, "streamx", PLAN_HASH_BYTE, 0);
    let other_plan = block_key_of(FINGERPRINT_BYTE, "stream", 0x44, 0);
    let other_start = block_key_of(FINGERPRINT_BYTE, "stream", PLAN_HASH_BYTE, 1);
    let keys_and_blocks = [
        (first_key, block_of(&[clean])),
        (other_fingerprint, block_of(&[torn])),
        (other_name, block_of(&[clean, torn])),
        (other_plan, block_of(&[torn, clean])),
        (other_start, block_of(&[torn, torn])),
    ];
    for (block_key, block) in &keys_and_blocks {
        assert_eq!(
            store
                .store_verdict_block(block_key, &TEST_JUDGING_UNITS_FINGERPRINT, block, &[])
                .expect("存块"),
            VerdictBlockStoring::WrittenAndSynced,
            "块键 {block_key:?} 之前没存过"
        );
    }
    for (block_key, block) in &keys_and_blocks {
        assert_eq!(
            store
                .read_verdict_block(block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读块")
                .as_ref(),
            Some(block),
            "块键 {block_key:?} 下读回的要是它自己的块"
        );
    }
}

#[test]
fn violations_are_found_again_by_their_block_key() {
    let library = TestLibrary::new("violations");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let torn = store
        .register_verdict_vector(&verdict_vector(b"torn"))
        .expect("登记");
    let lost = store
        .register_verdict_vector(&verdict_vector(b"lost"))
        .expect("登记");
    let first_key = block_key_named("stream", 0);
    let key_with_a_longer_name = block_key_named("stream-longer", 0);
    let next_block_key = block_key_named("stream", 4);
    store
        .store_verdict_block(
            &first_key,
            &TEST_JUDGING_UNITS_FINGERPRINT,
            &block_of(&[clean, lost, torn, clean]),
            &[StateOffsetInBlock(2), StateOffsetInBlock(1)],
        )
        .expect("存块");
    store
        .store_verdict_block(
            &key_with_a_longer_name,
            &TEST_JUDGING_UNITS_FINGERPRINT,
            &block_of(&[torn, clean]),
            &[StateOffsetInBlock(0)],
        )
        .expect("存块");
    store
        .store_verdict_block(
            &next_block_key,
            &TEST_JUDGING_UNITS_FINGERPRINT,
            &block_of(&[clean, clean]),
            &[],
        )
        .expect("存块");
    assert_eq!(
        store
            .violations_in_block(&first_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读违例"),
        vec![
            StoredViolation {
                offset_in_block: StateOffsetInBlock(1),
                verdict_vector_number: lost,
            },
            StoredViolation {
                offset_in_block: StateOffsetInBlock(2),
                verdict_vector_number: torn,
            },
        ],
        "按块键查回这一块自己的违例，按块内序号排，编号取块里那个状态的"
    );
    assert_eq!(
        store
            .violations_in_block(&key_with_a_longer_name, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读违例"),
        vec![StoredViolation {
            offset_in_block: StateOffsetInBlock(0),
            verdict_vector_number: torn,
        }]
    );
    assert_eq!(
        store
            .violations_in_block(&next_block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读违例"),
        Vec::new(),
        "没有违例的块查回空的"
    );
}

#[test]
fn different_block_under_a_stored_block_key_is_refused_and_the_library_is_unchanged() {
    let library = TestLibrary::new("different-block");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let torn = store
        .register_verdict_vector(&verdict_vector(b"torn"))
        .expect("登记");
    let block_key = block_key_named("stream", 0);
    let block = block_of(&[clean, torn]);
    store
        .store_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT, &block, &[])
        .expect("存块");
    let entries_before = store.every_stored_entry();
    let different_block = store.store_verdict_block(
        &block_key,
        &TEST_JUDGING_UNITS_FINGERPRINT,
        &block_of(&[torn, torn]),
        &[],
    );
    assert!(
            matches!(
                different_block,
                Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { .. })
            ),
            "同一个块键下两份都不红而换一个块要拒：{different_block:?}"
        );
    assert_eq!(
        store.every_stored_entry(),
        entries_before,
        "拒了就一个字都不写"
    );
}

/// 记红、撤绿：同一个块键下先存一份不红的，再存一份块相同、头一个状态违例的（红），再存回不红的。
/// 预期：两次都交「换上」，读回是新存的那一份；红换绿之后违例行一行不剩。
/// 判别力（`crates/mutations.tsv`）：判定块那一处两份不同一律拒（不看红不红），这条红。
#[test]
fn a_red_verdict_block_replaces_a_green_one_under_its_key_and_a_green_one_replaces_it_back() {
    let library = TestLibrary::new("red-verdict-block-replaces");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let torn = store
        .register_verdict_vector(&verdict_vector(b"torn"))
        .expect("登记");
    let block_key = block_key_named("stream", 0);
    store
        .store_verdict_block(
            &block_key,
            &TEST_JUDGING_UNITS_FINGERPRINT,
            &block_of(&[clean, clean]),
            &[],
        )
        .expect("存块");
    assert_eq!(
        store
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[torn, clean]),
                &[StateOffsetInBlock(0)],
            )
            .expect("绿换红照换上"),
        VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed
    );
    assert_eq!(
        store
            .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读")
            .expect("存着"),
        block_of(&[torn, clean])
    );
    assert_eq!(
        store
            .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读")
            .len(),
        1,
        "记红：违例行一行"
    );
    assert_eq!(
        store
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[clean, clean]),
                &[],
            )
            .expect("红换绿照换上"),
        VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed
    );
    assert!(
        store
            .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读")
            .is_empty(),
        "红换绿：违例行一行不剩"
    );
}

#[test]
fn storing_the_identical_block_again_is_accepted_and_changes_nothing() {
    let library = TestLibrary::new("identical-block");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let torn = store
        .register_verdict_vector(&verdict_vector(b"torn"))
        .expect("登记");
    let block_key = block_key_named("stream", 0);
    let block = block_of(&[torn, clean]);
    store
        .store_verdict_block(
            &block_key,
            &TEST_JUDGING_UNITS_FINGERPRINT,
            &block,
            &[StateOffsetInBlock(0)],
        )
        .expect("存块");
    let entries_before = store.every_stored_entry();
    assert_eq!(
        store
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block,
                &[StateOffsetInBlock(0)]
            )
            .expect("再存同一块"),
        VerdictBlockStoring::AlreadyStoredIdentically,
        "续跑时重写逐字节相同的块要放行"
    );
    assert_eq!(store.every_stored_entry(), entries_before);
}

/// 判定向量编号加宽到 u32：第 257 种登记得上、拿到编号 256，块与违例里存得下、关库重开读得回，之后的接着往下给。
/// 判别力（`crates/mutations.tsv`）：编号退回一个字节宽（满 256 种就拒），这条红。
#[test]
fn the_257th_distinct_verdict_vector_gets_number_256_and_survives_closing_and_reopening() {
    let library = TestLibrary::new("vector-past-one-byte");
    let block_key = block_key_named("stream", 0);
    let the_257th_vector = verdict_vector(b"the 257th kind");
    let (clean, the_257th) = {
        let mut store = library.open();
        for expected_number in 0..256u32 {
            let number = store
                .register_verdict_vector(&verdict_vector(&expected_number.to_be_bytes()))
                .expect("前 256 种都登记得上");
            assert_eq!(number.as_u32(), expected_number);
        }
        let clean = store
            .register_verdict_vector(&verdict_vector(&0u32.to_be_bytes()))
            .expect("登记过的照旧");
        let the_257th = store
            .register_verdict_vector(&the_257th_vector)
            .expect("第 257 种登记得上");
        assert_eq!(the_257th.as_u32(), 256, "第 257 种拿到编号 256");
        assert_eq!(
            library.raw_value(
                &VerdictStoreKey::VerdictVectorContentByNumber(the_257th)
                    .to_key_bytes()
                    .0
            ),
            Some(the_257th_vector.0.clone()),
            "按编号那张表里编号 256 的键在、值是第 257 种的字节"
        );
        store
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[clean, the_257th]),
                &[StateOffsetInBlock(1)],
            )
            .expect("块里存得下编号 256");
        (clean, the_257th)
    };
    let mut store = library.open();
    assert_eq!(
        store
            .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读块"),
        Some(block_of(&[clean, the_257th])),
        "重开之后块里的编号 256 读得回"
    );
    assert_eq!(
        store
            .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读违例"),
        vec![StoredViolation {
            offset_in_block: StateOffsetInBlock(1),
            verdict_vector_number: the_257th,
        }],
        "重开之后违例里的编号 256 读得回"
    );
    assert_eq!(
        store.verdict_vector_of_number(the_257th),
        Some(&the_257th_vector)
    );
    assert_eq!(
        store
            .register_verdict_vector(&verdict_vector(b"the 258th kind"))
            .expect("第 258 种登记得上")
            .as_u32(),
        257,
        "重开之后新登记的接着往下给"
    );
}

/// 编号一直给到 u32 的最大值：已经登记了 u32::MAX 种时下一种拿到 u32::MAX，存成 4 字节大端、读回相同；2^32 种都登记过了才没有下一个编号。
#[test]
fn verdict_vector_numbers_reach_the_largest_u32_and_stop_only_after_it() {
    let registered_count_before_the_largest =
        usize::try_from(u32::MAX).expect("这个包只在 usize 至少 32 位的平台上编");
    let largest =
        VerdictVectorNumber::next_after_registered_count(registered_count_before_the_largest)
            .expect("已经登记了 u32::MAX 种时下一种还有编号");
    assert_eq!(largest.as_u32(), u32::MAX);
    assert_eq!(largest.to_stored_bytes(), [0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(
        stored_verdict_vector_numbers(&largest.to_stored_bytes()),
        Some(vec![u32::MAX]),
        "存成的字节读回同一个编号"
    );
    assert_eq!(
        VerdictVectorNumber::next_after_registered_count(registered_count_before_the_largest + 1),
        None,
        "2^32 种都登记过了，没有下一个编号"
    );
}

/// 块值引了一个服务的按编号表里也没有的编号：读成判定块那一张表、那一把键不合布局（错在引它的那一块）。
/// 判别力（`crates/mutations.tsv`）：从服务取编号时缺了那一号就报按编号表不合布局（报的表与键是按编号表的那一把，不是这一块），这条红。
#[test]
fn stored_block_naming_an_unregistered_verdict_vector_number_reads_as_a_layout_mismatch() {
    let library = TestLibrary::new("unregistered-number");
    let mut store = library.open();
    store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let block_key = block_key_named("stream", 0);
    // 绕过 store_verdict_block 直接写一个点名编号 1 的块值
    library.write_raw_rows(vec![(
        VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .to_key_bytes()
            .0,
        vec![0, 0, 0, 0, 0, 0, 0, 1],
    )]);
    let read = store.read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT);
    assert!(
        matches!(
            read,
            Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                table: VerdictStoreTable::VerdictBlocks,
                reason: "存着的判定向量编号在这个库里没登记过",
                ..
            })
        ),
        "只登记了编号 0，块值里有 1 要判不合布局：{read:?}"
    );
}

/// 块值的字节数不是 4（一个判定向量编号的字节数）的整数倍：读成不合布局，不丢掉零头照读、不把几段字节拼成一个编号。
#[test]
fn stored_block_value_whose_length_is_not_a_multiple_of_four_bytes_reads_as_a_layout_mismatch() {
    let library = TestLibrary::new("block-value-not-a-multiple-of-four-bytes");
    let mut store = library.open();
    store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let block_key = block_key_named("stream", 0);
    // 绕过 store_verdict_block 直接写一个五个字节的块值
    library.write_raw_rows(vec![(
        VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .to_key_bytes()
            .0,
        vec![0, 0, 0, 0, 0],
    )]);
    let read = store.read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT);
    assert!(
        matches!(
            read,
            Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                table: VerdictStoreTable::VerdictBlocks,
                reason: "存着的块值的字节数不是 4 字节编号的整数倍",
                ..
            })
        ),
        "五个字节的块值要判不合布局：{read:?}"
    );
}

#[test]
#[should_panic(expected = "违例状态的块内序号必须在块里")]
fn violating_state_outside_the_block_is_a_caller_bug() {
    let library = TestLibrary::new("violation-outside");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let block_key = block_key_named("stream", 0);
    let _outcome = store.store_verdict_block(
        &block_key,
        &TEST_JUDGING_UNITS_FINGERPRINT,
        &block_of(&[clean, clean]),
        &[StateOffsetInBlock(2)],
    );
}

#[test]
#[should_panic(expected = "块里的判定向量编号必须是这个库登记过的")]
fn verdict_vector_number_this_library_never_registered_is_a_caller_bug() {
    let library = TestLibrary::new("foreign-number");
    let mut store = library.open();
    store
        .register_verdict_vector(&verdict_vector(b"clean"))
        .expect("登记");
    let block_key = block_key_named("stream", 0);
    let _outcome = store.store_verdict_block(
        &block_key,
        &TEST_JUDGING_UNITS_FINGERPRINT,
        &block_of(&[VerdictVectorNumber(1)]),
        &[],
    );
}

/// 同一块在两个判法单元指纹下各存一份判定：各自读得回、互不相扰（复用只认同一版本），判定块表列出两行，判法单元指纹表记得两版的描述。
#[test]
fn the_same_block_judged_under_two_judging_units_fingerprints_keeps_both_rows_and_reads_only_its_own_fingerprint(
) {
    let library = TestLibrary::new("two-judge-versions");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let red = store
        .register_verdict_vector(&verdict_vector(b"red"))
        .expect("登记");
    let block_key = block_key_named("stream", 0);
    store
        .store_verdict_block(
            &block_key,
            &TEST_JUDGING_UNITS_FINGERPRINT,
            &block_of(&[clean, clean]),
            &[],
        )
        .expect("存旧版本的块");
    assert_eq!(
        store
            .read_verdict_block(&block_key, &OTHER_JUDGING_UNITS_FINGERPRINT)
            .expect("读块"),
        None,
        "另一个判法单元指纹下这一块还没判：旧版本的判定不复用"
    );
    store
        .store_verdict_block(
            &block_key,
            &OTHER_JUDGING_UNITS_FINGERPRINT,
            &block_of(&[clean, red]),
            &[StateOffsetInBlock(1)],
        )
        .expect("存新版本的块：与旧版本判得不同也不算冲突");
    assert_eq!(
        store
            .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读块"),
        Some(block_of(&[clean, clean])),
        "旧版本的行留着"
    );
    assert_eq!(
        store
            .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
            .expect("读违例"),
        Vec::new(),
        "旧版本没有违例"
    );
    assert_eq!(
        store
            .violations_in_block(&block_key, &OTHER_JUDGING_UNITS_FINGERPRINT)
            .expect("读违例")
            .len(),
        1,
        "新版本一处违例"
    );
    assert_eq!(
        store.stored_verdict_block_keys().expect("列块"),
        vec![
            (block_key.clone(), TEST_JUDGING_UNITS_FINGERPRINT),
            (block_key.clone(), OTHER_JUDGING_UNITS_FINGERPRINT),
        ],
        "同一块两版两行，按版本字节排"
    );
    let composition_of = |version: JudgingUnitsFingerprint, unit: &str| {
        JudgingUnitsKey::of_the_parts(
            version.0,
            vec![JudgingUnitFingerprint {
                unit: unit.to_string(),
                fingerprint: [version.0[0]; SHA256_DIGEST_BYTES],
            }],
        )
    };
    store
        .store_judging_units_composition(&composition_of(
            TEST_JUDGING_UNITS_FINGERPRINT,
            "old_judge",
        ))
        .expect("登组成清单");
    store
        .store_judging_units_composition(&composition_of(
            OTHER_JUDGING_UNITS_FINGERPRINT,
            "new_judge",
        ))
        .expect("登组成清单");
    assert_eq!(
        store.judging_units_fingerprints().expect("列指纹"),
        vec![
            TEST_JUDGING_UNITS_FINGERPRINT.0,
            OTHER_JUDGING_UNITS_FINGERPRINT.0
        ]
    );
    assert_eq!(
        store
            .read_judging_units_composition(&OTHER_JUDGING_UNITS_FINGERPRINT.0)
            .expect("读组成清单"),
        Some(
            composition_of(OTHER_JUDGING_UNITS_FINGERPRINT, "new_judge")
                .composition()
                .to_vec()
        )
    );
    // 另一个客户端（另一个进程开同一份库的样子）读到的是同一批行：不经并库、不经导入
    let another_client = library.open();
    assert_eq!(
        another_client
            .stored_verdict_block_keys()
            .expect("列块")
            .len(),
        2,
        "另一个客户端读得到两版两行"
    );
    assert_eq!(
        another_client
            .judging_units_fingerprints()
            .expect("列指纹")
            .len(),
        2,
        "另一个客户端读得到两份组成清单"
    );
}
