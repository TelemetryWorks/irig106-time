#!/usr/bin/env python3
"""Regenerate the docs snapshot from the CLI's packaged canonical schema."""
import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'apps/irig106-time-cli/schema/irigtime-report-v1.schema.json'
COPY = ROOT / 'docs/schema/irigtime-report-v1.schema.json'

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check:
        if not COPY.is_file() or COPY.read_bytes() != SOURCE.read_bytes():
            parser.error('schema snapshot differs; run python3 scripts/sync-cli-schema.py')
    else:
        COPY.write_bytes(SOURCE.read_bytes())
