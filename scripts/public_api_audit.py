"""Conservative source API inventory, not rustdoc expansion or a semver proof.

Includes public declarations and supporting types/constants, impl headers, macros, cfg/derive
attributes, direct dependency metadata and constant initializers. Public items
in private modules/private fields may over-report. External re-exported types,
conditional reachability and function-body behavior still need human review.
"""
import difflib
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib
from rust_safety_scan import RAW, QUOTED, CHAR

ROOT = Path(__file__).resolve().parent.parent
# Annotated v0.7.0 tag a5d128bba917fc97aa003cc7f5f53997288aa5c9, peeled commit:
BASE = '4ea0006723fc8d75925bd57b107e6dd8893be66e'
MIGRATION = ROOT / 'docs/MIGRATING-0.8.md'
WORD = re.compile(r'[A-Za-z_]\w*|\d[\w.]*|::|->|=>|\S')


def tokens(source):
    result, i = [], 0
    while i < len(source):
        if source[i].isspace(): i += 1; continue
        if source.startswith('//', i):
            end = source.find('\n', i)
            i = len(source) if end < 0 else end
            continue
        if source.startswith('/*', i):
            i, depth = i + 2, 1
            while i < len(source) and depth:
                if source.startswith('/*', i): depth += 1; i += 2
                elif source.startswith('*/', i): depth -= 1; i += 2
                else: i += 1
            if depth: raise ValueError('unterminated block comment')
            continue
        raw, quoted, char = RAW.match(source, i), QUOTED.match(source, i), CHAR.match(source, i)
        if raw:
            stop = '"' + raw.group(1)
            end = source.find(stop, raw.end())
            if end < 0: raise ValueError('unterminated raw literal')
            end += len(stop)
        elif quoted:
            end, closed = quoted.end(), False
            while end < len(source):
                if source[end] == '\\': end += 2
                elif source[end] == '"': end += 1; closed = True; break
                else: end += 1
            if not closed: raise ValueError('unterminated quoted literal')
        elif char: end = char.end()
        else: end = WORD.match(source, i).end()
        result.append(source[i:end]); i = end
    return result


def declarations(source, path):
    ts, pairs, stack = tokens(source), {}, []
    for i, token in enumerate(ts):
        if token in ('(', '[', '{'): stack.append(i)
        elif token in (')', ']', '}'):
            if not stack or ts[stack[-1]] != {')': '(', ']': '[', '}': '{'}[token]:
                raise ValueError(f'{path}: unmatched delimiter')
            pairs[stack.pop()] = i
    if stack: raise ValueError(f'{path}: unclosed delimiter')
    out = []

    def text(a, b): return ' '.join(ts[a:b])

    def walk(begin, end, scope):
        i = begin
        while i < end:
            if ts[i] == ';': i += 1; continue
            attrs = []
            while i < end and ts[i] == '#':
                j = i + 1 + int(i + 1 < end and ts[i + 1] == '!')
                if j >= end or ts[j] != '[': raise ValueError(f'{path}: unknown attribute')
                attrs.append(ts[j + 1:pairs[j]]); i = pairs[j] + 1
            if i >= end: break
            test = any(a == ['cfg', '(', 'test', ')'] or a == ['test'] for a in attrs)
            attrs = [a for a in attrs if a and a[0] not in {'doc', 'inline', 'allow', 'deny', 'warn', 'forbid'}]
            start, public = i, ts[i] == 'pub'
            if public:
                i += 1
                if i < end and ts[i] == '(':
                    public = False; i = pairs[i] + 1
            p = i
            while p < end and ts[p] in {'unsafe', 'async', 'default', 'auto'}: p += 1
            if p + 1 < end and ts[p:p + 2] == ['const', 'fn']: p += 1
            if p < end and ts[p] == 'extern' and p + 1 < end and ts[p + 1].startswith('"'):
                p += 2
            if p >= end: raise ValueError(f'{path}: incomplete declaration')
            kind, j = ts[p], p + 1
            # Item delimiters inside parameter/array/attribute groups are not bodies.
            while j < end and ts[j] != ';' and (ts[j] != '{' or kind in {'use', 'const', 'static', 'type'}):
                if ts[j] in ('(', '[', '{'): j = pairs[j] + 1
                else: j += 1
            if j >= end: raise ValueError(f'{path}: unknown item after {text(start, p+1)}')
            finish = pairs[j] + 1 if ts[j] == '{' else j + 1
            macro = p + 1 < end and ts[p + 1] == '!'
            if macro and ts[j] == '{' and finish < end and ts[finish] == ';': finish += 1
            if not test:
                prefix = path + (' :: ' + scope if scope else '')
                annotation = ' '.join(' '.join(a) for a in attrs)
                if public or kind in {'impl', 'type', 'const', 'static', 'struct', 'enum', 'trait'} or macro:
                    last = j if kind in {'fn', 'impl', 'mod'} else finish
                    out.append(f'{prefix} :: {annotation} {text(start, last)}'.strip())
                if kind in {'impl', 'mod'} and ts[j] == '{':
                    walk(j + 1, pairs[j], scope + ' / ' + text(p, j))
            i = finish
    walk(0, len(ts), '')
    return sorted(out)


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True)


def inventory(ref=None):
    read = (lambda name: git('show', f'{ref}:{name}')) if ref else lambda name: (ROOT / name).read_text()
    scopes = [('src/', 'src/', 'Cargo.toml')] if ref else [
        ('crates/irig106-time/src/', 'src/', 'crates/irig106-time/Cargo.toml'),
        ('apps/irig106-time-cli/src/', 'cli/src/', 'apps/irig106-time-cli/Cargo.toml')]
    # v0.7.0's CLI is binary-only, with no host-callable library API.
    items = []
    for prefix, logical, manifest_path in scopes:
        files = git('ls-tree', '-r', '--name-only', ref, prefix).splitlines() if ref else [
            p.relative_to(ROOT).as_posix() for p in (ROOT / prefix).rglob('*.rs')]
        for name in sorted(files):
            if name.endswith('.rs') and not name.endswith('_tests.rs') and '/tests/' not in name:
                items.extend(declarations(read(name), logical + name.removeprefix(prefix)))
        manifest = tomllib.loads(read(manifest_path))
        metadata = {name: manifest.get(name, {}) for name in ['features', 'dependencies', 'build-dependencies']}
        metadata['target'] = {target: {name: data.get(name, {}) for name in ['dependencies', 'build-dependencies']}
                              for target, data in manifest.get('target', {}).items()}
        workspace = tomllib.loads(read('Cargo.toml')).get('workspace', {}).get('dependencies', {})
        metadata['workspace_edges'] = {name: workspace.get(name) for name, data in metadata['dependencies'].items()
                                       if isinstance(data, dict) and data.get('workspace')}
        items.append(logical + 'Cargo = ' + json.dumps(metadata, sort_keys=True))
    return sorted(items)


def diff_report(before, after):
    diff = ''.join(difflib.unified_diff([s+'\n' for s in before], [s+'\n' for s in after],
                                     fromfile='v0.7.0@'+BASE, tofile='current-source', n=0))
    fingerprint = hashlib.sha256((BASE + '\n' + diff).encode()).hexdigest()
    return diff, fingerprint


def report():
    return diff_report(inventory(BASE), inventory())


def check(fingerprint, migration):
    marker = 'API-DIFF-SHA256: ' + fingerprint
    if marker not in migration.splitlines():
        raise ValueError('Public declarations/constants/dependencies changed or audit unacknowledged. '
                         'Review --report, update migration prose, then its API-DIFF-SHA256 marker: ' + fingerprint)


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--report', action='store_true', help='print complete source-declaration diff')
    args = parser.parse_args()
    try:
        diff, fingerprint = report()
        if args.report: print(diff, end='')
        print('API-DIFF-SHA256: ' + fingerprint)
        if args.check: check(fingerprint, MIGRATION.read_text())
    except (ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, str(error) + '\n')


if __name__ == '__main__': main()
