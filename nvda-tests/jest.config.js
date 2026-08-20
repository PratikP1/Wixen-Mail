// Plain CommonJS, no TypeScript and no transform: Jest reads `@guidepup/guidepup`
// and these test files exactly as written, with nothing converting either one
// first. Kept in this file rather than the "jest" key in package.json only so
// the comment above has somewhere to live.
module.exports = {
  testEnvironment: "node",
  // The app has to start, NVDA has to start, and a real announcement has to
  // reach a debounced speech capture on the other end. Jest's five-second
  // default is tuned for calling functions, not for that.
  testTimeout: 120000,
  // Jest runs test files in parallel worker processes by default. NVDA is a
  // single, machine-wide resource, the same way wxWidgets allows exactly one
  // live application per process: two test files racing to start their own
  // copy through Guidepup at the same time collide, the way running Guidepup
  // against a machine that already has a real NVDA session running does (see
  // the README). This package had only one file actually driving NVDA until
  // a second and third joined it, so the collision had never been possible
  // before and nothing caught it. One worker, so test files run one after
  // another and never share the machine's one NVDA instance.
  maxWorkers: 1,
};
