import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from '../types'

/**
 * 后端 AppError 的序列化形态。
 *
 * Rust 侧 `src-tauri/src/error.rs` 的 AppError 通过
 * `#[serde(tag = "type", content = "message")]` 序列化为
 * `{ type: "...", message: "..." }`，Tauri v2 IPC reject 时把它
 * 作为 rejection 值直接传给前端 catch。
 */
export interface AppErrorPayload {
  /** AppError variant 名（FileOperation / Database / AudioParse / AudioPlayback / DeviceNotFound / UnsupportedFormat / InvalidArgument / OperationFailed） */
  type: string
  /** 人类可读错误信息 */
  message: string
}

/**
 * 自定义 IPC 错误类，保留原始 AppError payload，便于调用方按 `errorType`
 * 分类处理（如 `InvalidArgument` 显示用户输入错误、`OperationFailed` 显示系统错误）。
 */
export class AppInvokeError extends Error {
  constructor(
    public readonly command: string,
    public readonly payload: AppErrorPayload | null,
    message: string,
  ) {
    super(`Command [${command}] failed: ${message}`)
    this.name = 'AppInvokeError'
  }

  /** 错误类别（AppError 的 type 字段），无结构化 payload 时返回 null */
  get errorType(): string | null {
    return this.payload?.type ?? null
  }
}

/** 类型守卫：判断 catch 到的错误是否为 AppInvokeError */
export function isAppInvokeError(e: unknown): e is AppInvokeError {
  return e instanceof AppInvokeError
}

/**
 * 从任意 catch 到的错误中提取可读消息 + 结构化 payload。
 * 兼容三种形态：
 * - AppError payload `{type, message}`（Rust 序列化形态）
 * - Error 实例（读 message）
 * - 字符串/其它（String() 兜底）
 */
function extractMessage(error: unknown): { message: string; payload: AppErrorPayload | null } {
  // Tauri v2 直接把 Rust 序列化的 AppError 作为 reject 值
  if (typeof error === 'object' && error !== null && 'type' in error && 'message' in error) {
    const payload = error as AppErrorPayload
    return { message: payload.message, payload }
  }
  if (error instanceof Error) {
    return { message: error.message, payload: null }
  }
  return { message: String(error), payload: null }
}

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
    const { message, payload } = extractMessage(error)
    console.error(`Command [${command}] failed:`, message, payload ? `type=${payload.type}` : '')

    if (showMessage) {
      // 这里可以集成 toast 通知（按 payload.type 分类显示）
      console.error(`操作失败: ${message}`)
    }

    throw new AppInvokeError(command, payload, message)
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
    const { message, payload } = extractMessage(error)
    console.error(`Command [${command}] failed:`, message, payload ? `type=${payload.type}` : '')
    return { error: message }
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
        const { message } = extractMessage(error)
        errors.push({ command, error: message })
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
