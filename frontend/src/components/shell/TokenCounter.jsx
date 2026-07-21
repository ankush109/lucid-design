import React from 'react';
import { useStore } from '../../store.js';

function fmtTokens(n) {
  n = Number(n) || 0;
  if (n < 1000) return String(n);
  if (n < 10000) return (n/1000).toFixed(2).replace(/0$/,'') + 'k';
  if (n < 1000000) return (n/1000).toFixed(1).replace(/\.0$/,'') + 'k';
  return (n/1000000).toFixed(2) + 'm';
}

export default function TokenCounter() {
  const t = useStore(s => s.tokens);
  const turnTotal = t.turnIn + t.turnOut;
  const sessTotal = t.sessionIn + t.sessionOut;
  const hover =
    `Design:  ${t.turnIn} in / ${t.turnOut} out` +
    `\nSession: ${t.sessionIn} in / ${t.sessionOut} out` +
    (t.estimated ? '\n(estimated from chars)' : '\n(reported by provider)');
  return (
    <div className={`tokens${t.estimated ? ' est' : ''}`} title={hover}>
      <div className="grp turn">
        <span className="lb">Design</span>
        <b>{fmtTokens(turnTotal)}</b>
      </div>
      <span className="sep">·</span>
      <div className="grp total">
        <span className="lb">Session</span>
        <b>{fmtTokens(sessTotal)}</b>
      </div>
    </div>
  );
}
