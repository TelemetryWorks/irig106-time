"""Source guard fixtures; no Rust fixture is executed."""
from pathlib import Path
import tempfile
import unittest
from rust_safety_scan import scan_project, mask, root_policies


class ScopeGuards(unittest.TestCase):
    def scan(self, lib, files=None, cli=None):
        with tempfile.TemporaryDirectory(prefix="time-scan-") as tmp:
            root = Path(tmp)
            paths = {"crates/irig106-time/src/lib.rs": lib, **(files or {})}
            if cli is not None:
                paths["apps/irig106-time-cli/src/main.rs"] = cli
            for name, text in paths.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
            return scan_project(root)

    def test_renamed_aliases_and_diagnostic_suppressors_are_listed(self):
        names = ['renamed_and_removed_lints', 'unknown_lints'] + [
            f'clippy::{kind}_{operation}_used'
            for kind in ['option', 'result'] for operation in ['unwrap', 'expect']]
        for directive in ['allow', 'expect']:
            for name in names:
                with self.subTest(directive=directive, name=name):
                    report = self.scan(f'#[{directive}({name})] fn f(){{}}')
                    self.assertEqual(len(report.allowances), 1)
                    self.assertEqual(report.allowances[0].lints, {name})
                    self.assertFalse(report.allowances[0].allowed)
        report = self.scan('#[allow(renamed_and_removed_lints, clippy::option_unwrap_used)] fn f(){}')
        self.assertEqual(report.allowances[0].lints,
                         {'renamed_and_removed_lints', 'clippy::option_unwrap_used'})
        self.assertFalse(report.allowances[0].allowed)

    def test_include_cannot_hide_allowance_in_non_rs_file(self):
        for cli in [False, True]:
            with self.subTest(cli=cli):
                source = 'include!("evil.inc");'
                prefix = 'apps/irig106-time-cli/src' if cli else 'crates/irig106-time/src'
                report = self.scan('' if cli else source,
                    {f'{prefix}/evil.inc': '#[allow(clippy::unwrap_used)] fn f(){}'},
                    cli=source if cli else None)
                self.assertTrue(any('include!' in e for e in report.source_errors))

    def test_include_cannot_hide_std_in_non_rs_file(self):
        report = self.scan('include!("evil.inc");', {
            'crates/irig106-time/src/evil.inc': 'extern crate std; fn f(){std::env::var("x");}'})
        self.assertTrue(any('include!' in e for e in report.source_errors))

    def test_metavariable_attribute_is_rejected_in_production_macros(self):
        source = 'macro_rules! m { ($l:ident) => { #[$l(clippy::unwrap_used)] fn f(){} }; }'
        for cli in [False, True]:
            with self.subTest(cli=cli):
                report = self.scan('' if cli else source, cli=source if cli else None)
                self.assertTrue(any('metavariable attribute' in e for e in report.source_errors))

    def test_metavariables_anywhere_in_production_attributes_are_rejected(self):
        attributes = ['#[cfg_attr(all(), $l(clippy::unwrap_used))]', '#[allow($n)]']
        for attr in attributes:
            source = 'macro_rules! m { ($l:ident, $n:path) => { ' + attr + ' fn f(){} }; }'
            for cli in [False, True]:
                with self.subTest(attr=attr, cli=cli):
                    report = self.scan('' if cli else source, cli=source if cli else None)
                    self.assertTrue(any('metavariable attribute' in e for e in report.source_errors))
        control = self.scan('''#[doc = "dollar $ inside a literal"] fn f(){}
#[cfg(test)] mod tests { macro_rules! m { ($n:path) => { #[allow($n)] fn f(){} }; } }
''')
        self.assertEqual(control.source_errors, [])

    def test_test_scopes_and_literals_do_not_trigger_source_guards(self):
        report = self.scan('''#[cfg(test)] mod tests {
include!("evil.inc");
macro_rules! m { ($l:ident) => { #[$l(clippy::unwrap_used)] fn f(){} }; }
}
const TEXT:&str = r"include!(not_code); #[$l(clippy::unwrap_used)]";
// include!(not_code); #[$l(clippy::unwrap_used)]
''', {'crates/irig106-time/src/tests/evil.inc': ''})
        self.assertEqual(report.source_errors, [])

    def test_root_allow_and_production_unwrap_are_rejected(self):
        report = self.scan('#![allow(clippy::unwrap_used)]\nfn f(x:Option<u8>){x.unwrap();}')
        self.assertEqual(len(report.allowances), 1)
        self.assertFalse(report.allowances[0].allowed)

    def test_raw_lint_identifier_is_not_a_bypass(self):
        report = self.scan('#![allow(clippy::r#unwrap_used)]')
        self.assertFalse(report.allowances[0].allowed)

    def test_policy_headers_in_raw_strings_are_not_enforcement(self):
        source = 'const X:&str=r#"\n#![deny(clippy::unwrap_used)]\n"#;'
        self.assertEqual(root_policies(source), [])

    def test_escaped_chars_do_not_change_brace_scope(self):
        code = mask(r"""fn f<'a>() { let open='\u{7b}'; let quote='\''; }""")
        self.assertEqual(code.count('{'), 1)
        self.assertEqual(code.count('}'), 1)

    def test_tests_filename_does_not_make_production_a_test(self):
        report = self.scan('#[path="bad_tests.rs"] mod child;', {
            "crates/irig106-time/src/bad_tests.rs": '#![allow(clippy::unwrap_used)]\nextern crate std;'})
        self.assertFalse(report.allowances[0].allowed)
        self.assertEqual(len(report.io_errors), 1)

    def test_conditional_production_allow_is_rejected(self):
        report = self.scan('#[cfg_attr(test,allow(clippy::expect_used))]\nfn f(){}')
        self.assertFalse(report.allowances[0].allowed)

    def test_production_child_allow_is_rejected(self):
        report = self.scan('mod child;', {"crates/irig106-time/src/child.rs": '#[allow(clippy::panic)] fn f(){}'})
        self.assertFalse(report.allowances[0].allowed)

    def test_inline_and_external_test_module_allowances(self):
        report = self.scan('''#[cfg(test)] #[allow(clippy::unwrap_used)] mod inline {}
#[cfg(test)] #[path="assertions.rs"] mod unit;''', {
            "crates/irig106-time/src/assertions.rs": '#![allow(clippy::expect_used)]\nfn f(){let _=std::env::var("x");}'})
        self.assertEqual(len(report.allowances), 2)
        self.assertTrue(all(a.allowed for a in report.allowances))
        self.assertEqual(report.io_errors, [])

    def test_cli_only_named_function_and_print_channel_are_allowed(self):
        report = self.scan('', cli='''#[allow(clippy::print_stdout)] fn main(){}
#[allow(clippy::print_stdout)] fn parse(){}
#[allow(clippy::unwrap_used)] fn cmd_csv(){}''')
        self.assertEqual([a.allowed for a in report.allowances], [True, False, False])

    def test_unsafe_and_blanket_allowances_have_no_exception(self):
        report = self.scan('#[cfg(test)] #[allow(unsafe_code,clippy::all)] mod tests {}')
        self.assertFalse(report.allowances[0].allowed)

    def test_child_std_import_alias_and_path_are_rejected(self):
        report = self.scan('mod util;', {"crates/irig106-time/src/util.rs":
            'extern crate std as host; fn f(){let _=std::time::SystemTime::now();}'})
        self.assertEqual(len(report.io_errors), 2)
        self.assertTrue(all('crates/irig106-time/src/util.rs' in e for e in report.io_errors))

    def test_test_only_std_import_is_not_production_io(self):
        report = self.scan('#[cfg(test)] extern crate std;\n#[cfg(test)] fn f(){std::env::var("x");}')
        self.assertEqual(report.io_errors, [])

    def test_comments_strings_raw_strings_and_chars_are_ignored(self):
        report = self.scan('''// #[allow(clippy::unwrap_used)] std::fs::read
const X:&str=r###"extern crate std; #[allow(unsafe_code)] std::env::var(\"x\"); }"###;
fn f<'a>(x:&'a str){let c='}'; let s="std::fs::read";}
''')
        self.assertEqual(report.allowances, [])
        self.assertEqual(report.io_errors, [])


if __name__ == "__main__":
    unittest.main()
