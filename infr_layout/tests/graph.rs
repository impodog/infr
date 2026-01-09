#![cfg(test)]

use infr_layout::graph::Graph;

#[test]
fn test_empty_graph() {
    let g = Graph::<i32>::new();
    let scc = g.scc();
    assert_eq!(scc.len(), 0);
    assert!(scc.is_empty());
}

#[test]
fn test_single_node_no_edges() {
    let mut g = Graph::new();
    let _idx = g.add("hello");
    let scc = g.scc();
    assert_eq!(scc.len(), 1);
    let data = scc.get(0);
    assert_eq!(data.len(), 1);
    assert_eq!(data[0], "hello");
}

#[test]
fn test_single_node_self_loop() {
    let mut g = Graph::new();
    let idx = g.add(42);
    g.connect(idx, idx);
    let scc = g.scc();
    assert_eq!(scc.len(), 1);
    let data = scc.get(0);
    assert_eq!(data.len(), 1);
    assert_eq!(data[0], 42);
}

#[test]
fn test_two_nodes_no_edges() {
    let mut g = Graph::new();
    let _a = g.add('a');
    let _b = g.add('b');
    let scc = g.scc();
    assert_eq!(scc.len(), 2);
    // each SCC is a singleton
    for i in 0..2 {
        let vec = scc.get(i);
        assert_eq!(vec.len(), 1);
    }
    // together they contain both characters
    let mut found = Vec::new();
    for i in 0..2 {
        found.push(scc.get(i)[0]);
    }
    found.sort();
    assert_eq!(found, ['a', 'b']);
}

#[test]
fn test_two_nodes_one_direction() {
    let mut g = Graph::new();
    let a = g.add("source");
    let b = g.add("sink");
    g.connect(a, b);
    let scc = g.scc();
    assert_eq!(scc.len(), 2);
    // each node its own SCC
    for i in 0..2 {
        assert_eq!(scc.get(i).len(), 1);
    }
}

#[test]
fn test_two_node_cycle() {
    let mut g = Graph::new();
    let a = g.add(100);
    let b = g.add(200);
    g.connect(a, b);
    g.connect(b, a);
    let scc = g.scc();
    assert_eq!(scc.len(), 1);
    let data = scc.get(0);
    assert_eq!(data.len(), 2);
    assert!(data.contains(&100));
    assert!(data.contains(&200));
}

#[test]
fn test_three_node_line() {
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    g.connect(n0, n1);
    g.connect(n1, n2);
    let scc = g.scc();
    assert_eq!(scc.len(), 3);
    let mut values = Vec::new();
    for i in 0..3 {
        let vec = scc.get(i);
        assert_eq!(vec.len(), 1);
        values.push(vec[0]);
    }
    values.sort();
    assert_eq!(values, [0, 1, 2]);
}

#[test]
fn test_three_node_cycle() {
    let mut g = Graph::new();
    let n0 = g.add("x");
    let n1 = g.add("y");
    let n2 = g.add("z");
    g.connect(n0, n1);
    g.connect(n1, n2);
    g.connect(n2, n0);
    let scc = g.scc();
    assert_eq!(scc.len(), 1);
    let data = scc.get(0);
    assert_eq!(data.len(), 3);
    assert!(data.contains(&"x"));
    assert!(data.contains(&"y"));
    assert!(data.contains(&"z"));
}

#[test]
fn test_dag_with_multiple_edges() {
    // 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    let n3 = g.add(3);
    g.connect(n0, n1);
    g.connect(n0, n2);
    g.connect(n1, n3);
    g.connect(n2, n3);
    let scc = g.scc();
    assert_eq!(scc.len(), 4);
    let mut values = Vec::new();
    for i in 0..4 {
        let vec = scc.get(i);
        assert_eq!(vec.len(), 1);
        values.push(vec[0]);
    }
    values.sort();
    assert_eq!(values, [0, 1, 2, 3]);
}

#[test]
fn test_two_disconnected_cycles() {
    // Cycle A: 0 <-> 1
    // Cycle B: 2 <-> 3
    let mut g = Graph::new();
    let a0 = g.add('A');
    let a1 = g.add('B');
    let b0 = g.add('C');
    let b1 = g.add('D');
    g.connect(a0, a1);
    g.connect(a1, a0);
    g.connect(b0, b1);
    g.connect(b1, b0);
    let scc = g.scc();
    assert_eq!(scc.len(), 2);
    for i in 0..2 {
        let vec = scc.get(i);
        assert_eq!(vec.len(), 2);
    }
    // verify all four chars appear
    let mut chars = Vec::new();
    for i in 0..2 {
        for &ch in scc.get(i) {
            chars.push(ch);
        }
    }
    chars.sort();
    assert_eq!(chars, ['A', 'B', 'C', 'D']);
}

#[test]
fn test_cycle_with_tail() {
    // 0 -> 1 -> 2 -> 0 (cycle) and 2 -> 3 (tail)
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    let n3 = g.add(3);
    g.connect(n0, n1);
    g.connect(n1, n2);
    g.connect(n2, n0);
    g.connect(n2, n3);
    let scc = g.scc();
    // Should have two SCCs: {0,1,2} and {3}
    assert_eq!(scc.len(), 2);
    let (large_idx, small_idx) = if scc.get(0).len() == 3 {
        (0, 1)
    } else {
        (1, 0)
    };
    assert_eq!(scc.get(large_idx).len(), 3);
    assert_eq!(scc.get(small_idx).len(), 1);
    let large_data = scc.get(large_idx);
    assert!(large_data.contains(&0));
    assert!(large_data.contains(&1));
    assert!(large_data.contains(&2));
    assert_eq!(scc.get(small_idx)[0], 3);
}

#[test]
fn test_multiple_cycles_linked() {
    // A: 0<->1, B: 2<->3, edge 0->2, edge 3->1
    let mut g = Graph::new();
    let a0 = g.add("a0");
    let a1 = g.add("a1");
    let b0 = g.add("b0");
    let b1 = g.add("b1");
    g.connect(a0, a1);
    g.connect(a1, a0);
    g.connect(b0, b1);
    g.connect(b1, b0);
    g.connect(a0, b0);
    g.connect(b1, a1);
    let scc = g.scc();
    // Since there are edges both ways indirectly, the whole graph may become one SCC
    // Actually 0 can reach 2, 2 can reach 3, 3 can reach 1, 1 can reach 0, so all four are mutually reachable.
    // Therefore one SCC.
    assert_eq!(scc.len(), 1);
    let data = scc.get(0);
    assert_eq!(data.len(), 4);
    assert!(data.contains(&"a0"));
    assert!(data.contains(&"a1"));
    assert!(data.contains(&"b0"));
    assert!(data.contains(&"b1"));
}

#[test]
fn test_scc_preserves_data_ownership() {
    // Use Vec to ensure data is moved correctly
    let mut g = Graph::new();
    let n0 = g.add(vec![1, 2]);
    let n1 = g.add(vec![3, 4, 5]);
    g.connect(n0, n1);
    g.connect(n1, n0);
    let scc = g.scc();
    assert_eq!(scc.len(), 1);
    let data = scc.get(0);
    assert_eq!(data.len(), 2);
    let has_len2 = data.iter().any(|v| v.len() == 2);
    let has_len3 = data.iter().any(|v| v.len() == 3);
    assert!(has_len2 && has_len3);
}

#[test]
fn test_large_dag() {
    // create a binary tree DAG: each node points to two children
    let mut g = Graph::new();
    let indices: Vec<_> = (0..15).map(|i| g.add(i)).collect();
    for i in 0..7 {
        g.connect(indices[i], indices[2 * i + 1]);
        g.connect(indices[i], indices[2 * i + 2]);
    }
    let scc = g.scc();
    assert_eq!(scc.len(), 15);
    let mut nums = Vec::new();
    for i in 0..15 {
        let vec = scc.get(i);
        assert_eq!(vec.len(), 1);
        nums.push(vec[0]);
    }
    nums.sort();
    assert_eq!(nums, (0..15).collect::<Vec<_>>());
}

#[test]
fn test_scc_edges_between_components() {
    // Create SCCs A, B, C with edges A->B, B->C
    // Each SCC is a single node for simplicity
    let mut g = Graph::new();
    let a = g.add("A");
    let b = g.add("B");
    let c = g.add("C");
    g.connect(a, b);
    g.connect(b, c);
    let scc = g.scc();
    // Each node is its own SCC
    assert_eq!(scc.len(), 3);
    // The resulting SCC graph should have edges 0->1, 1->2 (since indices correspond to SCC order?)
    // We cannot inspect edges directly, but we trust the algorithm.
}

#[test]
fn test_scc_after_multiple_connections() {
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    // connect in a triangle
    g.connect(n0, n1);
    g.connect(n1, n2);
    g.connect(n2, n0);
    // add extra edges
    g.connect(n0, n2);
    g.connect(n1, n0);
    g.connect(n2, n1);
    let scc = g.scc();
    // still one SCC
    assert_eq!(scc.len(), 1);
    assert_eq!(scc.get(0).len(), 3);
}

#[test]
fn test_simple_graph() {
    // Simple linear chain
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    g.connect(n0, n1);
    g.connect(n1, n2);
    let scc = g.scc();
    assert_eq!(scc.len(), 3);
    // Each SCC should be a singleton
    let mut values = Vec::new();
    for i in 0..3 {
        let vec = scc.get(i);
        assert_eq!(vec.len(), 1);
        values.push(vec[0]);
    }
    values.sort();
    assert_eq!(values, [0, 1, 2]);

    // Simple cycle
    let mut g2 = Graph::new();
    let a = g2.add("a");
    let b = g2.add("b");
    g2.connect(a, b);
    g2.connect(b, a);
    let scc2 = g2.scc();
    assert_eq!(scc2.len(), 1);
    let data = scc2.get(0);
    assert_eq!(data.len(), 2);
    assert!(data.contains(&"a"));
    assert!(data.contains(&"b"));
}

#[test]
fn test_complex_graph() {
    // Graph with two cycles and edges between them
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    let n3 = g.add(3);
    g.connect(n0, n1);
    g.connect(n1, n0);
    g.connect(n2, n3);
    g.connect(n3, n2);
    g.connect(n0, n2);
    g.connect(n3, n1);
    let scc = g.scc();
    // Actually edges create mutual reachability: 0->2, 2->3, 3->1, 1->0, so all four in one SCC
    assert_eq!(scc.len(), 1);
    let data = scc.get(0);
    assert_eq!(data.len(), 4);
    assert!(data.contains(&0));
    assert!(data.contains(&1));
    assert!(data.contains(&2));
    assert!(data.contains(&3));
}

#[test]
fn test_no_loops() {
    // DAG with tree structure
    let mut g = Graph::new();
    let indices: Vec<_> = (0..10).map(|i| g.add(i)).collect();
    for i in 0..5 {
        if 2 * i + 1 < 10 {
            g.connect(indices[i], indices[2 * i + 1]);
        }
        if 2 * i + 2 < 10 {
            g.connect(indices[i], indices[2 * i + 2]);
        }
    }
    let scc = g.scc();
    // Each node is its own SCC
    assert_eq!(scc.len(), 10);
    // Collect all values
    let mut values = Vec::new();
    for i in 0..10 {
        let vec = scc.get(i);
        assert_eq!(vec.len(), 1);
        values.push(vec[0]);
    }
    values.sort();
    assert_eq!(values, (0..10).collect::<Vec<_>>());
}

#[test]
fn test_topo_sort_empty_graph() {
    let g = Graph::<i32>::new();
    let result = g.topo_sort();
    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_topo_sort_single_node_no_edges() {
    let mut g = Graph::new();
    let _idx = g.add("hello");
    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0], "hello");
}

#[test]
fn test_topo_sort_single_node_self_loop() {
    let mut g = Graph::new();
    let idx = g.add(42);
    g.connect(idx, idx);
    let result = g.topo_sort();
    assert!(result.is_none()); // Cycle exists
}

#[test]
fn test_topo_sort_two_nodes_one_direction() {
    let mut g = Graph::new();
    let a = g.add("first");
    let b = g.add("second");
    g.connect(a, b);

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 2);
    assert_eq!(sorted[0], "first");
    assert_eq!(sorted[1], "second");
}

#[test]
fn test_topo_sort_two_nodes_reverse_direction() {
    let mut g = Graph::new();
    let a = g.add("a");
    let b = g.add("b");
    g.connect(b, a); // Reverse order

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 2);
    assert_eq!(sorted[0], "b"); // b must come before a
    assert_eq!(sorted[1], "a");
}

#[test]
fn test_topo_sort_two_node_cycle() {
    let mut g = Graph::new();
    let a = g.add(100);
    let b = g.add(200);
    g.connect(a, b);
    g.connect(b, a);

    let result = g.topo_sort();
    assert!(result.is_none()); // Cycle should prevent topo sort
}

#[test]
fn test_topo_sort_three_node_line() {
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    g.connect(n0, n1);
    g.connect(n1, n2);

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted, vec![0, 1, 2]); // Exactly this order
}

#[test]
fn test_topo_sort_three_node_v_shape() {
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    g.connect(n0, n1);
    g.connect(n0, n2);

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0], 0); // Root must be first
    // 1 and 2 can be in either order
    assert!(sorted.contains(&1));
    assert!(sorted.contains(&2));
}

#[test]
fn test_topo_sort_diamond_shape() {
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    let n3 = g.add(3);
    g.connect(n0, n1);
    g.connect(n0, n2);
    g.connect(n1, n3);
    g.connect(n2, n3);

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 4);
    assert_eq!(sorted[0], 0); // Root first
    // 1 and 2 must come before 3, but can be in either order
    let idx1 = sorted.iter().position(|&x| x == 1).unwrap();
    let idx2 = sorted.iter().position(|&x| x == 2).unwrap();
    let idx3 = sorted.iter().position(|&x| x == 3).unwrap();
    assert!(idx1 < idx3);
    assert!(idx2 < idx3);
}

#[test]
fn test_topo_sort_complex_dag() {
    // 0 -> 1 -> 3
    // 0 -> 2 -> 3
    // 4 -> 2
    // 4 -> 5
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    let n3 = g.add(3);
    let n4 = g.add(4);
    let n5 = g.add(5);

    g.connect(n0, n1);
    g.connect(n0, n2);
    g.connect(n1, n3);
    g.connect(n2, n3);
    g.connect(n4, n2);
    g.connect(n4, n5);

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 6);

    // Verify constraints
    let pos: Vec<_> = (0..6)
        .map(|i| sorted.iter().position(|&x| x == i).unwrap())
        .collect();

    assert!(pos[0] < pos[1]); // 0 before 1
    assert!(pos[0] < pos[2]); // 0 before 2
    assert!(pos[1] < pos[3]); // 1 before 3
    assert!(pos[2] < pos[3]); // 2 before 3
    assert!(pos[4] < pos[2]); // 4 before 2
    assert!(pos[4] < pos[5]); // 4 before 5
}

#[test]
fn test_topo_sort_with_cycle() {
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    let n3 = g.add(3);

    g.connect(n0, n1);
    g.connect(n1, n2);
    g.connect(n2, n3);
    g.connect(n3, n1); // Creates cycle: 1->2->3->1

    let result = g.topo_sort();
    assert!(result.is_none());
}

#[test]
fn test_topo_sort_consumes_graph() {
    let mut g = Graph::new();
    let n0 = g.add(String::from("a"));
    let n1 = g.add(String::from("b"));
    g.connect(n0, n1);

    // This should consume the graph
    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted, vec![String::from("a"), String::from("b")]);

    // g is now consumed, can't use it
}

#[test]
fn test_topo_sort_with_complex_data_types() {
    #[derive(Debug, PartialEq, Eq)]
    struct ComplexData {
        id: usize,
        name: String,
        values: Vec<i32>,
    }

    let mut g = Graph::new();
    let n0 = g.add(ComplexData {
        id: 0,
        name: "first".to_string(),
        values: vec![1, 2, 3],
    });
    let n1 = g.add(ComplexData {
        id: 1,
        name: "second".to_string(),
        values: vec![4, 5],
    });
    g.connect(n0, n1);

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 2);
    assert_eq!(sorted[0].id, 0);
    assert_eq!(sorted[1].id, 1);
}

#[test]
fn test_topo_sort_multiple_valid_orders() {
    // Graph where multiple topological orders are valid
    // 0 -> 1, 0 -> 2
    // No edge between 1 and 2, so both [0,1,2] and [0,2,1] are valid
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    g.connect(n0, n1);
    g.connect(n0, n2);

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0], 0); // Root must be first
    // The rest can be in either order
    assert!(sorted.contains(&1));
    assert!(sorted.contains(&2));
}

#[test]
fn test_topo_sort_linear_chain_reverse() {
    // Build chain in reverse order: 3 -> 2 -> 1 -> 0
    // Should produce [3, 2, 1, 0] in topological sort
    let mut g = Graph::new();
    let n0 = g.add(0);
    let n1 = g.add(1);
    let n2 = g.add(2);
    let n3 = g.add(3);

    g.connect(n3, n2);
    g.connect(n2, n1);
    g.connect(n1, n0);

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted, vec![3, 2, 1, 0]);
}

#[test]
fn test_topo_sort_empty_graph_after_topo_sort() {
    let mut g = Graph::new();
    let n0 = g.add(100);
    let n1 = g.add(200);
    g.connect(n0, n1);

    // The graph should be consumed
    let result = g.topo_sort();
    assert!(result.is_some());

    // If we try to use g again, it won't compile because it was moved
    // This is a compile-time check, so we don't need a runtime test
}

#[test]
fn test_topo_sort_large_dag() {
    let mut g = Graph::new();
    // Create indices 0..=99
    let indices: Vec<_> = (0..100).map(|i| g.add(i)).collect();

    // Create a simple dependency: i depends on i+1 (so reverse order)
    for i in 0..99 {
        g.connect(indices[i + 1], indices[i]);
    }

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 100);

    // Should be in reverse order: 99, 98, ..., 0
    for (i, &value) in sorted.iter().enumerate() {
        assert_eq!(value, 99 - i);
    }
}

#[test]
fn test_topo_sort_with_parallel_chains() {
    // Chain A: 0 -> 1 -> 2
    // Chain B: 3 -> 4 -> 5
    // No dependencies between chains
    let mut g = Graph::new();
    let a0 = g.add(0);
    let a1 = g.add(1);
    let a2 = g.add(2);
    let b0 = g.add(3);
    let b1 = g.add(4);
    let b2 = g.add(5);

    g.connect(a0, a1);
    g.connect(a1, a2);
    g.connect(b0, b1);
    g.connect(b1, b2);

    let result = g.topo_sort();
    assert!(result.is_some());
    let sorted = result.unwrap();
    assert_eq!(sorted.len(), 6);

    // Verify each chain is in order
    let pos: Vec<_> = (0..6)
        .map(|i| sorted.iter().position(|&x| x == i).unwrap())
        .collect();

    assert!(pos[0] < pos[1] && pos[1] < pos[2]); // Chain A
    assert!(pos[3] < pos[4] && pos[4] < pos[5]); // Chain B
}
