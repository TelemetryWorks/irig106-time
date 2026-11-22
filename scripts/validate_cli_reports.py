#!/usr/bin/env python3
"""Validate live CLI JSON views against the packaged current Draft2020-12 schema.

Dev tooling only. CSV is its own19-column format, not a sixth JSON view.
Literal wire fixtures follow106-24R1 §11.2.1.1 and Figures11-12..17;
no production encoder generates the expected wire words.
"""
import argparse
import json
from pathlib import Path
import struct
import subprocess
import tempfile

from jsonschema import Draft202012Validator
from referencing import Registry

ROOT = Path(__file__).resolve().parent.parent
SCHEMA = ROOT / 'apps/irig106-time-cli/schema/irigtime-report-v1.schema.json'
VIEWS = ('summary', 'channels', 'jumps', 'timeline', 'correlate')


def validator(path=SCHEMA):
    schema = json.loads(path.read_text())
    if schema.get('$schema') != 'https://json-schema.org/draft/2020-12/schema':
        raise ValueError('Expected Draft2020-12 schema')
    Draft202012Validator.check_schema(schema)
    # All report references are local $defs. Never fetch a release URL/network
    # reference during validation, including the not-yet-published schema $id.
    return Draft202012Validator(schema, registry=Registry())


def validate_report(value, check):
    check.validate(value)


def packet(channel, kind, rtc_seconds, body):
    size = (24 + len(body) + 3) & ~3
    wire = bytearray(size)
    struct.pack_into('<HHII', wire, 0, 0xEB25, channel, size, len(body))
    wire[12], wire[15] = (6 if kind == 0x11 else 12), kind
    wire[16:22] = (rtc_seconds * 10_000_000).to_bytes(6, 'little')
    struct.pack_into('<H', wire, 22, sum(struct.unpack('<11H', wire[:22])) & 0xFFFF)
    wire[24:24+len(body)] = body
    wire[24+len(body):] = b'\xDD' * (size-24-len(body))  # Outside Data Length.
    return wire


def fixtures():
    # Fig11-12/13: FMT4/SRC1, unknown year, day10012:00; FMT5 native GPS.
    f1 = bytes.fromhex('41 00 00 00 00 00 00 12 00 01')
    gps = bytes.fromhex('51 00 00 00 00 00 00 12 00 01')
    # Fig11-15..17: NTF0 NTP, NTF1/2 IEEE1588. UTC2017-01-02 has
    # Unix1483315200; PTP adds37s. NTP era1 offset63104 =2036-02-08.
    ntp = struct.pack('<III', 1, 63104, 0x80000000)
    inventory = packet(7, 0x11, 0, f1) + packet(8, 0x11, 1, gps)
    for channel, word, seconds in [
        (9, 1, 63104), (10, 0x11, 1483315237),
        (11, 0x21, 1483315237), (12, 0x21, 1893542437),
        (13, 0x31, 0), (14, 0x23, 1483315237), (15, 0x20, 1483315237),
    ]:
        inventory += packet(channel, 0x12, channel, struct.pack('<III', word, seconds, 123))
    # Hard reset40->1; unknown-year F1 and data rows exercise segments,
    # ambiguous correlate answers and counter-only rows after resets.
    reset = bytearray()
    for rtc in [0, 40, 1, 2]:
        reset += packet(7, 0x11, rtc, f1)
        reset += packet(20, 9, rtc, b'\x01\x02\x03\x04')
    return [
        ('inventory', inventory, []), ('reset', reset, []),
        ('no-time', packet(20, 9, 0, b'\x01\x02\x03\x04'), []),
        ('ntp-no-era', packet(9, 0x12, 0, ntp), []),
        ('ntp-era1', packet(9, 0x12, 0, ntp), ['--ntp-era', '1']),
        ('ntp-sentinel', packet(9, 0x12, 0, bytes.fromhex('01 00 00 00') + bytes(8)),
         ['--ntp-era', '0']),
    ]


def validate_live_views(binary):
    check = validator()
    count = 0
    with tempfile.TemporaryDirectory(prefix='irigtime-schema-') as directory:
        for label, wire, options in fixtures():
            path = Path(directory) / f'{label}.ch10'
            path.write_bytes(wire)
            for command in VIEWS:
                args = [str(binary), command, str(path)]
                if command == 'correlate':
                    args.append('10000000' if label == 'reset' else '0')
                args += ['--format', 'json', '--fail-on', 'never', *options]
                result = subprocess.run(args, capture_output=True, timeout=30, check=False)
                if result.returncode != 0:
                    raise RuntimeError(f'{label}/{command}: exit{result.returncode}: {result.stderr!r}')
                if not result.stdout.isascii():
                    raise ValueError(f'{label}/{command}: non-ASCII JSON')
                try:
                    validate_report(json.loads(result.stdout), check)
                except Exception as error:
                    raise RuntimeError(f'{label}/{command}: schema validation failed') from error
                count += 1
                print(f'validated {label}/{command}', flush=True)
    print(f'Validated {count} live reports across all {len(VIEWS)} JSON views.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/debug/irigtime')
    args = parser.parse_args()
    validate_live_views(args.binary.resolve())
