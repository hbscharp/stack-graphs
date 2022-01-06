// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2021, stack-graphs authors.
// Licensed under either of Apache License, Version 2.0, or MIT license, at your option.
// Please see the LICENSE-APACHE or LICENSE-MIT files in this distribution for license details.
// ------------------------------------------------------------------------------------------------

use std::collections::BTreeSet;

use pretty_assertions::assert_eq;
use stack_graphs::graph::StackGraph;
use tree_sitter_stack_graphs::LoadError;
use tree_sitter_stack_graphs::StackGraphLanguage;

fn load_stack_graph(python_source: &str, tsg_source: &str) -> Result<StackGraph, LoadError> {
    let mut language = StackGraphLanguage::new(tree_sitter_python::language(), tsg_source).unwrap();
    let mut graph = StackGraph::new();
    let file = graph.get_or_create_file("test.py");
    language.load_stack_graph(&mut graph, file, python_source)?;
    Ok(graph)
}

fn check_stack_graph_edges(python_source: &str, tsg_source: &str, expected_edges: &[&str]) {
    let graph = load_stack_graph(python_source, tsg_source).expect("Could not load stack graph");
    let mut actual_edges = BTreeSet::new();
    for source in graph.iter_nodes() {
        for edge in graph.outgoing_edges(source) {
            actual_edges.insert(format!(
                "{} -{}-> {}",
                graph[source].display(&graph),
                edge.precedence,
                graph[edge.sink].display(&graph),
            ));
        }
    }
    let expected_edges = expected_edges
        .iter()
        .map(|s| s.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(expected_edges, actual_edges);
}

#[test]
fn can_create_edges() {
    let tsg = r#"
      (identifier) @id {
         node source
         attr (source) type = "definition", symbol = (source-text @id)
         node sink
         attr (sink) type = "reference", symbol = (source-text @id)
         edge source -> sink
      }
    "#;
    let python = "a";
    check_stack_graph_edges(
        python,
        tsg,
        &[
            "[test.py(0) definition a] -0-> [test.py(1) reference a]", //
        ],
    );
}

#[test]
fn can_create_edges_with_precedence() {
    let tsg = r#"
      (identifier) @id {
         node source
         attr (source) type = "definition", symbol = (source-text @id)
         node sink
         attr (sink) type = "reference", symbol = (source-text @id)
         edge source -> sink
         attr (source -> sink) precedence = 17
      }
    "#;
    let python = "a";
    check_stack_graph_edges(
        python,
        tsg,
        &[
            "[test.py(0) definition a] -17-> [test.py(1) reference a]", //
        ],
    );
}
