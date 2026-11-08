import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [
    react(),
    {
      name: 'development-csp',
      apply: 'serve',
      // React Refresh adds an inline preamble in development.
      transformIndexHtml: html => html.replace("script-src 'self';", "script-src 'self' 'unsafe-inline';"),
    },
  ],
  base: './',
  build: { outDir: 'dist' },
  server: { host: '127.0.0.1', port: 5173, strictPort: true },
});
