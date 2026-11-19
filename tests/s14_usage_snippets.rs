//! Documentation clauses only; no whole-requirement marker.
#[test]
fn network_usage_snippets_match_the_compiled_example() {
    let example = include_str!("../examples/format2_usage.rs");
    let usage = include_str!("../docs/usage.md");
    let section = usage
        .split("## 13. Processing Network Time Packets")
        .nth(1)
        .unwrap()
        .split("## 14.")
        .next()
        .unwrap();
    let snippets: Vec<_> = section
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
