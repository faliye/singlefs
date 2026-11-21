//! 记录用的读者：G2、G4 的读集，恢复与 checker 经它读镜像。

use super::*;

// ───────────────────────────── 记录用的读者（G2、G4 的读集） ─────────────────────────────

/// 读集里的一次调用：位置（调用种类 + 参数，PC2c / PC4c 按它去掉）与整次调用（位置 + 答案）的摘要。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LoggedCall {
    pub(super) position: Digest128,
    pub(super) digest: Digest128,
    pub(super) description: PositionDescription,
}

/// 位置的原样（PC2c、PC4c 报前十个判别位置时给人看）：调用种类与至多三个参数（盘、偏移、长度）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PositionDescription {
    pub(super) kind: u8,
    pub(super) arguments: [u64; 3],
}

impl PositionDescription {
    pub(super) fn of(kind: CallKind, arguments: &[u64]) -> Self {
        let mut padded = [0u64; 3];
        for (slot, argument) in padded.iter_mut().zip(arguments) {
            *slot = *argument;
        }
        Self {
            kind: kind as u8,
            arguments: padded,
        }
    }

    pub(super) fn text(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.kind, self.arguments[0], self.arguments[1], self.arguments[2]
        )
    }
}

/// 一次读的原样：给单元级检查重放、G1 计数、K4 集合用。
pub(super) struct LoggedRead {
    pub(super) device: u32,
    pub(super) offset: u64,
    pub(super) length: usize,
    pub(super) content: Option<(Digest128, Vec<u8>)>,
}

/// 调用种类的标签（进位置摘要）。
#[derive(Clone, Copy, Debug)]
pub(super) enum CallKind {
    PoolDeviceIdentities = 1,
    PoolDeviceSize = 2,
    PoolRead = 3,
    PoolJournalHint = 4,
    PoolPhysicalBlockSize = 5,
    ImageDevices = 11,
    ImageDeviceBytes = 12,
    ImageRead = 13,
    ImageCandidateUnitSlots = 14,
    ImageCandidateJournalSlots = 15,
}

pub(super) fn position_digest(kind: CallKind, arguments: &[u64]) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_POSITION);
    builder.feed_word(kind as u64);
    for argument in arguments {
        builder.feed_word(*argument);
    }
    builder.finish()
}

pub(super) fn call_digest(position: Digest128, answer: Option<Digest128>) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_CALL);
    builder.feed_digest(position);
    match answer {
        Some(digest) => {
            builder.feed_word(1);
            builder.feed_digest(digest);
        }
        None => builder.feed_word(0),
    }
    builder.finish()
}

pub(super) fn digest_of_words(words: &[u64]) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_CONTENT);
    builder.feed_word(u64::try_from(words.len()).expect("个数"));
    for word in words {
        builder.feed_word(*word);
    }
    builder.finish()
}

/// 这一遍读者属于哪一遍：恢复两遍与 checker 正常那一遍进内容表，走树那一遍只取读集。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecordedPass {
    Recovery,
    CheckerNormal,
    CheckerWalk,
}

/// 包在崩溃镜像外面、只转发只记的读者（5.1 第 2 步）。答案一律原样交回；走树那一遍的单元候选对每块盘交回空表。
pub(super) struct RecordingReader<'image, 'tables> {
    pub(super) image: &'image CrashImage<'image>,
    pub(super) pass: RecordedPass,
    pub(super) ordinal: u64,
    pub(super) tables: &'tables RefCell<ContentTables>,
    pub(super) calls: RefCell<Vec<LoggedCall>>,
    pub(super) reads: RefCell<Vec<LoggedRead>>,
}

impl<'image, 'tables> RecordingReader<'image, 'tables> {
    pub(super) fn new(
        image: &'image CrashImage<'image>,
        pass: RecordedPass,
        ordinal: u64,
        tables: &'tables RefCell<ContentTables>,
    ) -> Self {
        Self {
            image,
            pass,
            ordinal,
            tables,
            calls: RefCell::new(Vec::new()),
            reads: RefCell::new(Vec::new()),
        }
    }

    fn log(&self, kind: CallKind, arguments: &[u64], answer: Option<Digest128>) {
        let position = position_digest(kind, arguments);
        self.calls.borrow_mut().push(LoggedCall {
            position,
            digest: call_digest(position, answer),
            description: PositionDescription::of(kind, arguments),
        });
    }

    fn log_read(
        &self,
        kind: CallKind,
        device: u32,
        offset: u64,
        length: usize,
        answer: Option<&Vec<u8>>,
    ) {
        let content = answer.map(|bytes| (digest_of_bytes(bytes), bytes.clone()));
        if let Some((digest, bytes)) = &content {
            match self.pass {
                RecordedPass::Recovery | RecordedPass::CheckerNormal => {
                    self.tables.borrow_mut().insert(
                        ReadCategory::of(offset, length),
                        *digest,
                        bytes,
                        self.ordinal,
                        self.pass == RecordedPass::CheckerNormal
                            && (ReadCategory::of(offset, length) != ReadCategory::JournalRing
                                || u64::try_from(length).expect("读长")
                                    == LOCAL_JOURNAL_RECORD_BYTES),
                    )
                }
                RecordedPass::CheckerWalk => {}
            }
        }
        self.log(
            kind,
            &[
                u64::from(device),
                offset,
                u64::try_from(length).expect("读长"),
            ],
            content.as_ref().map(|(digest, _)| *digest),
        );
        self.reads.borrow_mut().push(LoggedRead {
            device,
            offset,
            length,
            content,
        });
    }
}

impl PoolReader for RecordingReader<'_, '_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        let answer = PoolReader::device_identities(self.image);
        let words: Vec<u64> = answer
            .iter()
            .map(|identity| u64::from(identity.0))
            .collect();
        self.log(
            CallKind::PoolDeviceIdentities,
            &[],
            Some(digest_of_words(&words)),
        );
        answer
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        let answer = PoolReader::device_size_in_bytes(self.image, device);
        self.log(
            CallKind::PoolDeviceSize,
            &[u64::from(device.0)],
            answer.map(|size| digest_of_words(&[size])),
        );
        answer
    }

    /// 只读挂载逐盘探测块宽那一问同样记进调用日志：答案进挂载的结论，去重要看它。
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        let answer = PoolReader::physical_block_size_probed_on_the_device(self.image, device);
        self.log(
            CallKind::PoolPhysicalBlockSize,
            &[u64::from(device.0)],
            answer.map(|physical_block_size| digest_of_words(&[u64::from(physical_block_size.0)])),
        );
        answer
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let answer = PoolReader::read(self.image, device, offset, length);
        self.log_read(
            CallKind::PoolRead,
            device.0,
            offset.0,
            length,
            answer.as_ref(),
        );
        answer
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let answer =
            PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes);
        let words = answer
            .as_ref()
            .map(|offsets| offsets.iter().map(|offset| offset.0).collect::<Vec<u64>>());
        self.log(
            CallKind::PoolJournalHint,
            &[u64::from(device.0), ring_start.0, ring_bytes],
            words.map(|offsets| digest_of_words(&offsets)),
        );
        answer
    }
}

impl ImageReader for RecordingReader<'_, '_> {
    fn devices(&self) -> Vec<u32> {
        let answer = ImageReader::devices(self.image);
        let words: Vec<u64> = answer.iter().map(|device| u64::from(*device)).collect();
        self.log(CallKind::ImageDevices, &[], Some(digest_of_words(&words)));
        answer
    }

    fn device_bytes(&self, device: u32) -> Option<u64> {
        let answer = ImageReader::device_bytes(self.image, device);
        self.log(
            CallKind::ImageDeviceBytes,
            &[u64::from(device)],
            answer.map(|size| digest_of_words(&[size])),
        );
        answer
    }

    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let answer = ImageReader::read(self.image, device, offset, length);
        self.log_read(CallKind::ImageRead, device, offset, length, answer.as_ref());
        answer
    }

    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        let answer = match self.pass {
            RecordedPass::CheckerWalk => Some(Vec::new()),
            RecordedPass::Recovery | RecordedPass::CheckerNormal => {
                ImageReader::candidate_unit_slots(self.image, device)
            }
        };
        self.log(
            CallKind::ImageCandidateUnitSlots,
            &[u64::from(device)],
            answer.as_deref().map(digest_of_words),
        );
        answer
    }

    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        let answer = ImageReader::candidate_journal_slots(self.image, device);
        self.log(
            CallKind::ImageCandidateJournalSlots,
            &[u64::from(device)],
            answer.as_deref().map(digest_of_words),
        );
        answer
    }
}
