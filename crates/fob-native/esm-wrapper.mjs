import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const binding = require('./index.js')

export const OutputFormat = Object.freeze({
  Esm: 'esm',
  Cjs: 'cjs',
  Iife: 'iife',
})

export const SourceMapMode = Object.freeze({
  External: 'external',
  Inline: 'inline',
  Hidden: 'hidden',
  Disabled: 'false',
})

export const Fob = binding.Fob
export const bundleSingle = binding.bundleSingle
export const initLogging = binding.initLogging
export const initLoggingFromEnv = binding.initLoggingFromEnv
export const version = binding.version

export default {
  ...binding,
  OutputFormat,
  SourceMapMode,
}
