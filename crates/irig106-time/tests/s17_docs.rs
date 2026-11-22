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

#[path = "../examples/migration_0_8.rs"]
mod migration_0_8;

#[test]
fn typed_migration_snippets_match_the_executed_example() {
    let example = include_str!("../examples/migration_0_8.rs");
    let fixture = include_str!("fixtures/migration_0_8.md");
    let snippets: Vec<_> = fixture
        .split("```rust\n")
        .skip(1)
        .map(|s| s.split("\n```").next().unwrap())
        .collect();
    for (i, name) in ["durations", "leap"].iter().enumerate() {
        let start = format!("// BEGIN {name}\n");
        let end = format!("// END {name}");
        let expected = example
            .split(&start)
            .nth(1)
            .unwrap()
            .split(&end)
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
        assert_eq!(normalized(snippets[i]), normalized(expected));
    }
    assert_eq!(snippets.len(), 2);
}
