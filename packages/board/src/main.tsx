import '@mikata/theme/tokens.css';
// The interface's faces, served by the board itself (the CSP allows no other font source).
import '@fontsource/manrope/latin-400.css';
import '@fontsource/manrope/latin-600.css';
import '@fontsource/manrope/latin-700.css';
import '@fontsource/manrope/latin-800.css';
import '@fontsource/jetbrains-mono/latin-400.css';
import '@fontsource/jetbrains-mono/latin-600.css';
import '@fontsource/jetbrains-mono/latin-800.css';
import './board.css';
import './chat.css';
import './ui/dialogs.css';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App.tsx';
import { takeKey } from './data/session.ts';
import { apply, applyScheme, stored, storedScheme } from './theme.ts';

// The key leaves the address bar before anything reads it; the theme is set before the first paint, so a light system never flashes dark.
takeKey();
apply(stored());
applyScheme(storedScheme());

const root = document.getElementById('root');
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
