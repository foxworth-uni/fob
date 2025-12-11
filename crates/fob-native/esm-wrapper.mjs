import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const wrapper = require('./wrapper.js')

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

// Use wrapped Fob class that supports flexible entries
export const Fob = wrapper.Fob
export const bundleSingle = wrapper.bundleSingle
export const initLogging = wrapper.initLogging
export const initLoggingFromEnv = wrapper.initLoggingFromEnv
export const version = wrapper.version
export const normalizeEntries = wrapper.normalizeEntries

export default {
  ...wrapper,
  OutputFormat,
  SourceMapMode,
}
