import { useEffect } from 'react';
import { installIpcBridge, installChatMirror, ipcSend } from '../ipc.js';

// Installed once at App root. Wires the __onEvent dispatcher + chat mirror,
// then kicks the initial `list_projects` request identical to legacy ui.html.
export function useIpcEvents() {
  useEffect(() => {
    installIpcBridge();
    installChatMirror();
    ipcSend('list_projects', '');
  }, []);
}
