import process from 'process';
import color from '@oclif/color';

const currentNodeVersion = process.versions.node;
const semver = currentNodeVersion.split('.');
const major = parseInt(<string> semver[0], 10);
// const _minor = parseInt(<string> semver[1], 10);

if (major < 18) {
  console.error(
    color.red(
      `You are running Node.js ${currentNodeVersion}.\n` +
      'Cube.js CLI requires Node.js 20 or higher \n' +
      'Please update your Node.js version.'
    )
  );
  process.exit(1);
}

if (major === 19) {
  process.emitWarning(
    color.red(
      `You are running Node.js ${currentNodeVersion}.\n` +
      'Support for Node.js 19 not guaranty. Please upgrade to Node.js 20 or higher.'
    )
  );
}

if (major === 18) {
  process.emitWarning(
    color.red(
      `You are running Node.js ${currentNodeVersion}.\n` +
      'Support for Node.js 18 will be removed soon. Please upgrade to Node.js 20 or higher.'
    )
  );
}
