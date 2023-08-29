# Graph Motif Calculator

This program counts specific graph motifs. It consumes an adjacency
matrix (CSV) and outputs a list of the kinds of motifs.

This program is intended for processing biologically realistic neural networks.

The program is also intended to very efficient, so networks of 100s
of cells will complete effectively instantaneously.

# Using
This program is written in [Rust](https://www.rust-lang.org/), a blazingly fast systems programming
language.

On Unix-like systems such as Linux and MacOS, you can install the Rust compiler like so:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

And then you can run this program with

```cargo install --git https://github.com/njaard/graph-motif```

The program can then be run like so:

```
graph-motif <your_adjacency_matrix.csv> [--count-by-category] [--verbose]
```

The option `--count-by-category` considers the inhibitory or excitatory status
of each Node (see below). The option `--verbose` has the program output
each and every connection listed. The nodes are counted from 0.


# Input file
The input file must be a CSV file containing a directed square adjacency matrix, with no headers.
Each row represents the "from" node, and each column represents the "to" node.

For example, the following represents a graph where
node '0' connects to node '1', which connects to node '2', and
node '2' has no outgoing connections. Therefor, this graph contains a
Chain Motif.

```csv
0,1,0
0,0,1
0,0,1
```

The values can be negative, 0, or positive. '0' represents
no connection, positive values represent an excititory connection,
and negative values represent inhibitory connections.

Each row must contain connections of only the same sign, those that
do not would violate [Dale's Law](https://en.wikipedia.org/wiki/Dale%27s_principle).
This is a block-structured connectivity model.

# Connection types
The motifs counted are Chain, Divergent, Convergent, and Reciprocal. Only
those motifs with 3 participating nodes are considered, or 2 for Reciprocal.

With the option `--count-by-category`, the motifs are further divided into
the following:

 * ChainEE: Excititory -> Excititory -> Any
 * ChainEI: Excititory -> Inhibitory -> Any
 * ChainIE: Inhibitory -> Excititory -> Any
 * ChainII: Inhibitory -> Inhibitory -> Any
 * ConvergentEE: Excititory -> Any <- Excititory
 * ConvergentII: Inhibitory -> Any <- Inhibitory
 * ConvergentEI: Excititory -> Any <- Inhibitory
 * DivergentE: Any <- Excititory -> Any
 * DivergentI: Any <- Inhibitory -> Any
 * ReciprocalEE: Excititory <-> Excititory
 * ReciprocalII: Inhibitory <-> Inhibitory
 * ReciprocalEI: Inhibitory <-> Excititory

# Example output

With no options:

```
Chain: 261402
Convergent: 111172
Divergent: 182509
Reciprocal: 1253
```

And with the `--count-by-category` options:

```
ChainEE: 10843
ChainEI: 139632
ChainIE: 38803
ChainII: 72124
ConvergentEE: 34299
ConvergentII: 32801
ConvergentEI: 44072
DivergentE: 17904
DivergentI: 164605
ReciprocalEE: 20
ReciprocalII: 282
ReciprocalEI: 951
```

# Citation

```
@misc{Samuels_2023, url={https://github.com/njaard/graph-motif}, journal={Graph Motif Calculator}, author={Samuels, Charles}, year={2023}, month={Aug}}
```
