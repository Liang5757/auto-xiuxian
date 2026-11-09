const { app, BrowserWindow, Tray, Menu, ipcMain, nativeImage, dialog } = require('electron');
const path = require('node:path');
const { Settings } = require('./settings.cjs');
const { Engine } = require('./engine.cjs');
const { validate } = require('./commands.cjs');

let window;
let tray;
let engine;
let settings;
let quitting = false;
let state = { backendError: null };

function publish(update) {
  state = { ...update, backendError: state.backendError };
  if (window && !window.isDestroyed()) window.webContents.send('assistant:state', state);
}
function restore() {
  if (!window || window.isDestroyed()) return;
  window.show();
  if (window.isMinimized()) window.restore();
  window.focus();
}
async function quit() {
  if (quitting) return;
  quitting = true;
  try {
    if (settings) await settings.pending;
    if (engine) await engine.stop();
    tray?.destroy();
    app.exit(0);
  } catch (error) {
    dialog.showErrorBox('退出失败', error.message);
    quitting = false;
  }
}
if (!app.isPackaged) {
  app.setPath('userData', path.join(app.getPath('appData'), app.getName() + '-dev'));
}

if (!app.requestSingleInstanceLock()) app.quit();
else {
  app.on('second-instance', restore);
  app.on('before-quit', (event) => {
    event.preventDefault();
    void quit();
  });
  app.on('window-all-closed', () => {});
  app
    .whenReady()
    .then(async () => {
      app.setAppUserModelId('com.liang.xiuxian');
      const iconPath = path.join(app.getAppPath(), 'assets/app.ico');
      window = new BrowserWindow({
        width: 800,
        height: 570,
        minWidth: 760,
        minHeight: 540,
        title: '修仙助手',
        icon: iconPath,
        backgroundColor: '#f6f7fa',
        show: false,
        autoHideMenuBar: true,
        webPreferences: {
          preload: path.join(__dirname, 'preload.cjs'),
          contextIsolation: true,
          nodeIntegration: false,
          sandbox: true,
          backgroundThrottling: true,
        },
      });
      window.on('close', (event) => {
        if (!quitting && tray) {
          event.preventDefault();
          window.hide();
        }
      });
      window.on('minimize', () => {
        if (tray && !quitting) window.hide();
      });
      window.once('ready-to-show', () => window.show());
      window.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));
      window.webContents.on('will-navigate', (event) => event.preventDefault());
      window.webContents.on('before-input-event', (event, input) => {
        if (input.control && input.key.toLowerCase() === 'q') {
          event.preventDefault();
          void quit();
        }
      });

      const icon = nativeImage.createFromPath(iconPath);
      tray = new Tray(icon);
      tray.setToolTip('修仙助手 · 双击打开，右键退出');
      tray.setContextMenu(
        Menu.buildFromTemplate([
          { label: '显示窗口', click: restore },
          { type: 'separator' },
          { label: '退出', click: () => void quit() },
        ]),
      );
      tray.on('double-click', restore);

      settings = new Settings(path.join(app.getPath('userData'), 'settings.json'));
      await settings.load();
      engine = new Engine(app, publish, (error) => {
        state = {
          ...state,
          backendError: error.message,
          dispatchRunning: false,
          dispatchStopping: false,
          dispatchStatus: '后台进程不可用',
        };
        if (state.farm)
          state.farm = { ...state.farm, running: false, stopping: false, status: '后台进程不可用' };
        if (state.clicker) state.clicker = { ...state.clicker, enabled: false, status: '已停止' };
        if (window && !window.isDestroyed()) window.webContents.send('assistant:state', state);
      });
      for (const command of settings.commands) await engine.command(command);
      ipcMain.handle('assistant:get-state', () => state);
      ipcMain.handle('assistant:command', async (_event, command) => {
        try {
          if (state.backendError) throw new Error(state.backendError);
          await settings.apply(engine, validate(command));
          return { ok: true };
        } catch (error) {
          return { ok: false, error: error.message };
        }
      });
      ipcMain.on('assistant:background', () => {
        if (tray) window.hide();
      });

      if (process.env.VITE_DEV_SERVER_URL && !app.isPackaged) {
        void window.loadURL(process.env.VITE_DEV_SERVER_URL);
      } else void window.loadFile(path.join(app.getAppPath(), 'dist/index.html'));
    })
    .catch((error) => {
      dialog.showErrorBox('启动失败', error.message);
      app.exit(1);
    });
}
