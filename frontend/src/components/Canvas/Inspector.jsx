import React, { useEffect, useRef, useState } from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';

// ── Inspector: Style + Layers tabs, driven by the selected element in the
// preview iframe (data-ov-sel). Wires all inputs generically.

function cssRgbToHex(str) {
  if (!str) return '#000000';
  if (str.startsWith('#')) return str;
  const m = String(str).match(/rgba?\((\d+)\s*,\s*(\d+)\s*,\s*(\d+)/);
  if (!m) return '#000000';
  return '#' + [+m[1], +m[2], +m[3]].map(x => x.toString(16).padStart(2, '0')).join('');
}

function esc(s) {
  return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

export default function Inspector({ open, onClose, frameRef }) {
  const currentTarget = useStore(s => s.currentTarget);
  const [tab, setTab] = useState('style');
  const bodyRef = useRef(null);
  const treeRef = useRef(null);
  const [tick, setTick] = useState(0); // force re-populate

  // Whenever the target changes or panel opens, populate widgets from the iframe.
  useEffect(() => {
    if (!open) return;
    populate();
    if (tab === 'layers') renderTree();
  }, [open, tab, currentTarget, tick]);

  // Listen for iframe edit events → re-render.
  useEffect(() => {
    function onMsg(e) {
      if (!e.data) return;
      if (e.data.type === 'html_edited' || e.data.type === 'selection') {
        setTick(t => t + 1);
      }
    }
    window.addEventListener('message', onMsg);
    return () => window.removeEventListener('message', onMsg);
  }, []);

  function frameDoc() {
    const f = frameRef.current;
    return f && f.contentDocument;
  }

  function selectedEl() {
    const doc = frameDoc();
    return doc && doc.querySelector('[data-ov-sel]');
  }

  function populate() {
    const el = selectedEl();
    const body = bodyRef.current;
    if (!body) return;
    const sections = body.querySelectorAll('[data-ins-section]');
    const emptyMsg = body.querySelector('.ins-empty');
    if (!el) {
      sections.forEach(s => s.style.display = 'none');
      if (emptyMsg) emptyMsg.style.display = '';
      return;
    }
    sections.forEach(s => s.style.display = '');
    if (emptyMsg) emptyMsg.style.display = 'none';
    const cs = el.ownerDocument.defaultView.getComputedStyle(el);

    setColor('ins-bg-color',     'ins-bg-text',     cs.backgroundColor);
    setColor('ins-text-color',   'ins-text-text',   cs.color);
    setColor('ins-border-color', 'ins-border-text', cs.borderColor);

    setRange('ins-radius',    parseFloat(cs.borderRadius) || 0, v => Math.round(v) + 'px');
    setRange('ins-opacity',   Math.round((parseFloat(cs.opacity) || 1) * 100), v => v + '%');
    setRange('ins-font-size', parseFloat(cs.fontSize) || 16, v => Math.round(v) + 'px');
    setRange('ins-letter-sp', Math.round((parseFloat(cs.letterSpacing) || 0) * 10), v => (v/10).toFixed(1) + 'px');
    const lh = parseFloat(cs.lineHeight) / (parseFloat(cs.fontSize) || 16);
    setRange('ins-line-h', Math.round((isFinite(lh) ? lh : 1.5) * 100), v => (v/100).toFixed(2));
    setRange('ins-padding',   parseFloat(cs.paddingTop) || 0, v => Math.round(v) + 'px');
    setRange('ins-margin',    parseFloat(cs.marginTop)  || 0, v => Math.round(v) + 'px');
    setRange('ins-gap',       parseFloat(cs.gap)        || 0, v => Math.round(v) + 'px');
    setRange('ins-border',    parseFloat(cs.borderTopWidth) || 0, v => Math.round(v) + 'px');

    const weight = cs.fontWeight;
    const weightSel = body.querySelector('#ins-font-weight');
    if (weightSel) {
      const opts = Array.from(weightSel.options).map(o => o.value);
      weightSel.value = opts.includes(String(weight)) ? String(weight) : '400';
    }
    const shadowSel = body.querySelector('#ins-shadow');
    if (shadowSel) shadowSel.value = '';
  }

  function setColor(colorId, textId, cssColor) {
    const hex = cssRgbToHex(cssColor);
    const body = bodyRef.current; if (!body) return;
    const c = body.querySelector('#' + colorId); if (c) c.value = hex;
    const t = body.querySelector('#' + textId); if (t) t.value = hex;
  }
  function setRange(id, value, fmt) {
    const body = bodyRef.current; if (!body) return;
    const el = body.querySelector('#' + id);
    if (el) el.value = value;
    const valEl = body.querySelector('#' + id + '-val');
    if (valEl) valEl.textContent = fmt ? fmt(value) : String(value);
  }

  function applyInput(input) {
    const doc = frameDoc(); if (!doc) return;
    const el = doc.querySelector('[data-ov-sel]'); if (!el) return;

    const prop = input.dataset.prop;
    const unit = input.dataset.unit || '';
    const transform = input.dataset.transform || '';
    let raw = input.value;

    // Mirror color text ↔ swatch.
    if (input.type === 'text' && /color$/i.test(prop)) {
      const swId = input.id.replace('-text', '-color');
      const sw = bodyRef.current?.querySelector('#' + swId);
      if (sw && /^#[0-9a-f]{6}$/i.test(raw)) sw.value = raw;
    }
    if (input.type === 'color') {
      const textId = input.id.replace('-color', '-text');
      const tx = bodyRef.current?.querySelector('#' + textId);
      if (tx) tx.value = raw;
    }

    let value = raw;
    if (transform === 'divide100') value = String(parseFloat(raw) / 100);
    if (transform === 'tenth')     value = String(parseFloat(raw) / 10);
    if (unit) value = value + unit;

    if (prop === 'border-width' && parseFloat(raw) > 0) {
      el.style.setProperty('border-style', 'solid');
    }
    el.style.setProperty(prop, value);

    const valEl = bodyRef.current?.querySelector('#' + input.id + '-val');
    if (valEl) {
      if (prop === 'opacity')       valEl.textContent = Math.round(parseFloat(value) * 100) + '%';
      else if (prop === 'line-height') valEl.textContent = parseFloat(value).toFixed(2);
      else if (prop === 'letter-spacing') valEl.textContent = parseFloat(value).toFixed(1) + 'px';
      else                          valEl.textContent = value;
    }

    syncFromIframe();
  }

  function syncFromIframe() {
    const doc = frameDoc(); if (!doc || !doc.documentElement) return;
    const html = '<!DOCTYPE html>' + doc.documentElement.outerHTML;
    useStore.getState().setCurrentHTML(html);
    ipcSend('sync_design', html);
  }

  function onInput(e) { const t = e.target; if (t && t.dataset && t.dataset.prop) applyInput(t); }

  function renderTree() {
    const tree = treeRef.current; if (!tree) return;
    const doc = frameDoc();
    if (!doc || !doc.body) {
      tree.innerHTML = '<div class="ins-empty">No design loaded yet.</div>';
      return;
    }
    tree.innerHTML = '';
    buildTreeNode(doc.body, tree, 0, doc);
  }
  function buildTreeNode(el, parent, depth, doc) {
    if (!el || el.nodeType !== 1) return;
    if (['SCRIPT','STYLE','LINK','META','NOSCRIPT','IFRAME'].includes(el.tagName)) return;
    if ((el.id || '').startsWith('__ov') || el.id === '__edit_support' || el.id === '__ov_style') return;

    const node = document.createElement('div');
    node.className = 'ins-node';
    if (el.hasAttribute && el.hasAttribute('data-ov-sel')) node.classList.add('sel');
    node.style.paddingLeft = (4 + depth * 12) + 'px';

    const kids = Array.from(el.children || []).filter(k =>
      !['SCRIPT','STYLE','LINK','META','NOSCRIPT','IFRAME'].includes(k.tagName)
      && !(k.id || '').startsWith('__ov'));

    const tag = el.tagName.toLowerCase();
    const idStr = el.id ? '<span class="id">#' + esc(el.id) + '</span>' : '';
    node.innerHTML =
      '<span class="disc">' + (kids.length ? '▸' : '·') + '</span>' +
      '<span class="tag">' + tag + '</span>' + idStr;
    node.addEventListener('click', (e) => {
      e.stopPropagation();
      doc.querySelectorAll('[data-ov-sel]').forEach(n => n.removeAttribute('data-ov-sel'));
      el.setAttribute('data-ov-sel', '');
      useStore.getState().setCurrentTarget({ selector: (el.id ? '#' + el.id : tag), tag });
      el.scrollIntoView({ block: 'center', behavior: 'smooth' });
      setTick(t => t + 1);
    });
    parent.appendChild(node);
    for (const c of kids) buildTreeNode(c, parent, depth + 1, doc);
  }

  const label = currentTarget?.selector || 'select an element…';

  return (
    <div className={`inspector${open ? ' on' : ''}`}>
      <div className="ins-head">
        <div className="ins-title-wrap">
          <span className="ins-title">Inspector</span>
          <span className="ins-selector">{label}</span>
        </div>
        <button className="ins-close" onClick={onClose} title="Close">×</button>
      </div>

      <div className="ins-tabs">
        <button className={`ins-tab${tab === 'style' ? ' active' : ''}`} onClick={() => setTab('style')}>Style</button>
        <button className={`ins-tab${tab === 'layers' ? ' active' : ''}`} onClick={() => setTab('layers')}>Layers</button>
      </div>

      {tab === 'style' && (
        <div className="ins-body" ref={bodyRef} onInput={onInput} onChange={onInput}>
          <div className="ins-empty">Click any element on the canvas to edit its colour, spacing, radius, and typography here.</div>

          <div className="ins-group" data-ins-section style={{ display: 'none' }}>
            <h5>Appearance</h5>
            <div className="ins-row"><span>Background</span>
              <div className="ins-color-combo">
                <input type="color" id="ins-bg-color" data-prop="background-color" />
                <input type="text"  id="ins-bg-text"  data-prop="background-color" placeholder="#f5efe4" />
              </div>
            </div>
            <div className="ins-row"><span>Text color</span>
              <div className="ins-color-combo">
                <input type="color" id="ins-text-color" data-prop="color" />
                <input type="text"  id="ins-text-text"  data-prop="color" placeholder="#1c1a17" />
              </div>
            </div>
            <div className="ins-row"><span>Radius</span>
              <input type="range" id="ins-radius" min="0" max="60" data-prop="border-radius" data-unit="px" />
              <span className="val" id="ins-radius-val">—</span>
            </div>
            <div className="ins-row"><span>Opacity</span>
              <input type="range" id="ins-opacity" min="0" max="100" data-prop="opacity" data-transform="divide100" />
              <span className="val" id="ins-opacity-val">—</span>
            </div>
          </div>

          <div className="ins-group" data-ins-section style={{ display: 'none' }}>
            <h5>Typography</h5>
            <div className="ins-row"><span>Size</span>
              <input type="range" id="ins-font-size" min="8" max="96" data-prop="font-size" data-unit="px" />
              <span className="val" id="ins-font-size-val">—</span>
            </div>
            <div className="ins-row"><span>Weight</span>
              <select id="ins-font-weight" data-prop="font-weight">
                <option value="300">300 · Light</option>
                <option value="400">400 · Regular</option>
                <option value="500">500 · Medium</option>
                <option value="600">600 · Semibold</option>
                <option value="700">700 · Bold</option>
                <option value="800">800 · ExtraBold</option>
              </select>
            </div>
            <div className="ins-row"><span>Letter-sp</span>
              <input type="range" id="ins-letter-sp" min="-40" max="40" data-prop="letter-spacing" data-unit="px" data-transform="tenth" />
              <span className="val" id="ins-letter-sp-val">—</span>
            </div>
            <div className="ins-row"><span>Line height</span>
              <input type="range" id="ins-line-h" min="80" max="220" data-prop="line-height" data-transform="divide100" />
              <span className="val" id="ins-line-h-val">—</span>
            </div>
          </div>

          <div className="ins-group" data-ins-section style={{ display: 'none' }}>
            <h5>Spacing</h5>
            <div className="ins-row"><span>Padding</span>
              <input type="range" id="ins-padding" min="0" max="80" data-prop="padding" data-unit="px" />
              <span className="val" id="ins-padding-val">—</span>
            </div>
            <div className="ins-row"><span>Margin</span>
              <input type="range" id="ins-margin" min="0" max="80" data-prop="margin" data-unit="px" />
              <span className="val" id="ins-margin-val">—</span>
            </div>
            <div className="ins-row"><span>Gap</span>
              <input type="range" id="ins-gap" min="0" max="80" data-prop="gap" data-unit="px" />
              <span className="val" id="ins-gap-val">—</span>
            </div>
          </div>

          <div className="ins-group" data-ins-section style={{ display: 'none' }}>
            <h5>Border</h5>
            <div className="ins-row"><span>Border</span>
              <input type="range" id="ins-border" min="0" max="12" data-prop="border-width" data-unit="px" />
              <span className="val" id="ins-border-val">—</span>
            </div>
            <div className="ins-row"><span>Border color</span>
              <div className="ins-color-combo">
                <input type="color" id="ins-border-color" data-prop="border-color" />
                <input type="text"  id="ins-border-text"  data-prop="border-color" placeholder="#d9ccb2" />
              </div>
            </div>
            <div className="ins-row"><span>Shadow</span>
              <select id="ins-shadow" data-prop="box-shadow">
                <option value="">— none —</option>
                <option value="0 1px 2px rgba(0,0,0,0.05)">Micro</option>
                <option value="0 2px 6px rgba(0,0,0,0.08)">Soft</option>
                <option value="0 4px 12px rgba(0,0,0,0.10)">Elevated</option>
                <option value="0 8px 24px rgba(0,0,0,0.14)">Floating</option>
                <option value="0 16px 40px rgba(0,0,0,0.18)">Dramatic</option>
              </select>
            </div>
          </div>
        </div>
      )}

      {tab === 'layers' && (
        <div className="ins-body">
          <div className="ins-tree" ref={treeRef}>
            <div className="ins-empty">No design loaded yet.</div>
          </div>
        </div>
      )}
    </div>
  );
}
