//! 验证包模块：crash_identity、crash_library、verdict_store
//! 崩溃判定库只走 prune-server：每条用例建一份一次性库（`crash_library::DisposableLibrary`：它自己在这个进程里起一份临时 prune-server，
//! 库是那份临时服务里的一段名字空间，各条用例各一份、各从空库起），经 `CrashLibrary::open`（一次性库设成
//! 这个进程开的那一份）与 `DisposableLibrary::open_store`（一个新的客户端）读写它；读写不成一律交回错误（`PruneServerFailureHandling::ReturnTheError`），
//! 不退出进程。临时服务起不来当场失败，不跳过；不写真服务。几个客户端各开同一份一次性库，就是几个进程、两台
//! 同时读写服务那一份的样子：一个客户端写下的，别的客户端马上读得到（不经并库、不经导入）。

use singlefs_verification::crash_identity::{ConclusionKind, ConclusionVersion};
use singlefs_verification::crash_library::{CrashLibrary, CrashLibraryClass, DisposableLibrary};
use singlefs_verification::sha256::DIGEST_BYTES;
use singlefs_verification::verdict_store::{
    BlockCountVector, BlockStartStateIndex, ConclusionBlockRecord, EncodedVerdictVector,
    EnumerationPlanHash, JudgingDevice, NodeInputDigest, PruneServerFailureHandling,
    StateOffsetInBlock, VerdictBlock, VerdictBlockKey, VerdictStore, VerdictVectorNumber,
};

/// 用例存结论用的那一把版本。
const TEST_CONCLUSION_VERSION: ConclusionVersion = ConclusionVersion([7; DIGEST_BYTES]);

fn block_key_with_the_content_byte(content_byte: u8) -> VerdictBlockKey {
    VerdictBlockKey {
        node_input_digest: NodeInputDigest([content_byte; DIGEST_BYTES]),
        enumeration_plan_hash: EnumerationPlanHash([9; DIGEST_BYTES]),
        block_start: BlockStartStateIndex(0),
    }
}

/// 一块一个状态、编号是 `number` 的结论；`is_red` 时那一个状态记违例。
fn conclusion_of_one_state(number: VerdictVectorNumber, is_red: bool) -> ConclusionBlockRecord {
    ConclusionBlockRecord {
        block: VerdictBlock::new(vec![number]).expect("一块一个状态"),
        violating_states: if is_red {
            vec![StateOffsetInBlock(0)]
        } else {
            Vec::new()
        },
        count_vector: BlockCountVector::default(),
        judged_on: JudgingDevice::Cpu,
        recovery_words: None,
    }
}

/// 临时服务里的一份一次性库；临时 prune-server 由它自己在这个进程里起，起不来当场失败。
fn disposable_library(label: &str) -> DisposableLibrary {
    // 一次性库留着：存放布局经服务的自证，各条从空库起
    DisposableLibrary::new(label).unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// 开一次性库的一个新客户端（另一个进程开同一份库的样子）。
fn another_client_of(library: &DisposableLibrary) -> VerdictStore {
    library
        .open_store(PruneServerFailureHandling::ReturnTheError)
        .unwrap_or_else(|refusal| panic!("开不了一次性库：{refusal}"))
}

/// 一块读回的头一个状态的判定向量（经读它的那个客户端认编号）。
fn verdict_vector_of_the_first_state(
    reader: &VerdictStore,
    block_key: &VerdictBlockKey,
) -> Option<Vec<u8>> {
    let stored = reader
        .read_conclusion_block(
            block_key,
            &ConclusionKind::ImageJudgement,
            &TEST_CONCLUSION_VERSION,
        )
        .expect("读得了服务")?;
    let number = *stored
        .block
        .verdict_vector_number_of_each_state()
        .first()
        .expect("一块一个状态");
    reader
        .verdict_vector_of_number(number)
        .map(|vector| vector.0.clone())
}

/// 经 `CrashLibrary::open`（这个进程开的那一份，设成一次性库）存一块结论：不放手，另一个客户端马上读得回同一份判定向量
/// （写一块就是写进服务那一份，没有先写本机、收尾再并回这一步）。
/// 判别力：存块只记在这个进程里、不写进服务（另一个客户端读不到），这条红。
#[test]
fn conclusion_stored_through_the_crash_library_is_read_back_by_another_client_at_once() {
    let library = disposable_library("prune-server-lands");
    let _the_library_of_this_process = library.make_it_the_library_of_this_process();
    let crash_library =
        CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration).expect("开得了库");
    let block_key = block_key_with_the_content_byte(0x51);
    {
        let mut store = crash_library.lock();
        let number = store
            .register_verdict_vector(&EncodedVerdictVector(b"lands-in-the-server".to_vec()))
            .expect("登记得了判定向量");
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &TEST_CONCLUSION_VERSION,
                &conclusion_of_one_state(number, false),
            )
            .expect("存得了结论");
    }
    assert_eq!(
        verdict_vector_of_the_first_state(&another_client_of(&library), &block_key),
        Some(b"lands-in-the-server".to_vec()),
        "库还开着、没放手，另一个客户端就读得回这一块与它的判定向量"
    );
}

/// 一个客户端存下一块的结论：另一个客户端问这一块缺哪几种，答一种都不缺。
/// 判别力：问「缺不缺」读的不是服务那一份（读不到别的客户端写的），答「缺」，这条红。
#[test]
fn block_concluded_by_one_client_reads_as_present_from_another() {
    let library = disposable_library("prune-server-present");
    let mut writer = another_client_of(&library);
    let number = writer
        .register_verdict_vector(&EncodedVerdictVector(b"present".to_vec()))
        .expect("服务里登记得了判定向量");
    writer
        .store_conclusion_block(
            &block_key_with_the_content_byte(0x61),
            &ConclusionKind::ImageJudgement,
            &TEST_CONCLUSION_VERSION,
            &conclusion_of_one_state(number, false),
        )
        .expect("服务里存得了结论");
    assert_eq!(
        another_client_of(&library)
            .conclusions_missing_for_block(
                &block_key_with_the_content_byte(0x61),
                &[(ConclusionKind::ImageJudgement, TEST_CONCLUSION_VERSION)],
            )
            .expect("问得了服务"),
        Vec::new(),
        "服务里有的那一种不缺"
    );
}

/// 两个客户端（两个客户端身份）同时登记同一种新的判定向量，各存一块指到它：服务里这一种只有一个编号，两块都指到它。
/// 判别力：登记不在服务里认领编号（各按自己的表续号），两份拿到的编号不同、同一种判定向量登记两个编号，这条红。
#[test]
fn two_clients_registering_one_verdict_vector_get_one_number_in_the_prune_server() {
    let library = disposable_library("prune-server-one-number");
    let mut first = another_client_of(&library);
    let mut second = another_client_of(&library);
    let verdict_vector = EncodedVerdictVector(b"registered-by-two-clients".to_vec());
    let (first_number, second_number) = std::thread::scope(|scope| {
        let first_registration = scope.spawn(|| {
            first
                .register_verdict_vector(&verdict_vector)
                .expect("第一个客户端登记得了")
        });
        let second_registration = scope.spawn(|| {
            second
                .register_verdict_vector(&verdict_vector)
                .expect("第二个客户端登记得了")
        });
        (
            first_registration.join().expect("第一个登记线程没 panic"),
            second_registration.join().expect("第二个登记线程没 panic"),
        )
    });
    assert_eq!(
        first_number, second_number,
        "同一种判定向量两个客户端拿到同一个编号"
    );
    for (store, content_byte) in [(&mut first, 0x81_u8), (&mut second, 0x82_u8)] {
        store
            .store_conclusion_block(
                &block_key_with_the_content_byte(content_byte),
                &ConclusionKind::ImageJudgement,
                &TEST_CONCLUSION_VERSION,
                &conclusion_of_one_state(first_number, false),
            )
            .expect("存得了结论");
    }
    let reader = another_client_of(&library);
    for content_byte in [0x81_u8, 0x82_u8] {
        assert_eq!(
            verdict_vector_of_the_first_state(
                &reader,
                &block_key_with_the_content_byte(content_byte)
            ),
            Some(verdict_vector.0.clone()),
            "两块都指到同一种判定向量"
        );
    }
    assert_eq!(
        reader
            .registered_verdict_vectors()
            .iter()
            .filter(|registered| **registered == verdict_vector)
            .count(),
        1,
        "服务里这一种判定向量只登记了一个编号"
    );
}

/// 另一个客户端在这一份开库之后登记、存下的那一块：这一份读它时编号还不认得，从服务取来，读出的是那个客户端登记的那一种判定向量。
/// 判别力：读到不认得的编号不去服务取（照旧报编号没登记过），这条红。
#[test]
fn number_registered_by_another_client_after_opening_reads_as_its_verdict_vector() {
    let library = disposable_library("prune-server-fetched");
    let reader = another_client_of(&library);
    let mut writer = another_client_of(&library);
    let number = writer
        .register_verdict_vector(&EncodedVerdictVector(b"registered-after-opening".to_vec()))
        .expect("服务里登记得了");
    writer
        .store_conclusion_block(
            &block_key_with_the_content_byte(0xD1),
            &ConclusionKind::ImageJudgement,
            &TEST_CONCLUSION_VERSION,
            &conclusion_of_one_state(number, false),
        )
        .expect("服务里存得了");
    assert_eq!(
        verdict_vector_of_the_first_state(&reader, &block_key_with_the_content_byte(0xD1)),
        Some(b"registered-after-opening".to_vec()),
        "读出的是另一个客户端登记的那一种"
    );
}

/// 这一份开库之后另一个客户端登记了一种、存了一块；这一份再登记另一种：在服务里往后认领到下一个编号，不与那一种撞号，
/// 读那一块读出的仍是那个客户端登记的那一种。
/// 判别力：登记不在服务里认领编号（照这一份自己的表续号，拿到和那一种同一个号），读那一块读出这一份登记的那一种，这条红。
#[test]
fn registering_after_another_client_takes_the_next_number_in_the_prune_server() {
    let library = disposable_library("prune-server-next-number");
    let mut late = another_client_of(&library);
    let mut early = another_client_of(&library);
    let early_number = early
        .register_verdict_vector(&EncodedVerdictVector(b"registered-early".to_vec()))
        .expect("先登记的那一种登记得了");
    early
        .store_conclusion_block(
            &block_key_with_the_content_byte(0xE1),
            &ConclusionKind::ImageJudgement,
            &TEST_CONCLUSION_VERSION,
            &conclusion_of_one_state(early_number, false),
        )
        .expect("服务里存得了");
    let late_number = late
        .register_verdict_vector(&EncodedVerdictVector(b"registered-late".to_vec()))
        .expect("后登记的那一种登记得了");
    assert_ne!(late_number, early_number, "后登记的那一种拿到别的编号");
    assert_eq!(
        verdict_vector_of_the_first_state(&late, &block_key_with_the_content_byte(0xE1)),
        Some(b"registered-early".to_vec()),
        "那一块读出的仍是先登记的那一种"
    );
}

/// 服务里那一份先存一块红的、再存同键不红的：照「有红整份换」换上，旧的违例行在服务里删掉（写成删除记号），另一个客户端读回没有违例。
/// 判别力：删除记号读回时不当成不在（空值照旧算一行违例），这条红。
#[test]
fn red_conclusion_replaced_in_the_prune_server_leaves_no_violation_behind() {
    let library = disposable_library("prune-server-replaced");
    let mut writer = another_client_of(&library);
    let number = writer
        .register_verdict_vector(&EncodedVerdictVector(b"replaced".to_vec()))
        .expect("登记得了判定向量");
    let block_key = block_key_with_the_content_byte(0x91);
    for is_red in [true, false] {
        writer
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &TEST_CONCLUSION_VERSION,
                &conclusion_of_one_state(number, is_red),
            )
            .expect("存得了结论");
    }
    assert!(
        !another_client_of(&library)
            .conclusion_block_has_violations(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &TEST_CONCLUSION_VERSION,
            )
            .expect("读得了服务"),
        "换上不红的之后服务里这一块没有违例"
    );
}
