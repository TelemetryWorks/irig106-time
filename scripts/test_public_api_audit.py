"""API source/fingerprint negative probes; no Rust build or dependency download."""
from pathlib import Path
import unittest
from public_api_audit import BASE, check, declarations, diff_report, tokens


class ApiAudit(unittest.TestCase):
    def inventory(self, source):
        return declarations(source, 'src/example.rs')

    def test_return_argument_and_constant_changes_require_migration_ack(self):
        before = self.inventory('impl C { pub const DEFAULT: u64 = 2_000_000_000; pub fn f(x:u64)->i32 {0} }')
        _, accepted = diff_report(before, before)
        migration = 'API-DIFF-SHA256: ' + accepted
        for source in [
            'impl C { pub const DEFAULT:u64=1_100_000_000; pub fn f(x:u64)->i32{0} }',
            'impl C { pub const DEFAULT:u64=2_000_000_000; pub fn f(x:NanosDuration)->i32{0} }',
            'impl C { pub const DEFAULT:u64=2_000_000_000; pub fn f(x:u64)->Result<i32>{Ok(0)} }',
        ]:
            with self.subTest(source=source):
                diff, changed = diff_report(before, self.inventory(source))
                self.assertIn('-src/example.rs', diff)
                self.assertIn('+src/example.rs', diff)
                with self.assertRaisesRegex(ValueError, 'update migration prose'):
                    check(changed, migration)
                check(changed, 'API-DIFF-SHA256: ' + changed)
        with self.assertRaises(ValueError): check(accepted, 'no acknowledgement')

    def test_docs_format_and_function_bodies_do_not_change_signatures(self):
        a = self.inventory('/// pub fn fake(){}\n pub fn f(x:u64)->u64 {x+1}')
        b = self.inventory('#[doc="changed"] pub fn f ( x : u64 ) -> u64 { x+2 }')
        self.assertEqual(a, b)
        self.assertEqual(len(a), 1)

    def test_literals_nested_comments_and_test_modules(self):
        source = '''/* outer /* pub fn fake(){} */ */
            pub const LABEL:&str = r#"pub fn literal(){ } // literal"#;
            #[cfg(test)] mod tests { pub struct Fake; }
            #[test] pub fn fixture() {}
            pub const LETTER:char = '{';'''
        items = self.inventory(source)
        self.assertEqual(len(items), 2)
        self.assertTrue(any('LABEL' in i and '// literal' in i for i in items))
        self.assertFalse(any('Fake' in i or 'fixture' in i for i in items))
        for bad in ['/* open', '"open', 'r#"open']:
            with self.subTest(bad=bad), self.assertRaises(ValueError): tokens(bad)

    def test_type_traits_cfg_reexports_macros_and_impl_headers_are_visible(self):
        source = '''#[cfg(target_os="windows")] #[derive(Clone)]
            pub struct S { pub x:u64, private:u8 }
            pub enum E { A, B(u64) }
            pub trait T { fn f(&self)->u64; }
            pub use hidden::{A, B as C};
            impl T for S { fn f(&self)->u64 {0} }
            macro_rules! value { () => { pub struct Generated; } }
            value!();'''
        items = '\n'.join(self.inventory(source))
        for fragment in ['target_os', 'derive', 'private', 'enum E', 'trait T',
                         'B as C', 'impl T for S', 'macro_rules', 'value !']:
            self.assertIn(fragment, items)
        with self.assertRaises(ValueError): self.inventory('pub struct S {')

    def test_private_alias_and_constant_indirection_are_reviewed(self):
        a=self.inventory('type Offset=u64; const VALUE:u64=2; pub const DEFAULT:Offset=VALUE;')
        b=self.inventory('type Offset=i64; const VALUE:i64=1; pub const DEFAULT:Offset=VALUE;')
        self.assertNotEqual(diff_report(a,a)[1],diff_report(a,b)[1])

    def test_ci_fetches_the_pinned_baseline_and_runs_the_review_gate(self):
        workflow = (Path(__file__).resolve().parent.parent / '.github/workflows/ci.yml').read_text()
        for command in [f'git fetch --no-tags --depth=1 origin {BASE}',
                        'python -m unittest discover -s scripts -p test_public_api_audit.py',
                        'python scripts/public_api_audit.py --check --report']:
            self.assertIn('      - run: ' + command + '\n', workflow)
