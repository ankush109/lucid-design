import React, { useState } from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';

export default function CritiqueFixes({ msg }) {
  const items = Array.isArray(msg.content) ? msg.content : [];
  const [applied, setApplied] = useState(new Set());

  function applyFix(idx) {
    const st = useStore.getState();
    if (st.status.state === 'busy') return;
    const fix = items[idx];
    if (!fix || applied.has(idx)) return;
    st.addUser(`Apply: ${fix.label}`);
    st.setStatus('busy', 'Applying fix…');
    if (st.currentHTML) ipcSend('sync_design', st.currentHTML);
    ipcSend('user_message', fix.prompt);
    const next = new Set(applied); next.add(idx); setApplied(next);
  }

  return (
    <div className="msg assistant">
      <span className="msg-label">Critique</span>
      <div className="bubble">
        <div style={{ lineHeight: 1.55 }}>A few things I'd tighten. Click one to apply it:</div>
        <div className="theme-picker">
          {items.map((f, i) => (
            <button
              key={i}
              className="theme-chip"
              style={applied.has(i) ? { opacity: 0.5, pointerEvents: 'none' } : undefined}
              onClick={(e) => { e.preventDefault(); applyFix(i); }}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
