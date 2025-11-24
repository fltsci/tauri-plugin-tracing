# Changelog

## \[0.2.0-canary.4]

- [`a8d659b`](https://github.com/fltsci/tauri-plugin-tracing/commit/a8d659bf6a132994cc00a1cd17fb8ec1f5af0d60) ([#38](https://github.com/fltsci/tauri-plugin-tracing/pull/38) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Skipping a version previously published to satisfy NPM publishing rules.

## \[0.2.0-canary.3]

- [`96c37b6`](https://github.com/fltsci/tauri-plugin-tracing/commit/96c37b62b92486c72749607e005e6ed7bb37ac73) ([#36](https://github.com/fltsci/tauri-plugin-tracing/pull/36) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Fix a lockup possibly caused by the timings' feature's use of a sync mutex.

## \[0.2.0-canary.2]

- [`16ee339`](https://github.com/fltsci/tauri-plugin-tracing/commit/16ee339387ede491544544dd7feaf56d019e6b8c) ([#34](https://github.com/fltsci/tauri-plugin-tracing/pull/34) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add [release channel check](https://docs.npmjs.com/adding-dist-tags-to-packages) to publish job CI.
- [`16ee339`](https://github.com/fltsci/tauri-plugin-tracing/commit/16ee339387ede491544544dd7feaf56d019e6b8c) ([#34](https://github.com/fltsci/tauri-plugin-tracing/pull/34) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) -   Update Tauri to version 2.9.3
  - Update npm dependencies to latest
  - Update example to latest implementation

## \[0.2.0-canary.1]

- [`87086a7`](https://github.com/fltsci/tauri-plugin-tracing/commit/87086a7e7995737d6399a34c6c75ab5938361680) ([#24](https://github.com/fltsci/tauri-plugin-tracing/pull/24) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Update examples and Cargo dependencies.

## \[0.2.0-canary.0]

- [`f865b4a`](https://github.com/fltsci/tauri-plugin-tracing/commit/f865b4aeb0fe23a4b81490059edf7c9f18670ddc) ([#22](https://github.com/fltsci/tauri-plugin-tracing/pull/22) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add timing.

## \[0.1.2-canary.7]

- [`b0fd006`](https://github.com/fltsci/tauri-plugin-tracing/commit/b0fd006759d281a83b6cfb6d54d9e83d76e5bff6) ([#20](https://github.com/fltsci/tauri-plugin-tracing/pull/20) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Refine webview path, change initialization from global to default subscriber

## \[0.1.2-canary.6]

- [`c72e376`](https://github.com/fltsci/tauri-plugin-tracing/commit/c72e37632f064b3f2cc8dea354a690622bf14a4e) ([#18](https://github.com/fltsci/tauri-plugin-tracing/pull/18) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Use sync methods to better emulate the JavaScript console.

## \[0.1.2-canary.5]

- [`ccddfd9`](https://github.com/fltsci/tauri-plugin-tracing/commit/ccddfd9d98c6bf32cb3a6ac77a119efecac1ce92) ([#16](https://github.com/fltsci/tauri-plugin-tracing/pull/16) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Add `specta` feature for integration with other Tauri & typescript plugins.
- [`ccddfd9`](https://github.com/fltsci/tauri-plugin-tracing/commit/ccddfd9d98c6bf32cb3a6ac77a119efecac1ce92) ([#16](https://github.com/fltsci/tauri-plugin-tracing/pull/16) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Fix various bugs that have popped up in testing.

## \[0.1.2-canary.4]

- [`549f972`](https://github.com/fltsci/tauri-plugin-tracing/commit/549f972627fc348d8227bcf4c5e1b97e24c639a7) Move NPM package to GitHub packages.

## \[0.1.2-canary.3]

- [`e041ec2`](https://github.com/fltsci/tauri-plugin-tracing/commit/e041ec22c232e78df7e3011ac170376588979cd2) ([#14](https://github.com/fltsci/tauri-plugin-tracing/pull/14) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) In the event webview returns an empty callstack, tauri-plugin-tracing will now use an empty string as the callstack.

## \[0.1.2-canary.2]

- [`6977ca8`](https://github.com/fltsci/tauri-plugin-tracing/commit/6977ca88896d01671048c2b384985b8877c32598) ([#12](https://github.com/fltsci/tauri-plugin-tracing/pull/12) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Users can now filter targets using [tracing_subscriber::Target::with_taget(...)](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/targets/struct.Targets.html) syntax.

## \[0.1.2-canary.1]

- [`6c43912`](https://github.com/fltsci/tauri-plugin-tracing/commit/6c439128ba328244843967d24a1a7531e390c383) ([#9](https://github.com/fltsci/tauri-plugin-tracing/pull/9) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Fine tune publishing jobs, use a release branch.

## \[0.1.2-canary.0]

- [`847a016`](https://github.com/fltsci/tauri-plugin-tracing/commit/847a016916292305babbd91bcb6bb5a1a364d764) ([#3](https://github.com/fltsci/tauri-plugin-tracing/pull/3) by [@johncarmack1984](https://github.com/fltsci/tauri-plugin-tracing/../../johncarmack1984)) Set up publishing to kellnr registry and npm via covector. Add CI jobs for linting and formatting. Adhere to tauri-plugin repo standards wherever practical.
