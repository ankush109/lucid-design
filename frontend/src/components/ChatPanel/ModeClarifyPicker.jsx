import React from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';

// Shown when the backend classifies an idea as Ambiguous. User picks
// Landing or App; we send set_mode, then re-fire the queued start_design
// payload (React held it in lastStartDesignPayload) so generation resumes
// without them having to retype the idea.
export default function ModeClarifyPicker({ msg }) {
  const brief = msg.content || '';

  function choose(mode) {
    const s = useStore.getState();
    // Send set_mode first so Rust's current_mode is updated before the
    // subsequent start_design runs.
    ipcSend('set_mode', mode);
    s.setMode(mode);
    s.removeModeClarify();

    const payload = s.lastStartDesignPayload;
    if (payload) {
      s.setStatus('busy', 'Generating…');
      s.setGenerating(true);
      ipcSend('start_design', JSON.stringify(payload));
    }
  }

  return (
    <div className="msg assistant">
      <span className="msg-label">Assistant</span>
      <div className="bubble">
        <div style={{ lineHeight: 1.55, marginBottom: 10 }}>
          I couldn't tell whether <em>{brief}</em> is a landing page or a full
          app / dashboard. Pick one so I load the right design patterns.
        </div>
        <div className="theme-picker" style={{ display: 'flex', gap: 8 }}>
          <button className="theme-chip" style={{ fontWeight: 600 }} onClick={() => choose('landing')}>
            Landing page
          </button>
          <button className="theme-chip" style={{ fontWeight: 600 }} onClick={() => choose('app')}>
            App / dashboard
          </button>
        </div>
        <div className="theme-hint" style={{ marginTop: 10 }}>
          The choice sticks per project — future ideas skip this step.
        </div>
      </div>
    </div>
  );
}
