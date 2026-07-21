import React from 'react';
import { useStore } from '../../store.js';

export default function MetaChip() {
  const meta = useStore(s => s.meta);
  return (
    <div className="meta-chip" title="Active provider · model">
      <span className="provider">{meta.provider}</span>
      <span className="sep">·</span>
      <span className="model">{meta.model}</span>
    </div>
  );
}
