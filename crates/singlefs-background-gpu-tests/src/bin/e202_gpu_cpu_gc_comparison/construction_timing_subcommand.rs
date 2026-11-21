//! 子命令 construction-timing：造池分阶段计时。

use super::*;

// ---- 子命令 construction-timing（造池分阶段计时：在给定的几个池上照同一套造法造起步镜像，按阶段打用时与内存；不跑 GC、不要卡；登记第十二节修订第 14 条的量 B）----

impl ChunkStorage {
    pub(super) const fn name(self) -> &'static str {
        match self {
            ChunkStorage::EachDeviceCopiesItsChunks => "each-device-copies",
            ChunkStorage::SharedWhenTheBytesAreEqual => "shared",
        }
    }

    pub(super) fn of_the_name(name: &str) -> Option<ChunkStorage> {
        [
            ChunkStorage::EachDeviceCopiesItsChunks,
            ChunkStorage::SharedWhenTheBytesAreEqual,
        ]
        .into_iter()
        .find(|storage| storage.name() == name)
    }
}

/// `construction-timing` 的设定：盘取哪一档的大小、造哪几个 P（各造一遍、按给的次序串着造，常驻内存读的是整个进程）、块怎么存。
pub(super) struct ConstructionTimingCommand {
    pub(super) tier: UnitAreaTier,
    pub(super) live_data_units: Vec<u64>,
    pub(super) storage: ChunkStorage,
    pub(super) product: PathBuf,
}

/// 造池认的 P：大于 0、是 32 的倍数（交错那一段写满整段，P ÷ 2 也就是 K = 16 的倍数），不超过这一档登记的 P。
pub(super) fn the_construction_accepts_the_live_data_units(
    tier: UnitAreaTier,
    live_data_units: u64,
) -> bool {
    live_data_units > 0
        && live_data_units.is_multiple_of(data_units_of_a_cluster_segment())
        && live_data_units <= tier.live_data_units()
}

pub(super) fn construction_timing_command(command: &ConstructionTimingCommand) -> i32 {
    let mut product = match ProductWriter::appending_to(&command.product) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let joined_live_data_units: Vec<String> =
        command.live_data_units.iter().map(u64::to_string).collect();
    if let Err(exit_code) = opening_of_the_command(
        &mut product,
        &command.product,
        &format!(
            "command=construction-timing tier={} chunks={} live_data_units={}",
            command.tier.name(),
            command.storage.name(),
            joined_live_data_units.join(",")
        ),
        CardsOfTheCommand::NotNeeded,
    ) {
        product.finish();
        return exit_code;
    }
    for live_data_units in &command.live_data_units {
        let mut progress_to_the_standard_error = |line: &str| eprintln!("{line}");
        let mut clock = ConstructionClock::new(
            format!("{}-P{live_data_units}", command.tier.name()),
            CONSTRUCTION_PROGRESS_INTERVAL,
            &mut progress_to_the_standard_error,
        );
        let image = match construct_a_starting_image_on_the_clock(
            command.tier,
            command.tier.device_bytes(),
            *live_data_units,
            command.storage,
            &mut clock,
        ) {
            Ok(image) => image,
            Err(stopped) => {
                emit_the_construction_stop(&mut product, command.tier, *live_data_units, &stopped);
                product.finish();
                return EXIT_STOPPED;
            }
        };
        let construction = &image.construction;
        let histogram: Vec<String> = construction
            .geometry
            .live_slot_histogram
            .iter()
            .map(|(live, segments)| format!("{live}:{segments}"))
            .collect();
        let chunk_bytes = chunk_bytes_of_the_pool(&image.devices);
        product.emit(
            "construction_timing",
            &format!(
                "tier={} device_bytes={} live_data_units={live_data_units} chunks={} construction_seconds={:.3} publishes={} \
                 floor_raises={} release_rounds={} live_data_units_found={} empty_segments={} mapped_units={} written_segments={} \
                 written_segments_with_half_live={} forced_segment_start_slot={} forced_live_slots={} \
                 forced_holds_only_data_units={} live_slot_histogram={} geometry_formed={} alignment_spacer_units={} \
                 chunk_bytes_held_by_the_devices={} chunk_bytes_stored={} resident_bytes={} {}",
                command.tier.name(),
                command.tier.device_bytes(),
                command.storage.name(),
                construction.construction_seconds,
                construction.publishes,
                construction.floor_raises,
                construction.release_rounds,
                construction.live_data_units_found,
                construction.empty_segments,
                construction.mapped_units,
                construction.geometry.written_segments,
                construction.geometry.written_segments_with_half_live,
                construction
                    .geometry
                    .forced
                    .start_slot
                    .map_or_else(|| "none".to_string(), |slot| slot.to_string()),
                construction.geometry.forced.live_slots,
                construction.geometry.forced.holds_only_data_units,
                histogram.join(","),
                construction.geometry.is_formed(),
                construction.alignment_spacer_units,
                chunk_bytes.held_by_the_devices,
                chunk_bytes.stored,
                resident_bytes_of_this_process()
                    .map_or_else(|| "unread".to_string(), |bytes| bytes.to_string()),
                whole_file_construction_fields(construction),
            ),
        );
        for timing in &construction.phases {
            product.emit(
                "construction_phase",
                &format!(
                    "tier={} live_data_units={live_data_units} chunks={} {}",
                    command.tier.name(),
                    command.storage.name(),
                    timing.fields()
                ),
            );
        }
        if !construction.geometry.is_formed() {
            product.emit(
                "stopped",
                &format!(
                    "tier={} live_data_units={live_data_units} {GEOMETRY_NOT_FORMED_STOP_FIELDS}",
                    command.tier.name()
                ),
            );
            product.finish();
            return EXIT_STOPPED;
        }
    }
    product.finish();
    EXIT_SUCCESS
}
