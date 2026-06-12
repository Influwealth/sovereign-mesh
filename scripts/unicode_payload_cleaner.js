const fs = require('fs');
const path = require('path');

// File scanning and cleanup logic
const suspiciousDecoderPattern = /(\bcodePointAt\b\s*\(.*?\)\s*(?:>=\s*0xFE00\s*&&\s*<=\s*0xFE0F|>=\s*0xE0100\s*&&\s*<=\s*0xE01EF).*?eval|Buffer\.from\s*\().*?;/g;

// Recursive scan for .js and .ts files
const findFiles = (dir, extensions) => {
  let results = [];
  const list = fs.readdirSync(dir);
  list.forEach(file => {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    if (stat && stat.isDirectory()) {
      results = results.concat(findFiles(filePath, extensions));
    } else if (extensions.includes(path.extname(file))) {
      results.push(filePath);
    }
  });
  return results;
};

// Scan and clean a specific file
const scanAndCleanFile = (filePath) => {
  const content = fs.readFileSync(filePath, 'utf-8');
  const matches = content.match(suspiciousDecoderPattern);

  if (matches) {
    console.log(`Suspicious code found in ${filePath}`);
    console.log(`Matches: ${matches}`);

    // Remove suspicious code
    const cleanedContent = content.replace(suspiciousDecoderPattern, '');
    fs.writeFileSync(filePath, cleanedContent, 'utf-8');

    // Git commit logic
    execSync(`git add "${filePath}"`);
    execSync(`git commit -m "security: remove obfuscated Unicode payload"`);
  }
};

// Directories and files to scan
const filesToScan = [
  'deepflex-core/main.js',
  'roblox-wealthbridge-east-flatbush/preinstall.js',
  'wealthbridge-agent-framework/preinstall.js',
  'deepflex-core/preload.js',
];

const dirsToScan = [
  'code-cat',
  'eastflatbush-90s-unity',
];

filesToScan.forEach(scanAndCleanFile);
dirsToScan.forEach(dir => {
  const files = findFiles(dir, ['.js', '.ts']);
  files.forEach(scanAndCleanFile);
});