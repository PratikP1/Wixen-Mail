// Proving the measurement: this check has to be able to see a real failure.
//
// A guardrail that stays quiet whatever the config says is worse than none,
// because it looks like something is watching. Each case below is one state
// the setting can be in.
//
// Run: node .claude/guardrails/tdd-mode-check.test.js

const assert = require('node:assert');
const { test } = require('node:test');
const { tddModeIsOn, whatToSay } = require('./tdd-mode-check.js');

test('tdd_mode on is the only state that passes', () => {
  assert.strictEqual(tddModeIsOn({ workflow: { tdd_mode: true } }), true);
});

test('tdd_mode off is not on', () => {
  assert.strictEqual(tddModeIsOn({ workflow: { tdd_mode: false } }), false);
});

test('a config that never mentions tdd_mode is not on', () => {
  // GSD's default. The whole reason this check exists: absent reads as off,
  // and off means the planner decides per task.
  assert.strictEqual(tddModeIsOn({ workflow: {} }), false);
});

test('a truthy value that is not true is not on', () => {
  // "true" the string is what a hand-edited config ends up holding, and GSD
  // compares against the boolean. Treating it as on would report a setting
  // that is not in force.
  assert.strictEqual(tddModeIsOn({ workflow: { tdd_mode: 'true' } }), false);
});

test('a directory that is not a GSD project is left alone', () => {
  // Every other project on this machine opens here. Warning about a GSD
  // setting where GSD is not in use is noise, and noise is how a warning
  // stops being read.
  assert.strictEqual(whatToSay(false, null), null);
});

test('a GSD project with tdd_mode on says nothing', () => {
  assert.strictEqual(whatToSay(true, { workflow: { tdd_mode: true } }), null);
});

test('a GSD project with tdd_mode off says so and names the fix', () => {
  const said = whatToSay(true, { workflow: { tdd_mode: false } });
  assert.ok(said, 'the check stayed silent with the setting off');
  assert.match(said, /config-set workflow\.tdd_mode true/);
});

test('a config that could not be read warns rather than passing', () => {
  // Cannot-tell is not the same as satisfied. Reporting silence for an
  // unreadable config is exactly the shape this whole guardrail is about.
  const said = whatToSay(true, null);
  assert.ok(said, 'an unreadable config was reported as satisfied');
});
