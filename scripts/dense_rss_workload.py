"""Bounded generator: Ch11 §11.2.1.1 headers and Fig11-13 F1 DOY words.
Opaque non-time payloads are transport data, not a PCM decoder fixture.
"""
import struct

TICKS = 10_000_000  # 100ns RTC; Fig11-13 fraction is 0.01s.
GROUP_BYTES = 36 + 9 * 28
REFERENCES = [(n * TICKS, n * 100) for n in range(41)]
REFERENCES += [(39*TICKS, 3900), (41*TICKS, 4100),
               (40*TICKS, 4000), (40*TICKS + TICKS//2, 4050), (42*TICKS, 4200)]
for origin, end in [(60, 12), (100, 40)]:
    REFERENCES += [(n*TICKS, (origin+n+(5 if n >= 4 else 0))*100)
                   for n in range(1, end+1)]
CYCLE_BYTES = len(REFERENCES) * GROUP_BYTES
TIMELINE_LIMIT = 1000


def packet(kind, channel, rtc):
    length, data_length = (36, 10) if kind == 0x11 else (28, 4)
    p = bytearray(length)
    struct.pack_into('<HHII', p, 0, 0xEB25, channel, length, data_length)
    # Table11-4 current F1 data-type version 0x06; older known 0x01 for opaque data.
    p[12], p[15] = (6 if kind == 0x11 else 1), kind
    p[16:22] = rtc.to_bytes(6, 'little')
    struct.pack_into('<H', p, 22, sum(struct.unpack('<11H', p[:22])) & 0xFFFF)
    if kind == 0x11:
        p[34:36] = b'\xDD\xDD'  # Filler outside Data Length, never decoded.
    return p


def bcd(n):
    return (n // 10 << 4) | n % 10


def write_workload(path, cycles):
    if cycles < 1:
        raise ValueError('cycles must be positive')
    template, fields = bytearray(), []
    for rtc, clock in REFERENCES:
        fields.append((len(template) + 28, clock))
        template.extend(packet(0x11, 7, rtc))  # CSDW=0, FMT0/DATE0.
        for n in range(1, 10):
            template.extend(packet(9, 9, rtc + n * (TICKS//10)))
    with path.open('wb') as out:
        for cycle in range(cycles):
            block = bytearray(template)  # One 28KiB cycle; never the whole file.
            for offset, clock in fields:
                days, cs = divmod(4_320_000 + cycle*18_000 + clock, 8_640_000)
                seconds, fraction = divmod(cs, 100)
                hours, seconds = divmod(seconds, 3600)
                minutes, seconds = divmod(seconds, 60)
                day = 100 + days
                if day > 365:
                    raise ValueError('fixture exceeds its unknown-year DOY range')
                block[offset:offset+6] = bytes([bcd(fraction), bcd(seconds), bcd(minutes),
                                               bcd(hours), bcd(day % 100), day // 100])
            out.write(block)
    return dict(cycles=cycles, bytes=cycles*CYCLE_BYTES, packets=cycles*980,
                time_packets=cycles*98, resets=cycles*3-1,
                jumps=cycles*2, late=cycles*3, segments=cycles*3)


def warnings_for(command, meta):
    base = meta['resets'] + meta['late']
    if command in ('summary', 'jumps'):
        return base + meta['jumps']
    if command == 'correlate':
        return base + 1  # One per-channel overlap warning, not one per segment.
    if command == 'csv':
        return base + meta['time_packets']*9 - 387
    if command == 'timeline':
        groups = min(meta['time_packets'], TIMELINE_LIMIT//10)
        return base + 9*sum(g in (41, 43, 44) or g >= 46 for g in range(groups))
    return base


def write_steady_workload(path, rounds=100_000):
    """1025 same-counter references, then four channels at 100Hz for 1000s.

    The initial burst fills the shared correlation cap and exceeds the native
    quality pending cap (1024) before any RTC advance can drain it. The following
    400k reset-free references test long-lived storage; neither cap is per-channel.
    """
    warmup = 1025
    with path.open('wb') as out:
        for i in range(warmup):
            p = packet(0x11, 7 + i % 4, 0)
            p[28:34] = bytes.fromhex('00 00 00 12 00 01')
            out.write(p)
            for _ in range(9):
                out.write(packet(9, 11, 0))
        for n in range(rounds):
            block = bytearray()
            seconds, fraction = divmod(n, 100)
            minutes, seconds = divmod(seconds, 60)
            hours, minutes = divmod(minutes, 60)
            for channel in range(7, 11):
                p = packet(0x11, channel, n*(TICKS//100))
                p[28:34] = bytes([bcd(fraction), bcd(seconds), bcd(minutes),
                                  bcd(12+hours), 0, 1])
                block.extend(p)
                for i in range(1, 10):
                    block.extend(packet(9, 11, n*(TICKS//100)+i*(TICKS//1000)))
            out.write(block)
    return dict(bytes=(rounds*4+warmup)*GROUP_BYTES, packets=rounds*40+warmup*10,
                time_packets=rounds*4+warmup, resets=0, late=0, jumps=0, segments=0,
                steady=True, cap_warmup_references=warmup)
