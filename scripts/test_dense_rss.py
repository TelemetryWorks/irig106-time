"""Small decoder-independent wire/resource checks; never generate the CI file."""
from pathlib import Path
import struct
import tempfile
import unittest


class DenseRss(unittest.TestCase):
    def test_ci_runs_dense_gate(self):
        workflow = Path('.github/workflows/ci.yml').read_text()
        self.assertIn('python3 scripts/check-dense-rss.py', workflow)
        self.assertIn('test_dense_rss.py', workflow)

    def test_one_cycle_has_valid_headers_ratio_resets_and_literal_bcd(self):
        from dense_rss_workload import write_workload, CYCLE_BYTES
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'small.ch10'
            meta = write_workload(path, cycles=1)
            data = path.read_bytes()  # One 28 KiB cycle, never the large fixture.
        self.assertEqual(len(data), CYCLE_BYTES)
        offset, stamps = 0, []
        while offset < len(data):
            h = data[offset:offset + 24]
            sync, channel, length, data_length = struct.unpack_from('<HHII', h)
            self.assertEqual(sync, 0xEB25)
            self.assertEqual(length % 4, 0)
            self.assertLessEqual(data_length, length - 24)
            self.assertEqual(struct.unpack_from('<H', h, 22)[0],
                             sum(struct.unpack('<11H', h[:22])) & 0xFFFF)
            if h[15] == 0x11:
                self.assertEqual(channel, 7)
                self.assertEqual(data_length, 10)
                stamps.append(int.from_bytes(h[16:22], 'little'))
                if len(stamps) == 1:
                    # Fig11-13: day100, 12:00:00.00, not encoder-derived.
                    self.assertEqual(data[offset + 24:offset + 34],
                                     bytes.fromhex('00 00 00 00 00 00 00 12 00 01'))
                    self.assertEqual(data[offset + 34:offset + 36], b'\xDD\xDD')
                if len(stamps) == 45:
                    self.assertEqual(data[offset+28:offset+34], bytes.fromhex('50 40 00 12 00 01'))
                if len(stamps) == 50:
                    self.assertEqual(data[offset+28:offset+34], bytes.fromhex('00 09 01 12 00 01'))
                if len(stamps) == 62:
                    self.assertEqual(data[offset+28:offset+34], bytes.fromhex('00 49 01 12 00 01'))
            else:
                self.assertEqual((channel, length, data_length), (9, 28, 4))
            offset += length
        self.assertEqual(offset, len(data))
        self.assertEqual(len(stamps), 98)
        self.assertEqual(meta['packets'], 980)
        self.assertEqual(stamps[40:46], [400_000_000, 390_000_000,
                         410_000_000, 400_000_000, 405_000_000, 420_000_000])
        self.assertEqual(stamps[46:50], [10_000_000, 20_000_000, 30_000_000, 40_000_000])
        self.assertEqual(meta['resets'], 2)
        self.assertEqual(meta['jumps'], 2)
        self.assertEqual(meta['late'], 3)

    def test_multiple_cycles_and_expected_findings(self):
        from dense_rss_workload import write_workload, warnings_for, CYCLE_BYTES
        with tempfile.TemporaryDirectory() as tmp:
            meta = write_workload(Path(tmp) / 'two.ch10', cycles=2)
        self.assertEqual(meta['bytes'], 2 * CYCLE_BYTES)
        self.assertEqual(meta['resets'], 5)
        self.assertEqual(warnings_for('summary', meta), 15)
        self.assertEqual(warnings_for('correlate', meta), 12)
        self.assertEqual(warnings_for('timeline', meta), 524)
        self.assertEqual(warnings_for('csv', meta), 1388)


if __name__ == '__main__':
    unittest.main()
