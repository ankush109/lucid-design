import React, { useState } from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';
import { openModal } from './Modal.jsx';

export default function PageSuggestions({ msg }) {
  const [disabled, setDisabled] = useState(false);
  const [text, setText] = useState('');
  const candidates = Array.isArray(msg.content) ? msg.content : [];
  if (candidates.length === 0) return null;

  async function startPage(name) {
    const st = useStore.getState();
    if (st.status.state === 'busy' || disabled) return;
    const cleanName = (name || '').trim();
    if (!cleanName) return;
    const brief = await openModal({
      eyebrow: 'New page',
      title:   `Design the <em>${escapeHtml(cleanName)}</em> page.`,
      desc:    'Describe features, data, and layout — sections, key actions, any preferred archetype. The shell (nav, sidebar, theme) will carry over.',
      placeholder: 'e.g. weekly volume chart, PR list, workouts table; split-view with sidebar filters',
      okText:  'Design page',
    });
    const briefText = (brief === true || brief === null) ? '' : String(brief || '').slice(0, 500);
    setDisabled(true);
    const userMsg = briefText
      ? `Design the ${cleanName} page — ${briefText}`
      : `Design the ${cleanName} page`;
    st.addUser(userMsg);
    st.setStatus('busy', `Designing ${cleanName}…`);
    ipcSend('create_page', JSON.stringify({ name: cleanName, brief: briefText }));
  }

  function submitFree() {
    const name = text.trim();
    if (name) startPage(name);
  }

  return (
    <div className="msg assistant">
      <span className="msg-label">Assistant</span>
      <div className="bubble">
        Your design links to <strong>{candidates.length}</strong> other page{candidates.length === 1 ? '' : 's'}.
        Pick one to design next — I'll ask what it should contain, then build it in a new tab.
        The shell (nav, sidebar, theme) carries over.
        <div className="theme-picker" style={{ marginTop: 12 }}>
          {candidates.map((c, i) => (
            <button
              key={i}
              className="theme-chip page-suggest-chip"
              disabled={disabled}
              onClick={(e) => { e.preventDefault(); startPage(c.name); }}
            >
              {c.name}
            </button>
          ))}
        </div>
        <div className="page-suggest-freeform" style={{ display: 'flex', gap: 8, marginTop: 12, alignItems: 'center' }}>
          <input
            type="text" className="page-suggest-input"
            placeholder="Or type a page name…"
            style={{ flex: 1 }}
            maxLength={60}
            value={text}
            disabled={disabled}
            onChange={e => setText(e.target.value)}
            onKeyDown={e => { if (e.key === 'Enter') { e.preventDefault(); submitFree(); }}}
          />
          <button className="theme-chip page-suggest-freeform-btn" style={{ fontWeight: 600 }} disabled={disabled} onClick={(e) => { e.preventDefault(); submitFree(); }}>Design page</button>
        </div>
      </div>
    </div>
  );
}

function escapeHtml(s) {
  return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}
