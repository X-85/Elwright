// B6 e2e 的 IPC mock：浏览器 e2e 无 Tauri 运行时，注入本函数作为
// __TAURI_INTERNALS__.invoke 的实现，使前端走真实 tauriBridge 全链路。
// 注意：本文件以字符串形式嵌入页面（new Function），不能 import 任何模块；
// 反斜杠路径一律 String.fromCharCode(92) 运行时构造，避免序列化吃转义。

/* eslint-disable @typescript-eslint/no-explicit-any */

export function makeInvoke() {
  return (cmd, args = {}) => {
    switch (cmd) {
      case 'plugin:dialog|open':
        // 选择项目目录 → D:\demo\myproj（join 拼反斜杠）
        return ['D:', 'demo', 'myproj'].join(String.fromCharCode(92))
      case 'code_browser_tree':
        // 按 rel 分层返回，避免同一目录在自身子层重复导致 UI 无限递归
        if (args.rel === 'src')
          return [
            {
              path: 'src/main/java/com/example/App.java',
              name: 'App.java',
              kind: 'file',
              size: 20,
              readable: true,
              sensitive: false,
            },
          ]
        return [{ path: 'src', name: 'src', kind: 'dir', size: 0, readable: true, sensitive: false }]
      case 'code_browser_read':
        return {
          path: args.rel,
          name: 'App.java',
          language: 'java',
          size: 20,
          truncated: false,
          content: 'public class App {}\n',
          sensitive: false,
          notice: '',
        }
      case 'code_browser_recent_load':
      case 'code_browser_recent_open':
        return { projects: [], files: [], favorites: [], bookmarks: [] }
      case 'code_browser_scan_symbols':
        return []
      case 'list_capabilities':
        // App.vue 首屏对 capabilities.value 调 filter，必须是数组
        return []
      case 'check_update':
        return { version: '0.0.0-test', notes: '', url: '' }
      default:
        // 其余命令统一空数组兜底（绝大多数调用方期待可迭代结果）
        return []
    }
  }
}
