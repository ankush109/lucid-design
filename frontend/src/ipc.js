// Bridge to the wry-hosted native side.
// Outbound: window.ipc.postMessage(JSON.stringify({kind, content}))
// Inbound:  Rust calls window.__onEvent({type, ...}) via evaluateScript.

import { useStore } from './store.js';

export function ipcSend(kind, content) {
  const body = JSON.stringify({ kind, content: content ?? '' });
  if (window.ipc && typeof window.ipc.postMessage === 'function') {
    window.ipc.postMessage(body);
  } else {
    console.warn('[ipc] no bridge; would have sent', kind, content);
  }
}

let _extraHandler = null;
export function onIpcEvent(handler) { _extraHandler = handler; }

// ── Streaming chunk accumulator (module-local state, mirrors ui.html). ──
const SECTION_PATTERNS = [
  { id: 'css',    label: 'CSS & Variables',   re: /<style[\s>]/i },
  { id: 'nav',    label: 'Navigation',        re: /<nav[\s>]|id="nav|class="[^"]*navbar/i },
  { id: 'hero',   label: 'Hero',              re: /id="hero|class="[^"]*\bhero\b|<header[\s>]/i },
  { id: 'feat',   label: 'Features',          re: /id="features|class="[^"]*\bfeatures?\b/i },
  { id: 'how',    label: 'How it works',      re: /id="how|id="steps|how.it.works/i },
  { id: 'price',  label: 'Pricing',           re: /id="pricing|class="[^"]*\bpricing\b/i },
  { id: 'test',   label: 'Testimonials',      re: /id="testimonials?|testimonial/i },
  { id: 'cta',    label: 'Call to action',    re: /id="cta|class="[^"]*\bcta\b/i },
  { id: 'faq',    label: 'FAQ',               re: /id="faq|class="[^"]*\bfaq\b/i },
  { id: 'footer', label: 'Footer',            re: /<footer[\s>]/i },
];

let accHtml = '';
let sections = [];

function scanSections(html) {
  const ids = new Set(sections.map(s => s.id));
  for (const p of SECTION_PATTERNS) {
    if (!ids.has(p.id) && p.re.test(html)) {
      sections.push({ id: p.id, label: p.label });
      ids.add(p.id);
    }
  }
}

function resetStream() { accHtml = ''; sections = []; }

export function installIpcBridge() {
  window.__onEvent = (msg) => {
    const s = useStore.getState();
    switch (msg.type) {
      case 'assistant': {
        s.removeProgress();
        s.clearEphemeralStatus();
        s.addAssistant(msg.content);
        // setProcessing(false)
        s.setStatus('ready', 'Ready');
        break;
      }
      case 'status': {
        // Ephemeral status bubble (spinner in chat).
        s.addStatus(msg.content);
        break;
      }
      case 'design': {
        s.removeProgress();
        s.clearEphemeralStatus();
        s.setCurrentHTML(msg.content);
        s.setAssemblyPreview('');
        s.setCurrentTarget(null);
        s.closeKitPicker();
        s.setStatus('ready', 'Ready');
        s.bumpHistoryReset();
        resetStream();
        break;
      }
      case 'chunk': {
        s.clearEphemeralStatus();
        // Start progress card if none yet.
        const hasProgress = s.messages.some(m => m.kind === 'progress');
        if (!hasProgress) {
          resetStream();
          s.addProgress();
        }
        accHtml += msg.content || '';
        scanSections(accHtml);
        const kb = (accHtml.length / 1024).toFixed(1);
        s.updateProgress({ kb, sections: [...sections] });
        // Live preview during single-layout streaming.
        const isMulti = /[<]!\x2D\x2D\s*LAYOUT-\d/i.test(accHtml);
        if (!isMulti && (accHtml.length % 300 < (msg.content || '').length || accHtml.includes('</html>'))) {
          s.setAssemblyPreview(accHtml);
        }
        break;
      }
      case 'generating': {
        s.setGenerating(!!msg.value);
        if (msg.value) s.setStatus('busy', 'Generating');
        break;
      }
      case 'projects': {
        s.setProjects(msg.items);
        break;
      }
      case 'project_opened': {
        s.setCurrentProject({ slug: msg.slug, name: msg.name });
        s.resetMessages();
        s.setCurrentHTML(msg.html || '');
        s.setAssemblyPreview('');
        s.setCurrentTarget(null);
        s.setPendingIdea(null);
        s.closeKitPicker();
        s.bumpHistoryReset();
        // Restore chat: legacy string arriving in msg.chat isn't structured;
        // parse if JSON, else drop a synthetic system bubble.
        let chatArr = [];
        try {
          const parsed = JSON.parse(msg.chat || '[]');
          if (Array.isArray(parsed)) chatArr = parsed;
        } catch(_) {}
        if (chatArr.length === 0) {
          s.addAssistant(msg.html && msg.html.trim()
            ? `Opened ${msg.name}. Edit the canvas directly, or describe changes.`
            : `Started ${msg.name}. Describe what you're designing — mention layout style and palette if you have preferences.`
          );
        } else {
          for (const e of chatArr) {
            if (!e || !e.role) continue;
            if (e.role === 'critique' && Array.isArray(e.items)) {
              s.addCritique(e.items);
            } else if (e.role === 'user' || e.role === 'assistant') {
              s.addMessage({
                role: e.role, label: e.label || (e.role === 'user' ? 'You' : 'Assistant'),
                kind: 'text', content: e.text || '',
              });
            }
          }
        }
        s.setStatus('ready', 'Ready');
        // Reset multi-page state — a `pages` event will follow.
        s.setPages([], 'home');
        break;
      }
      case 'meta': {
        s.setMeta(msg.provider, msg.model);
        break;
      }
      case 'tokens': {
        s.setTokens(msg);
        break;
      }
      case 'critique': {
        s.addCritique(msg.items);
        break;
      }
      case 'assembly_preview': {
        // Interim preview only — doesn't touch currentHTML.
        s.setAssemblyPreview(msg.content);
        break;
      }
      case 'pages': {
        s.setPages(msg.pages, msg.active);
        break;
      }
      case 'page_suggestions': {
        s.addPageSuggestions(msg.candidates || []);
        break;
      }
      case 'session_snapshot': {
        s.applySessionSnapshot({
          mode:       msg.mode,
          brief:      msg.brief,
          tokens_in:  msg.tokens_in,
          tokens_out: msg.tokens_out,
        });
        s.removeModeClarify();
        break;
      }
      case 'mode_set': {
        s.setMode(msg.mode);
        // Once mode is resolved, any lingering clarify card can go.
        if (msg.mode && msg.mode !== 'ambiguous') s.removeModeClarify();
        break;
      }
      case 'mode_clarify': {
        // Backend classified as Ambiguous — ask the user to pick.
        s.addModeClarify(msg.brief || '');
        // Undo the setProcessing that fired on start_design; user needs to act.
        s.setStatus('ready', 'Ready');
        s.setGenerating(false);
        break;
      }
      default:
        break;
    }
    if (_extraHandler) _extraHandler(msg);
  };
}

// Debounced save_chat mirror. Callers just push messages via store actions
// and this hook (installed once) mirrors the chat to Rust.
export function installChatMirror() {
  let timer = null;
  let lastKey = '';
  useStore.subscribe((state, prev) => {
    if (state.messages === prev.messages && state.session.currentProject === prev.session.currentProject) return;
    if (!state.session.currentProject) return;
    // Only mirror stable text/critique entries, matching legacy shape.
    const entries = state.messages
      .map(m => {
        if (m.kind === 'text') return { role: m.role, label: m.label, text: m.content };
        if (m.kind === 'critique') return { role: 'critique', items: m.content };
        return null;
      })
      .filter(Boolean);
    const key = JSON.stringify(entries);
    if (key === lastKey) return;
    lastKey = key;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      try { ipcSend('save_chat', key); } catch(_) {}
    }, 250);
  });
}
