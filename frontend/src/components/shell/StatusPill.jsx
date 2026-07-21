import React from 'react';
import { useStore } from '../../store.js';

export default function StatusPill() {
  const status = useStore(s => s.status);
  return (
    <div className={`status-pill ${status.state}`}>
      <span className="dot" />
      <span>{status.text}</span>
    </div>
  );
}
