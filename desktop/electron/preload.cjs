const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('assistant', {
  getState: () => ipcRenderer.invoke('assistant:get-state'),
  command: (command) => ipcRenderer.invoke('assistant:command', command),
  background: () => ipcRenderer.send('assistant:background'),
  speed: (request) => ipcRenderer.invoke('assistant:speed', request),
  onState: (listener) => {
    const callback = (_event, state) => listener(state);
    ipcRenderer.on('assistant:state', callback);
    return () => ipcRenderer.removeListener('assistant:state', callback);
  },
});
