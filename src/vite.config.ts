import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import vue from '@vitejs/plugin-vue'
import { defineConfig, type Plugin } from 'vite'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const resourcesRoot = path.join(repoRoot, 'resources')

// 桌面壳版本（tauri.conf.json 是发版真相源）；浏览器预览用它显示「当前版本」
const appVersion = JSON.parse(
  fs.readFileSync(path.join(repoRoot, 'src-tauri', 'tauri.conf.json'), 'utf8'),
).version as string

export default defineConfig({
  plugins: [vue(), elwrightDataApi()],
  define: { __APP_VERSION__: JSON.stringify(appVersion) },
})

/// Dev-only read-only endpoints so the browser preview talks to the real
/// capabilities.json and resources/ files instead of mock data.
/// - GET /api/capabilities -> capabilities.json
/// - GET /api/file?path=resources/... -> file content（仅允许 resources/ 前缀）
function elwrightDataApi(): Plugin {
  return {
    name: 'elwright-data-api',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = new URL(req.url ?? '/', 'http://localhost')
        if (url.pathname === '/api/capabilities') {
          sendText(res, path.join(repoRoot, 'capabilities.json'))
        } else if (url.pathname === '/api/file') {
          const rel = url.searchParams.get('path') ?? ''
          const abs = path.resolve(repoRoot, rel)
          if (!abs.startsWith(resourcesRoot + path.sep) && abs !== resourcesRoot) {
            res.statusCode = 403
            res.end('仅允许访问 resources/ 下的文件')
          } else {
            sendText(res, abs)
          }
        } else {
          next()
        }
      })
    },
  }
}

function sendText(res: { statusCode: number; setHeader: (k: string, v: string) => void; end: (s: string) => void }, file: string) {
  fs.readFile(file, 'utf8', (err, data) => {
    if (err) {
      res.statusCode = 404
      res.end(`not found: ${path.basename(file)}`)
      return
    }
    res.setHeader('Content-Type', 'text/plain; charset=utf-8')
    res.end(data)
  })
}
