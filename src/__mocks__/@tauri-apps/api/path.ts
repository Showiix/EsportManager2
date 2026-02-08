export async function appDataDir() {
  return '/mock/app/data'
}

export async function join(...paths: string[]) {
  return paths.join('/')
}
