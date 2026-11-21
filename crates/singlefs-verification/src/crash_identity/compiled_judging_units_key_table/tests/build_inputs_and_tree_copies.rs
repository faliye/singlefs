//! 构建脚本读的 crates/ 之外的文件、编进去的被监视文件摘要，与拷树重算键时拷了哪些。

use super::*;

/// 场景：被监视的文件里有 `crates/` 之外的数据文件、`crates/` 之外的 crate 的源文件、`crates/` 底下的源文件与根上的两份清单（清单现造，不读编进去的表）。
/// 预期：挑出 `crates/` 之外的那两份，按给的次序；`crates/` 底下的与根上的 `Cargo.toml`、`Cargo.lock` 不挑。
/// 判别力（`crates/mutations.tsv`）：拷树只认 `crates/` 底下的文件，这条红。
#[test]
fn only_the_watched_files_outside_the_crates_and_the_root_manifests_are_copied_beside_the_crates() {
    let watched_files = [
        ".claude/heavy-thread-default",
        "Cargo.lock",
        "Cargo.toml",
        "crates/singlefs-verification/src/heavy_threads.rs",
        "prune-server/src/lib.rs",
    ]
    .map(String::from);
    assert_eq!(
        watched_files_outside_the_crates_and_the_root_manifests(&watched_files),
        [".claude/heavy-thread-default", "prune-server/src/lib.rs"],
        "挑出 crates/ 之外的数据文件与源文件，crates/ 底下的与根上两份清单不挑"
    );
}

/// 编进去的那份表里没有 `crates/` 之外的文件，带不带 `gpu` 特性编都一样：进键的几组只按一组固定的 cfg 收（构建脚本交登记表那一份的特性，
/// 不照这一次编译开了哪些），闭包不沿依赖图上一个产品包都够不到的 crate 收（判定库服务 prune-server：验证包经 path 依赖用它，它不依赖任何产品包；
/// `singlefs_unit_graph::closure::units_of_the_crates_reaching_none_of_the_listed_packages`），各组闭包里也没有一句 `include_str!` 读到 `crates/` 之外。
/// 这里红了，拷树重算多拷的那几份就是红的输出点名的，看它们是不是新进闭包的输入。
/// 判别力（`crates/mutations.tsv`）：拷树不挑、把 `crates/` 底下的也当数据文件另拷，这条红；只键产品包的那几组照旧沿产品包够不到的 crate 收，
/// 这条红在 prune-server 的库文件上。
#[test]
fn the_build_watches_no_file_outside_the_crates_with_or_without_the_gpu_feature() {
    assert_eq!(
        data_files_outside_the_crates_the_build_read(),
        Vec::<&str>::new(),
        "构建脚本看着的 crates 之外的文件（带不带 gpu 特性编都该一份没有）"
    );
}

/// 摘要表里的文件此刻有改过的、有读不了的，都点名出来；没改的不点名。
/// 判别力（`crates/mutations.tsv`）：比摘要那一句改成一律当没改，这条红。
#[test]
fn watched_files_whose_digest_differs_from_the_build_are_named_and_unchanged_ones_are_not() {
    let unchanged_bytes = b"pub fn unchanged() {}\n".to_vec();
    let digests_text = format!(
            "{}\tcrates/sample/src/lib.rs\n{}\tcrates/sample/src/edited.rs\n{}\tcrates/sample/src/removed.rs\n",
            sha256_of(&unchanged_bytes).hexadecimal(),
            sha256_of(b"pub fn before_the_edit() {}\n").hexadecimal(),
            sha256_of(b"pub fn removed() {}\n").hexadecimal()
        );
    let read_file = |path: &str| match path {
        "crates/sample/src/lib.rs" => Some(unchanged_bytes.clone()),
        "crates/sample/src/edited.rs" => Some(b"pub fn after_the_edit() {}\n".to_vec()),
        _ => None,
    };
    assert_eq!(
        watched_files_changed_since_the_build(&digests_text, &read_file),
        [
            "crates/sample/src/edited.rs",
            "crates/sample/src/removed.rs"
        ],
        "改过的与读不了的点名、没改的不点名"
    );
}

/// 编进去的摘要表罩着判器键表点名的每份被监视文件，连同 `Cargo.lock` 与构建脚本自己（构建脚本照 `rerun-if-changed` 那一份写全）。
/// 判别力（`crates/mutations.tsv`）：构建脚本只写判器键表点名的那几份，这条红在 `Cargo.lock` 那一句。
#[test]
fn the_compiled_watched_file_digests_cover_the_lock_file_the_build_script_and_every_watched_file() {
    let digested: BTreeSet<&str> = COMPILED_WATCHED_FILE_DIGESTS_TEXT
        .lines()
        .filter_map(|line| line.split_once('\t').map(|(_digest, path)| path))
        .collect();
    for path in ["Cargo.lock", "crates/singlefs-verification/build.rs"] {
        assert!(digested.contains(path), "摘要表里有 {path}：{digested:?}");
    }
    let not_digested: Vec<&String> = compiled_judging_units_keys()
        .watched_files
        .iter()
        .filter(|path| !digested.contains(path.as_str()))
        .collect();
    assert!(
        not_digested.is_empty(),
        "判器键表点名的被监视文件都在摘要表里：{not_digested:?}"
    );
}

/// 真仓上工作区收进来的 `crates/` 之外的 crate 目录，恰是根清单 `members` 点名的 GPU 批处理验证服务 gpu-verification-server
/// 与经 path 依赖收进来的判定库服务 prune-server（验证包依赖它）。
/// 判别力（`crates/mutations.tsv`）：`crates/` 底下的包也收进来，这条红。
#[test]
fn the_crates_outside_the_crates_directory_are_the_gpu_verification_server_and_the_prune_server() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("本 crate 在仓根的 crates/ 下")
        .to_path_buf();
    assert_eq!(
        crate_directories_outside_the_crates_directory(&repository),
        BTreeSet::from([
            PathBuf::from("gpu-verification-server"),
            PathBuf::from("prune-server")
        ]),
        "工作区收进来的 crates/ 之外的 crate 目录"
    );
}

/// 场景：现造一棵小仓（空的 `crates/`、根上两份清单、`crates/` 之外两份文件：清单里的 `.claude/heavy-thread-default` 与不在清单里的
/// `.claude/not-listed`），拷树那一步（[`RepositoryCopy::of_the_repository`]）交一张只含 `.claude/heavy-thread-default` 的清单。
/// 预期：临时根底下 `.claude/heavy-thread-default` 在（上级目录现建）、内容逐字相同；不在清单里的那一份不拷；`crates/` 与根上两份清单照拷。
/// 不读活树：小仓与清单都在用例里现造。
/// 判别力（`crates/mutations.tsv`）：拷树删掉逐份拷清单里那几份的循环，这条红。
#[test]
fn copying_the_tree_copies_each_listed_file_outside_the_crates_to_the_same_path_under_the_temporary_root(
) {
    let small_repository =
        crate::crash_library::ExclusiveTemporaryDirectory::create("judging-units-small-repository")
            .expect("建得了一次性临时目录");
    std::fs::create_dir_all(small_repository.join("crates")).expect("建得了 crates/");
    std::fs::write(small_repository.join("Cargo.toml"), "[workspace]\n").expect("写得了根清单");
    std::fs::write(small_repository.join("Cargo.lock"), "version = 4\n").expect("写得了根锁文件");
    std::fs::create_dir_all(small_repository.join(".claude")).expect("建得了 .claude/");
    let listed_file_bytes = b"listed file outside the crates\n";
    std::fs::write(
        small_repository.join(".claude/heavy-thread-default"),
        listed_file_bytes,
    )
    .expect("写得了清单里那一份");
    std::fs::write(small_repository.join(".claude/not-listed"), b"not listed\n")
        .expect("写得了不在清单里的那一份");
    let copy = RepositoryCopy::of_the_repository(
        "listed-file-outside-the-crates",
        &small_repository,
        &[".claude/heavy-thread-default"],
    );
    assert_eq!(
        std::fs::read(copy.root.join(".claude/heavy-thread-default")).ok(),
        Some(listed_file_bytes.to_vec()),
        "清单里那一份拷到临时根底下同样的相对路径、逐字相同"
    );
    assert!(
        !copy.root.join(".claude/not-listed").exists(),
        "不在清单里的 crates/ 之外的文件不拷"
    );
    assert!(
        copy.root.join("crates").is_dir()
            && copy.root.join("Cargo.toml").is_file()
            && copy.root.join("Cargo.lock").is_file(),
        "crates/ 与根上两份清单照拷"
    );
}

/// 场景：现造一棵小仓，根清单 `[workspace]` 的 `members` 点名 `crates/inside-member` 与 `crates/` 之外的 `outside-member`，
/// `exclude` 点名 `outside-path-dependency`；`crates/inside-member` 经 path 依赖用 `outside-path-dependency`；`not-a-member`
/// 有 `Cargo.toml`，既不是成员也没有谁依赖它。拷树那一步（[`RepositoryCopy::of_the_repository`]）交空的另拷清单。
/// 预期：`crates/` 之外的成员与顺 path 依赖收进来的那一个连同各自的 `Cargo.toml` 拷到临时根底下同样的相对路径；
/// `not-a-member` 不拷；`crates/inside-member` 照拷。
/// 不读活树：小仓在用例里现造。
/// 判别力（`crates/mutations.tsv`）：拷树只收顺 path 依赖收进来的工作区外 crate、不收根清单 `members` 点名的 `crates/` 之外的成员，
/// 这条红在 `outside-member` 那一句。
#[test]
fn copying_the_tree_copies_the_members_and_the_path_dependencies_outside_the_crates_directory() {
    let small_repository = crate::crash_library::ExclusiveTemporaryDirectory::create(
        "judging-units-small-repository-with-members-outside-the-crates",
    )
    .expect("建得了一次性临时目录");
    let manifests = [
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"crates/inside-member\", \"outside-member\"]\nexclude = [\"outside-path-dependency\"]\n",
            ),
            (
                "crates/inside-member/Cargo.toml",
                "[package]\nname = \"inside-member\"\n\n[dependencies]\noutside-path-dependency = { path = \"../../outside-path-dependency\" }\n",
            ),
            ("outside-member/Cargo.toml", "[package]\nname = \"outside-member\"\n"),
            (
                "outside-path-dependency/Cargo.toml",
                "[package]\nname = \"outside-path-dependency\"\n",
            ),
            ("not-a-member/Cargo.toml", "[package]\nname = \"not-a-member\"\n"),
        ];
    for (relative, text) in manifests {
        let path = small_repository.join(relative);
        std::fs::create_dir_all(path.parent().expect("小仓里的清单都有上一级目录"))
            .expect("建得了清单所在的目录");
        std::fs::write(&path, text).unwrap_or_else(|error| panic!("写得了 {relative}：{error}"));
    }
    std::fs::write(small_repository.join("Cargo.lock"), "version = 4\n").expect("写得了根锁文件");
    let copy =
        RepositoryCopy::of_the_repository("members-outside-the-crates", &small_repository, &[]);
    assert!(
        copy.root.join("outside-member/Cargo.toml").is_file(),
        "根清单 members 点名的 crates/ 之外的成员连同清单照拷"
    );
    assert!(
        copy.root
            .join("outside-path-dependency/Cargo.toml")
            .is_file(),
        "顺 path 依赖收进来的 crates/ 之外的 crate 连同清单照拷"
    );
    assert!(
        !copy.root.join("not-a-member").exists(),
        "既不是成员、也没有谁经 path 依赖用到的目录不拷"
    );
    assert!(
        copy.root.join("crates/inside-member/Cargo.toml").is_file(),
        "crates/ 底下的成员照拷"
    );
}
