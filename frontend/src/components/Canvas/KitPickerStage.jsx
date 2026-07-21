import React from 'react';
import { useStore } from '../../store.js';
import { ipcSend } from '../../ipc.js';
import { KIT_THEMES, KIT_PALETTES, KIT_VARIANTS } from '../../kits.js';

const STEPS = [
  { cat: 'theme',        label: 'Theme',        prompt: 'Which tone package should carry the design?',       options: KIT_THEMES },
  { cat: 'palette',      label: 'Palette',      prompt: 'Which colour palette?',                              options: KIT_PALETTES },
  { cat: 'navbar',       label: 'Navbar',       prompt: 'How should the navigation feel?',                    options: KIT_VARIANTS.navbar },
  { cat: 'hero',         label: 'Hero',         prompt: 'What shape should the hero take?',                   options: KIT_VARIANTS.hero },
  { cat: 'features',     label: 'Features',     prompt: 'How should the feature section be laid out?',        options: KIT_VARIANTS.features },
  { cat: 'testimonials', label: 'Testimonials', prompt: 'How should social proof be shown?',                  options: KIT_VARIANTS.testimonials },
  { cat: 'pricing',      label: 'Pricing',      prompt: 'What pricing structure?',                            options: KIT_VARIANTS.pricing },
  { cat: 'cta',          label: 'CTA',          prompt: 'What kind of closing call to action?',               options: KIT_VARIANTS.cta },
  { cat: 'footer',       label: 'Footer',       prompt: 'How should the footer close things out?',            options: KIT_VARIANTS.footer },
];

function labelFor(cat, id) {
  const step = STEPS.find(s => s.cat === cat);
  const opt  = step && step.options.find(o => o.id === id);
  return opt ? opt.label : id;
}

export default function KitPickerStage() {
  const kitPicker = useStore(s => s.kitPicker);
  if (!kitPicker) return null;

  const { idea, step, picks } = kitPicker;
  const isLast = step >= STEPS.length - 1;
  const current = STEPS[step];

  function choose(id) {
    const s = useStore.getState();
    const nextPicks = { ...s.kitPicker.picks, [current.cat]: id };
    if (isLast) {
      commit(nextPicks);
    } else {
      s.updateKitPicker({ picks: nextPicks, step: step + 1 });
    }
  }

  function goTo(index) {
    const s = useStore.getState();
    if (index < 0 || index >= STEPS.length) return;
    s.updateKitPicker({ step: index });
  }

  function commit(finalPicks) {
    const s = useStore.getState();
    const summary = Object.entries(finalPicks)
      .filter(([, v]) => v && v !== 'auto')
      .map(([k, v]) => `${k}=${v}`).join(', ');
    s.addUser(summary ? `Build with: ${summary}` : 'Build with all Auto');
    s.closeKitPicker();
    s.setPendingIdea(null);
    s.setStatus('busy', 'Assembling…');
    s.setGenerating(true);
    ipcSend('assemble_design', JSON.stringify({
      idea,
      theme:        finalPicks.theme        || 'auto',
      palette:      finalPicks.palette      || 'auto',
      navbar:       finalPicks.navbar       || 'auto',
      hero:         finalPicks.hero         || 'auto',
      features:     finalPicks.features     || 'auto',
      testimonials: finalPicks.testimonials || 'auto',
      pricing:      finalPicks.pricing      || 'auto',
      cta:          finalPicks.cta          || 'auto',
      footer:       finalPicks.footer       || 'auto',
    }));
  }

  const answered = step;   // count of prior answered rows
  const total    = STEPS.length;
  const pct      = Math.round((answered / total) * 100);

  return (
    <div className="kit-stage">
      <div className="kit-stage-inner">
        {/* Header: eyebrow + brief + progress */}
        <div className="kit-stage-head">
          <div className="eyebrow accent">Assistant · Kit</div>
          <h1 className="kit-stage-idea">Designing <em>{idea}</em></h1>
          <div className="kit-stage-progress">
            <div className="kit-stage-progress-bar">
              <div className="fill" style={{ width: `${pct}%` }} />
            </div>
            <div className="kit-stage-progress-meta mono">
              <span>Step {step + 1} of {total}</span>
              <span className="dot">·</span>
              <span>{current.label}</span>
            </div>
          </div>
        </div>

        {/* Question + options */}
        <div className="kit-stage-body">
          <h2 className="kit-stage-question">{current.prompt}</h2>
          <div className="kit-stage-options">
            {current.options.map((o) => {
              const isAuto  = o.id === 'auto';
              const isChosen = picks[current.cat] === o.id;
              return (
                <button
                  key={o.id}
                  className={`kit-option${isAuto ? ' auto' : ''}${isChosen ? ' chosen' : ''}`}
                  onClick={() => choose(o.id)}
                >
                  <span className="kit-option-label">{o.label}</span>
                  {isAuto && <span className="kit-option-hint">Let me pick</span>}
                </button>
              );
            })}
          </div>
        </div>

        {/* Summary + back */}
        <div className="kit-stage-foot">
          <button
            className="kit-nav-btn"
            onClick={() => goTo(step - 1)}
            disabled={step === 0}
            title="Back one step"
          >← Back</button>

          {answered > 0 && (
            <div className="kit-summary" role="list">
              {STEPS.slice(0, answered).map((s, i) => {
                const id = picks[s.cat];
                if (!id) return null;
                return (
                  <button
                    key={s.cat}
                    className={`kit-summary-chip${id === 'auto' ? ' auto' : ''}`}
                    onClick={() => goTo(i)}
                    title="Change this pick"
                    role="listitem"
                  >
                    <span className="kit-summary-cat">{s.label}</span>
                    <span className="kit-summary-val">{labelFor(s.cat, id)}</span>
                  </button>
                );
              })}
            </div>
          )}

          <button
            className="kit-nav-btn cancel"
            onClick={() => { const s = useStore.getState(); s.closeKitPicker(); s.setPendingIdea(null); s.setStatus('ready','Ready'); }}
            title="Cancel and drop the kit"
          >Cancel</button>
        </div>

        <div className="kit-stage-footnote">
          Nothing spends tokens until every step is answered. Pick <b>Auto</b> to
          skip a decision — I'll choose based on your idea and theme.
        </div>
      </div>
    </div>
  );
}
