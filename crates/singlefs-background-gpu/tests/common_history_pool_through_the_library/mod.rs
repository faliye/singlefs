//! 端到端对拍（真卡上推整理、经运行库挂载）共用：两边都从它起步的那份历史（`common_pool_on_memory_devices` 的
//! `pool_after_a_sequential_write_in_an_earlier_mount`）过判定库。录入段在 CPU 上搭出历史、把每块盘写过的扇区写进库，只录那一趟到此返回、不开卡；
//! 判段从库里读回、搭回一个池，选 `cpu` 与选 `gpu` 的两个池都从读回的那一份拷。一边选 `cpu`、一边选 `gpu` 的对拍，判段的对拍结论按键入库复用
//! （[`history_pool_concluded_on_the_cards_through_the_pairing_library`]），库里有两边一致的不开卡。
//! 要先 `mod common_pool_on_memory_devices;` 与 `mod common_pairing_input_through_the_library;` 再 mod 进这一份。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use crate::common_pairing_input_through_the_library::PairingJudged;
use crate::common_pairing_input_through_the_library::{
    pairing_concluded_on_the_cards_through_the_pairing_library,
    pairing_input_and_reference_read_back_through_the_pairing_library,
    pairing_input_bytes_of_the_written_sectors,
    pairing_input_read_back_through_the_pairing_library,
    written_sectors_of_the_pairing_input_bytes,
};
use crate::common_pool_on_memory_devices::{
    pool_after_a_sequential_write_in_an_earlier_mount, PoolOnMemoryDevices,
};

/// 那份历史编成对拍输入的字节：在 CPU 上搭出历史，每块盘写过的扇区（[`pairing_input_bytes_of_the_written_sectors`] 的布局）。
#[must_use]
pub fn pairing_input_bytes_of_the_history() -> Vec<u8> {
    pairing_input_bytes_of_the_written_sectors(
        &pool_after_a_sequential_write_in_an_earlier_mount().written_sectors_of_every_device(),
    )
}

/// 持卡判那一段照读回的扇区搭回那份历史的池（录制流是空的）。
///
/// # Panics
/// 读回的字节与布局对不上。
#[must_use]
pub fn history_pool_of_the_input_read_back(input_bytes: &[u8]) -> PoolOnMemoryDevices {
    PoolOnMemoryDevices::of_the_written_sectors(written_sectors_of_the_pairing_input_bytes(
        input_bytes,
    ))
}

/// 两边都从那份历史起步、一边选 `cpu` 一边选 `gpu` 的真卡对拍用例走这一条：历史过判定库，只录那一段另调 `reference_of_the_input`
/// 把选 `cpu` 的那一趟在 CPU 上跑完、它留下的结果一起写进库（录入键拼进登记表里 `reference_units_group` 那一组的判法单元键）；
/// 判段的对拍结论按键入库复用（`crate::common_pairing_input_through_the_library::pairing_concluded_on_the_cards_through_the_pairing_library`），
/// 要现判的才调 `judged_on_the_cards`（这条用例的持卡判入口：照读回的扇区搭池、只跑选 `gpu` 的那一趟、与读回的结果比）。
///
/// # Panics
/// 同 `pairing_concluded_on_the_cards_through_the_pairing_library`。
pub fn history_pool_concluded_on_the_cards_through_the_pairing_library(
    test_name: &str,
    reference_units_group: &str,
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
    judged_on_the_cards: impl FnOnce(&str, &[u8], &[u8]) -> PairingJudged,
) {
    pairing_concluded_on_the_cards_through_the_pairing_library(
        test_name,
        reference_units_group,
        pairing_input_bytes_of_the_history,
        reference_of_the_input,
        judged_on_the_cards,
    );
}

/// 一条端到端对拍用例的头一步：那份历史过判定库（项名用用例名）。只录那一趟交回 `None`（用例到此返回，不开卡）；
/// 判段交回照读回的扇区搭的池（录制流是空的）。
///
/// # Panics
/// 库开不了、写不进、读不回；读回的字节与布局对不上。
#[must_use]
pub fn history_pool_read_back_through_the_pairing_library(
    test_name: &str,
) -> Option<PoolOnMemoryDevices> {
    pairing_input_read_back_through_the_pairing_library(
        test_name,
        pairing_input_bytes_of_the_history,
    )
    .map(|input_read_back| history_pool_of_the_input_read_back(&input_read_back))
}

/// 两边都从那份历史起步、一边选 `cpu` 一边选 `gpu` 的端到端对拍用例的头一步：历史过判定库，只录那一段另调 `reference_of_the_input`
/// 把选 `cpu` 的那一趟在 CPU 上跑完、它留下的结果一起写进库（录入键拼进登记表里 `reference_units_group` 那一组的判法单元键）。
/// 只录那一趟交回 `None`（用例到此返回，不开卡）；判段交回照读回的扇区搭的池与选 `cpu` 那一趟的结果，持卡的那一段只跑选 `gpu` 的那一趟。
///
/// # Panics
/// 库开不了、写不进、读不回；读回的字节与布局对不上。
#[must_use]
pub fn history_pool_and_reference_read_back_through_the_pairing_library(
    test_name: &str,
    reference_units_group: &str,
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Option<(PoolOnMemoryDevices, Vec<u8>)> {
    pairing_input_and_reference_read_back_through_the_pairing_library(
        test_name,
        reference_units_group,
        pairing_input_bytes_of_the_history,
        reference_of_the_input,
    )
    .map(|read_back| {
        (
            history_pool_of_the_input_read_back(&read_back.input),
            read_back.reference,
        )
    })
}

/// 一趟跑完留下的几样各编成字节之后拼成参照的字节：样数，逐样是带长度的字节。
#[must_use]
pub fn reference_bytes_of_the_pieces(pieces: &[(&'static str, Vec<u8>)]) -> Vec<u8> {
    let mut reference_bytes = Vec::new();
    crate::common_pairing_input_through_the_library::append_count(
        &mut reference_bytes,
        pieces.len(),
    );
    for (_name, piece) in pieces {
        crate::common_pairing_input_through_the_library::append_length_prefixed_bytes(
            &mut reference_bytes,
            piece,
        );
    }
    reference_bytes
}

/// 选 `gpu` 那一趟留下的几样（`pieces_on_the_gpus`）与库里读回的参照（[`reference_bytes_of_the_pieces`] 写下的）逐样比，只比 `names_compared` 点名的那几样。
///
/// # Panics
/// 样数对不上；点名的某一样两边不相同；点名的名字不在 `pieces_on_the_gpus` 里。
pub fn assert_the_pieces_equal_the_reference(
    pieces_on_the_gpus: &[(&'static str, Vec<u8>)],
    reference_bytes: &[u8],
    names_compared: &[&str],
) {
    let mut reference =
        crate::common_pairing_input_through_the_library::PairingInputBeingRead::from_the_start_of(
            reference_bytes,
        );
    let piece_count = reference.take_count();
    assert_eq!(
        piece_count,
        pieces_on_the_gpus.len(),
        "参照与这一趟留下的样数相同"
    );
    let mut compared = 0;
    for (name, piece_on_the_gpus) in pieces_on_the_gpus {
        let reference_piece = reference.take_length_prefixed_bytes();
        if names_compared.contains(name) {
            compared += 1;
            assert!(
                piece_on_the_gpus.as_slice() == reference_piece,
                "{name}：选 gpu 的那一趟与库里读回的选 cpu 的那一趟不相同\n gpu：{}\n cpu：{}",
                String::from_utf8_lossy(&piece_on_the_gpus[..piece_on_the_gpus.len().min(2000)]),
                String::from_utf8_lossy(&reference_piece[..reference_piece.len().min(2000)])
            );
        }
    }
    reference.finish();
    assert_eq!(
        compared,
        names_compared.len(),
        "点名要比的每一样都在这一趟留下的里面"
    );
}
