// Visualizes the Poseidon Merkle tree and highlights the proof path for a given leaf.
// Usage: node scripts/visualize_tree.js [leafIndex]
//   leafIndex defaults to 3 (value 42)

const { buildPoseidon } = require("circomlibjs");

const nLevels = 3;
const leaves = [10n, 20n, 30n, 42n, 50n, 60n, 70n, 80n];

// ANSI colors
const RESET = "\x1b[0m";
const GREEN = "\x1b[32m";
const YELLOW = "\x1b[33m";
const BOLD = "\x1b[1m";
const DIM = "\x1b[2m";

function shortHash(n, level = 0) {
  const s = n.toString();
  const sides = [4, 7, 18, 34];
  const side = sides[level] ?? (4 + level * 10);
  const minLen = side * 2 + 1;
  return s.length > minLen ? s.slice(0, side) + "…" + s.slice(-side) : s;
}

async function main() {
  const targetLeafIndex = parseInt(process.argv[2] ?? "3", 10);
  if (targetLeafIndex < 0 || targetLeafIndex >= leaves.length) {
    console.error(`Leaf index must be 0–${leaves.length - 1}`);
    process.exit(1);
  }

  const poseidon = await buildPoseidon();
  const F = poseidon.F;

  function hashPair(left, right) {
    return F.toObject(poseidon([left, right]));
  }

  // Build tree bottom-up: raw values as leaves, Poseidon(2) for internal nodes
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

  // Determine which nodes are on the proof path and which are siblings
  const pathNodes = new Set();
  const siblingNodes = new Set();
  let idx = targetLeafIndex;
  for (let level = 0; level <= nLevels; level++) {
    pathNodes.add(`${level}:${idx}`);
    if (level < nLevels) {
      siblingNodes.add(`${level}:${idx ^ 1}`);
      idx = Math.floor(idx / 2);
    }
  }

  function plainLabel(level, i) {
    const hash = shortHash(tree[level][i], level);
    if (level === 0) return `${leaves[i]} ${hash}`;
    return hash;
  }

  // Compute positions bottom-up
  const gap = 3;
  const positions = [];
  for (let level = 0; level <= nLevels; level++) positions.push([]);

  let cursor = 0;
  for (let i = 0; i < tree[0].length; i++) {
    const w = plainLabel(0, i).length;
    const left = cursor;
    const center = left + Math.floor(w / 2);
    positions[0].push({ left, center, right: left + w });
    cursor = left + w + gap;
  }

  for (let level = 1; level <= nLevels; level++) {
    for (let i = 0; i < tree[level].length; i++) {
      const leftChild = positions[level - 1][i * 2];
      const rightChild = positions[level - 1][i * 2 + 1];
      const center = Math.floor((leftChild.center + rightChild.center) / 2);
      const w = plainLabel(level, i).length;
      const left = center - Math.floor(w / 2);
      positions[level].push({ left, center, right: left + w });
    }
  }

  function colorLabel(level, i) {
    const key = `${level}:${i}`;
    const hash = shortHash(tree[level][i], level);
    const onPath = pathNodes.has(key);
    const isSibling = siblingNodes.has(key);

    if (level === 0) {
      if (onPath) return `${GREEN}${BOLD}[${leaves[i]}] ${hash}${RESET}`;
      if (isSibling) return `${YELLOW}(${leaves[i]}) ${hash}${RESET}`;
      return `${DIM}${leaves[i]} ${hash}${RESET}`;
    }
    if (onPath) return `${GREEN}${BOLD}${hash}${RESET}`;
    if (isSibling) return `${YELLOW}${hash}${RESET}`;
    return `${DIM}${hash}${RESET}`;
  }

  function renderLevel(level) {
    let line = "";
    let col = 0;
    for (let i = 0; i < tree[level].length; i++) {
      const pos = positions[level][i];
      if (pos.left > col) {
        line += " ".repeat(pos.left - col);
        col = pos.left;
      }
      line += colorLabel(level, i);
      col += plainLabel(level, i).length;
    }
    return line;
  }

  function renderConnectors(parentLevel) {
    const childLevel = parentLevel - 1;
    let line = "";
    let col = 0;
    for (let i = 0; i < tree[parentLevel].length; i++) {
      const leftChild = positions[childLevel][i * 2];
      const rightChild = positions[childLevel][i * 2 + 1];

      const lCol = leftChild.center;
      const rCol = rightChild.center;

      const leftKey = `${childLevel}:${i * 2}`;
      const rightKey = `${childLevel}:${i * 2 + 1}`;
      const leftColor = pathNodes.has(leftKey) ? GREEN : siblingNodes.has(leftKey) ? YELLOW : DIM;
      const rightColor = pathNodes.has(rightKey) ? GREEN : siblingNodes.has(rightKey) ? YELLOW : DIM;

      if (lCol > col) { line += " ".repeat(lCol - col); col = lCol; }
      line += `${leftColor}/${RESET}`; col++;
      const midSpaces = Math.max(0, rCol - col);
      line += " ".repeat(midSpaces); col += midSpaces;
      line += `${rightColor}\\${RESET}`; col++;
    }
    return line;
  }

  // Print header
  console.log();
  console.log(`${BOLD}Merkle Tree Visualization${RESET}  (depth ${nLevels}, ${leaves.length} leaves, Poseidon hash)`);
  console.log(`Proving leaf[${targetLeafIndex}] = ${BOLD}${leaves[targetLeafIndex]}${RESET}`);
  console.log();
  console.log(`  ${GREEN}■${RESET} Proof path    ${YELLOW}■${RESET} Sibling (witness)    ${DIM}■${RESET} Other nodes`);
  console.log();

  const prefix = "  ";
  const levelNames = [];
  for (let level = nLevels; level >= 0; level--) {
    const tag = level === nLevels ? "Root  " : level === 0 ? "Leaves" : `Lvl ${nLevels - level} `;
    levelNames[level] = tag;
  }

  for (let level = nLevels; level >= 0; level--) {
    console.log(`${prefix}${DIM}${levelNames[level]}${RESET}  ${renderLevel(level)}`);
    if (level > 0) {
      console.log(`${prefix}        ${renderConnectors(level)}`);
    }
  }

  // Print proof details
  console.log();
  console.log(`${BOLD}Public inputs:${RESET}`);
  console.log(`  Root:        ${GREEN}${BOLD}${shortHash(tree[nLevels][0], nLevels)}${RESET}`);

  console.log();
  console.log(`${BOLD}Private inputs:${RESET}`);
  console.log(`  Key (index): ${targetLeafIndex}`);
  console.log(`  Value:       ${leaves[targetLeafIndex]}`);

  idx = targetLeafIndex;
  for (let level = 0; level < nLevels; level++) {
    const sibIdx = idx ^ 1;
    const direction = idx % 2 === 0 ? "right →" : "← left";
    console.log(`  Sibling[${level}]:  ${YELLOW}${shortHash(tree[level][sibIdx], level)}${RESET}  (${direction} of path node)`);
    idx = Math.floor(idx / 2);
  }
  console.log();
}

main().catch(console.error);
