export function encodeBase64Utf8(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let binary = '';
  for (const b of bytes) {
    binary += String.fromCharCode(b);
  }
  return btoa(binary);
}

export function decodeBase64Utf8(b64: string): string {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return new TextDecoder().decode(bytes);
}

export function encodeHtmlEntities(input: string): string {
  return input
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

export function decodeHtmlEntities(input: string): string {
  const el = document.createElement('textarea');
  el.innerHTML = input;
  return el.value;
}

export function escapeUnicode(input: string): string {
  const parts: string[] = [];
  for (const ch of input) {
    const cp = ch.codePointAt(0);
    if (cp === undefined) {
      continue;
    }
    if (cp <= 0xffff) {
      parts.push(`\\u${cp.toString(16).padStart(4, '0')}`);
    } else {
      parts.push(`\\u{${cp.toString(16)}}`);
    }
  }
  return parts.join('');
}

export function unescapeUnicode(input: string): string {
  return input
    .replaceAll(/\\u\{([0-9a-fA-F]+)\}/g, (_m, hex) => String.fromCodePoint(Number.parseInt(hex, 16)))
    .replaceAll(/\\u([0-9a-fA-F]{4})/g, (_m, hex) => String.fromCharCode(Number.parseInt(hex, 16)));
}
