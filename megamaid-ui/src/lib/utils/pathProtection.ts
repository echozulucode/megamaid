const PROTECTED_DIR_MARKERS = ['.git', '.hg', '.svn'];
const MANIFESTS = ['package.json', 'Cargo.toml', 'pyproject.toml'];

export function isProtectedPath(path: string): boolean {
  const lower = path.toLowerCase();
  if (lower === '.' || lower === '/' || lower === '\\') return true;

  for (const marker of PROTECTED_DIR_MARKERS) {
    if (lower.includes(`/${marker}`) || lower.includes(`\\${marker}`)) return true;
  }

  for (const manifest of MANIFESTS) {
    if (lower.includes(`/${manifest}`) || lower.includes(`\\${manifest}`)) return true;
  }

  return false;
}
