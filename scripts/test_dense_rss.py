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

    def test_reset_free_multichannel_phase(self):
        from dense_rss_workload import write_steady_workload
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'steady.ch10'
            meta = write_steady_workload(path, rounds=3)
            data = path.read_bytes()
        self.assertEqual(meta['time_packets'], 12)
        self.assertEqual(meta['packets'], 120)
        stamps = {}
        offset = 0
        while offset < len(data):
            h = data[offset:offset+24]
            length = int.from_bytes(h[4:8], 'little')
            self.assertEqual(int.from_bytes(h[22:24], 'little'),
                             sum(struct.unpack('<11H', h[:22])) & 0xffff)
            if h[15] == 0x11:
                channel = int.from_bytes(h[2:4], 'little')
                stamps.setdefault(channel, []).append(int.from_bytes(h[16:22], 'little'))
                if channel == 7 and len(stamps[channel]) == 2:
                    self.assertEqual(data[offset+28:offset+34], bytes.fromhex('01 00 00 12 00 01'))
            offset += length
        self.assertEqual(stamps, {ch: [0, 100000, 200000] for ch in range(7, 11)})

    def test_streamed_summary_footer_after_eight_kib_is_checked(self):
        import io
        import runpy
        contains = runpy.run_path('scripts/check-dense-rss.py')['log_contains']
        class LinesOnly(io.StringIO):
            def read(self, *args):
                raise AssertionError('scan lines, do not slurp the log')
        log = LinesOnly('native session span\n' * 1000 + 'Findings: error=0, warning=7\nTotal: 2 jump(s)\n')
        self.assertTrue(contains(log, 'Findings: error=0, warning=7'))
        self.assertTrue(contains(log, 'Total: 2 jump(s)'))
        self.assertFalse(contains(log, 'warning=8'))

    def test_true_measurement_and_32_mib_gate(self):
        source = Path('scripts/check-dense-rss.py').read_text()
        self.assertIn('/usr/bin/time', source)
        self.assertNotIn('os.wait4', source)
        self.assertIn('32*1024', source)

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
