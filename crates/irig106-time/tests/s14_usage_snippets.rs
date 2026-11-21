//! Documentation clauses only; no whole-requirement marker.
// Reuse the example's #[test] fixture in the normal integration-test harness.
#[path = "../examples/format2_usage.rs"]
mod format2_usage;

#[test]
fn network_usage_snippets_match_the_compiled_example() {
    let example = include_str!("../examples/format2_usage.rs");
    let usage = include_str!("fixtures/format2_usage.md");
    let snippets: Vec<_> = usage
        .split("```rust\n")
        .skip(1)
        .map(|s| s.split("\n```").next().unwrap().trim())
        .collect();
    for (i, name) in ["workflow", "comparison"].iter().enumerate() {
        let start = format!("// BEGIN {name}\n");
        let end = format!("// END {name}");
        let expected = example
            .split(&start)
            .nth(1)
            .unwrap()
            .split(&end)
            .next()
            .unwrap()
            .trim();
        assert_eq!(snippets[i], expected, "{name} snippet must stay compiled");
    }
    assert_eq!(snippets.len(), 2);
}
