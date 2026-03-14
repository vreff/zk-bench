// Visualizes the SHA-256 Merkle tree and highlights the proof path for a given leaf.
// Computes hash values using Node.js crypto (same SHA-256 as the SP1 guest).
// Usage: node scripts/visualize_tree.js [leafIndex]
//   leafIndex defaults to 3 (value 42)

const crypto = require("crypto");

const nLevels = 3;
const leaves = [10n, 20n, 30n, 42n, 50n, 60n, 70n, 80n];

// ANSI colors
const RESET = "\x1b[0m";
const GREEN = "\x1b[32m";
const YELLOW = "\x1b[33m";
const BOLD = "\x1b[1m";
const DIM = "\x1b[2m";

function hashLeaf(value) {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(value);
  return crypto.createHash("sha256").update(buf).digest("hex");
}

function hashPair(leftHex, rightHex) {
  const left = Buffer.from(leftHex, "hex");
  const right = Buffer.from(rightHex, "hex");
  return crypto.createHash("sha256").update(Buffer.concat([left, right])).digest("hex");
}

function shortHash(hex, level = 0) {
  const sides = [4, 7, 18, 34];
  const side = sides[level] ?? (4 + level * 10);
  const minLen = side * 2 + 1;
  return hex.length > minLen ? hex.slice(0, side) + "…" + hex.slice(-side) : hex;
}

function buildTree() {
  const tree = [];
  tree.push(leaves.map((v) => hashLeaf(v)));
  for (let level = 0; level < nLevels; level++) {
    const prev = tree[level];
    const next = [];
    for (let i = 0; i < prev.length; i += 2) {
      next.push(hashPair(prev[i], prev[i + 1]));
    }
    tree.push(next);
  }
  return tree;
}

function main() {
  const targetLeafIndex = parseInt(process.argv[2] ?? "3", 10);
  if (targetLeafIndex < 0 || targetLeafIndex >= leaves.length) {
    console.error(`Leaf index must be 0–${leaves.length - 1}`);
    process.exit(1);
  }

  const tree = buildTree();

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

  console.log(`\n${BOLD}Merkle Tree Visualization${RESET}  (depth ${nLevels}, ${leaves.length} leaves, SHA-256 hash)`);
  console.log(`Proving leaf[${targetLeafIndex}] = ${BOLD}${leaves[targetLeafIndex]}${RESET}`);
  console.log();
  console.log(`  ${GREEN}■${RESET} Proof path    ${YELLOW}■${RESET} Sibling (witness)    ${DIM}■${RESET} Other nodes`);
  console.log();

  const prefix = "  ";
  for (let level = nLevels; level >= 0; level--) {
    const tag = level === nLevels ? "Root  " : level === 0 ? "Leaves" : `Lvl ${nLevels - level} `;
    console.log(`${DIM}${tag}${RESET} ${prefix}${renderLevel(level)}`);
    if (level > 0) {
      console.log(`       ${prefix}${renderConnectors(level)}`);
    }
  }

  console.log();
  console.log(`${BOLD}Proof details:${RESET}`);
  console.log(`  ${DIM}Public:${RESET}  root = 0x${tree[nLevels][0]}`);
  console.log(`  ${DIM}Private:${RESET} leaf = ${leaves[targetLeafIndex]}, index = ${targetLeafIndex}`);
  let sidx = targetLeafIndex;
  for (let level = 0; level < nLevels; level++) {
    console.log(`           sibling[${level}] = 0x${tree[level][sidx ^ 1]}`);
    sidx = Math.floor(sidx / 2);
  }
  console.log();
}

main();
