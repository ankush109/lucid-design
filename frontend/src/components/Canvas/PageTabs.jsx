import React from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';
import { openModal } from '../ChatPanel/Modal.jsx';

export default function PageTabs() {
  const tabs        = useStore(s => s.session.tabs);
  const active      = useStore(s => s.session.currentPage);
  const currentHTML = useStore(s => s.currentHTML);
  const status      = useStore(s => s.status);
  const isProcessing = status.state === 'busy';

  // Hide unless project has pages and there is a design shown.
  if (!tabs || tabs.length === 0 || !currentHTML) return null;

  function switchPage(slug) {
    if (isProcessing) return;
    if (!slug || slug === active) return;
    useStore.getState().setStatus('busy', 'Switching page…');
    ipcSend('switch_page', slug);
  }

  async function promptNewPage() {
    const s = useStore.getState();
    if (s.status.state === 'busy') return;
    if (!s.currentHTML) {
      s.addAssistant('Design the Home page first — new pages inherit its shell.');
      return;
    }
    const name = await openModal({
      eyebrow: 'New page',
      title:   'Design a <em>new page</em>.',
      desc:    'The nav, sidebar, and theme carry over from the home page.',
      placeholder: 'Settings',
      okText:  'Design page',
    });
    if (!name) return;
    s.addUser(`Design the ${name} page`);
    s.setStatus('busy', `Designing ${name}…`);
    ipcSend('create_page', JSON.stringify({ name, brief: '' }));
  }

  return (
    <div className="page-tabs">
      <div className="page-tabs-inner">
        {tabs.map(p => {
          // Un-built pages that have a skeleton get a dashed tab with a
          // small "wireframe" glyph so the user knows one click will
          // preview the wireframe.
          const isWire = p.built === false && p.has_skeleton === true;
          return (
            <button
              key={p.slug}
              className={`page-tab${p.slug === active ? ' active' : ''}${isWire ? ' wireframe' : ''}`}
              onClick={() => switchPage(p.slug)}
              title={isWire ? 'Wireframe — click to preview, then Build to upgrade' : ''}
            >
              {isWire && <span className="wire-dot" aria-hidden />}
              {p.name}
            </button>
          );
        })}
      </div>
      <button className="page-tab page-tab-new" onClick={promptNewPage} title="Design a new page in this project">+ page</button>
    </div>
  );
}
