import { useState } from 'react';
import Sidebar from '../Sidebar';
import StatusCard from '../StatusCard';
import DispatchPage from '../DispatchPage';
import ClickerPage from '../ClickerPage';
import { useEngineState } from '../../useEngineState';
import type { Command, Page } from '../../types';
import styles from './index.module.scss';

export default function App() {
  const { state, ready } = useEngineState();
  const [page, setPage] = useState<Page>('dispatch');
  const [pending, setPending] = useState(false);
  const [error, setError] = useState('');
  const dispatch = page === 'dispatch';
  const on = dispatch ? state.dispatchRunning : state.clicker.enabled;
  const disabled = !ready || pending || state.closing || Boolean(state.backendError);

  async function perform(command: Command) {
    if (!window.assistant) return false;
    setPending(true);
    setError('');
    try {
      const result = await window.assistant.command(command);
      if (!result.ok) setError(result.error || '操作失败');
      return result.ok;
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
      return false;
    } finally {
      setPending(false);
    }
  }
  return (
    <div className={styles.app}>
      <Sidebar page={page} onNavigate={setPage} connected={state.gameConnected} />
      <main className={styles.main}>
        <header className={styles.header}>
          <div>
            <h1>{dispatch ? '自动派遣' : '连点器'}</h1>
            <p>{dispatch ? '小队归来后，重新派遣并返回原地图' : '为游戏窗口设置连续点击'}</p>
          </div>
          <button onClick={() => window.assistant?.background()}>后台运行</button>
        </header>
        {(error || state.backendError) && (
          <p className={styles.error} role="alert">
            {state.backendError || error}
          </p>
        )}
        <StatusCard
          on={on}
          status={dispatch ? state.dispatchStatus : state.clicker.status}
          detail={dispatch ? '检查间隔 30 秒' : '仅在鼠标指向游戏时点击'}
          disabled={disabled || (dispatch && state.dispatchStopping)}
          onToggle={() =>
            void perform(
              dispatch
                ? { type: on ? 'stop_dispatch' : 'start_dispatch' }
                : { type: 'set_clicker', enabled: !on },
            )
          }
        />
        {dispatch ? (
          <DispatchPage state={state} />
        ) : (
          <ClickerPage state={state} disabled={disabled} perform={perform} />
        )}
      </main>
    </div>
  );
}
