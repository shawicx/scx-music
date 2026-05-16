import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from '../types'

/**
 * 统一的 Tauri 命令调用错误处理
 * @param command Tauri 命令名称
 * @param args 命令参数
 * @param showMessage 是否显示错误消息给用户 (默认 true)
 * @returns Promise<T>
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
  showMessage = true
): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    console.error(`Command [${command}] failed:`, errorMessage)

    if (showMessage) {
      // 这里可以集成 toast 通知
      console.error(`操作失败: ${errorMessage}`)
    }

    throw new Error(`Command [${command}] failed: ${errorMessage}`)
  }
}

/**
 * 包装 API 响应，确保类型安全
 * @param command Tauri 命令名称
 * @param args 命令参数
 * @returns Promise<ApiResponse<T>>
 */
export async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<ApiResponse<T>> {
  try {
    const data = await invoke<T>(command, args)
    return { data }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    console.error(`Command [${command}] failed:`, errorMessage)
    return { error: errorMessage }
  }
}

/**
 * 创建 Result 类型的辅助函数
 */
export function success<T>(data: T) {
  return { success: true as const, data }
}

export function failure(error: string) {
  return { success: false as const, error }
}

/**
 * 批量执行命令，收集错误
 * @param commands 命令数组
 * @returns 成功数量和失败信息
 */
export async function batchInvoke(
  commands: Array<{ command: string; args?: Record<string, unknown> }>
): Promise<{ successCount: number; errors: Array<{ command: string; error: string }> }> {
  const errors: Array<{ command: string; error: string }> = []
  let successCount = 0

  await Promise.all(
    commands.map(async ({ command, args }) => {
      try {
        await invoke(command, args)
        successCount++
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error)
        errors.push({ command, error: errorMessage })
      }
    })
  )

  return { successCount, errors }
}

/**
 * 重试机制
 * @param fn 要执行的函数
 * @param maxRetries 最大重试次数
 * @param delay 重试延迟 (ms)
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxRetries = 3,
  delay = 1000
): Promise<T> {
  let lastError: Error | undefined

  for (let i = 0; i <= maxRetries; i++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error))
      if (i < maxRetries) {
        await new Promise(resolve => setTimeout(resolve, delay * (i + 1)))
      }
    }
  }

  throw lastError!
}