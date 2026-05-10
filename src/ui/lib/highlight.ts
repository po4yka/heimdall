/**
 * Minimal syntax highlighter for artifact body rendering.
 * No external dependencies — pure token-walk approach.
 * Returns HTML-safe string with <span class="hl-*"> wrappers.
 */

type TT = 'comment' | 'string' | 'keyword' | 'number' | 'plain';
interface Token { t: TT; v: string }

const KEYWORDS: Record<string, ReadonlyArray<string>> = {
  python: ['False','None','True','and','as','assert','async','await','break','class',
    'continue','def','del','elif','else','except','finally','for','from','global',
    'if','import','in','is','lambda','not','or','pass','raise','return','try',
    'while','with','yield'],
  rust: ['Self','as','async','await','break','const','continue','crate','dyn','else',
    'enum','extern','false','fn','for','if','impl','in','let','loop','match','mod',
    'move','mut','pub','ref','return','self','static','struct','super','trait','true',
    'type','union','unsafe','use','where','while'],
  javascript: ['async','await','break','case','catch','class','const','continue','debugger',
    'default','delete','do','else','export','extends','false','finally','for','from',
    'function','if','import','in','instanceof','let','new','null','of','return',
    'static','super','switch','this','throw','true','try','typeof','undefined',
    'var','void','while','with','yield'],
  typescript: ['abstract','any','as','async','await','boolean','break','case','catch',
    'class','const','constructor','continue','declare','default','delete','do','else',
    'enum','export','extends','false','finally','for','from','function','if','implements',
    'import','in','infer','instanceof','interface','is','keyof','let','module','namespace',
    'never','new','null','number','object','of','override','private','protected','public',
    'readonly','return','static','string','super','switch','symbol','this','throw','true',
    'try','type','typeof','undefined','unique','unknown','var','void','while','with','yield'],
  go: ['break','case','chan','const','continue','default','defer','else','fallthrough',
    'for','func','go','goto','if','import','interface','map','package','range','return',
    'select','struct','switch','type','var','true','false','nil'],
  java: ['abstract','assert','boolean','break','byte','case','catch','char','class',
    'const','continue','default','do','double','else','enum','extends','final','finally',
    'float','for','goto','if','implements','import','instanceof','int','interface','long',
    'native','new','null','package','private','protected','public','return','short',
    'static','strictfp','super','switch','synchronized','this','throw','throws',
    'transient','true','try','var','void','volatile','while'],
  sql: ['SELECT','FROM','WHERE','JOIN','LEFT','RIGHT','INNER','OUTER','ON','AS','AND',
    'OR','NOT','IN','LIKE','BETWEEN','IS','NULL','ORDER','BY','GROUP','HAVING','LIMIT',
    'OFFSET','INSERT','INTO','VALUES','UPDATE','SET','DELETE','CREATE','TABLE','INDEX',
    'DROP','ALTER','ADD','COLUMN','PRIMARY','KEY','FOREIGN','REFERENCES','UNIQUE',
    'DEFAULT','CONSTRAINT','IF','EXISTS','WITH','UNION','ALL','DISTINCT','COUNT',
    'SUM','AVG','MIN','MAX','CASE','WHEN','THEN','ELSE','END'],
  bash: ['if','then','else','elif','fi','for','in','do','done','while','until','case',
    'esac','function','return','break','continue','exit','export','local','readonly',
    'declare','unset','set','shift','source','alias','true','false'],
};

// Normalize aliases
KEYWORDS['js'] = KEYWORDS['javascript']!;
KEYWORDS['ts'] = KEYWORDS['typescript']!;
KEYWORDS['sh'] = KEYWORDS['bash']!;
KEYWORDS['shell'] = KEYWORDS['bash']!;
KEYWORDS['zsh'] = KEYWORDS['bash']!;
KEYWORDS['kotlin'] = KEYWORDS['java']!;

function escHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function tokenize(code: string, lang: string): Token[] {
  const kw = new Set<string>(KEYWORDS[lang.toLowerCase()] ?? []);
  const isHtmlLike = ['html', 'xml', 'svg'].includes(lang.toLowerCase());
  const isPyLike = ['python', 'py', 'bash', 'sh', 'shell', 'zsh'].includes(lang.toLowerCase());
  const tokens: Token[] = [];
  let i = 0;
  const n = code.length;

  while (i < n) {
    const ch = code[i]!;

    // HTML/XML comment: <!-- -->
    if (isHtmlLike && ch === '<' && code.startsWith('!--', i + 1)) {
      const end = code.indexOf('-->', i + 4);
      const j = end === -1 ? n : end + 3;
      tokens.push({ t: 'comment', v: code.slice(i, j) });
      i = j;
      continue;
    }

    // Block comment: /* */
    if (ch === '/' && code[i + 1] === '*') {
      const end = code.indexOf('*/', i + 2);
      const j = end === -1 ? n : end + 2;
      tokens.push({ t: 'comment', v: code.slice(i, j) });
      i = j;
      continue;
    }

    // Line comment: //
    if (ch === '/' && code[i + 1] === '/' && !isHtmlLike) {
      const end = code.indexOf('\n', i);
      const j = end === -1 ? n : end;
      tokens.push({ t: 'comment', v: code.slice(i, j) });
      i = j;
      continue;
    }

    // Line comment: # (Python, Bash, Ruby, etc.)
    if (ch === '#' && isPyLike) {
      const end = code.indexOf('\n', i);
      const j = end === -1 ? n : end;
      tokens.push({ t: 'comment', v: code.slice(i, j) });
      i = j;
      continue;
    }

    // SQL line comment: --
    if (ch === '-' && code[i + 1] === '-' && lang.toLowerCase() === 'sql') {
      const end = code.indexOf('\n', i);
      const j = end === -1 ? n : end;
      tokens.push({ t: 'comment', v: code.slice(i, j) });
      i = j;
      continue;
    }

    // Triple-quoted strings (Python docstrings): """ or '''
    if (isPyLike && (ch === '"' || ch === "'") && code[i + 1] === ch && code[i + 2] === ch) {
      const q = ch.repeat(3);
      const end = code.indexOf(q, i + 3);
      const j = end === -1 ? n : end + 3;
      tokens.push({ t: 'string', v: code.slice(i, j) });
      i = j;
      continue;
    }

    // Template literal / backtick string (no nested expression tracking — best-effort)
    if (ch === '`' || ch === '"' || ch === "'") {
      const q = ch;
      let j = i + 1;
      while (j < n) {
        if (code[j] === '\\') { j += 2; continue; }
        if (code[j] === q) { j++; break; }
        // Newline ends single/double quoted strings but not backtick
        if (q !== '`' && code[j] === '\n') break;
        j++;
      }
      tokens.push({ t: 'string', v: code.slice(i, j) });
      i = j;
      continue;
    }

    // Numeric literals (hex, float, int, binary)
    if (ch >= '0' && ch <= '9') {
      let j = i + 1;
      while (j < n) {
        const c = code[j]!;
        if ((c >= '0' && c <= '9') || c === '.' || c === '_' || c === 'x' || c === 'X' ||
            c === 'b' || c === 'B' || c === 'o' || c === 'O' ||
            (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F') ||
            c === 'e' || c === 'E' || c === '+' || c === '-') {
          j++;
        } else break;
      }
      tokens.push({ t: 'number', v: code.slice(i, j) });
      i = j;
      continue;
    }

    // Identifiers and keywords
    if ((ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch === '_' || ch === '$') {
      let j = i + 1;
      while (j < n) {
        const c = code[j]!;
        if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
            (c >= '0' && c <= '9') || c === '_' || c === '$') {
          j++;
        } else break;
      }
      const word = code.slice(i, j);
      tokens.push({ t: kw.has(word) ? 'keyword' : 'plain', v: word });
      i = j;
      continue;
    }

    // Plain character (operator, whitespace, punctuation, etc.)
    tokens.push({ t: 'plain', v: ch });
    i++;
  }

  return tokens;
}

/**
 * Highlight `code` for the given language.
 * Returns an HTML string safe to inject via `dangerouslySetInnerHTML`.
 * Falls back to plain escaped text for unknown/empty languages.
 */
export function highlight(code: string, lang: string): string {
  if (!lang || lang === 'text' || lang === 'plain' || lang === 'plaintext') {
    return escHtml(code);
  }
  const tokens = tokenize(code, lang);
  return tokens
    .map(({ t, v }) => (t === 'plain' ? escHtml(v) : `<span class="hl-${t}">${escHtml(v)}</span>`))
    .join('');
}
