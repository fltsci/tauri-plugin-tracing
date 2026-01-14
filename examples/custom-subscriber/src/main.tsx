import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { interceptConsole, attachConsole } from '@fltsci/tauri-plugin-tracing'

// Send JS console calls to Rust tracing, but also keep original console output
interceptConsole({ preserveOriginal: true })

// Also listen for Rust tracing events and show them in browser console
attachConsole()

createRoot(
  document.getElementById('root') ?? document.createElement('root')
).render(
  <StrictMode>
    <App />
  </StrictMode>
)
