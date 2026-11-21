//! 装载文件与布置暂存目录、规划写启动、起虚拟机并读来宾行、本机状态读数。

use super::*;

// ============================== 宿主：附加根（照 scripts/devices/e152-stage-root.sh:59-75、:88、:133-137 的办法） ==============================

fn find_on_path(program: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")
        .map(|path_value| std::env::split_paths(&path_value).map(|directory| directory.join(program)).collect::<Vec<PathBuf>>())
        .unwrap_or_default()
        .into_iter()
        .find(|candidate_path| candidate_path.is_file())
}

fn install_file(source: &Path, destination_root: &Path, guest_path: &str, mode: u32) -> Result<(), String> {
    let target = destination_root.join(guest_path.trim_start_matches('/'));
    if target.exists() {
        return Ok(());
    }
    let parent = target.parent().expect("来宾路径有父目录");
    fs::create_dir_all(parent).map_err(|error| format!("建不了 {}：{error}", parent.display()))?;
    let resolved = fs::canonicalize(source).map_err(|error| format!("解不开 {}：{error}", source.display()))?;
    fs::copy(&resolved, &target).map_err(|error| format!("拷不了 {} → {}：{error}", resolved.display(), target.display()))?;
    fs::set_permissions(&target, fs::Permissions::from_mode(mode)).map_err(|error| format!("设不了 {} 的权限：{error}", target.display()))
}

/// `ldd` 解出的库与加载器，按宿主上的路径装进来宾的同名路径，0755（库与加载器都要执行位，`.claude/kb/vm-harness.md`）。
fn install_binary_with_libraries(source: &Path, destination_root: &Path, guest_path: &str) -> Result<usize, String> {
    let resolution = run_tool("ldd", &[&source.to_string_lossy()], None)?;
    if resolution.contains("not found") {
        return Err(format!("{} 有库解析不到：{}", source.display(), sanitized(&resolution)));
    }
    install_file(source, destination_root, guest_path, 0o755)?;
    let mut installed_libraries = 0usize;
    for line in resolution.lines() {
        let trimmed = line.trim();
        let library_path = if let Some((_name, rest)) = trimmed.split_once("=> ") {
            rest.split(" (").next().unwrap_or("").to_string()
        } else if trimmed.starts_with('/') {
            trimmed.split(" (").next().unwrap_or("").to_string()
        } else {
            continue;
        };
        if !library_path.starts_with('/') {
            continue;
        }
        install_file(Path::new(&library_path), destination_root, &library_path, 0o755)?;
        installed_libraries += 1;
    }
    Ok(installed_libraries)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StageSummary {
    file_count: usize,
    total_bytes: u64,
    pub(super) staged_selftest_passed: bool,
    staged_formatter_version: String,
    staged_partitioner_version: String,
}

fn directory_file_totals(directory: &Path) -> (usize, u64) {
    let Ok(entries) = fs::read_dir(directory) else { return (0, 0) };
    let mut totals = (0usize, 0u64);
    for entry in entries.flatten() {
        let Ok(metadata) = fs::symlink_metadata(entry.path()) else { continue };
        if metadata.is_dir() {
            let (files, bytes) = directory_file_totals(&entry.path());
            totals.0 += files;
            totals.1 += bytes;
        } else if metadata.is_file() {
            totals.0 += 1;
            totals.1 += metadata.len();
        }
    }
    totals
}

/// 写盘附加根：S4 二进制、mkfs.ext4、sfdisk 连同库与加载器，/etc/mke2fs.conf，/etc/mtab；搭完用搭好的加载器实跑三样。
pub(super) fn stage_write_root(destination: &Path) -> Result<StageSummary, String> {
    fs::create_dir_all(destination).map_err(|error| format!("建不了 {}：{error}", destination.display()))?;
    if fs::read_dir(destination).map_err(|error| error.to_string())?.next().is_some() {
        return Err(format!("{} 不是空目录", destination.display()));
    }
    let executable = std::env::current_exe().map_err(|error| format!("找不到自己的可执行文件：{error}"))?;
    install_binary_with_libraries(&executable, destination, STAGED_BINARY_GUEST_PATH)?;
    // release 配置带 debug = true，二进制约 450 MB；附加根整份进 initramfs，验盘开机还要再带设备侧日志，
    // 所以只拿掉调试信息（代码段不动，搭完用搭好的加载器实跑 selftest 核过）。
    let staged_binary = destination.join(STAGED_BINARY_GUEST_PATH.trim_start_matches('/')).to_string_lossy().to_string();
    run_tool("strip", &["--strip-debug", &staged_binary], None)?;
    let formatter = find_on_path("mkfs.ext4").ok_or("宿主上没有 mkfs.ext4（装 e2fsprogs）")?;
    install_binary_with_libraries(&formatter, destination, GUEST_FILESYSTEM_FORMATTER)?;
    let partitioner = find_on_path("sfdisk").ok_or("宿主上没有 sfdisk（装 fdisk / util-linux）")?;
    install_binary_with_libraries(&partitioner, destination, GUEST_PARTITIONER)?;
    install_file(Path::new("/etc/mke2fs.conf"), destination, "/etc/mke2fs.conf", 0o644)?;
    std::os::unix::fs::symlink("/proc/self/mounts", destination.join("etc/mtab")).map_err(|error| format!("建不了 etc/mtab：{error}"))?;
    let loader = destination.join(STAGED_LOADER_GUEST_PATH.trim_start_matches('/'));
    let library_path = format!("{}:{}", destination.join("lib/x86_64-linux-gnu").display(), destination.join("usr/lib/x86_64-linux-gnu").display());
    let staged = |guest_path: &str| destination.join(guest_path.trim_start_matches('/')).to_string_lossy().to_string();
    let loader_text = loader.to_string_lossy().to_string();
    let selftest = run_tool(&loader_text, &["--library-path", &library_path, &staged(STAGED_BINARY_GUEST_PATH), "--role", "selftest"], None);
    let staged_selftest_passed = selftest.as_ref().is_ok_and(|text| text.contains("anchor_mismatches=0") && text.contains("name=done"));
    let staged_formatter_version = run_tool(&loader_text, &["--library-path", &library_path, &staged(GUEST_FILESYSTEM_FORMATTER), "-V"], None)
        .map(|text| sanitized(text.lines().next().unwrap_or("")))
        .unwrap_or_else(|error| format!("failed:{}", sanitized(&error)));
    let staged_partitioner_version = run_tool(&loader_text, &["--library-path", &library_path, &staged(GUEST_PARTITIONER), "--version"], None)
        .map(|text| sanitized(text.lines().next().unwrap_or("")))
        .unwrap_or_else(|error| format!("failed:{}", sanitized(&error)));
    let (file_count, total_bytes) = directory_file_totals(destination);
    Ok(StageSummary { file_count, total_bytes, staged_selftest_passed, staged_formatter_version, staged_partitioner_version })
}

pub(super) fn stage_summary_line(destination: &Path, summary: &StageSummary) -> String {
    format!(
        "name=stage_root destination={} files={} bytes={} staged_selftest_passed={} staged_formatter={} staged_partitioner={}",
        destination.display(),
        summary.file_count,
        summary.total_bytes,
        summary.staged_selftest_passed,
        summary.staged_formatter_version,
        summary.staged_partitioner_version
    )
}

pub(super) fn run_stage_role(arguments: &RoleArguments) -> i32 {
    let Some(destination) = arguments.text("destination") else {
        eprintln!("  ✗ s4-stage 要 --destination <空目录>");
        return USAGE_EXIT_CODE;
    };
    let mut output = ProductLines::to_standard_output();
    let exit_code = match stage_write_root(Path::new(destination)) {
        Ok(summary) => {
            output.line(stage_summary_line(Path::new(destination), &summary));
            if summary.staged_selftest_passed {
                0
            } else {
                1
            }
        }
        Err(detail) => halt_line(&mut output, "H10", "stage", &detail),
    };
    output.finish();
    exit_code
}

/// 验盘附加根：写盘附加根逐个硬链接过来（vm-bench 会整份拷进 initramfs，硬链接只省宿主的盘），再放日志与计划。
pub(super) fn link_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| format!("建不了 {}：{error}", destination.display()))?;
    for entry in fs::read_dir(source).map_err(|error| format!("列不了 {}：{error}", source.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let target = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.is_dir() {
            link_tree(&entry.path(), &target)?;
        } else if metadata.file_type().is_symlink() {
            let pointee = fs::read_link(entry.path()).map_err(|error| error.to_string())?;
            std::os::unix::fs::symlink(pointee, &target).map_err(|error| error.to_string())?;
        } else {
            fs::hard_link(entry.path(), &target).map_err(|error| format!("硬链接不了 {}：{error}", target.display()))?;
        }
    }
    Ok(())
}

pub(super) fn copy_log_prefix(log_path: &Path, byte_count: u64, destination: &Path) -> Result<(), String> {
    let source = File::open(log_path).map_err(|error| format!("打不开 {}：{error}", log_path.display()))?;
    let mut target = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| format!("建不了 {}：{error}", destination.display()))?;
    let copied = std::io::copy(&mut source.take(byte_count), &mut target).map_err(|error| error.to_string())?;
    if copied != byte_count {
        return Err(format!("日志只拷了 {copied} 字节，要 {byte_count}"));
    }
    Ok(())
}

// ============================== 宿主：驱动 ==============================

/// 写盘开机一次：臂、块数、在开机次序里的号。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PlannedWriteBoot {
    pub(super) order: usize,
    pub(super) arm: PowerCutArm,
    pub(super) block_count: u64,
}

/// 补 12 第四段的开机次序：PC4-C → 各臂 PC4-L、PC4-NB、PC4-FD → 各臂主格 → 各臂 S4-bs10。
pub(super) fn planned_write_boots(main_arms: &[PowerCutArm]) -> Vec<PlannedWriteBoot> {
    let mut arms = vec![PowerCutArm::WriteCacheModel];
    for &arm in main_arms {
        arms.push(arm.lost_page_cache_control());
        arms.push(arm.lost_write_cache_control());
        if arm == PowerCutArm::HardenedFile {
            arms.push(PowerCutArm::HardenedFileWithoutDirectorySync);
        }
    }
    arms.extend(main_arms.iter().copied());
    arms.extend(main_arms.iter().map(|arm| arm.small_block_sampling_point()));
    arms.into_iter()
        .enumerate()
        .map(|(order, arm)| PlannedWriteBoot { order, arm, block_count: arm.registered_block_count() })
        .collect()
}

/// 一次 vm-bench 调用：环境变量与参数（参数跟在 launcher 之后，来宾里再跟在 /dev/vda 之后）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VirtualMachineRun {
    environment: Vec<(String, String)>,
    arguments: Vec<String>,
}

pub(super) fn write_boot_run(boot: &PlannedWriteBoot, log_directory: &Path, write_root: &Path, kernel: &str) -> VirtualMachineRun {
    VirtualMachineRun {
        environment: vec![
            ("VM_BLKLOGWRITES_DIR".to_string(), log_directory.display().to_string()),
            ("VM_DISKS".to_string(), "1".to_string()),
            ("VM_DISK_MB".to_string(), DISK_MEBIBYTES.to_string()),
            ("VM_LOG_MB".to_string(), LOG_DISK_MEBIBYTES.to_string()),
            ("VM_MEM".to_string(), WRITE_BOOT_MEMORY_MEBIBYTES.to_string()),
            ("VM_CPUS".to_string(), WRITE_BOOT_PROCESSORS.to_string()),
            ("VM_TIMEOUT".to_string(), WRITE_BOOT_TIMEOUT_SECONDS.to_string()),
            ("VM_EXTRA_ROOT".to_string(), write_root.display().to_string()),
            ("SINGLEFS_KERNEL".to_string(), kernel.to_string()),
        ],
        arguments: vec![
            "--role".to_string(),
            "s4-write-guest".to_string(),
            "--arm".to_string(),
            boot.arm.index().to_string(),
            "--blocks".to_string(),
            boot.block_count.to_string(),
        ],
    }
}

pub(super) fn verify_boot_run(verify_root: &Path, kernel: &str) -> VirtualMachineRun {
    VirtualMachineRun {
        environment: vec![
            ("VM_DISKS".to_string(), "1".to_string()),
            ("VM_MEM".to_string(), VERIFY_BOOT_MEMORY_MEBIBYTES.to_string()),
            ("VM_TIMEOUT".to_string(), VERIFY_BOOT_TIMEOUT_SECONDS.to_string()),
            ("VM_EXTRA_ROOT".to_string(), verify_root.display().to_string()),
            ("SINGLEFS_KERNEL".to_string(), kernel.to_string()),
        ],
        arguments: vec![
            "--role".to_string(),
            "s4-verify-guest".to_string(),
            "--log".to_string(),
            GUEST_LOG_PATH.to_string(),
            "--plan".to_string(),
            GUEST_PLAN_PATH.to_string(),
        ],
    }
}

fn vm_bench_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/devices/vm-bench.sh"))
}

pub(super) fn run_description(run: &VirtualMachineRun, launcher: &Path) -> String {
    let environment: Vec<String> = run.environment.iter().map(|(key, value)| format!("{key}={value}")).collect();
    format!(
        "environment={} command=bash,{},{},{}",
        environment.join(";"),
        vm_bench_path().display(),
        launcher.display(),
        run.arguments.join(",")
    )
}

/// 起 vm-bench，把它的标准输出整份读到 EOF 再解析（不在读的循环里打时间戳）；返回来宾的 E7RESULT 行（去掉前缀）与退出码。
pub(super) fn execute_virtual_machine(run: &VirtualMachineRun, launcher: &Path) -> (Vec<String>, Option<i32>) {
    let mut command = Command::new("bash");
    command.arg(vm_bench_path()).arg(launcher).args(&run.arguments).stdout(Stdio::piped()).stderr(Stdio::inherit());
    for (key, value) in &run.environment {
        command.env(key, value);
    }
    let Ok(output) = command.output() else { return (Vec::new(), None) };
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let guest_lines = text
        .lines()
        .filter_map(|line| line.find("E7RESULT ").map(|position| line[position + "E7RESULT ".len()..].trim_end().to_string()))
        .collect();
    (guest_lines, output.status.code())
}

/// 来宾的行转进产物：在 `name=…` 之后插上 `boot=<号>`。
pub(super) fn tagged_guest_line(boot_label: &str, guest_line: &str) -> String {
    match guest_line.split_once(' ') {
        Some((name_field, rest)) => format!("{name_field} boot={boot_label} {rest}"),
        None => format!("{guest_line} boot={boot_label}"),
    }
}

pub(super) fn available_memory_bytes() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .unwrap_or_default()
        .lines()
        .find_map(|line| line.strip_prefix("MemAvailable:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|kibibytes| kibibytes.parse::<u64>().ok())
        .map_or(0, |kibibytes| kibibytes * 1024)
}

pub(super) fn available_disk_bytes(directory: &Path) -> u64 {
    let directory_text = directory.to_string_lossy().to_string();
    run_tool("df", &["-B1", "--output=avail", &directory_text], None)
        .ok()
        .and_then(|text| text.lines().last().and_then(|line| line.trim().parse().ok()))
        .unwrap_or(0)
}

/// 补 5.3「宿主资源」：机器上有没有别的虚机或性能测量（按 /proc/*/cmdline 的第一个参数认，不按模式杀任何东西）。
pub(super) fn interfering_processes() -> Vec<String> {
    let own = std::process::id().to_string();
    let Ok(entries) = fs::read_dir("/proc") else { return Vec::new() };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.chars().all(|character| character.is_ascii_digit()) || name == own {
            continue;
        }
        let Ok(command_line) = fs::read(entry.path().join("cmdline")) else { continue };
        let words: Vec<String> = command_line.split(|&byte| byte == 0).map(|word| String::from_utf8_lossy(word).to_string()).collect();
        let program = words.first().map(|word| word.rsplit('/').next().unwrap_or("").to_string()).unwrap_or_default();
        let script = words.get(1).map(|word| word.rsplit('/').next().unwrap_or("").to_string()).unwrap_or_default();
        let interferes = program.starts_with("qemu-system")
            || program == "fio"
            || program.starts_with("e152-file-system-benchmark")
            || script == "vm-bench.sh";
        if interferes {
            found.push(format!("{name}:{program}"));
        }
    }
    found
}

/// `${TMPDIR:-/tmp}/e162-s4-<pid>/`，Drop 时整目录删掉；E162_KEEP_SCRATCH=1 时留下并打印路径。
pub(super) struct ScratchDirectory {
    pub(super) path: PathBuf,
}

impl ScratchDirectory {
    pub(super) fn create() -> Result<ScratchDirectory, String> {
        let base = std::env::var_os("TMPDIR").map_or_else(|| PathBuf::from("/tmp"), PathBuf::from);
        let path = base.join(format!("e162-s4-{}", std::process::id()));
        fs::create_dir(&path).map_err(|error| format!("建不了 {}：{error}", path.display()))?;
        Ok(ScratchDirectory { path })
    }
}

impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        if std::env::var("E162_KEEP_SCRATCH").is_ok_and(|value| value == "1") {
            eprintln!("E162_KEEP_SCRATCH=1：现场留在 {}", self.path.display());
        } else {
            let _cleanup_outcome = fs::remove_dir_all(&self.path);
        }
    }
}
