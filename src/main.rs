use clap::Parser;
use std::io::Read;
use std::cmp::Ordering;
use csv::{ReaderBuilder,StringRecord};

use int_enum::IntEnum;

#[derive(Parser)]
#[command(version,about)]
struct Args
{
	#[arg()]
	connectivity: std::path::PathBuf,

	#[arg(help="Separate the motif count by the polarity of their nodes",long="count-by-category")]
	count_by_category: bool,

	#[arg(help="Output all the individual identified motifs", long="verbose")]
	verbose: bool,
}

fn main()
{
	let args = Args::parse();

	let csv_reader = ReaderBuilder::new()
		.has_headers(false)
		.from_path(&args.connectivity)
		.expect("opening file");

	if args.count_by_category
	{
		let mut counters = [0; 12];

		let nodes = load_nodes(csv_reader);

		process(
			&nodes,
			|motif|
			{
				let cat = determine_motif_category_by_node_type(&nodes, &motif);
				counters[cat as usize] += 1;

				if args.verbose
				{
					println!("{motif} ({cat:?})");
				}
			}
		);

		for (idx,counter) in counters.iter().enumerate()
		{
			let t_c = MotifCategoryByNodeType::from_int(idx).unwrap();
			println!("{t_c:?}: {counter}");
		}

	}
	else
	{
		let mut counters = [0; 4];

		let nodes = load_nodes(csv_reader);

		process(
			&nodes,
			|motif|
			{
				let cat = determine_motif_category_basic(&motif);
				counters[cat as usize] += 1;

				if args.verbose
				{
					println!("{motif} ({cat:?})");
				}
			}
		);

		for (idx,counter) in counters.iter().enumerate()
		{
			let t_c = MotifCategoryBasic::from_int(idx).unwrap();
			println!("{t_c:?}: {counter}");
		}

	}
}

#[derive(Default,Clone,Copy,Eq,PartialEq,Debug)]
enum Type
{
	#[default]
	Undetermined,
	Excitatory,
	Inhibitory,
}

#[derive(Default,Debug)]
struct Node
{
	edges_to: smallvec::SmallVec<[usize;4]>,
	edges_from: smallvec::SmallVec<[usize;4]>,
	typ: Type,
}


#[derive(Debug,Eq,PartialEq)]
enum MotifShape
{
	Chain(usize,usize,usize),
	Convergent(usize,usize,usize),
	Divergent(usize,usize,usize),
	Reciprocal(usize,usize),
}

#[repr(usize)]
#[derive(Debug,Eq,PartialEq,Copy,Clone,int_enum::IntEnum)]
enum MotifCategoryByNodeType
{
	ChainEE=0,
	ChainEI=1,
	ChainIE=2,
	ChainII=3,
	ConvergentEE=4,
	ConvergentII=5,
	ConvergentEI=6,
	DivergentE=7,
	DivergentI=8,
	ReciprocalEE=9,
	ReciprocalII=10,
	ReciprocalEI=11,
}

fn determine_motif_category_by_node_type(nodes: &[Node], shape: &MotifShape) -> MotifCategoryByNodeType
{
	use Type::*;
	use MotifCategoryByNodeType::*;
	match *shape
	{
		MotifShape::Chain(a, b, _) =>
		{
			match (nodes[a].typ, nodes[b].typ)
			{
				(Excitatory, Excitatory) => ChainEE,
				(Excitatory, Inhibitory) => ChainEI,
				(Inhibitory, Excitatory) => ChainIE,
				(Inhibitory, Inhibitory) => ChainII,
				a => panic!("invalid {a:?}"),
			}
		}
		MotifShape::Convergent(a, _, b) =>
		{
			match (nodes[a].typ, nodes[b].typ)
			{
				(Excitatory, Excitatory) => ConvergentEE,
				(Inhibitory, Inhibitory) => ConvergentII,
				(Inhibitory, Excitatory) => ConvergentEI,
				(Excitatory, Inhibitory) => ConvergentEI,
				a => panic!("invalid {a:?}"),
			}
		}
		MotifShape::Divergent(_, a, _) =>
		{
			match nodes[a].typ
			{
				Excitatory => DivergentE,
				Inhibitory => DivergentI,
				a => panic!("invalid {a:?}"),
			}
		}
		MotifShape::Reciprocal(a, b) =>
		{
			match (nodes[a].typ, nodes[b].typ)
			{
				(Excitatory,Excitatory) => ReciprocalEE,
				(Inhibitory,Inhibitory) => ReciprocalII,
				(Excitatory,Inhibitory) => ReciprocalEI,
				(Inhibitory,Excitatory) => ReciprocalEI,
				a => panic!("invalid {a:?}"),
			}
		}
	}
}

#[repr(usize)]
#[derive(Debug,Eq,PartialEq,Copy,Clone,int_enum::IntEnum)]
enum MotifCategoryBasic
{
	Chain=0,
	Convergent=1,
	Divergent=2,
	Reciprocal=3,
}

fn determine_motif_category_basic(shape: &MotifShape) -> MotifCategoryBasic
{
	use MotifCategoryBasic::*;
	match *shape
	{
		MotifShape::Chain(..) => Chain,
		MotifShape::Convergent(..) => Convergent,
		MotifShape::Divergent(..) => Divergent,
		MotifShape::Reciprocal(..) => Reciprocal,
	}
}

impl std::fmt::Display for MotifShape
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		match self
		{
			Self::Chain(a,b,c) =>
				write!(f, "chain: {a} → {b} → {c}"),
			Self::Convergent(a,b,c) =>
				write!(f, "convergent: {a} → {b} ← {c}"),
			Self::Divergent(a,b,c) =>
				write!(f, "divergent: {a} ← {b} → {c}"),
			Self::Reciprocal(a,b) =>
				write!(f, "reciprocal: {a} ↔ {b}"),
		}

	}
}

fn load_nodes(mut csv_reader: csv::Reader<impl Read>) -> Vec<Node>
{
	fn is_connected(s: &str) -> bool
	{
		if s.is_empty() { return false; }
		! (s == "0.0" || s == "0")
	}


	let mut nodes = Vec::new();

	// load all the nodes and edges
	let mut record = StringRecord::new();
	let mut node_idx = 0;
	while csv_reader.read_record(&mut record).expect("reading row")
	{
		if node_idx == 0
		{
			nodes.resize_with(record.len(), || Node::default());
		}

		if nodes.len() != record.len()
		{
			panic!("row {node_idx} doesn't have {} columns", nodes.len());
		}

		for (col_idx,field) in record.iter().enumerate()
		{
			if is_connected(field)
			{
				nodes[node_idx].edges_to.push(col_idx);
				let Ok(s) = field.parse::<f64>() else {panic!("Can't parse node={node_idx} value=\"{field}\"") };

				match (s.total_cmp(&0.0), nodes[node_idx].typ)
				{
					(Ordering::Less, Type::Undetermined) => nodes[node_idx].typ = Type::Inhibitory,
					(Ordering::Less, Type::Inhibitory) => {},
					(Ordering::Less, Type::Excitatory) => panic!("node {node_idx} violating Dale's Law"),
					(Ordering::Equal, _) => {},
					(Ordering::Greater, Type::Undetermined) => nodes[node_idx].typ = Type::Excitatory,
					(Ordering::Greater, Type::Excitatory) => {},
					(Ordering::Greater, Type::Inhibitory) => panic!("node {node_idx} violating Dale's Law"),
				}

				nodes[col_idx].edges_from.push(node_idx);
			}
		}
		node_idx+=1;
	}

	nodes
}

fn process(nodes: &[Node], mut item: impl FnMut(MotifShape))
{
	// find the motifs
	for (idx, node) in nodes.iter().enumerate()
	{
		let edges_from = &node.edges_from;
		let edges_to = &node.edges_to;

		// detect convergent motifs: `node` should have multiple `in`s
		if edges_from.len() >= 2
		{
			for i in 0 .. edges_from.len()-1
			{
				for j in i+1 .. edges_from.len()
				{
					item(MotifShape::Convergent(edges_from[i], idx, edges_from[j]));
				}
			}
		}

		// detect divergent motifs: `node` should have multiple outs
		if edges_to.len() >= 2
		{
			for i in 0 .. edges_to.len()-1
			{
				for j in i+1 .. edges_to.len()
				{
					item(MotifShape::Divergent(edges_to[i], idx, edges_to[j]));
				}
			}
		}

		// detect chain motifs, `node` has an in and an out
		if edges_from.len() >= 1 && edges_to.len() >= 1
		{
			for &edge_from in edges_from
			{
				for &edge_to in edges_to
				{
					if edge_to != edge_from
					{
						item(MotifShape::Chain(edge_from, idx, edge_to));
					}
				}
			}
		}

		// detect reciprocal motifs, `node` has connects to another node that connects to the first
		if edges_from.len() >= 1 && edges_to.len() >= 1
		{
			for &edge_to in edges_to
			{
				for &edge_from in edges_from
				{
					// always output reciprical with idx < edge_to (so we don't duplicate them)
					if edge_to == edge_from && idx < edge_to
					{
						item(MotifShape::Reciprocal(idx, edge_from));
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests
{
	use csv::ReaderBuilder;
	use super::{process,MotifShape,load_nodes};

	#[test]
	fn chain()
	{
		let conn = "\
0,1,0
0,0,1
0,0,0\
		";
		let csv_reader = ReaderBuilder::new()
			.has_headers(false)
			.from_reader(conn.as_bytes());

		let mut out = vec![];
		process(&load_nodes(csv_reader), |item| out.push(item));
		assert_eq!(out, vec![MotifShape::Chain(0,1,2)]);

	}
	#[test]
	fn div()
	{
		let conn = "\
0,1,1
0,0,0
0,0,0\
		";
		let csv_reader = ReaderBuilder::new()
			.has_headers(false)
			.from_reader(conn.as_bytes());

		let mut out = vec![];
		process(&load_nodes(csv_reader), |item| out.push(item));
		assert_eq!(out, vec![MotifShape::Divergent(1,0,2)]);
	}
	#[test]
	fn conv()
	{
		let conn = "\
0,0,0
1,0,0
1,0,0\
		";
		let csv_reader = ReaderBuilder::new()
			.has_headers(false)
			.from_reader(conn.as_bytes());

		let mut out = vec![];
		process(&load_nodes(csv_reader), |item| out.push(item));
		assert_eq!(out, vec![MotifShape::Convergent(1,0,2)]);
	}
	#[test]
	fn recip()
	{
		let conn = "\
0,1,0
1,0,0\
		";
		let csv_reader = ReaderBuilder::new()
			.has_headers(false)
			.from_reader(conn.as_bytes());

		let mut out = vec![];
		process(&load_nodes(csv_reader), |item| out.push(item));

		assert_eq!(out, vec![MotifShape::Reciprocal(0,1)]);
	}
}
