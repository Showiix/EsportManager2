const mockResponses = new Map<string, unknown>()

export function __setMockResponse(command: string, response: unknown) {
  mockResponses.set(command, response)
}

export function __clearMockResponses() {
  mockResponses.clear()
}

export async function invoke<T>(command: string): Promise<T> {
  if (mockResponses.has(command)) return mockResponses.get(command) as T
  throw new Error(`No mock for command: ${command}`)
}
