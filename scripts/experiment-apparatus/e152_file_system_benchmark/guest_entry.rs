//! 来宾入口：参数解析、环境准备、配置行、内核模块装载、负载与失败报告。

use super::*;

#[derive(Debug, PartialEq, Eq)]
struct GuestArguments {
    device_paths: Vec<String>,
    pub(super) configuration: Configuration,
    pub(super) round: u32,
}

fn parse_guest_arguments(arguments: &[String]) -> Result<GuestArguments, String> {
    let [device_paths @ .., configuration_text, round_text] = arguments else {
        return Err("用法：/bench <设备...> <配置> <轮次>".to_string());
    };
    let configuration =
        Configuration::parse(configuration_text).ok_or_else(|| format!("不认识的配置 {configuration_text}"))?;
    let round = round_text
        .parse::<u32>()
        .map_err(|error| format!("轮次 {round_text} 不是数：{error}"))?;
    if device_paths.len() != configuration.device_count() {
        return Err(format!(
            "{} 要 {} 块盘，拿到 {} 块",
            configuration.label(),
            configuration.device_count(),
            device_paths.len()
        ));
    }
    Ok(GuestArguments {
        device_paths: device_paths.to_vec(),
        configuration,
        round,
    })
}

fn prepare_guest_environment() -> Result<(), GuestFailure> {
    // zpool 给整盘分区之后要看得到分区节点，mdev -s 只扫一次，devtmpfs 由内核自己维护（登记第三节）
    run_checked(&CommandLine::new("/bin/mount", &["-t", "devtmpfs", "devtmpfs", "/dev"]), "mount_devtmpfs")?;
    // mdadm 在 /run/mdadm 里记它建过的阵列（登记第十一节）；initramfs 里没有 /run
    std::fs::create_dir_all("/run/mdadm").map_err(|error| GuestFailure::new("run_directory", error.to_string()))?;
    std::fs::create_dir_all(MOUNT_POINT).map_err(|error| GuestFailure::new("mount_point", error.to_string()))
}

fn emit_configuration_line(parsed: &GuestArguments, emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map_err(|error| GuestFailure::new("kernel_release", error.to_string()))?;
    let memory_information_text =
        std::fs::read_to_string("/proc/meminfo").map_err(|error| GuestFailure::new("memory_total", error.to_string()))?;
    let memory_total = memory_information_text
        .lines()
        .find(|line| line.starts_with("MemTotal:"))
        .map_or_else(|| "NA".to_string(), single_token);
    let mut device_sizes = Vec::new();
    for device_path in &parsed.device_paths {
        device_sizes.push(format!("{}:{}", device_name(device_path)?, read_device_bytes(device_path)?));
    }
    emitter.emit(
        "configuration",
        &format!("kernel={} memory_total={memory_total} devices={}", kernel.trim(), device_sizes.join(",")),
    );
    Ok(())
}

fn load_kernel_modules(configuration: Configuration, emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let Some(order_name) = configuration.module_order_name() else {
        return Ok(());
    };
    let order_path = format!("{MODULE_ORDER_DIRECTORY}/{order_name}.order");
    let order = std::fs::read_to_string(&order_path).map_err(|error| GuestFailure::new("module_order", format!("{order_path}：{error}")))?;
    let mut loaded = 0_u32;
    for module_path in order.lines().filter(|line| !line.trim().is_empty()) {
        run_checked(&CommandLine::new("/bin/insmod", &[module_path]), "insmod")?;
        loaded += 1;
    }
    emitter.emit("modules", &format!("order={order_name} loaded={loaded}"));
    Ok(())
}

fn run_guest_workload(parsed: &GuestArguments, emitter: &mut Emitter) -> Result<(), GuestFailure> {
    prepare_guest_environment()?;
    emit_configuration_line(parsed, emitter)?;
    load_kernel_modules(parsed.configuration, emitter)?;
    match parsed.configuration {
        Configuration::RawDevice | Configuration::RawSoftwareRaidMirror => {
            run_raw_device_round(parsed.configuration, &parsed.device_paths, emitter)
        }
        Configuration::Singlefs => run_singlefs(&parsed.device_paths, emitter),
        Configuration::FourthExtendedFileSystem
        | Configuration::Xfs
        | Configuration::FlashFriendlyFileSystem
        | Configuration::Btrfs
        | Configuration::Bcachefs
        | Configuration::Zfs
        | Configuration::FourthExtendedFileSystemOnSoftwareRaidMirror
        | Configuration::XfsOnSoftwareRaidMirror
        | Configuration::FlashFriendlyFileSystemOnSoftwareRaidMirror
        | Configuration::BtrfsMirror
        | Configuration::BcachefsTwoReplicas
        | Configuration::ZfsMirror => run_file_system_round(parsed.configuration, &parsed.device_paths, emitter),
    }
}

/// 登记第八节第 1、4 条：失败的那一轮要附来宾 dmesg 末尾。
fn report_failure(failure: &GuestFailure, emitter: &mut Emitter) {
    emitter.emit(
        "failure",
        &format!("stage={} detail={}", failure.stage, single_token(&failure.detail)),
    );
    match Command::new("/bin/dmesg").output() {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = text.lines().collect();
            let start = lines.len().saturating_sub(DMESG_TAIL_LINES);
            for (offset, line) in lines[start..].iter().enumerate() {
                emitter.emit("dmesg", &format!("line={} text={}", start + offset, single_token(line)));
            }
        }
        Err(error) => emitter.emit("dmesg_unavailable", &format!("detail={}", single_token(&error.to_string()))),
    }
}

pub(super) fn run_guest(arguments: &[String]) -> ExitCode {
    let parsed = match parse_guest_arguments(arguments) {
        Ok(parsed) => parsed,
        Err(reason) => {
            eprintln!("{reason}");
            return ExitCode::from(2);
        }
    };
    std::env::set_var("PATH", "/usr/sbin:/usr/bin:/sbin:/bin");
    std::env::set_var("LD_LIBRARY_PATH", "/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu");
    // initramfs 里没有 udev，mdadm 不许等它（登记第十一节）
    std::env::set_var("MDADM_NO_UDEV", "1");
    let mut emitter = Emitter {
        emitted: 0,
        context: format!("configuration={} round={}", parsed.configuration.label(), parsed.round),
    };
    let exit_code = match run_guest_workload(&parsed, &mut emitter) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            report_failure(&failure, &mut emitter);
            ExitCode::from(1)
        }
    };
    emitter.finish();
    std::io::stdout().flush().expect("控制台写不出去，结果行也就没了");
    exit_code
}

#[cfg(test)]
#[path = "guest_entry_tests.rs"]
mod tests;
