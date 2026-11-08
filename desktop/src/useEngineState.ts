import { useEffect, useState } from 'react';
import type { DesktopState } from './types';

const initial: DesktopState = {
  dispatchRunning: false,
  dispatchStopping: false,
  dispatchStatus: '连接中',
  map: '待识别',
  success: 0,
  failure: 0,
  gameConnected: false,
  closing: false,
  logs: [],
  backendError: null,
  clicker: { enabled: false, count: 0, status: '已停止', interval: 100, right: false, key: 117 },
};

export function useEngineState() {
  const [state, setState] = useState(initial);
  const [ready, setReady] = useState(false);
  useEffect(() => {
    const api = window.assistant;
    if (!api) {
      setState((current) => ({
        ...current,
        backendError: '请通过 Electron 打开应用以连接自动化进程。',
      }));
      return;
    }
    let active = true;
    let received = false;
    const update = (next: Partial<DesktopState>) => {
      if (!active) return;
      setState((current) => ({ ...current, ...next }));
      if ('dispatchRunning' in next) setReady(true);
    };
    const unsubscribe = api.onState((next) => {
      received = true;
      update(next);
    });
    api
      .getState()
      .then((next) => {
        if (!received) update(next);
      })
      .catch((error) => {
        update({ backendError: String(error) });
      });
    return () => {
      active = false;
      unsubscribe();
    };
  }, []);
  return { state, ready };
}
