use rs_graph::linkedlistgraph::LinkedListGraphBuilder;
use rs_graph::maxflow::edmondskarp;
use rs_graph::traits::Indexable;
use rs_graph::Builder;

pub fn main() {
    let mut builder: LinkedListGraphBuilder<u32, (), ()> = LinkedListGraphBuilder::new();
    let mut edges: Vec<usize> = Vec::new();

    let source = builder.add_node();
    let destination = builder.add_node();
    let i = builder.add_node();
    let j = builder.add_node();

    println!(
        "source {:?} destination {:?} i {:?}",
        source, destination, i
    );

    builder.add_edge(source, i);
    edges.push(43);
    builder.add_edge(i, destination);
    edges.push(42);

    builder.add_edge(i, destination);
    edges.push(0);
    builder.add_edge(source, destination);
    edges.push(0);
    builder.add_edge(j, destination);
    edges.push(20);

    let graph = builder.into_graph();

    let (f, e, n) = edmondskarp(&graph, source, destination, |e| edges[e.index()]);
    for (edge, i) in e.iter() {
        println!("edge: {:?} {:?}", edge, i);
    }

    println!("output {} {:?}", f, n);
}
