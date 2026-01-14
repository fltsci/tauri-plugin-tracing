import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { takeoverConsole } from '@fltsci/tauri-plugin-tracing'

// Full console takeover: JS console → Rust tracing → browser console
// All logs flow through Rust's tracing infrastructure (file logging, filtering, etc.)
takeoverConsole()

createRoot(
  document.getElementById('root') ?? document.createElement('root')
).render(
  <StrictMode>
    <App />
  </StrictMode>
)
