use super::*;

// ───────────────────────────── library：判定库里已录的同键块（附带） ─────────────────────────────

/// 每条候选流的块键在 prune-server 里那一份崩溃判定库录入表里的个数（只读，不写；连哪一处照
/// `crash_library::prune_server_route_of_the_configuration`，连不上照 prune-server 客户端的退出码退出）。
pub(super) fn run_library_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(arguments, &["--keys"], &[]) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let key_directories = match parsed.paths("--keys") {
        Ok(directories) => directories,
        Err(message) => return usage_error(&message),
    };
    let key_paths = match keys_file_paths(&key_directories, Expansion::Full) {
        Ok(key_paths) => key_paths,
        Err(message) => {
            eprintln!("读块键文件失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    let recorded = match LibraryOpenInThisProcess::open_or_share().and_then(|library| {
        let store = library.lock();
        store
            .recorded_blocks()
            .map_err(|error| format!("读录入表失败：{error:?}"))
    }) {
        Ok(recorded) => recorded,
        Err(refusal) => {
            eprintln!("开库或读录入表失败：{refusal}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    let recorded_keys: BTreeSet<VerdictBlockKey> =
        recorded.iter().map(|(key, _states)| key.clone()).collect();
    let mut output = Output::new();
    for path in &key_paths {
        let file = match read_keys_file(path, Expansion::Full) {
            Ok(file) => file,
            Err(message) => {
                eprintln!("读块键文件失败：{message}");
                return EXIT_CODE_STOP_CLAUSE_FAILED;
            }
        };
        let in_library = file
            .blocks
            .iter()
            .filter(|block| recorded_keys.contains(&block.key))
            .count();
        output.emit(&format!(
            "name=library_share flow={} family={} blocks={} in_library={in_library} fraction={}",
            file.row.label,
            file.row.family,
            file.blocks.len(),
            if file.blocks.is_empty() {
                "none".to_string()
            } else {
                fraction_text(in_library as f64 / file.blocks.len() as f64)
            },
        ));
    }
    output.emit_closing_line(
        "library_done",
        &format!(
            "flows={} recorded_blocks={}",
            key_paths.len(),
            recorded_keys.len()
        ),
    );
    EXIT_CODE_OK
}

#[cfg(test)]
mod tests;
