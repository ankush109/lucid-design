import React, { useState } from 'react';
import { KIT_THEMES, KIT_PALETTES, KIT_VARIANTS } from '../../kits.js';

// Stepped kit picker: one question at a time. Previous picks stay visible as
// a compact summary above; the current row shows chip options; remaining
// rows are hidden until the user advances. When the last row is answered,
// the composer's Send fires assemble_design (reads picks from msg.meta).

const STEPS = [
  { cat: 'theme',        label: 'Theme',        options: KIT_THEMES },
  { cat: 'palette',      label: 'Palette',      options: KIT_PALETTES },
  { cat: 'navbar',       label: 'Navbar',       options: KIT_VARIANTS.navbar },
  { cat: 'hero',         label: 'Hero',         options: KIT_VARIANTS.hero },
  { cat: 'features',     label: 'Features',     options: KIT_VARIANTS.features },
  { cat: 'testimonials', label: 'Testimonials', options: KIT_VARIANTS.testimonials },
  { cat: 'pricing',      label: 'Pricing',      options: KIT_VARIANTS.pricing },
  { cat: 'cta',          label: 'CTA',          options: KIT_VARIANTS.cta },
  { cat: 'footer',       label: 'Footer',       options: KIT_VARIANTS.footer },
];

function labelFor(cat, id) {
  const step = STEPS.find(s => s.cat === cat);
  const opt  = step && step.options.find(o => o.id === id);
  return opt ? opt.label : id;
}

export default function KitPicker({ msg }) {
  const [picks, setPicks] = useState(msg.meta?.picks || {});
  const [step,  setStep]  = useState(msg.meta?.step  ?? 0);

  React.useEffect(() => {
    if (!msg.meta) return;
    msg.meta.picks = picks;
    msg.meta.step  = step;
  }, [picks, step, msg]);

  const finished = step >= STEPS.length;
  const current  = finished ? null : STEPS[step];

  function choose(id) {
    if (!current) return;
    setPicks({ ...picks, [current.cat]: id });
    setStep(step + 1);
  }
  function back() {
    if (step === 0) return;
    setStep(step - 1);
  }

  return (
    <div className="msg assistant" data-kit-picker-id={msg.id}>
      <span className="msg-label">Assistant · Kit</span>
      <div className="bubble">
        <div style={{ lineHeight: 1.55, marginBottom: 12 }}>
          Pick the pieces for <em>{msg.content}</em> — one at a time. Leave any on <b>Auto</b>
          and I'll choose. When every row's answered, hit <b>Send ↵</b> to assemble.
        </div>

        {/* Summary of picks so far */}
        {step > 0 && (
          <div className="kit-summary" style={{
            display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 14,
            paddingBottom: 12, borderBottom: '1px solid var(--line-soft)',
          }}>
            {STEPS.slice(0, step).map(s => {
              const id = picks[s.cat] || 'auto';
              return (
                <button
                  key={s.cat}
                  onClick={(e) => { e.preventDefault(); setStep(STEPS.indexOf(s)); }}
                  className="theme-chip"
                  style={{ opacity: 0.85, fontSize: 11 }}
                  title="Click to change"
                >
                  <span style={{ opacity: 0.55, marginRight: 6 }}>{s.label}:</span>
                  <b>{labelFor(s.cat, id)}</b>
                </button>
              );
            })}
          </div>
        )}

        {/* Current question */}
        {current && (
          <div className="kit-row">
            <div className="kit-lbl">{current.label}</div>
            <div className="theme-picker">
              {current.options.map(o => (
                <button
                  key={o.id}
                  className="theme-chip kit-chip"
                  onClick={(e) => { e.preventDefault(); choose(o.id); }}
                  data-kit-cat={current.cat}
                  data-kit-id={o.id}
                >
                  {o.label}
                </button>
              ))}
            </div>
            <div style={{
              display: 'flex', justifyContent: 'space-between', alignItems: 'center',
              marginTop: 10, fontSize: 11, color: 'var(--muted)',
            }}>
              <span>Step {step + 1} of {STEPS.length}</span>
              {step > 0 && (
                <button
                  onClick={(e) => { e.preventDefault(); back(); }}
                  className="theme-chip"
                  style={{ fontSize: 11 }}
                >← Back</button>
              )}
            </div>
          </div>
        )}

        {finished && (
          <div style={{
            padding: '10px 12px', border: '1px dashed var(--accent)',
            borderRadius: 4, background: 'var(--accent-soft)',
            fontSize: 12, color: 'var(--ink-2)',
          }}>
            All set. Hit <b>Send ↵</b> to assemble.
          </div>
        )}

        <div className="theme-hint" style={{ marginTop: 12 }}>
          Nothing spends tokens until you press Send.
        </div>
      </div>
    </div>
  );
}
