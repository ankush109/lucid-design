import React, { useEffect, useRef, useState } from 'react';

// Imperative modal: openModal(opts) returns a Promise, mirroring ui.html.
// The <Modal /> component installs a global handler at App level.

let _open = null;

export function openModal(opts) {
  return new Promise((resolve) => {
    if (_open) _open(opts, resolve);
    else resolve(null);
  });
}

export default function Modal() {
  const [opts, setOpts] = useState(null);
  const [value, setValue] = useState('');
  const resolverRef = useRef(null);
  const inputRef = useRef(null);
  const okRef = useRef(null);

  useEffect(() => {
    _open = (o, resolve) => {
      resolverRef.current = resolve;
      setOpts(o || {});
      setValue(o?.initial || '');
      setTimeout(() => {
        if (o?.mode === 'confirm') okRef.current?.focus();
        else { inputRef.current?.focus(); inputRef.current?.select(); }
      }, 0);
    };
    return () => { _open = null; };
  }, []);

  useEffect(() => {
    function onKey(e) {
      if (!opts) return;
      if (e.key === 'Escape') { e.preventDefault(); close(null); }
      else if (e.key === 'Enter') { e.preventDefault(); submit(); }
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [opts, value]);

  function close(v) {
    if (resolverRef.current) { resolverRef.current(v); resolverRef.current = null; }
    setOpts(null);
    setValue('');
  }
  function submit() {
    if (!opts) return;
    if (opts.mode === 'confirm') close(true);
    else close((value || '').trim() || null);
  }
  function onBackdrop(e) {
    if (e.target === e.currentTarget) close(null);
  }

  return (
    <div className={`modal-backdrop ${opts ? 'open' : ''}`} onMouseDown={onBackdrop}>
      {opts && (
        <div className="modal" onMouseDown={e => e.stopPropagation()}>
          <div className="m-eyebrow">{opts.eyebrow || 'Prompt'}</div>
          <h3 className="serif" dangerouslySetInnerHTML={{ __html: opts.title || 'Enter <em>value</em>.' }} />
          {opts.desc && <p dangerouslySetInnerHTML={{ __html: opts.desc }} />}
          {opts.mode !== 'confirm' && (
            <input
              ref={inputRef}
              type="text"
              value={value}
              onChange={e => setValue(e.target.value)}
              placeholder={opts.placeholder || ''}
              autoComplete="off"
            />
          )}
          <div className="modal-actions">
            <button className="modal-btn" onClick={() => close(null)}>Cancel</button>
            <button
              ref={okRef}
              className={`modal-btn ${opts.danger ? 'danger' : 'primary'}`}
              onClick={submit}
            >
              {opts.okText || 'OK'}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
