use super::*;

#[test]
fn sha256_matches_the_standard_test_vectors() {
    assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(sha256_hex(&[b'a'; 1_000]).len(), 63 + 1);
}

/// 第四段续跑单测用的小项：F8 大文件改写删，数据单元数不同就是不同的项（一项算得快）。
fn small_rewrite_item(data_units: u64) -> SegmentFourItem {
    SegmentFourItem::HistoryCell {
        history: History::LargeFileRewriteAndDelete { data_units, rewrites: 0, overwrite_before_unlink: true, batch: 1 },
        cell: write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0),
    }
}

/// 单测自己的项库：给的那一份内存库（几把代码键共用一份就传同一份），组成清单照 device-code-key.py 打的那几行写（一行 code_key）。
fn test_library(store: &DeviceLibrary, code_key: &str) -> SegmentFourLibrary {
    let composition = format!(
        "{COMPOSITION_FORM_LINE}\njudged\t2026-10-15\nhost\t本机\ncode_key\te7_index_bench::segment_four_item_lines\t{code_key}\n\
             unit\te7_index_bench::segment_four_item_lines\tfn\te7_index_bench::<bin>::segment_four_item_lines\tscripts/experiments/e178.rs:1-2\t{code_key}\n"
    );
    segment_four_library_from_composition(store.clone(), &composition).expect("单测写的组成清单读得出")
}

fn library_options(library: &SegmentFourLibrary) -> SegmentFourOptions {
    SegmentFourOptions { library: library.clone() }
}

fn item_files_in(library: &SegmentFourLibrary) -> usize {
    library.store.count_entries("items/", ".out")
}

/// 登记修订 4 之六单测表「项键随代码键变」（M27 抓）：同一项、两个代码键 → 项键不等；项键的绝对值另由 python 的 hashlib 算出钉住。
#[test]
fn item_key_changes_with_the_code_key() {
    let item = SegmentFourItem::DirectRollbackConstruction;
    assert_eq!(segment_four_item_key("A", item), "15b1a55c608c669c7e110a594d8fb4165ad050c30b8f84c510d749d2da62269e");
    assert_eq!(segment_four_item_key("B", item), "70c33db17155490d2820d10bcdd2af7f335a311f3c74255fdb9009ac18698c0f");
    assert_ne!(segment_four_item_key("A", item), segment_four_item_key("B", item));
}

/// 「项键随每个参数字段变」（M23 抓）：只差 θ、只差容量、只差借臂、只差删序、只差批的项 → 项键两两不等（同一个代码键）。
#[test]
fn item_key_changes_with_every_parameter_field() {
    let item = |borrow_arm, leaf_capacity, merge_threshold, order, batch| SegmentFourItem::HistoryCell {
        history: History::ManyObjectsWithData { inode_count: 300, order, batch },
        cell: write_path_cell(borrow_arm, leaf_capacity, merge_threshold),
    };
    let items = [
        item(BorrowArm::RaiseSeparator, 16, 4, DeletionOrder::Ascending, 144),
        item(BorrowArm::RaiseSeparator, 16, 8, DeletionOrder::Ascending, 144),
        item(BorrowArm::RaiseSeparator, 32, 4, DeletionOrder::Ascending, 144),
        item(BorrowArm::RenewIdentity, 16, 4, DeletionOrder::Ascending, 144),
        item(BorrowArm::RaiseSeparator, 16, 4, DeletionOrder::EvenThenOdd, 144),
        item(BorrowArm::RaiseSeparator, 16, 4, DeletionOrder::Ascending, 1),
    ];
    let keys: BTreeSet<String> = items.iter().map(|listed_item| segment_four_item_key("A", *listed_item)).collect();
    assert_eq!(keys.len(), items.len(), "六项只差一个参数字段，项键要两两不等");
}

/// 「项键不看序号与名字」（M24 抓）：清单前插一项再跑，原有各项全部复用，只现判插进来的那一项。
#[test]
fn inserting_an_item_in_front_reuses_every_item_already_judged() {
    let library = test_library(&DeviceLibrary::in_memory(), "a1");
    let first = compute_or_reuse_the_items(&[small_rewrite_item(3), small_rewrite_item(4)], 2, &library_options(&library));
    let second = compute_or_reuse_the_items(&[small_rewrite_item(5), small_rewrite_item(3), small_rewrite_item(4)], 2, &library_options(&library));
    assert_eq!((first.computed, first.reused), (2, 0));
    assert_eq!((second.computed, second.reused), (1, 2), "前插一项之后，原有两项的项键不变、读回");
}

/// 「别的代码键写的项不读回」（M25 抓）：代码键 A 下写库、B 下再跑（同一份库） → 全部现判，A 写的项留着（库里两份并存）。
#[test]
fn items_written_under_another_code_key_are_not_read_back() {
    let store = DeviceLibrary::in_memory();
    let library_under_the_original_code_key = test_library(&store, "a1");
    let library_under_another_code_key = test_library(&store, "b2");
    let items = [small_rewrite_item(3), small_rewrite_item(4)];
    compute_or_reuse_the_items(&items, 2, &library_options(&library_under_the_original_code_key));
    let second = compute_or_reuse_the_items(&items, 2, &library_options(&library_under_another_code_key));
    let files = item_files_in(&library_under_another_code_key);
    assert_eq!((second.computed, second.reused), (2, 0), "B 下不读 A 写的项");
    assert_eq!(files, 4, "A 写的两份留着，B 又写了两份");
}

/// 「键对不上的项数」（M26 抓）：代码键 A 跑完、B 再跑 → 成功句报键对不上 = 项数，那一句点名变了的单元。
#[test]
fn another_code_key_reports_every_item_as_mismatched() {
    let store = DeviceLibrary::in_memory();
    let library_under_the_original_code_key = test_library(&store, "a1");
    let library_under_another_code_key = test_library(&store, "b2");
    let items = [small_rewrite_item(3), small_rewrite_item(4)];
    let first = compute_or_reuse_the_items(&items, 2, &library_options(&library_under_the_original_code_key));
    let second = compute_or_reuse_the_items(&items, 2, &library_options(&library_under_another_code_key));
    assert_eq!(first.mismatched, 0, "空库上第一趟没有上一趟可比");
    assert_eq!(second.mismatched, 2);
    let sentence = second.changed_units.expect("键对不上时有那一句");
    assert!(sentence.contains("改了 e7_index_bench::<bin>::segment_four_item_lines"), "{sentence}");
}

/// 续跑（修订 2 之十三、修订 4 之六单测表「续跑拼出同一份产物」）：小输入上第一遍现判、第二遍全读回，两遍拼出的产物逐行相同；现判的照样按项键写回。
#[test]
fn resuming_from_the_item_files_assembles_the_same_product() {
    let items = vec![small_rewrite_item(3), SegmentFourItem::DirectRollbackConstruction];
    let library = test_library(&DeviceLibrary::in_memory(), "c3");
    let fresh = compute_or_reuse_the_items(&items, 2, &library_options(&library));
    let files_after_fresh = item_files_in(&library);
    let reused = compute_or_reuse_the_items(&items, 1, &library_options(&library));
    assert_eq!((fresh.computed, fresh.reused, reused.computed, reused.reused), (2, 0, 0, 2));
    assert_eq!(files_after_fresh, 2, "现判的照样按项键写回");
    assert_eq!(assemble_segment_four(&items, &fresh.item_lines), assemble_segment_four(&items, &reused.item_lines));
}

/// 「给了代码键就要转交的管道」：带 --code-key-file 却没经 device-code-key.py run 起（环境里没有管道） → 参数解析报错（main 照它退 2），不退回本地库。
#[test]
fn code_key_without_the_relay_is_refused() {
    let composition_path = std::env::temp_dir().join(format!("e178-segment-four-relay-missing-{}.txt", std::process::id()));
    std::fs::write(&composition_path, format!("{COMPOSITION_FORM_LINE}\ncode_key\te7_index_bench::segment_four_item_lines\ta1\n")).expect("写得进测试的组成清单");
    let refused = parse_with(&["--code-key-file", &composition_path.to_string_lossy()], &test_environment_without_relay());
    std::fs::remove_file(&composition_path).expect("删得掉测试的组成清单");
    let refused = refused.expect_err("没有转交的管道要报错");
    assert!(refused.contains("device-code-key.py run"), "{refused}");
}

/// 组成清单读法：第一行不是 form、code_key 不是恰好一行、入口不是 segment_four_item_lines，都报错；judged 与 host 读进 meta 用。
#[test]
fn the_composition_needs_the_form_line_and_exactly_one_entry_key() {
    let store = DeviceLibrary::in_memory();
    let library = test_library(&store, "abcdef0123456789");
    assert_eq!((library.code_key.as_str(), library.judged.as_str(), library.host.as_str()), ("abcdef0123456789", "2026-10-15", "本机"));
    let entry_line = "code_key\te7_index_bench::segment_four_item_lines\tabcdef0123456789\n";
    assert!(segment_four_library_from_composition(store.clone(), entry_line).is_err(), "缺 form 行");
    assert!(segment_four_library_from_composition(store.clone(), &format!("{COMPOSITION_FORM_LINE}\n")).is_err(), "缺 code_key 行");
    assert!(segment_four_library_from_composition(store.clone(), &format!("{COMPOSITION_FORM_LINE}\n{entry_line}{entry_line}")).is_err(), "两行 code_key");
    assert!(segment_four_library_from_composition(store, &format!("{COMPOSITION_FORM_LINE}\ncode_key\te7_index_bench::main\tabcdef0123456789\n")).is_err(), "入口不对");
}
