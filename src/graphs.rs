use std::{
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};

use rs_graph::{
    linkedlistgraph::{self as g, LinkedListGraphBuilder},
    maxflow::edmondskarp,
    traits::Indexable,
    Buildable, Builder, LinkedListGraph,
};

use crate::{models::State, OptionalOperation, UsablePlanet};

type NodeId = g::Node<usize>;
type EdgeId = g::Edge<usize>;

#[derive(Debug)]
pub struct Edge {
    wanted: Option<i32>,
}

#[derive(Debug, Copy, Clone)]
pub enum Type {
    Source,
    Destination,
}

#[derive(Debug, Clone)]
pub struct OperationNode<'b> {
    destination: EdgeId,
    planets: Vec<(usize, EdgeId)>,

    pub optional_operation: &'b OptionalOperation,
}

impl<'a> Into<Node<'a>> for OperationNode<'a> {
    fn into(self) -> Node<'a> {
        Node::Operation(self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlanetNode {
    id: usize,
    required: Vec<EdgeId>,
}

impl<'a> Into<Node<'a>> for PlanetNode {
    fn into(self) -> Node<'a> {
        Node::Planet(self)
    }
}

#[derive(Debug, Clone)]
pub enum Node<'a> {
    Special(Type),
    Operation(OperationNode<'a>),
    Planet(PlanetNode),
}

#[allow(unused)]
impl<'a> Node<'a> {
    fn op(&self) -> &OperationNode<'a> {
        match self {
            Node::Operation(ref n) => n,
            _ => panic!("Expected operatio node, but this was not the case"),
        }
    }
    fn op_mut(&mut self) -> &mut OperationNode<'a> {
        match self {
            Node::Operation(ref mut n) => n,
            _ => panic!("Expected operatio node, but this was not the case"),
        }
    }
    fn planet(&self) -> &PlanetNode {
        match self {
            Node::Planet(ref n) => n,
            _ => panic!("Expected operatio node, but this was not the case"),
        }
    }
    fn planet_mut(&mut self) -> &mut PlanetNode {
        match self {
            Node::Planet(ref mut n) => n,
            _ => panic!("Expected operatio node, but this was not the case"),
        }
    }
}

struct NodeOrchestrator<'a, 'b> {
    source: NodeId,
    destination: NodeId,

    created_planets: usize,
    blank_edge: EdgeId,

    edges: Vec<Edge>,
    nodes: Vec<Node<'b>>,
    builder: LinkedListGraphBuilder<usize, (), ()>,

    state: &'a State,

    /// 2D array, planets[1][0] indicates planet with id 1, at the now
    planets: Vec<Vec<NodeId>>,
}

impl<'a, 'b> Index<NodeId> for NodeOrchestrator<'a, 'b> {
    type Output = Node<'b>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.index()]
    }
}

impl<'a, 'b> IndexMut<NodeId> for NodeOrchestrator<'a, 'b> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.index()]
    }
}

impl<'a, 'b> Index<EdgeId> for NodeOrchestrator<'a, 'b> {
    type Output = Edge;

    fn index(&self, index: EdgeId) -> &Self::Output {
        &self.edges[index.index()]
    }
}

impl<'a, 'b> IndexMut<EdgeId> for NodeOrchestrator<'a, 'b> {
    fn index_mut(&mut self, index: EdgeId) -> &mut Self::Output {
        &mut self.edges[index.index()]
    }
}

impl<'a, 'b> NodeOrchestrator<'a, 'b> {
    fn new(state: &'a State) -> Self {
        let mut builder = LinkedListGraph::new_builder();
        let source = builder.add_node();
        let destination = builder.add_node();

        let blank_edge = builder.add_edge(source, destination);

        let planet_count = state.planets().len();
        let mut planets = Vec::new();
        planets.resize(planet_count, Vec::new());

        Self {
            source,
            created_planets: 0,
            blank_edge,
            destination,
            state,
            edges: vec![Edge { wanted: None }],
            nodes: vec![
                Node::Special(Type::Source),
                Node::Special(Type::Destination),
            ],
            builder,
            planets,
        }
    }

    fn add_node<N: Into<Node<'b>>>(&mut self, node: N) -> NodeId {
        self.nodes.push(node.into());
        self.builder.add_node()
    }

    fn add_edge(&mut self, source: NodeId, target: NodeId, wanted: Option<i32>) -> EdgeId {
        let edge = Edge { wanted };
        self.edges.push(edge);
        self.builder.add_edge(source, target)
    }

    fn add_usable_planet(&mut self, ub: &UsablePlanet, duration: usize) -> NodeId {
        let planets = self.state.planets();
        let index = duration - ub.dist;
        let planet_id = ub.id;

        for i in self.planets[planet_id].len()..index + 1 {
            // Spawn planet

            if i == 0 {
                let node = PlanetNode::default();
                let count = planets[ub.id][0].ships;
                let planet_node = self.add_node(node);
                self.planets[planet_id].push(planet_node);

                self.add_edge(self.source, planet_node, Some(count));
                let req = self.add_edge(planet_node, self.destination, Some(0));
                self[planet_node].planet_mut().required.push(req);
            } else {
                let parent = self.planets[planet_id][i - 1];
                let id = self.created_planets;
                self.created_planets += 1;
                let node = PlanetNode {
                    id,
                    required: self[parent].planet().required.clone(),
                };

                let planet_node = self.add_node(node);
                self.planets[planet_id].push(planet_node);

                self.add_edge(parent, planet_node, None);

                let delta = planets[ub.id][i].ships - planets[ub.id][i - 1].ships;

                // You are losing ships mate
                let req = if delta < 0 {
                    self.add_edge(self.source, planet_node, Some(0));
                    self.add_edge(planet_node, self.destination, Some(-1 * delta))
                } else {
                    self.add_edge(self.source, planet_node, Some(delta));
                    self.add_edge(planet_node, self.destination, Some(0))
                };
                self[planet_node].planet_mut().required.push(req);
            }
        }

        self.planets[planet_id][index]
    }

    fn add_optional_operation(&mut self, op: &'b OptionalOperation) {
        let op_id = self.add_node(OperationNode {
            destination: self.blank_edge,
            planets: Vec::new(),

            optional_operation: op,
        });

        let req = self.add_edge(op_id, self.destination, op.required_ships.into());
        self[op_id].op_mut().destination = req;

        for ub in &op.usable_planets {
            let node = self.add_usable_planet(ub, op.duration);
            let incoming = self.add_edge(node, op_id, None);

            let id = self[node].planet().id;

            let op = self[op_id].op_mut();
            op.planets.push((id, incoming));
        }
    }

    fn solve(self) -> Vec<Operation<'b>> {
        let graph = self.builder.into_graph();

        let (_, edges, _) = edmondskarp(&graph, self.source, self.destination, |e| {
            self.edges[e.index()].wanted.unwrap_or(i32::MAX)
        });

        let self_edges = self.edges;

        let good_edges: HashSet<EdgeId> = edges
            .iter()
            .filter_map(|(e, &i)| {
                let edge = &self_edges[e.index()];
                if let Some(wanted) = edge.wanted {
                    (i == wanted).then_some(e)
                } else {
                    Some(e)
                }
            })
            .collect();

        let edge_amounts: HashMap<EdgeId, i32> = edges.iter().map(|(e, &i)| (e, i)).collect();

        let good_planets: HashSet<_> = self
            .nodes
            .iter()
            .filter_map(|x| match x {
                Node::Planet(p) => Some(p),
                _ => None,
            })
            .filter_map(|p| {
                if p.required.iter().all(|x| good_edges.contains(x)) {
                    Some(p.id)
                } else {
                    None
                }
            })
            .collect();

        self.nodes
            .into_iter()
            .filter_map(|x| match x {
                Node::Operation(p) => Some(p),
                _ => None,
            })
            .filter_map(|x| {
                if x.planets.iter().all(|(x, _)| good_planets.contains(x))
                    && good_edges.contains(&x.destination)
                {
                    let target = x.optional_operation.target;
                    Some(Operation {
                        original: x.optional_operation,
                        solution: x
                            .planets
                            .iter()
                            .map(|(source, edge)| SolutionPart {
                                source: *source,
                                target,
                                ships: edge_amounts[edge],
                            })
                            .collect(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct SolutionPart {
    pub source: usize,
    pub target: usize,
    pub ships: i32,
}
pub struct Operation<'a> {
    pub original: &'a OptionalOperation,
    pub solution: Vec<SolutionPart>,
}

pub fn try_oo<'a, 'b>(operations: &'b [OptionalOperation], state: &'a State) -> Vec<Operation<'b>> {
    let mut orchestrator = NodeOrchestrator::new(state);

    for op in operations {
        orchestrator.add_optional_operation(op);
    }

    orchestrator.solve()
}
