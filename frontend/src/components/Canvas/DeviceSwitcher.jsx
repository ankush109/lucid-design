import React from 'react';

const DEVICES = [
  { label: 'Desktop', width: '100%' },
  { label: 'Tablet',  width: '768px' },
  { label: 'Mobile',  width: '390px' },
];

export default function DeviceSwitcher({ device, setDevice }) {
  return (
    <div className="tb-group">
      <span className="tb-label">Device</span>
      <div className="segmented">
        {DEVICES.map(d => (
          <button
            key={d.label}
            className={`device-btn${device === d.width ? ' active' : ''}`}
            onClick={() => setDevice(d.width)}
          >
            {d.label}
          </button>
        ))}
      </div>
    </div>
  );
}
