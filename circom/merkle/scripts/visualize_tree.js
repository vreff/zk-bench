// Visualizes the Merkle tree and highlights the proof path for a given leaf.
// Usage: node scripts/visualize_tree.js [leafIndex]
//   leafIndex defaults to 3 (value 42)

const { buildPoseidon } = require("circomlibjs");

const nLevels = 3;
const leaves = [10n, 20n, 30n, 42n, 50n, 60n, 70n, 80n];

// ANSI colors
const RESET = "\x1b[0m";
const GREEN = "\x1b[32m";
const YELLOW = "\x1b[33m";
const CYAN = "\x1b[36m";
const RED = "\x1b[31m";
const BOLD = "\x1b[1m";
const DIM = "\x1b[2m";

// Show more hash digits at higher levels where there's more horizontal room
function shortHash(n, level = 0) {
  const s = n.toString();
  // chars per side: 4 at leaves, growing exponentially toward root
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

  // Hash leaves
  const hashedLeaves = leaves.map((v) => F.toObject(poseidon([v])));

  // Build tree bottom-up: tree[0] = hashed leaves, tree[nLevels] = root
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

  // Determine which nodes are on the proof path and which are siblings
  const pathNodes = new Set();   // "level:index" keys on the path
  const siblingNodes = new Set(); // "level:index" keys used as siblings
  let idx = targetLeafIndex;
  for (let level = 0; level <= nLevels; level++) {
    pathNodes.add(`${level}:${idx}`);
    if (level < nLevels) {
      siblingNodes.add(`${level}:${idx ^ 1}`);
      idx = Math.floor(idx / 2);
    }
  }

  // Build plain-text labels for each node (no ANSI) to measure widths
  function plainLabel(level, i) {
    const hash = shortHash(tree[level][i], level);
    if (level === 0) return `${leaves[i]} ${hash}`;
    return hash;
  }

  // Compute the center position of every node, bottom-up
  const gap = 3; // min gap between adjacent leaf labels
  const positions = []; // positions[level][i] = { left, center, right }
  for (let level = 0; level <= nLevels; level++) positions.push([]);

  // Place leaves first
  let cursor = 0;
  for (let i = 0; i < tree[0].length; i++) {
    const w = plainLabel(0, i).length;
    const left = cursor;
    const center = left + Math.floor(w / 2);
    const right = left + w;
    positions[0].push({ left, center, right });
    cursor = right + gap;
  }

  // Place parents centered over their children
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

  // Colorize a label
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

  // Render a row of nodes at a given level
  function renderLevel(level) {
    const nodes = tree[level];
    let line = "";
    let col = 0;
    for (let i = 0; i < nodes.length; i++) {
      const pos = positions[level][i];
      if (pos.left > col) {
        line += " ".repeat(pos.left - col);
        col = pos.left;
      }
      const plain = plainLabel(level, i);
      line += colorLabel(level, i);
      col += plain.length;
    }
    return line;
  }

  // Render connector lines between a parent level and child level
  function renderConnectors(parentLevel) {
    const childLevel = parentLevel - 1;
    let line = "";
    let col = 0;
    for (let i = 0; i < tree[parentLevel].length; i++) {
      const leftChild = positions[childLevel][i * 2];
      const rightChild = positions[childLevel][i * 2 + 1];
      const parentPos = positions[parentLevel][i];

      // Left branch: from just left of parent center down to child center
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

  // Print tree top-down
  for (let level = nLevels; level >= 0; level--) {
    console.log(`${prefix}${DIM}${levelNames[level]}${RESET}  ${renderLevel(level)}`);
    if (level > 0) {
      console.log(`${prefix}        ${renderConnectors(level)}`);
    }
  }

  // Print proof details separated by visibility
  console.log();
  console.log(`${BOLD}Public inputs:${RESET}`);
  console.log(`  Root:        ${GREEN}${BOLD}${shortHash(tree[nLevels][0], nLevels)}${RESET}`);

  console.log();
  console.log(`${BOLD}Private inputs:${RESET}`);
  console.log(`  Key (index): ${targetLeafIndex}`);
  console.log(`  Value:       ${leaves[targetLeafIndex]}`);
  console.log(`  Leaf hash:   ${DIM}${hashedLeaves[targetLeafIndex].toString()}${RESET}`);

  idx = targetLeafIndex;
  for (let level = 0; level < nLevels; level++) {
    const sibIdx = idx ^ 1;
    const direction = idx % 2 === 0 ? "right →" : "← left";
    console.log(`  Sibling[${nLevels - 1 - level}]:  ${YELLOW}${shortHash(tree[level][sibIdx], level)}${RESET}  (${direction} of path node)`);
    idx = Math.floor(idx / 2);
  }
  console.log();
}

main().catch(console.error);
