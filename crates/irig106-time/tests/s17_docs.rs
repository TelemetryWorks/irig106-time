//! Current-guide examples only: clause evidence, no whole-requirement marker.
#[path = "../examples/qualified_usage.rs"]
mod qualified_usage;

#[test]
fn readme_wire_snippet_matches_the_executed_example() {
    let example = include_str!("../examples/qualified_usage.rs");
    let snippet = example
        .split("// BEGIN wire\n")
        .nth(1)
        .unwrap()
        .split("// END wire")
        .next()
        .unwrap();
    let readme = include_str!("../README.md");
    let actual = readme
        .split("```rust\n")
        .nth(1)
        .unwrap()
        .split("\n```")
        .next()
        .unwrap();
    let normalized = |s: &str| {
        s.lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_owned()
    };
    assert_eq!(normalized(snippet), normalized(actual));
}
