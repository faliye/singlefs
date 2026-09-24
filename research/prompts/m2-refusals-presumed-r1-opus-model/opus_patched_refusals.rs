//! 攻方腿自己提的改法，**只在副本上量过、被攻过零轮**：
//! Q1-F1（抬 F 时从现行根的指针上读实例表）恒开；R1-F1（树表 0 条的一版上不写行）与「跳过空池形状判定」由环境变量开。

mod common;

use common::{build_pool, file_content, format_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_writable, raise_rollback_floor, ShadowLedger};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// Q1-F1：同一条历史（mkfs 进程里 A + 三次覆盖写），改法之后抬 F 到 3 做得成，报得出上限与空发布。
#[test]
fn opus_q1_fix_lets_the_raise_go_through_on_the_mkfs_process_version() {
    let mut pool = build_pool("opus-q1-fix");
    for seed in [3usize, 5, 7] {
        overwrite_in_process(&mut pool, &content_of(4100, seed), InstanceGeneration(1));
    }
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(3),
        ShadowLedger::On,
    );
    match &raised {
        Ok(raised) => println!(
            "[Q1-F1] 抬 F 到 3 做成了：上限 {}，空发布 {} 次（txg {:?}），回收 {} 个落点",
            raised.ceiling.0,
            raised.publishes.len(),
            raised
                .publishes
                .iter()
                .map(|publish| publish.root.checkpoint_txg.0)
                .collect::<Vec<_>>(),
            raised.reclaimed.len()
        ),
        Err(error) => println!("[Q1-F1] 还是被拒：{error:?}"),
    }
    assert!(raised.is_ok(), "改法之后抬 F 做得成");
}

/// R1-F1 / R4：`OPUS_SKIP_R1` 开着时第二次可写挂载不再撞 R1 那道拒绝，下一道墙是哪一道照实打印；
/// 两道都跳开（`OPUS_SKIP_R4`）时挂载做得成，再试第一个文件版本，看第三道墙。
#[test]
fn opus_r1_fix_shows_the_next_wall_on_the_second_mount_of_a_formatted_pool() {
    let skip_r1 = std::env::var_os("OPUS_SKIP_R1").is_some();
    let skip_r4 = std::env::var_os("OPUS_SKIP_R4").is_some();
    println!("[R1-F1] OPUS_SKIP_R1 = {skip_r1}，OPUS_SKIP_R4 = {skip_r4}");
    let mut formatted = format_pool("opus-r1-fix");
    {
        let mut devices = formatted.reopen_recorded();
        mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
        formatted.devices = Some(devices);
    }
    let mut devices = formatted.reopen_recorded();
    let second = mount_writable(&parameters(), &mut devices);
    match &second {
        Ok(mounted) => println!(
            "[R1-F1] 第二次可写挂载做成了：实例 {}，写行 {} 条，写行那次 txg {}，暖机 {} 次（txg {:?}）",
            mounted.output.instance.0,
            mounted.output.rows_written.len(),
            mounted.output.row_publish.root().checkpoint_txg.0,
            mounted.output.warm_up_publishes.len(),
            mounted
                .output
                .warm_up_publishes
                .iter()
                .map(|publish| publish.root().checkpoint_txg.0)
                .collect::<Vec<_>>()
        ),
        Err(error) => println!("[R1-F1] 第二次可写挂载还是被拒：{error:?}"),
    }
    if !(skip_r1 && skip_r4) {
        formatted.devices = Some(devices);
        return;
    }
    let mounted = second.expect("两道都跳开之后挂载做得成");
    let mut allocator = mounted.allocator;
    let current = mounted.current;
    let current_root = *current.root();
    let current_record_bytes = current.record_bytes().to_vec();
    let content = file_content();
    let publish_parameters = parameters();
    let attempt = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &current_root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            mounted.output.instance,
            &current_record_bytes,
        )
    };
    match &attempt {
        Ok(output) => println!(
            "[R1-F1] 第二次挂载里第一个文件版本发出来了：txg {}",
            output.root.checkpoint_txg.0
        ),
        Err(error) => println!("[R1-F1] 第一个文件版本被拒：{error:?}"),
    }
    formatted.devices = Some(devices);
}
