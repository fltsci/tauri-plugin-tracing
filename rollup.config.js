import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { cwd } from 'node:process'
import { nodeResolve } from '@rollup/plugin-node-resolve'
import typescript from '@rollup/plugin-typescript'
import terser from '@rollup/plugin-terser'

/**
 * Create a base rollup config
 *
 * @param {object} [options] Configuration object
 * @param {string} [options.input] Input path
 * @param {import('rollup').ExternalOption} [options.external] External dependencies list
 * @param {import('rollup').RollupOptions | import('rollup').RollupOptions[]} [options.additionalConfigs] Additional rollup configurations
 *
 * @returns {import('rollup').RollupOptions}
 */
function createConfig(options = {}) {
  const pkg = JSON.parse(readFileSync(join(cwd(), 'package.json'), 'utf8'))

  const pluginJsName = pkg.name
    .replace('@fltsci/tauri-plugin-', '')
    .replace(/-./g, (x) => x[1].toUpperCase())
  const iifeVarName = `__TAURI_PLUGIN_${pkg.name
    .replace('@fltsci/tauri-plugin-', '')
    .replace('-', (x) => '_')
    .toUpperCase()}__`

  const {
    input = 'guest-js/index.ts',
    external = [
      /^@tauri-apps\/api/,
      ...Object.keys(pkg.dependencies || {}),
      ...Object.keys(pkg.peerDependencies || {})
    ],
    additionalConfigs = []
  } = options

  return [
    {
      input,
      output: [
        {
          file: pkg.exports.import,
          format: 'esm'
        },
        {
          file: pkg.exports.require,
          format: 'cjs'
        }
      ],
      plugins: [
        typescript({
          declaration: true,
          declarationDir: dirname(pkg.exports.import)
        })
      ],
      external,
      onwarn: (warning) => {
        throw Object.assign(new Error(), warning)
      }
    },

    {
      input,
      output: {
        format: 'iife',
        name: iifeVarName,
        // IIFE is in the format `var ${iifeVarName} = (() => {})()`
        // we check if __TAURI__ exists and inject the API object
        banner: "if ('__TAURI__' in window) {",
        // the last `}` closes the if in the banner
        footer: `Object.defineProperty(window.__TAURI__, '${pluginJsName}', { value: ${iifeVarName} }) }`,
        file: './dist-js/api-iife.js'
      },
      // and var is not guaranteed to assign to the global `window` object so we make sure to assign it
      plugins: [typescript(), terser(), nodeResolve()],
      onwarn: (warning) => {
        throw Object.assign(new Error(), warning)
      }
    },

    ...(Array.isArray(additionalConfigs)
      ? additionalConfigs
      : [additionalConfigs])
  ]
}

export default createConfig()
