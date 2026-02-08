import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@tauri-apps/api/core': resolve(__dirname, 'src/__mocks__/@tauri-apps/api/core.ts'),
      '@tauri-apps/api/path': resolve(__dirname, 'src/__mocks__/@tauri-apps/api/path.ts'),
    },
  },
  test: {
    globals: true,
    environment: 'happy-dom',
    include: ['src/**/*.test.ts'],
  },
})
