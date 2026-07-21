import React from 'react';
import { useStore } from '../../store.js';

const LABELS = {
  landing:   'LANDING',
  app:       'APP',
  ambiguous: '?',
};
const TITLES = {
  landing:   'Landing / marketing page mode',
  app:       'App / dashboard mode',
  ambiguous: 'Mode not yet decided',
};

export default function ModeBadge() {
  const mode = useStore(s => s.session.mode) || 'ambiguous';
  const label = LABELS[mode] || '?';
  return (
    <div className={`mode-badge mode-${mode}`} title={TITLES[mode] || ''}>
      <span className="dot" />
      <span>{label}</span>
    </div>
  );
}
