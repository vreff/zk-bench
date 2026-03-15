// Generates input.json for the doubleMerkle circuit.
// Builds TWO independent 8-leaf Poseidon Merkle trees and produces
// membership proofs for one leaf in each tree.

const { buildPoseidon } = require("circomlibjs");

const nLevels = 3;

function bitReverse(n, bits) {
  let result = 0;
  for (let i = 0; i < bits; i++) {
    result = (result << 1) | ((n >> i) & 1);
  }
  return result;
}

function buildTree(poseidon, F, leaves) {
  const hashedLeaves = leaves.map((v) => F.toObject(poseidon([v])));
  let currentLevel = hashedLeaves;
  const tree = [currentLevel];

  while (currentLevel.length > 1) {
    const nextLevel = [];
    for (let i = 0; i < currentLevel.length; i += 2) {
      nextLevel.push(F.toObject(poseidon([currentLevel[i], currentLevel[i + 1]])));
    }
    currentLevel = nextLevel;
    tree.push(currentLevel);
  }
  return tree;
}

function getProof(tree, targetLeafIndex) {
  const root = tree[tree.length - 1][0];
  const key = bitReverse(targetLeafIndex, nLevels);

  const pathSiblings = [];
  let idx = targetLeafIndex;
  for (let j = 0; j < nLevels; j++) {
    pathSiblings.push(tree[j][idx ^ 1]);
    idx = Math.floor(idx / 2);
  }

  const circuitSiblings = new Array(nLevels);
  for (let i = 0; i < nLevels; i++) {
    circuitSiblings[i] = pathSiblings[nLevels - 1 - i];
  }

  return { key, root, circuitSiblings };
}

async function main() {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;

  // Tree A: same leaves as single merkle
  const leavesA = [10n, 20n, 30n, 42n, 50n, 60n, 70n, 80n];
  const indexA = 3;

  // Tree B: different leaves
  const leavesB = [100n, 200n, 300n, 420n, 500n, 600n, 700n, 800n];
  const indexB = 3;

  const treeA = buildTree(poseidon, F, leavesA);
  const proofA = getProof(treeA, indexA);

  const treeB = buildTree(poseidon, F, leavesB);
  const proofB = getProof(treeB, indexB);

  const input = {
    keyA: proofA.key.toString(),
    valueA: leavesA[indexA].toString(),
    rootA: proofA.root.toString(),
    siblingsA: proofA.circuitSiblings.map((s) => s.toString()),
    keyB: proofB.key.toString(),
    valueB: leavesB[indexB].toString(),
    rootB: proofB.root.toString(),
    siblingsB: proofB.circuitSiblings.map((s) => s.toString()),
  };

  const fs = require("fs");
  fs.writeFileSync("input_double.json", JSON.stringify(input, null, 2));
  console.log("Written input_double.json:");
  console.log(JSON.stringify(input, null, 2));
}

main().catch(console.error);
