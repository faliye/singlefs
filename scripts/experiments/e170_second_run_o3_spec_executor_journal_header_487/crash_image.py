"""步 1：由基镜像与写流按落地向量建一个崩溃状态的镜像，并算镜像摘要（登记 5.3）。

.claude/kb/decisions/13-验证路线.md@已定项 4：崩溃点重放只枚举整写子集——
「没持久」的位置上放崩溃前那个位置的旧字节（不是零）；撕裂态那一次写的范围里放撕裂字节。
按流序叠：持久的写放新字节、撕裂的放 `torn-payload` 里那一段、没持久的不动（留着它之前的内容）。
镜像只存不全为零的 512 字节扇区；没存的扇区全为零。
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass

from crash_dump import OPERATION_KIND_WRITE_ZEROES, StreamDump

SECTOR_BYTES = 512  # D13（验证路线） 已定项 4「层 0 的扇区取 512 字节」
ZERO_SECTOR = bytes(SECTOR_BYTES)


@dataclass(frozen=True)
class PreparedWrite:
    """一次写在镜像上要改的扇区，事先切好；撕裂字节另切一份。"""

    write_ordinal: int
    stream_index: int
    device_identity: int
    byte_offset: int
    byte_count: int
    new_content: bytes
    torn_content: bytes | None


def prepare_writes(dump: StreamDump) -> list[PreparedWrite]:
    prepared = []
    for write_ordinal, stream_index in enumerate(dump.write_operation_indices):
        operation = dump.operations[stream_index]
        if operation.kind == OPERATION_KIND_WRITE_ZEROES:
            new_content = bytes(operation.byte_count)
        else:
            assert operation.payload is not None and len(operation.payload) == operation.byte_count
            new_content = operation.payload
        torn_content = dump.torn_payload_by_write_ordinal.get(write_ordinal)
        if torn_content is not None:
            assert len(torn_content) == operation.byte_count, "撕裂字节要罩住这次写的整个范围（D13 已定项 4）"
        prepared.append(PreparedWrite(write_ordinal, stream_index, operation.device_identity,
                                      operation.byte_offset, operation.byte_count, new_content, torn_content))
    return prepared


class SparseCrashImage:
    def __init__(self, sectors_by_device: dict[int, dict[int, bytes]]):
        self.sectors_by_device = sectors_by_device

    def device_identities(self) -> list[int]:
        return sorted(self.sectors_by_device)

    def write_bytes(self, device_identity: int, byte_offset: int, content: bytes) -> None:
        sectors = self.sectors_by_device.setdefault(device_identity, {})
        first_sector, offset_in_first = divmod(byte_offset, SECTOR_BYTES)
        if offset_in_first == 0 and len(content) % SECTOR_BYTES == 0:
            for sector_step in range(len(content) // SECTOR_BYTES):
                piece = content[sector_step * SECTOR_BYTES:(sector_step + 1) * SECTOR_BYTES]
                sectors[first_sector + sector_step] = piece
            return
        end_offset = byte_offset + len(content)
        for sector_index in range(first_sector, (end_offset + SECTOR_BYTES - 1) // SECTOR_BYTES):
            sector_start = sector_index * SECTOR_BYTES
            current = bytearray(sectors.get(sector_index, ZERO_SECTOR))
            overlap_start = max(sector_start, byte_offset)
            overlap_end = min(sector_start + SECTOR_BYTES, end_offset)
            current[overlap_start - sector_start:overlap_end - sector_start] = \
                content[overlap_start - byte_offset:overlap_end - byte_offset]
            sectors[sector_index] = bytes(current)

    def read_bytes(self, device_identity: int, byte_offset: int, byte_count: int) -> bytes | None:
        """设备不在池里返回 None（读不出）；在池里而没存的扇区读成全零。"""
        sectors = self.sectors_by_device.get(device_identity)
        if sectors is None:
            return None
        first_sector, offset_in_first = divmod(byte_offset, SECTOR_BYTES)
        last_sector = (byte_offset + byte_count + SECTOR_BYTES - 1) // SECTOR_BYTES
        joined = b"".join(sectors.get(sector_index, ZERO_SECTOR) for sector_index in range(first_sector, last_sector))
        return joined[offset_in_first:offset_in_first + byte_count]

    def nonzero_sector_indices_in(self, device_identity: int, byte_start: int, byte_end: int) -> list[int]:
        first_sector = byte_start // SECTOR_BYTES
        end_sector = (byte_end + SECTOR_BYTES - 1) // SECTOR_BYTES
        return sorted(sector_index for sector_index, content in self.sectors_by_device.get(device_identity, {}).items()
                      if first_sector <= sector_index < end_sector and content != ZERO_SECTOR)

    def digest(self) -> str:
        """镜像摘要（登记 5.3）：盘身份升序、不全为零的扇区按扇区号升序，依次喂盘身份 u32、扇区号 u64、512 字节。"""
        hasher = hashlib.sha256()
        for device_identity in sorted(self.sectors_by_device):
            sectors = self.sectors_by_device[device_identity]
            for sector_index in sorted(sectors):
                content = sectors[sector_index]
                if content == ZERO_SECTOR:
                    continue
                hasher.update(device_identity.to_bytes(4, "little"))
                hasher.update(sector_index.to_bytes(8, "little"))
                hasher.update(content)
        return hasher.hexdigest()


def copy_base_image(dump: StreamDump) -> SparseCrashImage:
    return SparseCrashImage({device: dict(sectors) for device, sectors in dump.base_sectors.items()}
                            | {device: {} for device in dump.device_identities if device not in dump.base_sectors})


LANDED_NOT_PERSISTED = "0"
LANDED_TORN = "1"
LANDED_PERSISTED = "2"


def build_crash_image(dump: StreamDump, prepared_writes: list[PreparedWrite], landing_vector: str) -> SparseCrashImage:
    assert len(landing_vector) == len(prepared_writes), "落地向量长度不等于写数（登记第十一节 V4）"
    image = copy_base_image(dump)
    for prepared in prepared_writes:
        landing = landing_vector[prepared.write_ordinal]
        if landing == LANDED_PERSISTED:
            image.write_bytes(prepared.device_identity, prepared.byte_offset, prepared.new_content)
        elif landing == LANDED_TORN:
            assert prepared.torn_content is not None, f"写 {prepared.write_ordinal} 没有撕裂字节却记成撕裂"
            image.write_bytes(prepared.device_identity, prepared.byte_offset, prepared.torn_content)
        else:
            assert landing == LANDED_NOT_PERSISTED, f"落地向量里有不认得的字符 {landing!r}"
    return image
