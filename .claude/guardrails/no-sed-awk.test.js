// Run with: node --test .claude/guardrails/no-sed-awk.test.js
const test = require('node:test');
const assert = require('node:assert');
const { spawnSync } = require('node:child_process');
const path = require('node:path');
const { runsSedOrAwk } = require('./no-sed-awk.js');

const refused = [
  "sed -n '1,20p' src/main.rs",
  "sed -i 's/a/b/' Cargo.toml",
  "cat log.txt | awk '{print $1}'",
  "git diff --stat | sed -n 1p",
  "cd /c/x && sed -n 5p f",
  "ls; awk 'NR==1' f",
  "x=$(sed -n 1p f)",
  "find . -name '*.rs' -exec sed -i 's/a/b/' {} +",
  "git ls-files | xargs sed -n 1p",
  "gawk '{print}' f",
  "/usr/bin/sed -n 1p f",
  "env sed -n 1p f",
  "bash -c 'awk 1 f'",
  "  sed -n 1p f",
  "echo a &&\nsed -n 1p f",
];

const allowed = [
  "bash scripts/build-installer.sh",
  "grep -n 'used' src/main.rs",
  "git log --grep='sed' --oneline",
  "cargo test --lib application::draft_copy::",
  "git commit -m 'parsed the closed folder'",
  "echo based",
  "python -c \"print('awkward')\"",
  "ls sediment/",
];

for (const command of refused) {
  test(`refuses: ${command}`, () => assert.strictEqual(runsSedOrAwk(command), true));
}
for (const command of allowed) {
  test(`allows: ${command}`, () => assert.strictEqual(runsSedOrAwk(command), false));
}

function hook(input) {
  return spawnSync(process.execPath, [path.join(__dirname, 'no-sed-awk.js')], { input: JSON.stringify(input), encoding: 'utf8' });
}

test('the hook blocks a Bash call that runs sed, with exit 2 and a reason', () => {
  const run = hook({ tool_name: 'Bash', tool_input: { command: "sed -n 1p f" } });
  assert.strictEqual(run.status, 2);
  assert.match(run.stderr, /Read and Grep/);
});

test('the hook blocks a PowerShell call that runs awk', () => {
  const run = hook({ tool_name: 'PowerShell', tool_input: { command: "awk 1 f" } });
  assert.strictEqual(run.status, 2);
});

test('the hook lets an ordinary command through', () => {
  const run = hook({ tool_name: 'Bash', tool_input: { command: "git status --porcelain" } });
  assert.strictEqual(run.status, 0);
});

test('the hook lets unreadable input through rather than blocking every call', () => {
  const run = spawnSync(process.execPath, [path.join(__dirname, 'no-sed-awk.js')], { input: 'not json', encoding: 'utf8' });
  assert.strictEqual(run.status, 0);
});
