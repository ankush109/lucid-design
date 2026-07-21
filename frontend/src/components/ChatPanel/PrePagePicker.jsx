import React, { useEffect, useRef, useState } from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';

export default function PrePagePicker({ msg }) {
  const [text, setText] = useState('');
  const [disabled, setDisabled] = useState(false);
  const inputRef = useRef(null);

  useEffect(() => { inputRef.current?.focus(); }, []);

  function submit(skip = false) {
    if (disabled) return;
    const raw = skip ? '' : text.trim();
    const pages = raw
      ? raw.split(',').map(s => s.trim()).filter(Boolean).slice(0, 10)
      : [];
    setDisabled(true);
    const s = useStore.getState();
    if (pages.length > 0) s.addUser('Other pages: ' + pages.join(', '));
    else s.addUser('Skip — just design the first page.');
    s.removePrePagePicker();
    startDesign(msg.content, 'auto', pages);
  }

  function onKey(e) {
    if (e.key === 'Enter') { e.preventDefault(); submit(false); }
  }

  return (
    <div className="msg assistant">
      <span className="msg-label">Assistant</span>
      <div className="bubble">
        <div style={{ lineHeight: 1.55, marginBottom: 12 }}>
          Before I design <em>{msg.content}</em> — are there other pages this app will have?
          I'll wire them into the sidebar as real links so we can build each one after.
        </div>
        <input
          ref={inputRef}
          className="pre-page-input"
          type="text"
          maxLength={200}
          placeholder="e.g. Workouts, Calendar, Profile"
          value={text}
          onChange={e => setText(e.target.value)}
          onKeyDown={onKey}
          disabled={disabled}
        />
        <div style={{ display: 'flex', gap: 8, marginTop: 10 }}>
          <button className="theme-chip pre-page-continue" style={{ fontWeight: 600 }} onClick={() => submit(false)} disabled={disabled}>Continue</button>
          <button className="theme-chip pre-page-skip" onClick={() => submit(true)} disabled={disabled}>Skip</button>
        </div>
        <div className="theme-hint" style={{ marginTop: 10 }}>
          Optional. Skip and I'll design just the first page.
        </div>
      </div>
    </div>
  );
}

export function startDesign(idea, theme, initialPages) {
  const s = useStore.getState();
  s.setPendingIdea(null);
  s.removeKitPicker();
  s.removePrePagePicker();
  s.removeModeClarify();
  s.setStatus('busy', 'Generating…');
  const payload = { idea, layouts: 'auto', theme };
  if (Array.isArray(initialPages) && initialPages.length > 0) payload.initial_pages = initialPages;
  // Remember the payload — if the backend classifies it as Ambiguous, the
  // ModeClarifyPicker replays this after the user picks.
  s.setLastStartDesign(payload);
  ipcSend('start_design', JSON.stringify(payload));
}
