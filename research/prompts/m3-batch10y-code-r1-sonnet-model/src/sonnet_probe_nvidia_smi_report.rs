//! 正推腿探针（草稿副本里的，不进仓）：把本机真跑的 `nvidia-smi -q -d MEMORY,PIDS` 喂给新解析，与 `--query-gpu` 对。
use singlefs_harness::gpu_memory_ledger::{
    card_memory_readings_of_one_nvidia_smi_report, pci_bus_identifier_without_the_domain,
    CardMemoryReading, NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS, OccupiedByThisProcess,
};
use std::process::Command;

fn run(arguments: &[&str]) -> String {
    let output = Command::new("nvidia-smi").args(arguments).output().expect("nvidia-smi 起得来");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// (总线号, 序号, 空闲, 总量, 保留)
fn query() -> Vec<(String, u32, u64, u64, u64)> {
    run(&["--query-gpu=pci.bus_id,index,memory.free,memory.total,memory.reserved", "--format=csv,noheader,nounits"])
        .lines()
        .map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            (pci_bus_identifier_without_the_domain(f[0]), f[1].parse().unwrap(), f[2].parse().unwrap(), f[3].parse().unwrap(), f[4].parse().unwrap())
        })
        .collect()
}

#[test]
fn live_report_free_equals_query_gpu_memory_free() {
    for attempt in 1..=6 {
        let before = query();
        let report = run(&NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS);
        let after = query();
        if before != after {
            println!("attempt {attempt}: 两次 --query-gpu 之间卡上的数变了，重来");
            continue;
        }
        let buses: Vec<String> = before.iter().map(|c| c.0.clone()).collect();
        let readings = card_memory_readings_of_one_nvidia_smi_report(&report, std::process::id(), &buses, &|_| false)
            .expect("真报告解析得出");
        println!("PARSED {readings:?}");
        println!("QUERY  {before:?}");
        assert_eq!(readings.len(), before.len());
        for (reading, (bus, _index, free, total, reserved)) in readings.iter().zip(&before) {
            assert_eq!(reading.free_mebibytes, *free, "总线 {bus} 的空闲");
            assert_eq!(reading.total_less_reserved_mebibytes, total - reserved, "总线 {bus} 的总量减保留");
            assert_eq!(reading.occupied_by_this_process, OccupiedByThisProcess::NotListed, "本进程没起设备：没列");
        }
        println!("LIVE_MATCH attempt={attempt} cards={}", readings.len());
        return;
    }
    panic!("六次都碰上卡上的数在变，核不了");
}

fn parse(report: &str, buses: &[&str]) -> Result<Vec<CardMemoryReading>, String> {
    let buses: Vec<String> = buses.iter().map(|b| pci_bus_identifier_without_the_domain(b)).collect();
    card_memory_readings_of_one_nvidia_smi_report(report, 4242, &buses, &|_| false)
}

#[test]
fn unrecognised_outputs_are_errors_not_silent_zeroes() {
    let real = std::fs::read_to_string(concat!(env!("HOME"), "/.local/share/singlefs/work/m3-batch10y-code-r1-sonnet/smi-q-1.txt")).unwrap();
    let cases: Vec<(&str, String)> = vec![
        ("空串", String::new()),
        ("乱码", "this is not an nvidia-smi report\nfoo : bar\n".to_string()),
        ("缺 Reserved（老驱动的样子）", real.replace("        Reserved                                       : 497 MiB\n", "")),
        ("Free 写 N/A", real.replacen("32110 MiB", "N/A", 1)),
        ("Free 缺单位", real.replacen("32110 MiB", "32110", 1)),
        ("没有 FB 小节", real.replace("FB Memory Usage", "Something Else")),
    ];
    for (name, report) in &cases {
        let parsed = parse(report, &["00000000:01:00.0"]);
        println!("CASE {name}: {parsed:?}");
        assert!(parsed.is_err(), "{name} 应该报错");
    }
    // 报告里卡有而点名的卡没有
    println!("CASE 点名的卡不在报告里: {:?}", parse(&real, &["00000000:99:00.0"]));
    assert!(parse(&real, &["00000000:99:00.0"]).is_err());
    // 进程号认不出
    let bad_pid = real.replacen("    Processes                                          : None\n", "    Processes\n        Process ID                                     : abc\n", 1);
    println!("CASE 进程号认不出: {:?}", parse(&bad_pid, &["00000000:01:00.0"]));
    assert!(parse(&bad_pid, &["00000000:01:00.0"]).is_err());
    // 本进程那一行占的量读不出：落在没列
    let na_used = real.replacen(
        "    Processes                                          : None\n",
        "    Processes\n        Process ID                                     : 4242\n            Used GPU Memory                            : N/A\n",
        1,
    );
    println!("CASE 本进程占的量 N/A: {:?}", parse(&na_used, &["00000000:01:00.0"]));
}
