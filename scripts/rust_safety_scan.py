"""Conservative source-scope checks for this repository's Rust module tree.

Not macro expansion: literals/comments are masked, ordinary module paths and
item scopes followed. Unknown allowance scopes fail closed; rustc still checks
syntax. Dependency/macro implementation behaviour needs the separate audit.
"""
from dataclasses import dataclass, field, replace
from pathlib import Path
import re

PROTECTED = {"clippy::" + n for n in (
    "unwrap_used", "expect_used", "panic", "todo", "unimplemented",
    "unreachable", "dbg_macro", "print_stdout", "print_stderr")}
# Old Clippy spellings and suppression of their diagnostics must remain visible.
PROTECTED |= {f"clippy::{kind}_{operation}_used"
              for kind in ("option", "result") for operation in ("unwrap", "expect")}
PROTECTED |= {"renamed_and_removed_lints", "unknown_lints"}
BROAD = {"warnings", "clippy::all", "clippy::restriction", "clippy::pedantic", "clippy::nursery"}
CLI_PRINTS = {name: {"clippy::" + channel for channel in channels} for name, channels in {
    "cmd_summary": ["print_stdout"], "cmd_channels": ["print_stdout"],
    "cmd_jumps": ["print_stdout", "print_stderr"], "cmd_timeline": ["print_stdout"],
    "cmd_csv": ["print_stderr"], "cmd_correlate": ["print_stdout", "print_stderr"],
    "print_usage": ["print_stdout"], "open_file": ["print_stderr"],
    "main": ["print_stdout", "print_stderr"],
}.items()}


RAW = re.compile(r'(?:br|cr|r)(#*)"')
QUOTED = re.compile(r'(?:b|c)?"')
CHAR = re.compile(r"'(?:\\u\{[0-9a-fA-F_]+\}|\\[^\n]|[^'\\\n])'")


def mask(source):
    out, i = list(source), 0
    while i < len(source):
        end = i
        if source.startswith("//", i):
            end = source.find("\n", i)
            end = len(source) if end < 0 else end
        elif source.startswith("/*", i):
            end, depth = i + 2, 1
            while end < len(source) and depth:
                if source.startswith("/*", end): depth += 1; end += 2
                elif source.startswith("*/", end): depth -= 1; end += 2
                else: end += 1
        else:
            raw = RAW.match(source, i)
            quoted = QUOTED.match(source, i)
            char = CHAR.match(source, i)
            if raw:
                stop = '"' + raw.group(1)
                last = source.find(stop, raw.end())
                end = len(source) if last < 0 else last + len(stop)
            elif quoted:
                end = quoted.end()
                while end < len(source):
                    if source[end] == "\\": end += 2
                    elif source[end] == '"': end += 1; break
                    else: end += 1
            elif char:
                end = char.end()
        if end > i:
            out[i:end] = ["\n" if c == "\n" else " " for c in source[i:end]]
            i = end
        else:
            i += 1
    return "".join(out)


def close(code, start, left, right):
    depth = 0
    for i in range(start, len(code)):
        if code[i] == left: depth += 1
        elif code[i] == right:
            depth -= 1
            if depth == 0: return i + 1
    raise ValueError(f"unclosed {left} at {start}")


def root_policies(source):
    code = mask(source)
    result = []
    for match in re.finditer(r"^#!\[(?:forbid|deny)\(", code, re.M):
        prefix = code[:match.start()]
        if prefix.count("{") == prefix.count("}"):
            end = close(code, match.start() + 2, "[", "]")
            result.append(source[match.start():end])
    return result


@dataclass
class Scope:
    test_module: bool = False
    test_code: bool = False
    cli: str | None = None
    directory: Path = Path()
    anchor: Path = Path()


@dataclass
class Allowance:
    location: str
    lints: set[str]
    allowed: bool
    scope: str


@dataclass
class Report:
    allowances: list[Allowance] = field(default_factory=list)
    io_errors: list[str] = field(default_factory=list)
    source_errors: list[str] = field(default_factory=list)


def scan_project(root, include_cli=True):
    root = root.resolve()
    report, seen = Report(), set()

    def visit(path, role, test_module=False, test_code=False):
        path = path.resolve()
        assert path.is_relative_to(root), f"module leaves repository: {path}"
        key = (path, role, test_module, test_code)
        if key in seen: return
        seen.add(key)
        source = path.read_text()
        code = mask(source)
        attrs = []
        for m in re.finditer(r"#!?\[", code):
            end = close(code, m.end() - 1, "[", "]")
            attrs.append({"start": m.start(), "end": end, "body": re.sub(r"\br#(?=[A-Za-z_])", "", code[m.end():end-1]),
                          "raw": source[m.start():end], "inner": code[m.start()+1] == "!"})
        by_start = {a["start"]: a for a in attrs}

        def after_attrs(pos):
            while True:
                while pos < len(code) and code[pos].isspace(): pos += 1
                if pos not in by_start: return pos
                pos = by_start[pos]["end"]

        items = []
        for m in re.finditer(r"\b(mod|fn)\s+([A-Za-z_]\w*)", code):
            if any(a["start"] <= m.start() < a["end"] for a in attrs): continue
            term = re.search(r"[;{]", code[m.end():])
            assert term, f"unrecognized item at {path}:{m.start()}"
            at = m.end() + term.start()
            end = close(code, at, "{", "}") if code[at] == "{" else at + 1
            items.append({"start": m.start(), "body": at, "end": end,
                          "kind": m.group(1), "name": m.group(2), "attrs": []})
        item_at = {i["start"]: i for i in items}
        for a in attrs:
            if a["inner"]: continue
            pos = after_attrs(a["end"])
            match = re.match(r"(?:pub(?:\([^)]*\))?\s+)?(?:(?:async|unsafe|const|extern)\s+)*(mod|fn)\s+\w+", code[pos:])
            if match:
                found = re.search(r"\b(?:mod|fn)\b", match.group())
                a["item"] = item_at[pos + found.start()]
                a["item"]["attrs"].append(a)
        directory = path.parent if path.name in {"lib.rs", "main.rs", "mod.rs"} else path.with_suffix("")
        initial = Scope(test_module, test_code, None, directory, path.parent)

        def enclosing(pos):
            parents = [i for i in items if i["body"] < pos < i["end"] and code[i["body"]] == "{"]
            return min(parents, key=lambda i: i["end"] - i["body"])["scope"] if parents else initial

        extra_test = []
        for item in sorted(items, key=lambda i: i["start"]):
            parent = enclosing(item["start"])
            item["parent"] = parent
            bodies = [re.sub(r"\s+", "", a["body"]) for a in item["attrs"]]
            gated = "cfg(test)" in bodies
            scope = replace(parent, test_code=parent.test_code or gated or "test" in bodies)
            if item["kind"] == "mod":
                scope.test_module |= gated
                if code[item["body"]] == "{":
                    scope.directory = parent.directory / item["name"]
                    scope.anchor = scope.directory
            elif role == "cli" and path == root / "irig106-time-cli/src/main.rs" and code[:item["start"]].count("{") == code[:item["start"]].count("}"):
                scope.cli = item["name"]
            item["scope"] = scope
            if scope.test_code: extra_test.append((item["start"], item["end"]))
        def production(pos):
            return role in {"library", "cli"} and not (
                enclosing(pos).test_code or any(a <= pos < b for a, b in extra_test))

        def source_error(pos, message):
            report.source_errors.append(
                f"{path.relative_to(root)}:{source.count(chr(10), 0, pos)+1}: {message}")

        for a in attrs:
            if re.sub(r"\s+", "", a["body"]) == "cfg(test)" and "item" not in a:
                pos = after_attrs(a["end"])
                term = re.search(r"[;{]", code[pos:])
                assert term
                at = pos + term.start()
                extra_test.append((pos, close(code, at, "{", "}") if code[at] == "{" else at+1))
            scope = a["item"]["scope"] if "item" in a else enclosing(a["start"])
            for allow in re.finditer(r"\b(?:allow|expect)\s*\(", a["body"]):
                start = allow.end() - 1
                args = a["body"][start+1:close(a["body"], start, "(", ")")-1]
                names = set(re.findall(r"(?:clippy\s*::\s*)?[A-Za-z_]\w*", args))
                names = {re.sub(r"\s+", "", n) for n in names} & (PROTECTED | BROAD | {"unsafe_code"})
                if not names: continue
                allowed = not (names & (BROAD | {"unsafe_code"})) and (scope.test_module or names <= CLI_PRINTS.get(scope.cli, set()))
                where = "cfg(test) module" if scope.test_module else f"CLI {scope.cli}" if scope.cli else "production/root"
                location = f"{path.relative_to(root)}:{source.count(chr(10), 0, a['start'])+1}"
                report.allowances.append(Allowance(location, names, allowed, where))
        # No expansion is attempted: reject these simple source indirections.
        for m in re.finditer(r"\b(?:r#)?include\s*!", code):
            if production(m.start()):
                source_error(m.start(), "production include! is forbidden")
        for a in attrs:
            if "$" in a["body"] and production(a["start"]):
                source_error(a["start"], "production metavariable attribute is forbidden")
        if role == "library":
            for m in re.finditer(r"\bextern\s+crate\s+(?:r#)?std\b|\b(?:r#)?std\s*::", code):
                if enclosing(m.start()).test_code or any(a <= m.start() < b for a,b in extra_test): continue
                report.io_errors.append(f"{path.relative_to(root)}:{source.count(chr(10),0,m.start())+1}: production {m.group().strip()}")
        for item in items:
            if item["kind"] != "mod" or code[item["body"]] != ";": continue
            explicit = next((re.search(r'\bpath\s*=\s*"([^"]+)"', a["raw"]) for a in item["attrs"] if re.search(r'\bpath\s*=', a["raw"])), None)
            parent = item["parent"]
            if explicit:
                child = parent.anchor / explicit.group(1)
            else:
                child = parent.directory / (item["name"] + ".rs")
                if not child.is_file(): child = parent.directory / item["name"] / "mod.rs"
            visit(child, role, item["scope"].test_module, item["scope"].test_code)

    visit(root / "src/lib.rs", "library")
    # Scan remaining library source files too; test-only files were classified
    # by their actual module declaration, never by a *_tests filename heuristic.
    for path in sorted((root / "src").rglob("*.rs")):
        if not any(p == path.resolve() for p, *_ in seen): visit(path, "library")
    if include_cli:
        for name in ["lib.rs", "main.rs", "bin/irigtime.rs"]:
            cli = root / "irig106-time-cli/src" / name
            if cli.is_file(): visit(cli, "cli")
    if include_cli:
        for directory in ["tests", "examples", "benches", "fuzz/fuzz_targets",
                          "irig106-time-cli/src", "irig106-time-cli/tests"]:
            for path in sorted((root / directory).rglob("*.rs")):
                if not any(p == path.resolve() for p, *_ in seen):
                    visit(path, "cli" if directory == "irig106-time-cli/src" else "auxiliary")
    return report
