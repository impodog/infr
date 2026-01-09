use std::{collections::VecDeque, ptr::NonNull};

/// A node storing data of type `T` in `Graph`.
struct Node<T> {
    data: NonNull<T>,
    next: Vec<usize>,
    low: usize,
    dfn: usize,
    /// In-degree of the node.
    deg: usize,
    in_stack: bool,
    /// The SCC index of this node.
    scc: usize,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Self {
            data: Box::leak(Box::new(data)).into(),
            next: Vec::new(),
            low: 0,
            dfn: 0,
            deg: 0,
            in_stack: false,
            scc: 0,
        }
    }
}

/// A graph that represents its nodes by indices returned when they were inserted.
pub struct Graph<T> {
    nodes: Vec<Node<T>>,
    current_dfn: usize,
    current_scc: usize,
    dfs_stack: Vec<usize>,
}

impl<T> Drop for Graph<T> {
    fn drop(&mut self) {
        for node in 0..self.nodes.len() {
            unsafe {
                // Drops the boxed node.
                self.get_boxed(node);
            }
        }
        self.nodes.clear();
    }
}

impl<T> Graph<T> {
    /// Creates an empty graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            // This will be changed by `Self::scc`
            current_dfn: 0,
            current_scc: 0,
            dfs_stack: Vec::new(),
        }
    }

    /// Adds a new node to the graph and returns its index.
    pub fn add(&mut self, data: T) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::new(data));
        index
    }

    /// Connects two nodes in the graph.
    ///
    /// Panics if either `from` or `to` is not a valid node index.
    pub fn connect(&mut self, from: usize, to: usize) {
        assert!(from < self.nodes.len(), "Invalid from node index");
        assert!(to < self.nodes.len(), "Invalid to node index");
        self.nodes[from].next.push(to);
        self.nodes[to].deg += 1;
    }

    /// Returns a reference to the data of a node.
    ///
    /// Panics if `node` is not a valid node index.
    pub fn get(&self, node: usize) -> &T {
        assert!(node < self.nodes.len(), "Invalid node index");
        unsafe { self.nodes[node].data.as_ref() }
    }

    /// Returns a mutable reference to the data of a node.
    ///
    /// Panics if `node` is not a valid node index.
    pub fn get_mut(&mut self, node: usize) -> &mut T {
        assert!(node < self.nodes.len(), "Invalid node index");
        unsafe { self.nodes[node].data.as_mut() }
    }

    /// Returns the degree of a node.
    ///
    /// Panics if `node` is not a valid node index.
    pub fn get_deg(&self, node: usize) -> usize {
        assert!(node < self.nodes.len(), "Invalid node index");
        self.nodes[node].deg
    }

    /// Returns a reference to the next nodes of a node.
    ///
    /// Panics if `node` is not a valid node index.
    pub fn get_next(&self, node: usize) -> &[usize] {
        assert!(node < self.nodes.len(), "Invalid node index");
        &self.nodes[node].next
    }

    /// Returns the number of nodes in the graph.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the graph contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns a boxed value of the data of a node.
    ///
    /// This must not violate memory rules by calling multiple times on the same node.
    ///
    /// Panics if `node` is not a valid node index.
    unsafe fn get_boxed(&self, node: usize) -> Box<T> {
        assert!(node < self.nodes.len(), "Invalid node index");
        unsafe { Box::from_raw(self.nodes[node].data.as_ptr()) }
    }

    /// Runs the tarjan algorithm to find SCC.
    ///
    /// It is called in `Self::scc`.
    fn tarjan(&mut self, u: usize) {
        self.nodes[u].dfn = self.current_dfn;
        self.nodes[u].low = self.current_dfn;
        self.nodes[u].in_stack = true;
        self.current_dfn += 1;
        self.dfs_stack.push(u);

        let next = self.nodes[u].next.clone();
        for v in next.into_iter() {
            if self.nodes[v].dfn == 0 {
                self.tarjan(v);
                self.nodes[u].low = self.nodes[u].low.min(self.nodes[v].low);
            } else if self.nodes[v].in_stack {
                self.nodes[u].low = self.nodes[u].low.min(self.nodes[v].dfn);
            }
        }
        if self.nodes[u].low == self.nodes[u].dfn {
            while let Some(w) = self.dfs_stack.pop() {
                self.nodes[w].scc = self.current_scc;
                self.nodes[w].in_stack = false;
                if w == u {
                    break;
                }
            }
            self.current_scc += 1;
        }
    }

    /// Find strongly connected components, and compress them into a new graph, with each node representing a SCC.
    pub fn scc(mut self) -> Graph<Vec<T>> {
        // Special case: there are no nodes.
        if self.nodes.is_empty() {
            return Graph::new();
        }

        // Initialize tarjan algorithm.
        self.current_dfn = 1;
        self.current_scc = 0;
        self.dfs_stack.clear();
        for u in 0..self.nodes.len() {
            self.nodes[u].in_stack = false;
        }

        for u in 0..self.nodes.len() {
            if self.nodes[u].dfn == 0 {
                self.tarjan(u);
            }
        }

        let mut result = Graph::<Vec<T>>::new();
        for _ in 0..self.current_scc {
            result.add(Vec::new());
        }
        for u in 0..self.nodes.len() {
            let scc = self.nodes[u].scc;
            for v in std::mem::take(&mut self.nodes[u].next).into_iter() {
                let v_scc = self.nodes[v].scc;
                if scc != v_scc {
                    result.connect(scc, v_scc);
                }
            }
        }
        for u in 0..self.nodes.len() {
            let scc = self.nodes[u].scc;
            let data = unsafe { self.get_boxed(u) };
            unsafe {
                result.nodes[scc].data.as_mut().push(*data);
            }
        }
        // Clears all nodes so that they don't get dropped again.
        self.nodes.clear();

        result
    }

    /// Performs topological sort on the graph, and returns one possible order.
    ///
    /// If the graph is not a DAG, `None` is returned.
    pub fn topo_sort(mut self) -> Option<Vec<T>> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        for u in 0..self.nodes.len() {
            if self.nodes[u].deg == 0 {
                queue.push_back(u);
            }
        }

        while let Some(u) = queue.pop_front() {
            result.push(u);
            let next = std::mem::take(&mut self.nodes[u].next);
            for v in next.into_iter() {
                self.nodes[v].deg -= 1;
                if self.nodes[v].deg == 0 {
                    queue.push_back(v);
                }
            }
        }

        if result.len() == self.nodes.len() {
            let result = result
                .into_iter()
                .map(|u| unsafe { *self.get_boxed(u) })
                .collect::<Vec<_>>();
            // Clear the nodes so they don't get dropped again.
            self.nodes.clear();
            Some(result)
        } else {
            // When the topo sort is incomplete, the graph is not a DAG.
            None
        }
    }
}
