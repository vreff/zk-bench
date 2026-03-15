// Generates witness args for the ZoKrates doubleMerkle circuit.
// Computes TWO independent Poseidon Merkle trees and outputs the combined
// witness arguments for zokrates compute-witness.

const { buildPoseidon } = require("circomlibjs");

const DEPTH = 3;
const leavesA = [10n, 20n, 30n, 42n, 50n, 60n, 70n, 80n];
const leavesB = [100n, 200n, 300n, 420n, 500n, 600n, 700n, 800n];
const targetIndex = 3;

async function main() {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;

  function hashPair(left, right) {
    return F.toObject(poseidon([left, right]));
  }

  function buildTree(leaves) {
    let currentLevel = leaves.map((v) => v);
    const tree = [currentLevel];
    while (currentLevel.length > 1) {
      const nextLevel = [];
      for (let i = 0; i < currentLevel.length; i += 2) {
        nextLevel.push(hashPair(currentLevel[i], currentLevel[i + 1]));
      }
      currentLevel = nextLevel;
      tree.push(currentLevel);
    }
    return tree;
  }

  function getProof(tree, leaves, index) {
    const root = tree[DEPTH][0];
    let idx = index;
    const siblings = [];
    for (let level = 0; level < DEPTH; level++) {
      siblings.push(tree[level][idx ^ 1]);
      idx = Math.floor(idx / 2);
    }
    return {
      leaf: leaves[index].toString(),
      index: index.toString(),
      siblings: siblings.map((s) => s.toString()),
      root: root.toString(),
    };
  }

  const treeA = buildTree(leavesA);
  const proofA = getProof(treeA, leavesA, targetIndex);

  const treeB = buildTree(leavesB);
  const proofB = getProof(treeB, leavesB, targetIndex);

  // doubleMerkle.zok main args: leafA indexA siblingsA[3] rootA leafB indexB siblingsB[3] rootB
  const args = [
    proofA.leaf, proofA.index, ...proofA.siblings, proofA.root,
    proofB.leaf, proofB.index, ...proofB.siblings, proofB.root,
  ];

  console.log("ZoKrates double witness arguments:");
  console.log(args.join(" "));

  // Write a shell script snippet for easy use
  const fs = require("fs");
  const cmd = `zokrates compute-witness -i build/doubleMerkle -o build/witness_double -a ${args.join(" ")}`;
  fs.writeFileSync("build/double_witness_cmd.sh", cmd + "\n");
  console.log("\nSaved command to build/double_witness_cmd.sh");
}

main().catch(console.error);
