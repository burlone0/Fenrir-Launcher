// Deterministic cover placeholder: same title always produces same color + initials.

const PALETTE = [
  "#6366f1", // indigo
  "#8b5cf6", // violet
  "#10b981", // emerald
  "#f59e0b", // amber
  "#f43f5e", // rose
  "#06b6d4", // cyan
  "#f97316", // orange
  "#14b8a6", // teal
  "#3b82f6", // blue
  "#ec4899", // pink
];

function hashTitle(title: string): number {
  let hash = 0;
  for (let i = 0; i < title.length; i++) {
    hash = (hash + title.charCodeAt(i)) | 0;
  }
  return hash;
}

function initials(title: string): string {
  if (!title) return "?";
  const words = title.trim().split(/\s+/);
  if (words.length === 1) {
    return words[0].slice(0, 2).toUpperCase();
  }
  return (words[0][0] + words[1][0]).toUpperCase();
}

export function coverFor(title: string): { bgColor: string; initials: string } {
  const hash = Math.abs(hashTitle(title));
  const bgColor = PALETTE[hash % PALETTE.length];
  return { bgColor, initials: initials(title) };
}
