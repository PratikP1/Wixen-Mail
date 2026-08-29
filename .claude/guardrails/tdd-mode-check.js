#!/usr/bin/env node
// Red/green TDD is not optional here, and this is what says so out loud.
//
// GSD decides per task whether to use TDD. With `workflow.tdd_mode` off — its
// default — the planner applies TDD "opportunistically", meaning it decides.
// With it on, every eligible task MUST be `type: tdd` and the RED and GREEN
// gate commits are checked. The standing rule for this machine is red/green on
// every change, so the opportunistic default is the wrong one and a sentence
// in CLAUDE.md asking for the right one is not a mechanism.
//
// This is the mechanism. It runs at session start, says nothing when the
// setting is right, and names the one command that fixes it when it is not.
// Advisory on purpose: refusing to start a session over a planning setting
// would be worse than the thing it is guarding against.
//
// Lives outside GSD's own file manifest, so a `gsd update` does not remove it.

const fs = require('fs');
const path = require('path');

const CONFIG = path.join(process.cwd(), '.planning', 'config.json');
const FIX = 'node .claude/gsd-core/bin/gsd-tools.cjs config-set workflow.tdd_mode true';

/** Whether this directory is a GSD project at all. */
function isGsdProject() {
  return fs.existsSync(CONFIG);
}

/** Whether every eligible task will be planned red/green. */
function tddModeIsOn(config) {
  return config?.workflow?.tdd_mode === true;
}

/** What to say, or null when there is nothing to say. */
function whatToSay(present, config) {
  if (!present) return null;
  if (tddModeIsOn(config)) return null;
  return (
    'GSD is set up here and workflow.tdd_mode is not on, so the planner will ' +
    'decide per task whether to write the test first. The standing rule is ' +
    'red/green on every change. Turn it on with:\n  ' +
    FIX
  );
}

function main() {
  let config = null;
  const present = isGsdProject();
  if (present) {
    try {
      config = JSON.parse(fs.readFileSync(CONFIG, 'utf8'));
    } catch {
      // An unreadable config is not an absent one. Saying nothing here would
      // report the setting as satisfied, which is the failure this exists to
      // stop, so it falls through to the warning with config as null.
      config = null;
    }
  }
  const said = whatToSay(present, config);
  if (said) {
    process.stderr.write(said + '\n');
  }
  process.exit(0);
}

module.exports = { isGsdProject, tddModeIsOn, whatToSay };

if (require.main === module) {
  main();
}
