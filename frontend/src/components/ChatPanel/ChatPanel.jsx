import React, { useEffect, useRef } from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';
import { LANDING_PAGE_PATTERN } from '../../kits.js';
import Message from './Message.jsx';
import PrePagePicker, { startDesign } from './PrePagePicker.jsx';
import PageSuggestions from './PageSuggestions.jsx';
import CritiqueFixes from './CritiqueFixes.jsx';
import ModeClarifyPicker from './ModeClarifyPicker.jsx';

export default function ChatPanel() {
  const messages       = useStore(s => s.messages);
  const status         = useStore(s => s.status);
  const currentHTML    = useStore(s => s.currentHTML);
  const pendingIdea    = useStore(s => s.pendingIdea);
  const currentTarget  = useStore(s => s.currentTarget);
  const generating     = useStore(s => s.status.generating);

  const messagesRef = useRef(null);
  const textareaRef = useRef(null);

  const isProcessing = status.state === 'busy';

  // Auto-scroll on message change
  useEffect(() => {
    const el = messagesRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages]);

  function autoResize(el) {
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 120) + 'px';
  }

  function onKey(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  }

  function send() {
    const ta = textareaRef.current;
    if (!ta) return;
    const text = (ta.value || '').trim();
    const s = useStore.getState();
    if (s.status.state === 'busy') return;

    // Kit picker is now hosted in the Canvas as a wizard. If it's open,
    // Send in the composer is a no-op — user drives it via the chip clicks.
    if (s.kitPicker) return;

    // ── Step 1: no design yet, no pending idea → this Send is the idea. ──
    if (!s.currentHTML && !s.pendingIdea && !s.currentTarget) {
      if (!text) return;
      s.addUser(text);
      ta.value = ''; ta.style.height = 'auto';

      // URL detection → skip picker, go straight to reference-based design.
      const urlMatch = text.match(/https?:\/\/\S+/);
      if (urlMatch) {
        s.addAssistant(
          'Detected reference URL. Rendering it in headless Chrome, extracting palette, typography, image assets, motion tokens (animations, transitions, gradients, shadows), and 3D usage (three.js / WebGL). If the reference uses 3D, I\'ll include a matching three.js signature element in the new design.'
        );
        s.setStatus('busy', 'Fetching reference…');
        startDesign(
          text,
          'Match the reference site\'s extracted palette and typography exactly. Improve execution: cleaner hierarchy, tighter type scale, visible focus rings, one signature element specific to the subject.'
        );
        return;
      }

      // Landing-page → open the canvas-side kit picker wizard.
      if (LANDING_PAGE_PATTERN.test(text)) {
        s.setPendingIdea(text);
        s.openKitPicker(text);
        s.addAssistant('I\'ll build a landing page — answer the questions on the right, then I\'ll assemble.');
        return;
      }

      // Everything else → pre-page picker → freeform.
      s.addPrePagePicker(text);
      return;
    }

    if (!text) return;

    // ── Element-scoped edit ──
    if (s.currentTarget && s.currentTarget.selector) {
      const outer = getSelectedOuterHTML();
      if (outer) {
        if (s.currentHTML) ipcSend('sync_design', s.currentHTML);
        s.addUser('→ ' + s.currentTarget.selector + '\n' + text);
        ta.value = ''; ta.style.height = 'auto';
        s.setStatus('busy', 'Working…');
        ipcSend('refine_element', JSON.stringify({
          selector:   s.currentTarget.selector,
          outer_html: outer,
          prompt:     text,
        }));
        return;
      }
    }

    // ── Wireframe refinement (when the active tab is showing a skeleton) ──
    // We route to a separate IPC that rewrites the .skeleton.html file
    // instead of the built page (which may not even exist yet).
    const activePage = (s.session.tabs || []).find(p => p.slug === s.session.currentPage);
    const activeMode = (s.canvasViewMode || {})[activePage?.slug]
      || (activePage && activePage.built === false && activePage.has_skeleton ? 'skeleton' : 'built');
    if (activePage && activeMode === 'skeleton') {
      if (s.currentHTML) ipcSend('sync_design', s.currentHTML);
      s.addUser(text);
      ta.value = ''; ta.style.height = 'auto';
      s.setStatus('busy', 'Tweaking wireframe…');
      ipcSend('refine_skeleton', JSON.stringify({ slug: activePage.slug, prompt: text }));
      return;
    }

    // ── Normal refine ──
    if (s.currentHTML) ipcSend('sync_design', s.currentHTML);
    s.addUser(text);
    ta.value = ''; ta.style.height = 'auto';
    s.setStatus('busy', 'Working…');
    ipcSend('user_message', text);
  }

  function stopGeneration() {
    ipcSend('stop_generation', '');
    const s = useStore.getState();
    s.setGenerating(false);
    s.removeProgress();
    s.setStatus('ready', 'Stopped');
  }

  function clearTarget() {
    const s = useStore.getState();
    s.setCurrentTarget(null);
    // Forward clearSelection to iframe.
    const frame = document.querySelector('iframe.preview-frame');
    if (frame && frame.contentWindow) {
      try { frame.contentWindow.postMessage({ cmd: 'clearSelection' }, '*'); } catch(_) {}
    }
  }

  return (
    <div className="chat-pane">
      <div className="chat-head">
        <div className="eyebrow accent">A conversation</div>
        <h2>What are we <em>designing</em>?</h2>
        <p>Describe a product, a page, or a mood — this canvas will draft it, iterate with you, and export clean HTML.</p>
      </div>

      <div className="messages" ref={messagesRef}>
        {messages.map(m => {
          if (m.kind === 'pre_page_picker')  return <PrePagePicker     key={m.id} msg={m} />;
          if (m.kind === 'page_suggestions') return <PageSuggestions   key={m.id} msg={m} />;
          if (m.kind === 'critique')         return <CritiqueFixes     key={m.id} msg={m} />;
          if (m.kind === 'mode_clarify')     return <ModeClarifyPicker key={m.id} msg={m} />;
          return <Message key={m.id} msg={m} />;
        })}
      </div>

      {generating && (
        <button className="stop-btn" onClick={stopGeneration}>■ Stop generation</button>
      )}

      <div className="footnote">
        Nothing is exported until you press <em>Export</em>. Refine as many times as you like.
      </div>

      <div className="input-area">
        <div className="eyebrow">Prompt</div>
        {currentTarget && currentTarget.selector && (
          <div className="target-chip on">
            <span className="label">Targeting</span>
            <span className="sel">{currentTarget.selector}</span>
            <button className="clear" onClick={clearTarget} title="Clear selection">×</button>
          </div>
        )}
        <div className="input-row">
          <textarea
            ref={textareaRef}
            className="input-textarea"
            placeholder="Describe your idea…"
            rows={1}
            onKeyDown={onKey}
            onInput={(e) => autoResize(e.currentTarget)}
          />
          <button className="send-btn" onClick={send} disabled={isProcessing} title="Send (Enter)">
            Send
            <svg viewBox="0 0 24 24"><path d="M2 21l21-9L2 3v7l15 2-15 2v7z" /></svg>
          </button>
        </div>
      </div>
    </div>
  );
}

// Reads the currently-selected element from the preview iframe.
function getSelectedOuterHTML() {
  try {
    const frame = document.querySelector('iframe.preview-frame');
    const doc = frame && frame.contentDocument;
    if (!doc) return null;
    const el = doc.querySelector('[data-ov-sel]');
    if (!el) return null;
    const clone = el.cloneNode(true);
    clone.removeAttribute('data-ov-sel');
    clone.removeAttribute('data-ov-hover');
    clone.removeAttribute('data-ov-dragging');
    clone.removeAttribute('contenteditable');
    return clone.outerHTML;
  } catch (_) {
    return null;
  }
}
