import React from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';
import { openModal } from '../ChatPanel/Modal.jsx';

function relTime(secs) {
  if (!secs) return 'just now';
  const now = Math.floor(Date.now() / 1000);
  const d = now - secs;
  if (d < 60) return 'just now';
  if (d < 3600) return Math.floor(d/60) + 'm ago';
  if (d < 86400) return Math.floor(d/3600) + 'h ago';
  if (d < 604800) return Math.floor(d/86400) + 'd ago';
  return new Date(secs * 1000).toLocaleDateString();
}
function formatSize(b) {
  if (!b) return 'empty';
  if (b < 1024) return b + ' b';
  if (b < 1024 * 1024) return (b/1024).toFixed(1) + ' kb';
  return (b/1024/1024).toFixed(1) + ' mb';
}

export default function ProjectsList() {
  const projects       = useStore(s => s.projects);
  const currentProject = useStore(s => s.session.currentProject);
  const status         = useStore(s => s.status);
  const isProcessing   = status.state === 'busy';

  async function createProject() {
    const name = await openModal({
      eyebrow: 'New project',
      title:   'Name this <em>project</em>.',
      desc:    'A file will be created in <span class="mono">~/Documents/lucid-design/</span>.',
      placeholder: 'Untitled',
      initial: '',
      okText:  'Create',
    });
    if (name === null) return;
    ipcSend('create_project', name || 'Untitled');
  }

  function openProject(slug) {
    if (isProcessing) return;
    ipcSend('open_project', slug);
  }

  async function deleteProject(e, p) {
    e.stopPropagation();
    const ok = await openModal({
      mode: 'confirm',
      eyebrow: 'Delete project',
      title:   `Delete <em>${escapeHtml(p.name)}</em>?`,
      desc:    'This removes the file from <span class="mono">~/Documents/lucid-design/</span>. Cannot be undone.',
      okText:  'Delete',
      danger:  true,
    });
    if (!ok) return;
    ipcSend('delete_project', p.slug);
    const st = useStore.getState();
    if (st.session.currentProject && st.session.currentProject.slug === p.slug) {
      st.setCurrentProject(null);
      st.resetCanvas();
      st.resetMessages();
    }
  }

  return (
    <div className="projects-rail">
      <div className="rail-head">
        <span className="eyebrow">Projects</span>
        <span className="count">{String(projects.length).padStart(2, '0')}</span>
      </div>
      <div className="rail-list">
        {projects.length === 0 ? (
          <div className="rail-empty">
            No projects yet. Start one below — it saves to{' '}
            <span className="mono" style={{ fontSize: 11 }}>~/Documents/lucid-design/</span>.
          </div>
        ) : projects.map((p, i) => {
          const num = String(i + 1).padStart(2, '0');
          const isActive = currentProject && currentProject.slug === p.slug;
          return (
            <div
              key={p.slug}
              className={`rail-item${isActive ? ' active' : ''}`}
              onClick={() => openProject(p.slug)}
            >
              <span className="num">{num}</span>
              <div className="body">
                <div className="name">{p.name}</div>
                <div className="meta">{relTime(p.updated_at)} · {formatSize(p.size)}</div>
              </div>
              <button className="del" onClick={(e) => deleteProject(e, p)} title="Delete">×</button>
            </div>
          );
        })}
      </div>
      <button className="rail-new" onClick={createProject}>
        <span>+ New project</span>
      </button>
    </div>
  );
}

function escapeHtml(s) {
  return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}
