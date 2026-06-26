import { invoke } from '@tauri-apps/api/core'

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
 *
 * UI 层的 toast 提示由各 composable 在 catch 中自行处理（useXxx 内的 showToast），
 * 此处只负责结构化错误传递，不做 UI 副作用。
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
 * 统一的 Tauri 命令调用入口。
 *
 * 仅做错误结构化：捕获后端 reject，转为 `AppInvokeError`（保留 AppError payload），
 * 重新抛出供调用方按 `errorType` 分类处理。UI 提示（toast）由各 composable 自行决定，
 * 避免本函数对"是否显示"做无效假设（原 showMessage 参数已移除）。
 *
 * @param command Tauri 命令名称
 * @param args 命令参数
 * @returns Promise<T>
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    const { message, payload } = extractMessage(error)
    console.error(`Command [${command}] failed:`, message, payload ? `type=${payload.type}` : '')
    throw new AppInvokeError(command, payload, message)
  }
}
