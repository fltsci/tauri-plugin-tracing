import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

createRoot(
  document.getElementById('root') ?? document.createElement('root')
).render(
  <StrictMode>
    <App />
  </StrictMode>
)
