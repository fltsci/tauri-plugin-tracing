import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

import * as tracing from '@fltsci/tauri-plugin-tracing'

console.log = tracing.info
console.debug = tracing.debug
console.info = tracing.info
console.warn = tracing.warn
console.error = tracing.error
console.trace = tracing.trace
console.time = tracing.time
console.timeEnd = tracing.timeEnd

createRoot(
  document.getElementById('root') ?? document.createElement('root')
).render(
  <StrictMode>
    <App />
  </StrictMode>
)
