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
};
