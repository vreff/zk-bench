// Generates a valid input.json for the merkle circuit.
// Builds an 8-leaf Poseidon Merkle tree and produces a membership proof.
//
// The circuit uses Num2Bits (LSB-first), where bit i of the key selects
// left(0)/right(1) at tree level i from the root. This means the circuit's
// key is the bit-reversal of the standard leaf array index.

const { buildPoseidon } = require("circomlibjs");

const nLevels = 3;

function bitReverse(n, bits) {
  let result = 0;
  for (let i = 0; i < bits; i++) {
    result = (result << 1) | ((n >> i) & 1);
  }
  return result;
}

async function main() {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;

  // 8 leaves for a depth-3 tree
  const leaves = [10n, 20n, 30n, 42n, 50n, 60n, 70n, 80n];
  const targetLeafIndex = 3; // standard array index → value 42

  // The circuit key is the bit-reversal of the standard leaf index
  const key = bitReverse(targetLeafIndex, nLevels);

  // Hash each leaf with Poseidon(1)
  const hashedLeaves = leaves.map((v) => F.toObject(poseidon([v])));

  // Build tree bottom-up: tree[0] = leaves, tree[nLevels] = root
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

  const root = tree[tree.length - 1][0];

  // Collect siblings walking from leaf to root
  const pathSiblings = []; // pathSiblings[j] = sibling at tree level j
  let idx = targetLeafIndex;
  for (let j = 0; j < nLevels; j++) {
    pathSiblings.push(tree[j][idx ^ 1]);
    idx = Math.floor(idx / 2);
  }

  // Circuit expects siblings[i] at circuit level i:
  //   levels[nLevels-1] (bottom) uses siblings[nLevels-1] → leaf-level sibling
  //   levels[0] (top) uses siblings[0] → root-level sibling
  // So siblings[i] = pathSiblings[nLevels - 1 - i]
  const circuitSiblings = new Array(nLevels);
  for (let i = 0; i < nLevels; i++) {
    circuitSiblings[i] = pathSiblings[nLevels - 1 - i];
  }

  const input = {
    key: key.toString(),
    value: leaves[targetLeafIndex].toString(),
    root: root.toString(),
    siblings: circuitSiblings.map((s) => s.toString()),
  };

  const fs = require("fs");
  fs.writeFileSync("input.json", JSON.stringify(input, null, 2));
  console.log("Written input.json:");
  console.log(JSON.stringify(input, null, 2));
}

main().catch(console.error);
