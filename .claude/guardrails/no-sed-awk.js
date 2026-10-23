// PreToolUse hook: refuses a Bash or PowerShell command that runs sed or awk.
//
// Wixen Mail forbids both, even read-only: files are read with the Read and
// Grep tools and changed with Edit or Write (CLAUDE.md, and Pratik's yes of
// 2026-10-20 after about sixty recorded slips). The match is on a program
// in command position, so a commit message or a grep pattern that merely
// contains the word is let through.
//
// It lives here, in the tracked guard folder beside tdd-mode-check.js, with
// its tests beside it in no-sed-awk.test.js, because .claude/hooks is GSD's
// and is rewritten on a reinstall. Run the tests with
// `node --test .claude/guardrails/no-sed-awk.test.js`.

const COMMAND_START = String.raw`(?:(?:^|[\n|;&(\`])\s*(?:(?:env|command|exec|sudo|nice|time|xargs(?:\s+-\S+)*)\s+)*|-exec\s+|\s-c\s+['"]\s*)`;
const SED_OR_AWK = String.raw`(?:\S*/)?(?:g?sed|[gmn]?awk)(?:\.exe)?(?=\s|$|['"])`;
const RUNS_SED_OR_AWK = new RegExp(COMMAND_START + SED_OR_AWK);

const REASON = 'This project forbids sed and awk, even read-only (CLAUDE.md, Pratik\'s rule). ' +
  'Use the Read and Grep tools to read a file, and Edit or Write to change one.';

function runsSedOrAwk(command) {
  return RUNS_SED_OR_AWK.test(command);
}

function main() {
  let input = '';
  process.stdin.on('data', chunk => { input += chunk; });
  process.stdin.on('end', () => {
    let call;
    try { call = JSON.parse(input); } catch { process.exit(0); }
    const command = call && call.tool_input && call.tool_input.command;
    if (typeof command === 'string' && runsSedOrAwk(command)) {
      process.stderr.write(REASON + '\n');
      process.exit(2);
    }
    process.exit(0);
  });
}

if (require.main === module) main();
module.exports = { runsSedOrAwk };
