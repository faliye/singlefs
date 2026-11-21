//! 观察者交回的盘：池级 checker 在 CPU 上、在卡上或经 GPU 批处理服务判这些盘（[`PoolCheckerOfTheImagesAfterTheObservers`]），
//! 建判表、按显存排队，连同判器交回的自证槽拼成观察者块。

use super::*;

// ------------------------------ 判段：观察者交回的盘上的池级 checker ------------------------------

/// 判段在观察者交回的盘上跑池级 checker 走哪一路：配了 GPU 在卡上判（判不了的照 记一条、判红），没配由 CPU 判
/// （CPU 判器只做参照与配置关 GPU 时的路）。
pub(crate) enum PoolCheckerOfTheImagesAfterTheObservers {
    OnTheCpu,
    /// 头一张要判的盘那一刻才取卡（崩溃放量与崩溃枚举用例的判段）：观察者一张盘都不交回的那一趟一张卡都不取、不读显卡配置；
    /// 取卡没成（调用方没交卡）的判表交 GPU 批处理服务判。
    #[cfg(feature = "gpu")]
    TakingTheGpuCardsAtTheFirstImageToJudge(CardsTakenAtTheFirstImageToJudge),
    /// 单测：照 CPU 那一路判，每交来一批盘记下交来的强制现判（判段把复用哨兵抽中强制现判的那一块交没交下来）。
    #[cfg(test)]
    OnTheCpuNotingTheForcedRejudging(std::sync::Mutex<Vec<crate::verdict_store::ForcedRejudging>>),
}

/// 观察者交回的盘上判池级 checker 的卡，头一张要判的盘那一刻才取（[`Self::cards_taken_now_or_before`]），取到的留到这一趟判完。
/// 取卡照的是判段入口记下的显卡来路（调用方交没交卡串、多机配置），不读取卡那一刻的环境。
/// 取不到的照 [`way_after_the_cards_are_not_taken`] 分：调用方没交卡（入口那一刻环境里没有服务交的卡串）的，这一趟的判表都经
/// `way_to_the_gpu_verification_server` 交 GPU 批处理服务判（[`WayToTheGpuVerificationServer::judge_the_packed_tables`]），不退回 CPU 判；
/// 交了卡串而取不成的（写法不对、卡池起不来或与它对不上），这一趟每张盘照取卡报的那一句报错。
#[cfg(feature = "gpu")]
pub(crate) struct CardsTakenAtTheFirstImageToJudge {
    cards_taken: std::sync::OnceLock<CardsOfTheImagesAfterTheObservers>,
    /// 调用方交的卡串（产品是判段入口记下的那一份，单测换成写死的；没设是 `None`，空白的照原样记、取卡没成时当没交）。
    card_string_handed_over_by_the_caller: Option<String>,
    /// 取卡（产品照判段入口记下的显卡来路取，[`gpu_cards_of_this_run_from_the_multi_host_configuration`]；单测换成写死的结局）。
    take_the_cards: Box<dyn Fn() -> Result<GpuCardsOfThisRun, String> + Send + Sync>,
    way_to_the_gpu_verification_server: WayToTheGpuVerificationServer,
}

/// 头一张要判的盘那一刻取卡的结局：取到了在本进程上卡判；调用方没交卡的交 GPU 批处理服务判；交了卡串而取不成的照取卡报的那一句报错。
#[cfg(feature = "gpu")]
enum CardsOfTheImagesAfterTheObservers {
    TakenInThisProcess(GpuCardsOfThisRun),
    NotTakenSoTheGpuVerificationServerJudges,
    CardStringHandedOverButNotTaken { reason: String },
}

#[cfg(feature = "gpu")]
impl CardsTakenAtTheFirstImageToJudge {
    /// 建的时候一张卡都不取：头一张要判的盘那一刻照 `card_sources_of_the_entry`（判段入口记下的显卡来路）取卡
    /// （[`gpu_cards_of_this_run_from_the_multi_host_configuration`]），调用方交没交卡串也照它；`way_to_the_gpu_verification_server`
    /// 是调用方没交卡时把判表交服务判的那一条路（产品是判段入口取的控制命令）。
    pub(crate) fn new(
        card_sources_of_the_entry: CardSourcesOfTheEngineStart,
        way_to_the_gpu_verification_server: WayToTheGpuVerificationServer,
    ) -> Self {
        Self {
            cards_taken: std::sync::OnceLock::new(),
            card_string_handed_over_by_the_caller: card_sources_of_the_entry
                .cards_handed_out_by_the_service
                .clone(),
            take_the_cards: Box::new(move || {
                gpu_cards_of_this_run_from_the_multi_host_configuration(&card_sources_of_the_entry)
            }),
            way_to_the_gpu_verification_server,
        }
    }

    /// 单测：取卡换成 `take_the_cards`、调用方交的卡串换成 `card_string_handed_over_by_the_caller`，都不读环境、不起卡池。
    #[cfg(test)]
    fn with_the_cards_of_the_test(
        take_the_cards: fn() -> Result<GpuCardsOfThisRun, String>,
        card_string_handed_over_by_the_caller: Option<String>,
        way_to_the_gpu_verification_server: WayToTheGpuVerificationServer,
    ) -> Self {
        Self {
            cards_taken: std::sync::OnceLock::new(),
            card_string_handed_over_by_the_caller,
            take_the_cards: Box::new(take_the_cards),
            way_to_the_gpu_verification_server,
        }
    }

    /// 取过就交取过的结局；没取过此刻取，取不到的照 [`way_after_the_cards_are_not_taken`]（看调用方交没交卡串）记成交服务判或报错
    /// （之后每张盘照它，不再取）。
    fn cards_taken_now_or_before(&self) -> &CardsOfTheImagesAfterTheObservers {
        self.cards_taken
            .get_or_init(|| match (self.take_the_cards)() {
                Ok(cards_of_this_run) => {
                    CardsOfTheImagesAfterTheObservers::TakenInThisProcess(cards_of_this_run)
                }
                Err(reason) => match way_after_the_cards_are_not_taken(
                    self.card_string_handed_over_by_the_caller.as_deref(),
                    reason,
                ) {
                    WayAfterTheCardsAreNotTaken::ThroughTheGpuVerificationServer => {
                        CardsOfTheImagesAfterTheObservers::NotTakenSoTheGpuVerificationServerJudges
                    }
                    WayAfterTheCardsAreNotTaken::CardStringHandedOverButNotTaken { reason } => {
                        CardsOfTheImagesAfterTheObservers::CardStringHandedOverButNotTaken {
                            reason,
                        }
                    }
                },
            })
    }
}

/// 判段要判的一张观察者交回的盘：去重键、它的基底、持久集合、观察者发的写、判哪几条。
pub(super) struct ImageAfterTheObserverToJudge<'judge> {
    pub(super) deduplication_key: [u8; DIGEST_BYTES],
    pub(super) foundation: &'judge ObserverFoundation,
    pub(super) image: &'judge ImageToJudgeAfterTheObserver,
}

impl PoolCheckerOfTheImagesAfterTheObservers {
    pub(super) fn judged_by(&self) -> JudgedBy {
        match self {
            Self::OnTheCpu => JudgedBy::CpuJudge,
            #[cfg(feature = "gpu")]
            Self::TakingTheGpuCardsAtTheFirstImageToJudge(_) => JudgedBy::GpuJudge,
            #[cfg(test)]
            Self::OnTheCpuNotingTheForcedRejudging(_) => JudgedBy::CpuJudge,
        }
    }

    /// 判这几张盘，交回与 `images` 同序的结论。头一张盘那一刻才取卡的那一路取不到卡（调用方没交卡）的，判表交 GPU 批处理服务判
    /// （[`CardsTakenAtTheFirstImageToJudge`]）；`forced` 是这几张盘在不在复用哨兵抽中、要现判的那一块上（交服务判的判表照它写强制现判，
    /// 库里早有的判表结论字也不用）。
    ///
    /// # Errors
    /// 配了 GPU 而卡用不上、借不到显存；调用方交了卡串却取不成；判表进库读回失败；交服务判那一条报的错
    /// （[`judge_packed_tables_through_the_gpu_verification_server`]）。
    #[cfg_attr(
        not(feature = "gpu"),
        allow(
            unused_variables,
            clippy::unnecessary_wraps,
            reason = "没编进 gpu 特性时只有 CPU 那一路：它不写库（判表只在卡上那一路进库）、不报错"
        )
    )]
    pub(super) fn judge(
        &self,
        library: &LibraryTakenAtEachReadOrWrite<'_>,
        images: &[ImageAfterTheObserverToJudge<'_>],
        forced: crate::verdict_store::ForcedRejudging,
    ) -> Result<Vec<ImageJudgementRecord>, String> {
        match self {
            Self::OnTheCpu => {
                // 拿到的线程活到这几张盘判完：切几段照拿到的条数
                let (_threads_taken, pieces) =
                    images_after_the_observers_cut_into_pieces_for_the_cpu(images.len());
                Ok(judge_the_images_after_the_observers_on_the_cpu(
                    images, &pieces,
                ))
            }
            #[cfg(test)]
            Self::OnTheCpuNotingTheForcedRejudging(forced_rejudging_handed_down) => {
                forced_rejudging_handed_down
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(forced);
                let (_threads_taken, pieces) =
                    images_after_the_observers_cut_into_pieces_for_the_cpu(images.len());
                Ok(judge_the_images_after_the_observers_on_the_cpu(
                    images, &pieces,
                ))
            }
            #[cfg(feature = "gpu")]
            Self::TakingTheGpuCardsAtTheFirstImageToJudge(cards_taken_at_the_first_image) => {
                match cards_taken_at_the_first_image.cards_taken_now_or_before() {
                    CardsOfTheImagesAfterTheObservers::TakenInThisProcess(cards_of_this_run) => {
                        judge_the_images_after_the_observers_on_these_cards(
                            library,
                            images,
                            &cards_of_this_run.cards,
                            cards_of_this_run.taken_from,
                        )
                    }
                    CardsOfTheImagesAfterTheObservers::NotTakenSoTheGpuVerificationServerJudges => {
                        judge_the_images_after_the_observers_through_the_gpu_verification_server(
                            library,
                            images,
                            &cards_taken_at_the_first_image.way_to_the_gpu_verification_server,
                            forced,
                        )
                    }
                    CardsOfTheImagesAfterTheObservers::CardStringHandedOverButNotTaken {
                        reason,
                    } => Err(format!(
                        "观察者交回的盘要在卡上判池级 checker，调用方交了卡串却取不成：{reason}"
                    )),
                }
            }
        }
    }
}

/// 在这一趟的卡上判观察者交回的这几张盘（[`judge_the_images_after_the_observers_on_the_gpu`]，上卡那一下照卡的来路排队：
/// [`judge_packed_tables_of_the_images_after_the_observers_on_a_card`]），交回与 `images` 同序的结论。
///
/// # Errors
/// 同 [`judge_the_images_after_the_observers_on_the_gpu`]。
#[cfg(feature = "gpu")]
fn judge_the_images_after_the_observers_on_these_cards(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    images: &[ImageAfterTheObserverToJudge<'_>],
    cards: &CardsInUse,
    taken_from: GpuCardsSource,
) -> Result<Vec<ImageJudgementRecord>, String> {
    // 这一路在一条线程上串着判、同时只有一个判器，排队照样经同一种写法（与给定状态表那一路只留一种在自己起的卡上建判器的写法）
    let memory_queue_of_the_cards_started_for_this_run =
        memory_queue_of_the_cards_started_for_this_run(Some(cards));
    judge_the_images_after_the_observers_on_the_gpu(
        library,
        images,
        &mut |packed_tables_read_back| {
            judge_packed_tables_of_the_images_after_the_observers_on_a_card(
                cards,
                taken_from,
                &memory_queue_of_the_cards_started_for_this_run,
                packed_tables_read_back,
            )
        },
    )
}

/// 调用方没交卡时判观察者交回的这几张盘（[`judge_the_images_after_the_observers_on_the_gpu`]，排判表、判表入库读回与解结论字与本进程上卡那一支同一套）：
/// 读回的每一张判表经 `way_to_the_gpu_verification_server` 交 GPU 批处理服务判，内核源码是产品那一份旋钮取的
/// （`ContainerCapacityKnobs::PRODUCT`，取出的就是本进程上卡那一支编的 `JudgeKernelSources::of_the_judge()`），交回与 `images` 同序的结论。
///
/// # Errors
/// 同 [`judge_the_images_after_the_observers_on_the_gpu`]；交服务判那一条报的错（[`WayToTheGpuVerificationServer::judge_the_packed_tables`]）。
#[cfg(feature = "gpu")]
fn judge_the_images_after_the_observers_through_the_gpu_verification_server(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    images: &[ImageAfterTheObserverToJudge<'_>],
    way_to_the_gpu_verification_server: &WayToTheGpuVerificationServer,
    forced: crate::verdict_store::ForcedRejudging,
) -> Result<Vec<ImageJudgementRecord>, String> {
    judge_the_images_after_the_observers_on_the_gpu(
        library,
        images,
        &mut |packed_tables_read_back| {
            let verdict_words = way_to_the_gpu_verification_server.judge_the_packed_tables(
                library,
                packed_tables_read_back,
                ContainerCapacityKnobs::PRODUCT,
                forced,
            )?;
            Ok((verdict_words.verdict_words, verdict_words.judged_on))
        },
    )
}

/// 观察者交回的盘：基底叠上这个状态持久了的写，再叠上观察者发的写。
fn pool_after_the_observer(image: &ImageAfterTheObserverToJudge<'_>) -> MemoryPool {
    let mut pool = pool_of_the_crash_image(
        &image.foundation.base,
        &image.foundation.enumerated_writes,
        &image.image.persisted,
    );
    pool.apply_writes(&image.image.writes_on_the_crash_image);
    pool
}

/// 观察者交回的 `image_count` 张盘在 CPU 上判之前拿线程、切段：从里层份额拿核对线程数与盘数里小的那个、至少一条
/// （[`crate::crash_library::InnerThreadsTaken`]：产品构建里照给，单测构建里从用例共用的那一份额度拿，可能少于要的），
/// 按拿到的条数把盘的下标切成连着的几段（[`images_after_the_observers_cut_by_the_threads_taken`]）。拿到的线程交回调用方，判完才还。
fn images_after_the_observers_cut_into_pieces_for_the_cpu(
    image_count: usize,
) -> (crate::crash_library::InnerThreadsTaken, Vec<Range<usize>>) {
    let threads_taken =
        crate::crash_library::InnerThreadsTaken::take(checking_threads().min(image_count).max(1));
    let pieces =
        images_after_the_observers_cut_by_the_threads_taken(image_count, threads_taken.threads());
    (threads_taken, pieces)
}

/// `image_count` 张盘的下标照拿到的 `threads_taken` 条线程均分成连着的几段，一段一条线程：段数是盘数与拿到的条数（至少 1）里小的那个，
/// 前「盘数除以段数的余数」段各比其余的段多一张；几段升序连着，每张盘恰好落在一段里，盘至少一张时每段不空，没有盘时一段都不切。
fn images_after_the_observers_cut_by_the_threads_taken(
    image_count: usize,
    threads_taken: usize,
) -> Vec<Range<usize>> {
    let piece_count = image_count.min(threads_taken.max(1));
    // 跨段带着：下一段从哪一张起，等于前面各段的张数之和
    let mut start_of_the_next_piece = 0;
    (0..piece_count)
        .map(|piece_index| {
            let piece_length =
                image_count / piece_count + usize::from(piece_index < image_count % piece_count);
            let piece = start_of_the_next_piece..start_of_the_next_piece + piece_length;
            start_of_the_next_piece += piece_length;
            piece
        })
        .collect()
}

/// CPU 判器那一路（配置关 GPU 时）：逐张重建盘、跑池级 checker，`pieces`（[`images_after_the_observers_cut_into_pieces_for_the_cpu`]）
/// 一段一条线程并行，结论按盘的次序交回。
///
/// 这里是观察者结论拼块那一组判法单元的入口（`crate::judging_units_entries`），切段的函数不在那一组的闭包里：起线程之前先核交进来的段
/// 恰好罩全盘的下标，结论对不对只由这里定，段切错了不会换出另一组结论。
///
/// # Panics
/// 交进来的段没有恰好罩全 `images` 的下标：几段要升序连着、头一段从 0 起、末一段到盘数止、每段不空（没有盘时段表为空）。
/// 消息点名哪一段、它的起止与盘数。
fn judge_the_images_after_the_observers_on_the_cpu(
    images: &[ImageAfterTheObserverToJudge<'_>],
    pieces: &[Range<usize>],
) -> Vec<ImageJudgementRecord> {
    let convention_of_the_pieces = "依赖的约定：调用方交的段要恰好罩全盘的下标（几段升序连着、头一段从 0 起、末一段到盘数止、每段不空，没有盘时段表为空）";
    // 跨段带着：前面各段罩到哪一张为止，下一段要从这里起
    let mut end_of_the_pieces_so_far = 0;
    for (piece_index, piece) in pieces.iter().enumerate() {
        assert!(
            piece.start == end_of_the_pieces_so_far
                && piece.start < piece.end
                && piece.end <= images.len(),
            "观察者盘在 CPU 上判：交进来的下标 {piece_index} 那一段是 {}..{}，盘数 {}：它不接着前面各段的末尾 {end_of_the_pieces_so_far} 起、是空段或越过了盘数；{convention_of_the_pieces}",
            piece.start,
            piece.end,
            images.len()
        );
        end_of_the_pieces_so_far = piece.end;
    }
    assert!(
        end_of_the_pieces_so_far == images.len(),
        "观察者盘在 CPU 上判：交进来的 {} 段止于 {end_of_the_pieces_so_far}（末一段是 {:?}），盘数 {}：没罩到末一张盘；{convention_of_the_pieces}",
        pieces.len(),
        pieces.last(),
        images.len()
    );
    std::thread::scope(|scope| {
        let handles: Vec<_> = pieces
            .iter()
            .map(|piece| {
                let chunk = &images[piece.clone()];
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|image| {
                            let pool = pool_after_the_observer(image);
                            let violated_invariants =
                                singlefs_checker::walk::check_pool_image(&pool)
                                    .into_iter()
                                    .filter(|(invariant, _)| {
                                        image.image.invariants_judged.includes(invariant)
                                    })
                                    .filter_map(|(invariant, verdict)| {
                                        match verdict {
                                    singlefs_checker::image::InvariantVerdict::Violated(_) => {
                                        Some(invariant.to_string())
                                    }
                                    singlefs_checker::image::InvariantVerdict::Holds
                                    | singlefs_checker::image::InvariantVerdict::NotApplicable(_)
                                    | singlefs_checker::image::InvariantVerdict::PartiallyJudged(
                                        _,
                                    ) => None,
                                }
                                    })
                                    .collect();
                            ImageJudgementRecord {
                                violated_invariants,
                                could_not_judge: None,
                                judged_on: JudgingDevice::Cpu,
                            }
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
            })
            .collect()
    })
}

/// 一张观察者交回的盘在判表的写表里的一次写是哪一次：同一基底的几张盘持久了同一次枚举写、或观察者发了逐字节相同的写，判表里只放一份
/// （甲报告问题 3 的分组：挂载写相同的几张盘共用一份写，掩码只差持久集合）。
#[cfg(any(feature = "gpu", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum WriteIdentityInTheJudgeTable {
    /// 基底枚举写表里的第几次写。
    EnumeratedWrite(usize),
    /// 观察者在崩溃镜像上发的一次写，按盘、起点、种类、FUA 与内容取的摘要（[`digest_of_a_write_the_observer_issued`]）。
    WriteTheObserverIssued([u8; DIGEST_BYTES]),
}

/// 观察者交回的一张盘落成判表时的一次写：它是哪一次、写的什么。
#[cfg(any(feature = "gpu", test))]
#[derive(Clone, Copy)]
struct WriteOfAnImageAfterTheObserver<'image> {
    identity: WriteIdentityInTheJudgeTable,
    write: &'image RetainedWrite,
}

/// 观察者发的一次写的摘要：盘、起点、种类码、FUA、内容（清零只带长度）；几张盘发的写这几样逐字节相同就是同一次。
#[cfg(any(feature = "gpu", test))]
fn digest_of_a_write_the_observer_issued(write: &RetainedWrite) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs write the observer issued 1\n".to_vec();
    message.extend_from_slice(&write.device.0.to_le_bytes());
    message.extend_from_slice(&write.offset.0.to_le_bytes());
    message.extend_from_slice(&crate::crash_judge_tables::step_kind_code(write.kind).to_le_bytes());
    message.push(u8::from(write.is_force_unit_access));
    match &write.contents {
        WrittenContents::Bytes(bytes) => {
            message.push(1);
            message.extend_from_slice(bytes);
        }
        WrittenContents::Zeros { length } => {
            message.push(0);
            message.extend_from_slice(&length.to_le_bytes());
        }
    }
    sha256_digest(&message)
}

/// 观察者交回的一张盘落成判表要的写：这个状态持久了的枚举写（按枚举写表次序）接上观察者发的写（按发的次序），同一位置（同一块盘、
/// 同一个起点）上的几次写并成最后落盘的那一次——后面有一次同起点、至少一样长的写，前一次的每个字节都被它盖住，不进判表（后面那次短的盖不全，
/// 两次都留）。照留下的这几次按次序叠在基底上，与 [`pool_after_the_observer`] 叠出的盘逐字节相同；判表一个落点最多
/// [`crate::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION`] 次写，挂载把系统配置槽写了几十次的盘不并就建不出表。
#[cfg(any(feature = "gpu", test))]
fn landed_writes_of_the_image_merged_on_each_location<'image>(
    image: &ImageAfterTheObserverToJudge<'image>,
) -> Vec<WriteOfAnImageAfterTheObserver<'image>> {
    let landed: Vec<WriteOfAnImageAfterTheObserver<'image>> = image
        .foundation
        .enumerated_writes
        .iter()
        .enumerate()
        .zip(&image.image.persisted)
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|((write_index, write), _)| WriteOfAnImageAfterTheObserver {
            identity: WriteIdentityInTheJudgeTable::EnumeratedWrite(write_index),
            write,
        })
        .chain(image.image.writes_on_the_crash_image.iter().map(|write| {
            WriteOfAnImageAfterTheObserver {
                identity: WriteIdentityInTheJudgeTable::WriteTheObserverIssued(
                    digest_of_a_write_the_observer_issued(write),
                ),
                write,
            }
        }))
        .collect();
    // 从最后落盘的那一次往前看：一个位置上更晚的写里最长的那一次有多长
    let mut longest_later_write_by_location: BTreeMap<(u32, u64), u64> = BTreeMap::new();
    let mut kept_from_the_last: Vec<WriteOfAnImageAfterTheObserver<'image>> =
        Vec::with_capacity(landed.len());
    for landed_write in landed.iter().rev() {
        let length = landed_write.write.length_in_bytes();
        let longest_later_write = longest_later_write_by_location
            .entry((landed_write.write.device.0, landed_write.write.offset.0))
            .or_insert(0);
        if *longest_later_write < length {
            kept_from_the_last.push(*landed_write);
        }
        *longest_later_write = (*longest_later_write).max(length);
    }
    kept_from_the_last.reverse();
    kept_from_the_last
}

/// 一组同基底的盘建出的判表：组里每张盘在交来的那一批里的位置（判表第 k 个状态就是第 k 张，持久掩码在判表的给定状态清单里）、
/// 判表的写表（几张盘并过之后的写去重：枚举写按枚举写表次序在前，观察者发的写按头一次出现的次序在后）、判表。
#[cfg(any(feature = "gpu", test))]
struct JudgeTableOfImagesAfterTheObservers {
    positions: Vec<usize>,
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "判表的写表留给单测核每张盘照掩码叠出来与原盘逐字节相同；GPU 那一路只用判表本身"
        )
    )]
    writes: Vec<RetainedWrite>,
    tables: crate::crash_judge_tables::JudgeTables,
}

/// 一批观察者交回的盘排出来的判表：建出了的一张，或单张盘也建不出（判段照 记判不了、判红，不退回 CPU 判）。
#[cfg(any(feature = "gpu", test))]
enum PlannedJudgeTableOfImagesAfterTheObservers {
    Built(Box<JudgeTableOfImagesAfterTheObservers>),
    NotBuildableForOneImage { position: usize, reason: String },
}

/// 一组盘为什么建不出一张判表。
#[cfg(any(feature = "gpu", test))]
#[derive(Debug)]
enum JudgeTableOfAGroupNotBuildable {
    /// 两张盘里有两次彼此重叠的写（字节范围交叠、同一块盘）先后相反：一张写表排不出两种次序，分到两张表里。单张盘不会撞上它（照它自己的次序排）。
    OverlappingWritesInOppositeOrders,
    Refused(crate::crash_judge_tables::JudgeTablesRefusal),
}

/// 一张判表里最多几张盘：一个落点的版本至多是表里盘的张数，不超过一个落点的版本数上限。
#[cfg(any(feature = "gpu", test))]
const IMAGES_AFTER_THE_OBSERVERS_PER_JUDGE_TABLE_AT_MOST: usize =
    crate::crash_judge_tables::MAXIMUM_VERSIONS_PER_LOCATION;

/// 同一基底的盘按次序几张一组（每张盘的写先照 [`landed_writes_of_the_image_merged_on_each_location`] 并过）：一组并起来的不同写不超过判表掩码的宽
/// （[`crate::crash_facts::MAXIMUM_ENUMERATED_WRITES`]）的一半、同一位置上不同的写不超过一个落点的写数上限的一半、一组不超过
/// [`IMAGES_AFTER_THE_OBSERVERS_PER_JUDGE_TABLE_AT_MOST`] 张。一半：建表时每次原地覆写在枚举写表末尾接一条撕裂镜像（`crash::crash_state_enumeration_tables`），
/// 它也罩着那个落点、也占掩码的一位，判表里每次写一段（[`judge_table_of_a_group_of_images_after_the_observers`]）所以不另接重放；
/// 别的建表时才撞上的上限，由建表那一步对半再建。
#[cfg(any(feature = "gpu", test))]
fn packed_groups_of_images_sharing_their_writes(
    merged_writes_of_each_image: &[Vec<WriteOfAnImageAfterTheObserver<'_>>],
    positions: &[usize],
) -> Vec<Vec<usize>> {
    use crate::crash_facts::MAXIMUM_ENUMERATED_WRITES;
    use crate::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION;
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut group: Vec<usize> = Vec::new();
    let mut identities_in_the_group: std::collections::HashSet<WriteIdentityInTheJudgeTable> =
        std::collections::HashSet::new();
    let mut identities_by_location: BTreeMap<(u32, u64), usize> = BTreeMap::new();
    for position in positions {
        let mut new_identities: std::collections::HashSet<WriteIdentityInTheJudgeTable> =
            std::collections::HashSet::new();
        let mut new_identities_by_location: BTreeMap<(u32, u64), usize> = BTreeMap::new();
        for landed_write in &merged_writes_of_each_image[*position] {
            if !identities_in_the_group.contains(&landed_write.identity)
                && new_identities.insert(landed_write.identity)
            {
                *new_identities_by_location
                    .entry((landed_write.write.device.0, landed_write.write.offset.0))
                    .or_insert(0) += 1;
            }
        }
        let fits_in_the_group = group.len() < IMAGES_AFTER_THE_OBSERVERS_PER_JUDGE_TABLE_AT_MOST
            && identities_in_the_group.len() + new_identities.len()
                <= MAXIMUM_ENUMERATED_WRITES / 2
            && new_identities_by_location
                .iter()
                .all(|(location, new_writes)| {
                    identities_by_location.get(location).copied().unwrap_or(0) + new_writes
                        <= MAXIMUM_WRITES_PER_LOCATION / 2
                });
        if !fits_in_the_group && !group.is_empty() {
            groups.push(std::mem::take(&mut group));
            identities_in_the_group.clear();
            identities_by_location.clear();
            // 这张盘开一组新的：它自己的写全算新的
            new_identities.clear();
            new_identities_by_location.clear();
            for landed_write in &merged_writes_of_each_image[*position] {
                if new_identities.insert(landed_write.identity) {
                    *new_identities_by_location
                        .entry((landed_write.write.device.0, landed_write.write.offset.0))
                        .or_insert(0) += 1;
                }
            }
        }
        group.push(*position);
        identities_in_the_group.extend(new_identities);
        for (location, new_writes) in new_identities_by_location {
            *identities_by_location.entry(location).or_insert(0) += new_writes;
        }
    }
    if !group.is_empty() {
        groups.push(group);
    }
    groups
}

/// 一组同基底的盘建一张给定状态的判表：写表是这几张盘并过之后的写去重（枚举写按枚举写表次序在前、观察者发的写按头一次出现的次序在后），
/// 每次写一段、不展开（同一段里原地覆写之后重叠的写建表时各接一份重放，同一位置上几十次写放一段会接出平方那么多份）；
/// 第 k 个状态的持久掩码是第 k 张盘留下的那几次写。每张盘照掩码按写表次序叠在基底上，与它自己的次序叠出的字节相同：
/// 枚举写在每张盘里本来就按枚举写表次序，观察者的写都在枚举写之后，剩下观察者的写之间两张盘先后相反、又彼此重叠的，这一组排不出。
///
/// # Errors
/// 两张盘重叠的写先后相反；建表拒了（[`crate::crash_judge_tables::JudgeTablesRefusal`]）。
#[cfg(any(feature = "gpu", test))]
fn judge_table_of_a_group_of_images_after_the_observers(
    images: &[ImageAfterTheObserverToJudge<'_>],
    merged_writes_of_each_image: &[Vec<WriteOfAnImageAfterTheObserver<'_>>],
    group: &[usize],
) -> Result<JudgeTableOfImagesAfterTheObservers, JudgeTableOfAGroupNotBuildable> {
    use crate::crash_judge_tables::{
        build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams, JudgeFlow,
    };
    let foundation = images[group[0]].foundation;
    let mut enumerated_writes_used: BTreeSet<usize> = BTreeSet::new();
    let mut writes_the_observers_issued: Vec<([u8; DIGEST_BYTES], &RetainedWrite)> = Vec::new();
    let mut issued_already: std::collections::HashSet<[u8; DIGEST_BYTES]> =
        std::collections::HashSet::new();
    // 这一组的盘都没留下一次写（都就是基底）时，写表里放一次哪张盘都不持久的枚举写，判表的写表不空（被判根的下标指得到一次写）
    if group
        .iter()
        .all(|position| merged_writes_of_each_image[*position].is_empty())
        && !foundation.enumerated_writes.is_empty()
    {
        enumerated_writes_used.insert(0);
    }
    for position in group {
        for landed_write in &merged_writes_of_each_image[*position] {
            match landed_write.identity {
                WriteIdentityInTheJudgeTable::EnumeratedWrite(write_index) => {
                    enumerated_writes_used.insert(write_index);
                }
                WriteIdentityInTheJudgeTable::WriteTheObserverIssued(digest) => {
                    if issued_already.insert(digest) {
                        writes_the_observers_issued.push((digest, landed_write.write));
                    }
                }
            }
        }
    }
    let mut table_index_of_each_identity: std::collections::HashMap<
        WriteIdentityInTheJudgeTable,
        usize,
    > = std::collections::HashMap::new();
    let mut writes: Vec<RetainedWrite> =
        Vec::with_capacity(enumerated_writes_used.len() + writes_the_observers_issued.len());
    for write_index in &enumerated_writes_used {
        table_index_of_each_identity.insert(
            WriteIdentityInTheJudgeTable::EnumeratedWrite(*write_index),
            writes.len(),
        );
        writes.push(foundation.enumerated_writes[*write_index].clone());
    }
    for (digest, write) in &writes_the_observers_issued {
        table_index_of_each_identity.insert(
            WriteIdentityInTheJudgeTable::WriteTheObserverIssued(*digest),
            writes.len(),
        );
        writes.push((*write).clone());
    }
    let table_indexes_of_each_image: Vec<Vec<usize>> = group
        .iter()
        .map(|position| {
            merged_writes_of_each_image[*position]
                .iter()
                .map(|landed_write| table_index_of_each_identity[&landed_write.identity])
                .collect()
        })
        .collect();
    // 每张盘里彼此重叠的两次写，在写表里要与它自己的先后相同
    for (position, table_indexes) in group.iter().zip(&table_indexes_of_each_image) {
        let landed_writes = &merged_writes_of_each_image[*position];
        let mut by_start: Vec<usize> = (0..landed_writes.len()).collect();
        by_start.sort_by_key(|order| {
            (
                landed_writes[*order].write.device.0,
                landed_writes[*order].write.offset.0,
            )
        });
        for (rank, earlier_by_start) in by_start.iter().enumerate() {
            let first = landed_writes[*earlier_by_start].write;
            let first_end = first.offset.0 + first.length_in_bytes();
            for later_by_start in &by_start[rank + 1..] {
                let second = landed_writes[*later_by_start].write;
                if second.device != first.device || second.offset.0 >= first_end {
                    break;
                }
                let (earlier, later) = if earlier_by_start < later_by_start {
                    (*earlier_by_start, *later_by_start)
                } else {
                    (*later_by_start, *earlier_by_start)
                };
                if table_indexes[earlier] > table_indexes[later] {
                    return Err(JudgeTableOfAGroupNotBuildable::OverlappingWritesInOppositeOrders);
                }
            }
        }
    }
    let segments: Vec<Vec<usize>> = (0..writes.len())
        .map(|write_index| vec![write_index])
        .collect();
    let not_expanded =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let enumerated_write_count = crate::crash::crash_state_enumeration_tables(
        &foundation.base,
        &writes,
        &segments,
        &not_expanded,
    )
    .enumerated_writes
    .len();
    let persisted_sets: Vec<Vec<bool>> = table_indexes_of_each_image
        .iter()
        .map(|table_indexes| {
            let mut persisted = vec![false; enumerated_write_count];
            for table_index in table_indexes {
                persisted[*table_index] = true;
            }
            persisted
        })
        .collect();
    let tables = build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams(
        &JudgeFlow {
            base: &foundation.base,
            writes: &writes,
            segments: &segments,
            judged_root_index: 0,
            versions: &[],
            expansion: &not_expanded,
        },
        &[],
        persisted_sets,
        &[],
    )
    .map_err(JudgeTableOfAGroupNotBuildable::Refused)?;
    Ok(JudgeTableOfImagesAfterTheObservers {
        positions: group.to_vec(),
        writes,
        tables,
    })
}

/// 单张盘也建不出判表时判红的那一句：建表拒在哪，这张盘是哪一类（基底、并过之后几次写、其中持久的枚举写与观察者发的写各几次、
/// 清零写几次、同一位置上最多几次）。
#[cfg(any(feature = "gpu", test))]
fn reason_no_judge_table_holds_the_image_after_the_observer(
    image: &ImageAfterTheObserverToJudge<'_>,
    merged_writes: &[WriteOfAnImageAfterTheObserver<'_>],
    not_buildable: &JudgeTableOfAGroupNotBuildable,
) -> String {
    let enumerated_writes = merged_writes
        .iter()
        .filter(|landed_write| {
            matches!(
                landed_write.identity,
                WriteIdentityInTheJudgeTable::EnumeratedWrite(_)
            )
        })
        .count();
    let zero_fills = merged_writes
        .iter()
        .filter(|landed_write| matches!(landed_write.write.contents, WrittenContents::Zeros { .. }))
        .count();
    let mut writes_by_location: BTreeMap<(u32, u64), usize> = BTreeMap::new();
    for landed_write in merged_writes {
        *writes_by_location
            .entry((landed_write.write.device.0, landed_write.write.offset.0))
            .or_insert(0) += 1;
    }
    let most_writes_on_one_location = writes_by_location.values().copied().max().unwrap_or(0);
    let not_buildable_text = match not_buildable {
        JudgeTableOfAGroupNotBuildable::Refused(refusal) => format!("{refusal:?}"),
        JudgeTableOfAGroupNotBuildable::OverlappingWritesInOppositeOrders => {
            "两次重叠的写先后相反".to_string()
        }
    };
    format!(
        "建不出 GPU 判表：{not_buildable_text}（单张盘也建不出，并过同一位置上的写之后还是）；这张盘：基底 {}，并过之后 {} 次写，其中持久的枚举写 {enumerated_writes} 次、观察者发的写 {} 次（观察者共发 {} 次），清零写 {zero_fills} 次，同一位置上最多 {most_writes_on_one_location} 次",
        hexadecimal_text(&image.image.foundation_digest[..8]),
        merged_writes.len(),
        merged_writes.len() - enumerated_writes,
        image.image.writes_on_the_crash_image.len()
    )
}

/// 一批观察者交回的盘排成几张给定状态的判表：每张盘的写先照 [`landed_writes_of_the_image_merged_on_each_location`] 并过同一位置上的几次，
/// 同一基底的盘按次序几张一组（[`packed_groups_of_images_sharing_their_writes`]，交回的写逐字节相同的共用一份写），一组建不出就对半再建，
/// 单张盘也建不出的记一句判不了（[`reason_no_judge_table_holds_the_image_after_the_observer`]）。每张盘恰好落在一张判表或一句判不了里。
/// 排出一张交一张给 `hand_on`（判完一张再建下一张，内存里只放一张判表）；`hand_on` 报错就停下、照原样交回。
///
/// # Errors
/// `hand_on` 报的错。
#[cfg(any(feature = "gpu", test))]
fn plan_the_judge_tables_of_the_images_after_the_observers(
    images: &[ImageAfterTheObserverToJudge<'_>],
    hand_on: &mut dyn FnMut(PlannedJudgeTableOfImagesAfterTheObservers) -> Result<(), String>,
) -> Result<(), String> {
    let merged_writes_of_each_image: Vec<Vec<WriteOfAnImageAfterTheObserver<'_>>> = images
        .iter()
        .map(landed_writes_of_the_image_merged_on_each_location)
        .collect();
    let mut positions_by_foundation: BTreeMap<[u8; DIGEST_BYTES], Vec<usize>> = BTreeMap::new();
    for (position, image) in images.iter().enumerate() {
        positions_by_foundation
            .entry(image.image.foundation_digest)
            .or_default()
            .push(position);
    }
    let mut pending_groups: Vec<Vec<usize>> = positions_by_foundation
        .into_values()
        .flat_map(|positions| {
            packed_groups_of_images_sharing_their_writes(&merged_writes_of_each_image, &positions)
        })
        .collect();
    pending_groups.reverse();
    while let Some(group) = pending_groups.pop() {
        match judge_table_of_a_group_of_images_after_the_observers(
            images,
            &merged_writes_of_each_image,
            &group,
        ) {
            Ok(table) => hand_on(PlannedJudgeTableOfImagesAfterTheObservers::Built(Box::new(
                table,
            )))?,
            Err(not_buildable) => {
                if group.len() > 1 {
                    let (front, back) = group.split_at(group.len() / 2);
                    pending_groups.push(back.to_vec());
                    pending_groups.push(front.to_vec());
                } else {
                    hand_on(
                        PlannedJudgeTableOfImagesAfterTheObservers::NotBuildableForOneImage {
                            position: group[0],
                            reason: reason_no_judge_table_holds_the_image_after_the_observer(
                                &images[group[0]],
                                &merged_writes_of_each_image[group[0]],
                                &not_buildable,
                            ),
                        },
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// 判器在这一趟自己起的卡上丢掉之后、还占位之前那一次等：等这张卡的设备上到此刻为止提交的活都做完（`submission_index: None`），
/// 最多等卡池借据丢掉时那一次等的同一个长度
/// （[`crate::gpu_unit_checks::LONGEST_WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE_OR_A_PLACE_IN_THE_MEMORY_QUEUE`]）。
/// 不设上限会让还占位的那条线程卡在一台出了事的设备上。
#[cfg(feature = "gpu")]
fn poll_type_before_returning_a_place_in_the_memory_queue() -> wgpu::PollType {
    wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(
            crate::gpu_unit_checks::LONGEST_WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE_OR_A_PLACE_IN_THE_MEMORY_QUEUE,
        ),
    }
}

/// 这一趟自己起的卡（[`GpuCardsSource::DevicesStartedForThisRun`]）的显存排队：每张卡的容量是那张卡给崩溃放量的额度
/// （[`crate::gpu_unit_checks::GpuCard::memory_budget_mebibytes`]）扣掉驱动那一份（[`crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER`]：
/// 这个进程在每张卡上一份，起设备时付过），次序照卡的先后。没有卡（不要卡、卡来自卡池）是空的。
#[cfg(feature = "gpu")]
pub(super) fn memory_queue_of_the_cards_started_for_this_run(
    cards: Option<&CardsInUse>,
) -> CardMemoryQueue {
    CardMemoryQueue::new(
        cards
            .map(|cards| {
                cards
                    .iter()
                    .map(|card| {
                        card.memory_budget_mebibytes().saturating_sub(
                            crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
    )
}

/// 在这一趟自己起的卡上判一份打包判表的全部状态（给定状态表与观察者交回的盘两路共用这一种写法）：一次派活至多一条草稿缓冲那么宽
/// （[`crate::crash_judge_dispatch::states_per_scratch_lane`]，照这几张卡里最小的存储缓冲绑定上限，与卡池那一支借的同宽），
/// 照这个宽度算判器要的显存（[`crate::crash_judge_dispatch::mebibytes_a_judge_needs`]），经 `memory_queue` 排进第一张放得下的卡（都放不下就等）；
/// 判器建在那张卡的一份克隆上，克隆的额度与空闲改成排进的那么多加驱动那一份，判器开出来的草稿区正好是排进的那么多
/// （与卡池借出去的卡同一种改法：`gpu_unit_checks::gpu_test_start::GpuCardPool` 的 `card_lent_for_the_booking`）。
/// 判完先丢判器、再等这张卡上提交了的活做完（丢掉的缓冲要等设备收拾才交回驱动），最后还占位。交回逐状态的结论与在哪张卡上判的。
///
/// # Errors
/// 判器要的比每张卡的容量都大（等多久都放不下：配了 GPU 就要 GPU 判，不换小、不退回 CPU）；判完之后等设备没等成
/// （超时或报错：没等成的占位怎么记，同卡按显存排队的账没定，第一版不支持，这一份不交结论、占位照还）。
///
/// # Panics
/// 同 [`crate::crash_judge_dispatch::GpuCrashJudge::on_card_compiling_these_kernel_sources`]；wgpu 在建缓冲、派活或等的时候报没接住的错。
#[cfg(feature = "gpu")]
pub(super) fn judge_on_a_card_started_for_this_run_queued_by_memory(
    cards: &CardsInUse,
    memory_queue: &CardMemoryQueue,
    packed: &crate::crash_judge_dispatch::PackedJudgeTablesReadBack,
    kernel_sources: &crate::crash_judge_gpu::JudgeKernelSources,
    judged_what: &str,
) -> Result<
    (
        Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>,
        JudgingDevice,
    ),
    String,
> {
    use crate::crash_judge_dispatch::{
        mebibytes_a_judge_needs, states_per_scratch_lane, GpuCrashJudge,
        MEBIBYTES_LEFT_FOR_THE_DRIVER,
    };
    let smallest_maximum_storage_buffer_binding_size = cards
        .iter()
        .map(|card| card.device().limits().max_storage_buffer_binding_size)
        .min()
        .ok_or_else(|| {
            format!("{judged_what}：配置开了 GPU，这一趟自己起的卡却一张都没有：不退回 CPU 判器")
        })?;
    let states_per_dispatch = states_per_scratch_lane(smallest_maximum_storage_buffer_binding_size);
    let need_mebibytes = mebibytes_a_judge_needs(packed, states_per_dispatch);
    let place = memory_queue
        .admit_on_the_first_card_with_room(need_mebibytes)
        .map_err(|larger| {
            format!(
                "{judged_what}：判器要 {} MiB，比这一趟每张卡给崩溃放量的额度扣掉驱动那一份都大（最大的一张 {} MiB）：\
                 配了 GPU 就要 GPU 判，不换小、不退回 CPU 判器",
                larger.need_mebibytes, larger.largest_ceiling_mebibytes
            )
        })?;
    let mut card_with_the_admitted_quota =
        crate::gpu_unit_checks::GpuCard::clone(&cards[place.card_position()]);
    card_with_the_admitted_quota.memory_quota_mebibytes =
        Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER);
    card_with_the_admitted_quota.free_memory_mebibytes =
        need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER;
    let judge = GpuCrashJudge::on_card_compiling_these_kernel_sources(
        &card_with_the_admitted_quota,
        packed,
        states_per_dispatch,
        kernel_sources,
    );
    let verdicts = judge.judge_states(0..packed.state_count());
    drop(judge);
    let waited = card_with_the_admitted_quota
        .device()
        .poll(poll_type_before_returning_a_place_in_the_memory_queue());
    drop(place);
    match &waited {
        Ok(wgpu::PollStatus::QueueEmpty | wgpu::PollStatus::WaitSucceeded) => {}
        Ok(wgpu::PollStatus::Poll) | Err(_) => {
            let failure = format!(
                "{judged_what}：判完之后等 {} 上提交的活做完没等成（{waited:?}）：没等成的占位怎么记，同卡按显存排队的账没定，\
                 第一版不支持，这一份不交结论",
                card_with_the_admitted_quota.name
            );
            card_with_the_admitted_quota
                .device_condition()
                .print_the_device_lost_marker_line_the_first_time(&failure);
            return Err(failure);
        }
    }
    Ok((
        verdicts,
        JudgingDevice::Gpu {
            card_type: card_with_the_admitted_quota.name.clone(),
            driver_version: card_with_the_admitted_quota.driver_version.clone(),
        },
    ))
}

/// 观察者交回的盘排成的一份判表在卡上判出的逐状态结论字（按状态序号排）与在哪张卡上判的。
#[cfg(feature = "gpu")]
type VerdictsOfThePackedTablesOnACard = (
    Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>,
    JudgingDevice,
);

/// GPU 判器那一路：盘先排成判表（[`plan_the_judge_tables_of_the_images_after_the_observers`]：每张盘同一位置上的写并过、同一基底交回的写相同的
/// 共用一份写、建不出就对半再建），判表先入库、再读库（[`crate::crash_library::record_the_judge_tables_and_read_them_back_at_each_read_or_write`]），
/// 读回的那一份交调用方给的 `judge_these_packed_tables_on_a_gpu_card` 上卡判（判段照卡的来路排队、定派活宽度、建判器的那一下：
/// [`judge_packed_tables_of_the_images_after_the_observers_on_a_card`]），只读池级 checker 那几个字。单张盘也建不出的照 H6 记判不了
/// （那一句写明是哪一类盘），不退回 CPU 判。
/// 观察者结论拼块那一组从这里收闭包：上卡那一下由调用方交进来，量显存额度、定派活宽度的函数与常量不在这一组的闭包里，
/// 改了它们拼过的块照旧复用（判器的代码另由 GPU 判器那一组并进池级 checker 那一把版本）。
///
/// # Errors
/// `judge_these_packed_tables_on_a_gpu_card` 报的错（卡用不上、借不到显存）；判表进库读回失败。
#[cfg(feature = "gpu")]
fn judge_the_images_after_the_observers_on_the_gpu(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    images: &[ImageAfterTheObserverToJudge<'_>],
    judge_these_packed_tables_on_a_gpu_card: &mut dyn FnMut(
        &crate::crash_judge_dispatch::PackedJudgeTablesReadBack,
    ) -> Result<
        VerdictsOfThePackedTablesOnACard,
        String,
    >,
) -> Result<Vec<ImageJudgementRecord>, String> {
    use crate::crash_judge_dispatch::PackedJudgeTables;
    let mut judgements: Vec<Option<ImageJudgementRecord>> = vec![None; images.len()];
    plan_the_judge_tables_of_the_images_after_the_observers(images, &mut |planned| {
        let JudgeTableOfImagesAfterTheObservers {
            positions: group,
            writes: _,
            tables,
        } = match planned {
            PlannedJudgeTableOfImagesAfterTheObservers::Built(built) => *built,
            PlannedJudgeTableOfImagesAfterTheObservers::NotBuildableForOneImage {
                position,
                reason,
            } => {
                judgements[position] = Some(ImageJudgementRecord {
                    violated_invariants: Vec::new(),
                    could_not_judge: Some(reason),
                    judged_on: JudgingDevice::Cpu,
                });
                return Ok(());
            }
        };
        let packed =
            crate::crash_library::record_the_judge_tables_and_read_them_back_at_each_read_or_write(
                library,
                &PackedJudgeTables::of(&tables),
            )?;
        drop(tables);
        let (verdicts, judged_on) = judge_these_packed_tables_on_a_gpu_card(&packed)?;
        for (position, verdict) in group.iter().zip(&verdicts) {
            let words = crate::crash_judge_gpu::cross_check_words_of_gpu_verdict(verdict);
            let violated_bits = pool_checker_violated_bits(verdict);
            let violated_invariants = IMPLEMENTED_INVARIANTS
                .iter()
                .enumerate()
                .filter(|(bit, invariant)| {
                    violated_bits & (1u128 << bit) != 0
                        && images[*position]
                            .image
                            .invariants_judged
                            .includes(invariant)
                })
                .map(|(_, invariant)| (*invariant).to_string())
                .collect();
            judgements[*position] = Some(ImageJudgementRecord {
                violated_invariants,
                could_not_judge: words
                    .gpu_could_not_judge
                    .then(|| "结论里判不了那一位置上了（表越过内核容量）".to_string()),
                judged_on: judged_on.clone(),
            });
        }
        Ok(())
    })?;
    Ok(judgements
        .into_iter()
        .map(|judgement| judgement.expect("每一张盘都落在某一组里判过或记了判不了"))
        .collect())
}

/// 观察者交回的盘排成的一份判表（从库里读回的那一份）在卡上判全部状态，照卡的来路分两支：这一趟自己起的卡经 `memory_queue_of_the_cards_started_for_this_run` 按显存排队
/// （[`judge_on_a_card_started_for_this_run_queued_by_memory`]）；卡池那一支借第一张此刻放得下的卡，一次派活一条草稿缓冲那么宽
/// （按池里绑定上限最小的那张卡估）。交回逐状态的结论字与在哪张卡上判的。量显存额度、定派活宽度都在这里，不在观察者结论拼块那一组的闭包里
/// （拼块经 [`judge_the_images_after_the_observers_on_the_gpu`] 的参数调它）。
///
/// # Errors
/// 配了 GPU 而这一趟没有起好的卡；判器要的比每张卡的容量都大、借不到显存；判完之后等设备没等成。
#[cfg(feature = "gpu")]
fn judge_packed_tables_of_the_images_after_the_observers_on_a_card(
    cards: &CardsInUse,
    taken_from: GpuCardsSource,
    memory_queue_of_the_cards_started_for_this_run: &CardMemoryQueue,
    packed: &crate::crash_judge_dispatch::PackedJudgeTablesReadBack,
) -> Result<VerdictsOfThePackedTablesOnACard, String> {
    use crate::crash_judge_dispatch::{
        mebibytes_a_judge_needs, states_per_scratch_lane, GpuCrashJudge,
    };
    use crate::crash_judge_gpu::JudgeKernelSources;
    match taken_from {
        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
            if cards.is_empty() {
                return Err(
                    "配置开了 GPU，观察者交回的盘上的池级 checker 走 GPU，这一趟却没有起好的卡：不退回 CPU 判器"
                        .to_string(),
                );
            }
            judge_on_a_card_started_for_this_run_queued_by_memory(
                cards,
                memory_queue_of_the_cards_started_for_this_run,
                packed,
                &JudgeKernelSources::of_the_judge(),
                "observer-images",
            )
        }
        GpuCardsSource::TheCardPoolOfThisProcess => {
            let pool = the_card_pool_this_run_takes_its_cards_from();
            let states_per_dispatch =
                states_per_scratch_lane(pool.smallest_maximum_storage_buffer_binding_size());
            let lease = pool
                .lease_the_first_card_with_room_now(
                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                    mebibytes_a_judge_needs(packed, states_per_dispatch),
                )
                .map_err(|refusal| {
                    let reason = given_states_lease_refusal_as_a_run_condition_not_met(
                        "observer-images",
                        &refusal,
                    );
                    println!("{reason}");
                    reason
                })?;
            let judge = GpuCrashJudge::on_card_with_packed_tables(
                lease.card(),
                packed,
                states_per_dispatch,
            );
            let verdicts = judge.judge_states(0..packed.state_count());
            let judged_on = JudgingDevice::Gpu {
                card_type: lease.card().name.clone(),
                driver_version: lease.card().driver_version.clone(),
            };
            drop(judge);
            drop(lease);
            Ok((verdicts, judged_on))
        }
    }
}

/// 判段从判器交回的逐盘自证槽数看的那一种观察者（两槽全废盘面观察者）在判段的一块：读判器连同结论块存下的那一份逐盘槽数
/// （[`SelfVerifiedSystemConfigurationSlotsHandedBackForABlock`]，键与判器结论块同一把：块键、判器结论的种类 `judge_kind`、版本 `judge_version`）
/// 与判器那一块结论的计数向量（按发布分的态数照它取），槽数相同的连着几个状态交 `observe` 看一次（它只随这个状态与基镜像两份槽数变），
/// 拼成与录入块同形的一块（[`ObserverBlockRecord`]），判段往下照录入块那样拼结论。`observe` panic 记成红项（同录入段的接法）。
///
/// # Errors
/// 库读失败；这一块的逐盘槽数或判器结论不在库里（判段第 ① 段没判这一块，或判器那一组拒算、结论记在占位下）；读回的字节解不出；
/// 记的状态数与这一块对不上。
pub(super) fn observer_block_from_the_self_verified_slots_the_judge_handed_back(
    store: &VerdictStore,
    key: &VerdictBlockKey,
    states_of_the_block: u64,
    observer_name: &ObserverName,
    observe: &ObserveTheSelfVerifiedSystemConfigurationSlotsOfACrashState<'_>,
    judge_kind: &ConclusionKind,
    judge_version: &ConclusionVersion,
) -> Result<ObserverBlockRecord, String> {
    let library_error = |error: VerdictStoreError| format!("读库失败：{error:?}");
    let not_in_the_library = |what: &str| {
        format!(
            "判器{what}不在库里（{} 版本 {}）：先跑判段第 ① 段，判器判一块时连同结论块一起存；判器那一组拒算、结论记在占位下时这里读不到",
            judge_kind.version_text(),
            hexadecimal_text(&judge_version.0[..8])
        )
    };
    let bytes = store
        .read_observer_block(key, judge_kind, judge_version)
        .map_err(library_error)?
        .ok_or_else(|| not_in_the_library("交回的逐盘自证槽数"))?;
    let handed_back = SelfVerifiedSystemConfigurationSlotsHandedBackForABlock::from_bytes(&bytes)
        .ok_or_else(|| "判器交回的逐盘自证槽数解不出".to_string())?;
    let recorded_states = handed_back.slots_of_the_states.state_count();
    if recorded_states != states_of_the_block {
        return Err(format!(
            "判器交回的逐盘自证槽数记了 {recorded_states} 个状态，这一块 {states_of_the_block} 个"
        ));
    }
    let judged = store
        .read_conclusion_block(key, judge_kind, judge_version)
        .map_err(library_error)?
        .ok_or_else(|| not_in_the_library("那一块的结论"))?;
    let mut observations: Vec<ObservationOfStates> = Vec::new();
    let mut index_of_each_observation: std::collections::HashMap<ObservationOfStates, u32> =
        std::collections::HashMap::new();
    let mut observation_of_each_state: Vec<u32> =
        Vec::with_capacity(usize::try_from(states_of_the_block).expect("一块装得进 usize"));
    // 迭代上界：段数（不超过这一块的状态数）；每段看一次、按它的状态数逐个记下标
    for run in &handed_back.slots_of_the_states.runs {
        let mut counts = CrashPointReplayObserverCounts::default();
        let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            observe(
                &run.slots_of_each_device,
                &handed_back.slots_of_each_device_on_the_base_image,
                &mut counts,
            );
        }));
        let observation = match observed {
            Ok(()) => ObservationOfStates::Counted {
                red_item: None,
                counts: counts
                    .counts_in_name_order()
                    .map(|(name, count)| (name.to_string(), count))
                    .collect(),
            },
            Err(payload) => ObservationOfStates::Counted {
                red_item: Some(observer_red_item(
                    observer_name,
                    &text_carried_by_the_panic_payload(payload.as_ref()),
                )),
                counts: Vec::new(),
            },
        };
        let index = *index_of_each_observation
            .entry(observation.clone())
            .or_insert_with(|| {
                observations.push(observation);
                u32::try_from(observations.len() - 1)
                    .expect("一块最多 2^16 个状态，观察结果的清单装得进 u32")
            });
        for _ in 0..run.states {
            observation_of_each_state.push(index);
        }
    }
    Ok(ObserverBlockRecord {
        observations,
        observation_of_each_state,
        count_vector: judged.count_vector,
    })
}

/// 观察者那一把版本照判不入库的缘由（判段读不到它的录入块时报错用）：没在 `OBSERVER_ENTRIES` 登记的写明要登记，编译时它那一组拒算的写明拒算
/// （[`observer_versions_keeping_the_refused_out`]：这两种，观察者那一把就从这个进程的占位拼）；算得出键的，读不到的是别的进程按它自己的占位录下的块
/// （录入与判段各在一个进程里，那一个进程认它没登记或拒算）。
pub(super) fn why_the_observer_is_kept_out_of_the_library(
    observer: &ObserverOnTheNodes<'_>,
) -> String {
    match observer_product_code_key_in(
        &observer_code_units_of(observer),
        ObserverJudgingUnitsKeyTable::OfTheVerificationPackage,
    ) {
        Err(JudgingUnitsKeyRefused::NotComputedAtTheBuild { .. }) => {
            "这个观察者的判法单元拒算（没在 OBSERVER_ENTRIES 登记）".to_string()
        }
        Err(JudgingUnitsKeyRefused::RefusedAtTheBuild { .. }) => {
            "这个观察者的判法单元拒算（编译时它的闭包里有认不出的形态）".to_string()
        }
        Ok(_) => {
            "这个观察者那一把版本照判不入库（录入它的那个进程认它没登记或拒算，块按那个进程的占位录下）"
                .to_string()
        }
    }
}

#[cfg(test)]
mod tests;
