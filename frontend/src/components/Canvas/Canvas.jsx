import React, { useEffect, useRef, useState } from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';
import { injectEditSupport, buildPrototypeHTML, stripEditArtifacts } from './injectEditSupport.js';
import DeviceSwitcher from './DeviceSwitcher.jsx';
import PageTabs from './PageTabs.jsx';
import Inspector from './Inspector.jsx';
import KitPickerStage from './KitPickerStage.jsx';

const MAX_HISTORY = 50;

export default function Canvas() {
  const currentHTML       = useStore(s => s.currentHTML);
  const assemblyPreview   = useStore(s => s.assemblyPreview);
  const editReady         = useStore(s => s.editReady);
  const currentTarget     = useStore(s => s.currentTarget);
  const historyResetToken = useStore(s => s.historyResetToken);
  const kitPicker         = useStore(s => s.kitPicker);
  const status            = useStore(s => s.status);
  const pageTabs          = useStore(s => s.session.tabs);
  const activePageSlug    = useStore(s => s.session.currentPage);
  const canvasViewMode    = useStore(s => s.canvasViewMode);

  // Metadata about the currently-viewed page (built / has_skeleton flags).
  const activePageInfo = React.useMemo(
    () => (pageTabs || []).find(p => p.slug === activePageSlug) || null,
    [pageTabs, activePageSlug]
  );
  const pageMode = (activePageInfo && canvasViewMode[activePageInfo.slug])
    || (activePageInfo && activePageInfo.built === false && activePageInfo.has_skeleton ? 'skeleton' : 'built');
  const showViewToggle = !!activePageInfo && activePageInfo.has_skeleton && activePageInfo.built === true;
  const showBuildCTA   = !!activePageInfo && activePageInfo.has_skeleton && activePageInfo.built === false && pageMode === 'skeleton';

  function setViewMode(mode) {
    if (!activePageInfo) return;
    const s = useStore.getState();
    s.setCanvasViewMode(activePageInfo.slug, mode);
    if (mode === 'skeleton') {
      ipcSend('get_page_skeleton', activePageInfo.slug);
    } else if (mode === 'built') {
      // Ask backend to switch back to built HTML.
      ipcSend('switch_page', activePageInfo.slug);
    }
  }
  function buildThisPage() {
    if (!activePageInfo) return;
    if (activePageInfo.slug === 'home') return;
    const s = useStore.getState();
    s.addUser(`Build the ${activePageInfo.name} page from its wireframe`);
    s.setStatus('busy', `Building ${activePageInfo.name}…`);
    ipcSend('build_page_from_skeleton', activePageInfo.slug);
  }

  const frameRef = useRef(null);
  const historyRef = useRef([]);   // edit history stack
  const redoRef    = useRef([]);
  const debounceRef = useRef(null);
  const lastAppliedRef = useRef(''); // to skip loop when we programmatically set srcdoc

  const [device, setDevice] = useState('100%');
  const [inspectorOpen, setInspectorOpen] = useState(false);
  const [playMode, setPlayMode] = useState(false);
  const [autoPlay, setAutoPlay] = useState(false);
  const playTimerRef = useRef(null);
  const [scenes, setScenes] = useState([]);
  const [sceneIdx, setSceneIdx] = useState(0);

  // ── srcdoc updates: prefer assemblyPreview if present, else currentHTML.
  useEffect(() => {
    const frame = frameRef.current;
    if (!frame) return;
    const html = assemblyPreview
      ? assemblyPreview
      : (currentHTML ? injectEditSupport(currentHTML) : '');
    if (!html) return;
    if (html === lastAppliedRef.current) return;
    lastAppliedRef.current = html;

    // For finished designs → scroll to top. For streaming previews → scroll
    // so the LATEST section sits at the bottom of the viewport (feels like
    // "the design is being built here"). Targeting the last section's
    // bounding rect avoids the failure mode where scrollHeight jumps past
    // the just-added content into empty margin/padding.
    const isPreview = !!assemblyPreview;
    frame.onload = () => {
      const doc = frame.contentDocument;
      if (!doc) return;
      const scroller = doc.scrollingElement || doc.documentElement || doc.body;
      if (!scroller) return;

      if (!isPreview) {
        scroller.scrollTop = 0;
        return;
      }

      // Preview: find the last body-level child that has real height, then
      // scroll so its bottom sits at the viewport bottom (with a small
      // pad). Wait one frame for layout so animations/fonts don't throw off
      // the getBoundingClientRect measurement.
      requestAnimationFrame(() => {
        const kids = doc.body?.children || [];
        let last = null;
        for (let i = kids.length - 1; i >= 0; i--) {
          const r = kids[i].getBoundingClientRect();
          if (r.height > 8) { last = kids[i]; break; }
        }
        if (last && typeof last.scrollIntoView === 'function') {
          // 'end' lines up the section's bottom with the viewport bottom —
          // the user sees the section that just landed, not empty space
          // beyond it.
          last.scrollIntoView({ block: 'end', inline: 'nearest', behavior: 'auto' });
        } else {
          // Fallback: original behavior. Bounded to actual content height so
          // we can't scroll past whatever's rendered so far.
          const contentBottom = doc.body ? doc.body.getBoundingClientRect().bottom + scroller.scrollTop : scroller.scrollHeight;
          scroller.scrollTop = Math.max(0, contentBottom - scroller.clientHeight + 24);
        }
      });
    };
    frame.srcdoc = html;
  }, [currentHTML, assemblyPreview]);

  // ── Reset history only when a new backend-issued design lands. Keyed on
  // historyResetToken (bumped in ipc.js on design/project_opened) so user
  // edits — which change currentHTML — don't wipe the undo stack.
  useEffect(() => {
    const seed = useStore.getState().currentHTML;
    historyRef.current = seed ? [seed] : [];
    redoRef.current = [];
  }, [historyResetToken]);

  // ── Iframe → parent messages
  useEffect(() => {
    function onMsg(e) {
      if (!e.data) return;
      const s = useStore.getState();
      if (e.data.type === 'html_edited') {
        // Strip injection artifacts before storing, so persisted HTML and
        // history snapshots stay clean.
        const clean = stripEditArtifacts(e.data.html);
        s.setCurrentHTML(clean);
        if (debounceRef.current) clearTimeout(debounceRef.current);
        debounceRef.current = setTimeout(() => {
          const h = historyRef.current;
          if (!h.length || h[h.length - 1] !== clean) {
            h.push(clean);
            if (h.length > MAX_HISTORY) h.shift();
            redoRef.current = [];
          }
        }, 400);
      } else if (e.data.type === 'selection') {
        if (e.data.selector) {
          s.setCurrentTarget({ selector: e.data.selector, tag: e.data.tag });
        } else {
          s.setCurrentTarget(null);
        }
      } else if (e.data.type === 'edit_ready') {
        s.setEditReady(true);
      } else if (e.data.type === 'canvas_undo_request') {
        if (e.data.redo) redoCanvas(); else undoCanvas();
      }
    }
    window.addEventListener('message', onMsg);
    return () => window.removeEventListener('message', onMsg);
  }, []);

  // ── Parent-level Cmd+Z / ⇧⌘Z
  useEffect(() => {
    function onKey(e) {
      if (!(e.metaKey || e.ctrlKey)) return;
      const k = (e.key || '').toLowerCase();
      if (k !== 'z') return;
      const ae = document.activeElement;
      if (ae && (ae.tagName === 'INPUT' || ae.tagName === 'TEXTAREA' || ae.isContentEditable)) return;
      if (ae && ae.tagName === 'IFRAME') return;
      e.preventDefault();
      if (e.shiftKey) redoCanvas(); else undoCanvas();
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, []);

  function applySnapshot(html) {
    if (!html) return;
    useStore.getState().setCurrentHTML(html);
    if (frameRef.current) {
      lastAppliedRef.current = injectEditSupport(html);
      frameRef.current.srcdoc = lastAppliedRef.current;
    }
    ipcSend('sync_design', html);
  }
  function undoCanvas() {
    if (debounceRef.current) { clearTimeout(debounceRef.current); debounceRef.current = null; }
    const h = historyRef.current;
    if (h.length < 2) return;
    const cur = h.pop();
    redoRef.current.push(cur);
    if (redoRef.current.length > MAX_HISTORY) redoRef.current.shift();
    applySnapshot(h[h.length - 1]);
  }
  function redoCanvas() {
    const r = redoRef.current;
    if (!r.length) return;
    const next = r.pop();
    historyRef.current.push(next);
    if (historyRef.current.length > MAX_HISTORY) historyRef.current.shift();
    applySnapshot(next);
  }

  // ── Toolbar handlers
  function tryDifferentLayout() {
    const s = useStore.getState();
    if (s.status.state === 'busy') return;
    s.setStatus('busy', 'New layout…');
    s.addUser('Try a different layout');
    ipcSend('try_different_layout', '');
  }
  function toggleInspector() { setInspectorOpen(o => !o); }
  function savePattern() { ipcSend('save_design', ''); }
  function exportHTML() { if (currentHTML) ipcSend('export', currentHTML); }
  function exportPrototype() { if (currentHTML) ipcSend('export_prototype', buildPrototypeHTML(currentHTML)); }

  function togglePlay() {
    if (!currentHTML) return;
    if (playMode) exitPlay(); else enterPlay();
  }
  function enterPlay() {
    setPlayMode(true);
    const frame = frameRef.current;
    if (frame?.contentWindow) {
      try {
        frame.contentWindow.postMessage({ cmd: 'clearSelection' }, '*');
        frame.contentWindow.postMessage({ cmd: 'setPlayMode', value: true }, '*');
      } catch(_) {}
    }
    detectScenes();
  }
  function exitPlay() {
    setPlayMode(false);
    stopAuto();
    const frame = frameRef.current;
    if (frame?.contentWindow) {
      try { frame.contentWindow.postMessage({ cmd: 'setPlayMode', value: false }, '*'); } catch(_) {}
    }
  }
  function detectScenes() {
    try {
      const doc = frameRef.current?.contentDocument;
      if (!doc || !doc.body) { setScenes([]); return; }
      const kids = Array.from(doc.body.children).filter(el =>
        !['SCRIPT','STYLE','LINK','META','NOSCRIPT','IFRAME'].includes(el.tagName)
        && !(el.id || '').startsWith('__ov')
        && el.getBoundingClientRect().height > 60
      );
      const list = kids.map(el => {
        const heading = el.querySelector('h1, h2, h3');
        const idBase = (el.id || '').trim();
        const label = heading && heading.textContent.trim()
          ? heading.textContent.trim().replace(/\s+/g, ' ').slice(0, 32)
          : (idBase ? idBase.replace(/[-_]/g, ' ').replace(/\b\w/g, c => c.toUpperCase()) : el.tagName.toLowerCase());
        return { el, label };
      });
      setScenes(list);
      setSceneIdx(0);
      if (list.length && list[0].el) list[0].el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    } catch(_) { setScenes([]); }
  }
  function jumpTo(i) {
    if (i < 0 || i >= scenes.length) return;
    setSceneIdx(i);
    const s = scenes[i];
    if (s?.el?.scrollIntoView) s.el.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
  function nextScene() { if (scenes.length) jumpTo((sceneIdx + 1) % scenes.length); }
  function prevScene() { if (scenes.length) jumpTo((sceneIdx - 1 + scenes.length) % scenes.length); }
  function toggleAutoPlay() {
    if (autoPlay) stopAuto(); else startAuto();
  }
  function startAuto() {
    if (!scenes.length) return;
    setAutoPlay(true);
    playTimerRef.current = setInterval(() => {
      setSceneIdx(i => {
        const next = scenes.length ? (i + 1) % scenes.length : 0;
        const s = scenes[next];
        if (s?.el?.scrollIntoView) s.el.scrollIntoView({ behavior: 'smooth', block: 'start' });
        return next;
      });
    }, 3200);
  }
  function stopAuto() {
    if (playTimerRef.current) { clearInterval(playTimerRef.current); playTimerRef.current = null; }
    setAutoPlay(false);
  }
  useEffect(() => () => stopAuto(), []);

  const hasDesign = !!currentHTML;

  return (
    <>
      <div className="design-pane">
        <div className="design-toolbar">
          <div className="tb-group">
            <span className="tb-label">Preview</span>
          </div>
          {editReady && hasDesign && (
            <div className="canvas-badge">
              <span className="lb-dot" /><span>Live canvas</span>
            </div>
          )}
          <div className="toolbar-spacer" />

          {/* View-mode toggle when the current page has both a skeleton
              and a built version. Shown to the LEFT of the device switcher
              so it's discoverable. */}
          {showViewToggle && (
            <div className="canvas-view-toggle" role="tablist" aria-label="Preview mode">
              <button
                role="tab"
                className={pageMode === 'skeleton' ? 'active' : ''}
                onClick={() => setViewMode('skeleton')}
              >Wireframe</button>
              <button
                role="tab"
                className={pageMode === 'built' ? 'active' : ''}
                onClick={() => setViewMode('built')}
              >Built</button>
            </div>
          )}

          {/* Prominent CTA when we're viewing a wireframe page that hasn't
              been built yet. Kicks off build_page_from_skeleton. */}
          {showBuildCTA && (
            <button
              className="build-page-cta"
              onClick={buildThisPage}
              disabled={status?.state === 'busy'}
              title="Upgrade this wireframe to a full-fidelity page"
            >Build this page →</button>
          )}

          <DeviceSwitcher device={device} setDevice={setDevice} />
          <div className="tb-sep" />
          {hasDesign && (
            <>
              <button className="tb-btn" onClick={tryDifferentLayout}>↻ Try different layout</button>
              <button className={`tb-btn${inspectorOpen ? ' active' : ''}`} onClick={toggleInspector}>▤ Inspector</button>
              <button className={`tb-btn${playMode ? ' active' : ''}`} onClick={togglePlay}>{playMode ? '✕ Exit' : '▶ Play'}</button>
              <button className="tb-btn" onClick={savePattern}>Save pattern</button>
              <button className="tb-btn" onClick={exportPrototype}>Prototype</button>
              <button className="tb-btn" onClick={exportHTML}>↓ Export HTML</button>
            </>
          )}
        </div>

        <PageTabs />

        <div className={`preview-area${playMode ? ' play-on' : ''}`}>
          {kitPicker && !hasDesign && !assemblyPreview && (
            <KitPickerStage />
          )}
          {!kitPicker && !hasDesign && !assemblyPreview && (
            <div className="empty-state">
              <div className="empty-mark">✦</div>
              <div className="empty-eyebrow">Empty canvas</div>
              <h2>Describe it in <em>chat</em>.</h2>
              <p>Three variants stream in here. Pick one and keep refining — everything else follows from that first message.</p>
            </div>
          )}
          <div className="preview-shell" style={{
            display: (hasDesign || assemblyPreview) ? 'flex' : 'none',
            maxWidth: device,
          }}>
            <div className="browser-bar">
              <div className="browser-dots">
                <div className="b-dot red" /><div className="b-dot yellow" /><div className="b-dot green" />
              </div>
              <div className="browser-url">yourapp.com</div>
            </div>
            <iframe
              ref={frameRef}
              className="preview-frame"
              sandbox="allow-same-origin allow-scripts"
            />
          </div>
        </div>

        {playMode && (
          <div className="play-bar on">
            <div className="play-title">
              <span>Playing</span>
              <span className="now">{scenes[sceneIdx]?.label || '—'}</span>
            </div>
            <div className="scenes">
              {scenes.length ? scenes.map((s, i) => (
                <div key={i} className={`scene-chip${i === sceneIdx ? ' active' : ''}`} onClick={() => jumpTo(i)}>
                  <span className="idx">{String(i + 1).padStart(2, '0')}</span>
                  <span>{s.label}</span>
                </div>
              )) : (
                <span style={{ color: 'rgba(255,255,255,0.5)', fontSize: 11, fontStyle: 'italic' }}>
                  No sections detected — try adding id="hero", id="features", etc.
                </span>
              )}
            </div>
            <span className="play-count">
              {scenes.length ? `${sceneIdx + 1} / ${scenes.length}` : '0 / 0'}
            </span>
            <div className="play-controls">
              <button className="play-btn" onClick={prevScene} title="Previous section">‹</button>
              <button className="play-btn primary" onClick={toggleAutoPlay} title="Auto-play">{autoPlay ? '❚❚' : '▶'}</button>
              <button className="play-btn" onClick={nextScene} title="Next section">›</button>
            </div>
            <button className="play-btn exit" onClick={exitPlay} title="Exit play mode">✕</button>
          </div>
        )}
      </div>

      <Inspector open={inspectorOpen} onClose={() => setInspectorOpen(false)} frameRef={frameRef} />
    </>
  );
}
