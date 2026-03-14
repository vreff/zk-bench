// Generates inputs for the ZoKrates Merkle membership proof circuit.
// Computes the Poseidon Merkle tree and outputs witness arguments.
// Usage: node scripts/generate_input.js [leafIndex]

const { buildPoseidon } = require("circomlibjs");

const DEPTH = 3;
const leaves = [10n, 20n, 30n, 42n, 50n, 60n, 70n, 80n];

async function main() {
  const targetIndex = parseInt(process.argv[2] ?? "3", 10);
  if (targetIndex < 0 || targetIndex >= leaves.length) {
    console.error(`Leaf index must be 0–${leaves.length - 1}`);
    process.exit(1);
  }

  const poseidon = await buildPoseidon();
  const F = poseidon.F;

  // Hash pairs of field elements: poseidon([left, right])
  function hashPair(left, right) {
    return F.toObject(poseidon([left, right]));
  }

  // Build tree bottom-up using raw values as leaves
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

  const root = tree[DEPTH][0];

  // Extract sibling path
  let idx = targetIndex;
  const siblings = [];
  for (let level = 0; level < DEPTH; level++) {
    const sibIdx = idx ^ 1;
    siblings.push(tree[level][sibIdx]);
    idx = Math.floor(idx / 2);
  }

  // ZoKrates witness arguments: leaf index siblings[0] siblings[1] siblings[2] root
  const args = [
    leaves[targetIndex].toString(),
    targetIndex.toString(),
    ...siblings.map((s) => s.toString()),
    root.toString(),
  ];

  console.log("ZoKrates witness arguments:");
  console.log(args.join(" "));
  console.log();
  console.log("Breakdown:");
  console.log(`  leaf      = ${leaves[targetIndex]}`);
  console.log(`  index     = ${targetIndex}`);
  for (let i = 0; i < DEPTH; i++) {
    console.log(`  sibling[${i}] = ${siblings[i]}`);
  }
  console.log(`  root      = ${root}`);
}

main().catch(console.error);
