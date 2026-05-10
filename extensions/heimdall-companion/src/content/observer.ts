/**
 * Content script: streaming-complete detector + ChatGPT citation scraper.
 *
 * Injected into claude.ai and chatgpt.com at document_idle.
 * Posts {type:'syncNow'} to the background service worker ~30s after the
 * last streaming-indicator mutation, giving the page time to settle.
 * On ChatGPT, also scrapes the citation sidebar and forwards the mapping
 * before the syncNow so the background can attach URLs to the next capture.
 */

const DEBOUNCE_MS = 30_000;
const vendor = location.hostname; // 'claude.ai' or 'chatgpt.com'

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function scheduleSync(): void {
  if (debounceTimer !== null) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    debounceTimer = null;
    if (vendor === 'chatgpt.com') scrapeChatGptCitations();
    chrome.runtime.sendMessage({ type: 'syncNow', vendor }).catch(() => {
      // Background service worker may be inactive — ignore.
    });
  }, DEBOUNCE_MS);
}

// ---------------------------------------------------------------------------
// Streaming-complete detection
// ---------------------------------------------------------------------------

function isStreamingActive(): boolean {
  if (vendor === 'claude.ai') {
    // Claude sets data-streaming="true" on response blocks while generating.
    return document.querySelector('[data-streaming="true"]') !== null;
  }
  if (vendor === 'chatgpt.com') {
    // ChatGPT adds .result-streaming to the active assistant turn.
    return document.querySelector('.result-streaming') !== null;
  }
  return false;
}

let wasStreaming = false;

const observer = new MutationObserver(() => {
  const streaming = isStreamingActive();
  if (wasStreaming && !streaming) {
    // Transition: streaming → complete. Trigger delayed sync.
    scheduleSync();
  }
  wasStreaming = streaming;
});

observer.observe(document.body, {
  subtree: true,
  childList: true,
  attributes: true,
  attributeFilter: ['data-streaming', 'class'],
});

// ---------------------------------------------------------------------------
// ChatGPT citation sidebar scraper
// ---------------------------------------------------------------------------

function currentChatGptConvId(): string | null {
  // URL shape: https://chatgpt.com/c/<conv-id>
  const m = location.pathname.match(/^\/c\/([^/]+)/);
  return m ? m[1] : null;
}

function scrapeChatGptCitations(): void {
  const convId = currentChatGptConvId();
  if (!convId) return;

  // Collect all anchor elements that look like source citations.
  // ChatGPT renders them in a sidebar or footnotes section — we collect all
  // external links that appear after assistant turns.
  const links = Array.from(
    document.querySelectorAll<HTMLAnchorElement>(
      '[data-message-author-role="assistant"] a[href^="http"], ' +
      '.source-link a[href^="http"], ' +
      'aside a[href^="http"]'
    )
  );

  if (links.length === 0) return;

  const mapping: Array<{ index: number; url: string; title: string }> = links
    .map((a, i) => ({
      index: i + 1,
      url: a.href,
      title: a.textContent?.trim() ?? '',
    }))
    .filter(m => m.url.startsWith('http'));

  if (mapping.length === 0) return;

  chrome.runtime.sendMessage({ type: 'chatgptCitations', convId, mapping }).catch(() => {
    // Service worker inactive — ignore.
  });
}
