use super::transaction; // Import transaction types

/// A node in any particular state-entry/transaction-based DAG.
pub struct Node<'a> {
    transaction: &'a transaction::Transaction<'a>,
}

/// A generic DAG used to store state entries, as well as transactions.
pub struct Graph<'a> {
    /// A list of nodes in the graph
    nodes: Vec<Node<'a>>,
}