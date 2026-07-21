import React from 'react';
import Topbar from './components/shell/Topbar.jsx';
import ProjectsList from './components/Sidebar/ProjectsList.jsx';
import ChatPanel from './components/ChatPanel/ChatPanel.jsx';
import Canvas from './components/Canvas/Canvas.jsx';
import Modal from './components/ChatPanel/Modal.jsx';
import { useIpcEvents } from './hooks/useIpcEvents.js';

export default function App() {
  useIpcEvents();
  return (
    <>
      <Topbar />
      <div className="main">
        <ProjectsList />
        <ChatPanel />
        <Canvas />
      </div>
      <Modal />
    </>
  );
}
