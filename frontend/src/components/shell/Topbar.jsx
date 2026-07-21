import React, { useEffect, useState } from 'react';
import { useStore } from '../../store.js';
import StatusPill from './StatusPill.jsx';
import ModeBadge from './ModeBadge.jsx';
import TokenCounter from './TokenCounter.jsx';
import MetaChip from './MetaChip.jsx';

const THEME_KEY = 'lucid-design-theme';
const RAIL_KEY  = 'lucid-design-rail';

function applyTheme(t) { document.documentElement.setAttribute('data-theme', t); }
function applyRail(r)  { document.documentElement.setAttribute('data-rail', r); }

export default function Topbar() {
  const crumb = useStore(s => s.crumb);
  const [theme, setTheme] = useState(() => {
    try { return localStorage.getItem(THEME_KEY) || 'light'; } catch(_) { return 'light'; }
  });
  const [rail, setRail] = useState(() => {
    try { return localStorage.getItem(RAIL_KEY) || 'open'; } catch(_) { return 'open'; }
  });

  useEffect(() => { applyTheme(theme); try { localStorage.setItem(THEME_KEY, theme); } catch(_) {} }, [theme]);
  useEffect(() => { applyRail(rail);   try { localStorage.setItem(RAIL_KEY, rail); } catch(_) {} }, [rail]);

  const toggleTheme = () => setTheme(t => t === 'light' ? 'dark' : 'light');
  const toggleRail  = () => setRail(r => r === 'open' ? 'closed' : 'open');

  return (
    <div className="topbar">
      <div className="brand">
        <span className="name">lucid</span>
        <span className="tag">design</span>
      </div>
      <div className="crumb">
        <span>workspace</span><span className="arrow">→</span>
        <span className="cur">{crumb}</span>
      </div>
      <div className="topbar-spacer" />
      <TokenCounter />
      <MetaChip />
      <ModeBadge />
      <StatusPill />
      <button className="icon-btn" onClick={toggleRail} title="Toggle projects panel">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
          <rect x="3" y="4" width="18" height="16" rx="2" />
          <line x1="9" y1="4" x2="9" y2="20" />
        </svg>
      </button>
      <button className="icon-btn" onClick={toggleTheme} title="Toggle theme">
        {theme === 'dark' ? (
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
          </svg>
        ) : (
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
          </svg>
        )}
      </button>
    </div>
  );
}
