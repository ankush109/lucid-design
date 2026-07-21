import React from 'react';

// Simple bubble for text/status. Special kinds (kit_picker, pre_page_picker,
// critique, page_suggestions, progress) render via dedicated components.

export default function Message({ msg }) {
  if (msg.kind === 'status_ephemeral') {
    return (
      <div className="msg status">
        <div className="bubble"><div className="spinner" />{msg.content}</div>
      </div>
    );
  }
  if (msg.kind === 'progress') {
    const { kb = 0, sections = [] } = msg.content || {};
    return (
      <div className="msg progress-wrap">
        <div className="progress-head">
          <div className="spinner" />Streaming design <span className="kb">· {kb} kb</span>
        </div>
        <div className="progress-list">
          {sections.length === 0 ? (
            <div className="sec-item active">
              <span className="num">00</span>
              <span className="glyph"><span className="dot-pulse" /></span>
              <span>Writing HTML structure…</span>
            </div>
          ) : sections.map((s, i) => {
            const done = i < sections.length - 1;
            const active = i === sections.length - 1;
            const num = String(i + 1).padStart(2, '0');
            return (
              <div key={s.id} className={`sec-item${done ? ' done' : active ? ' active' : ''}`}>
                <span className="num">{num}</span>
                <span className="glyph">
                  {done ? '✓' : (active ? <span className="dot-pulse" /> : '○')}
                </span>
                <span>{s.label}</span>
              </div>
            );
          })}
        </div>
      </div>
    );
  }
  // text / default
  return (
    <div className={`msg ${msg.role}`}>
      <span className="msg-label">{msg.label}</span>
      <div className="bubble">{msg.content}</div>
    </div>
  );
}
