import { useState } from 'react';
import PollingInterval from '../PollingInterval';
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
  const farming = page === 'farm';
  const taskPage = page !== 'clicker';
  const on = farming
    ? state.farm.running
    : taskPage
      ? state.dispatchRunning
      : state.clicker.enabled;
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
      <Sidebar
        page={page}
        onNavigate={setPage}
        connected={state.gameConnected}
        bothOn={state.farm.running && state.dispatchRunning}
        anyOn={state.farm.running || state.dispatchRunning}
        disabled={disabled || state.dispatchStopping || state.farm.stopping}
        toggleBoth={() =>
          void perform({
            type: state.farm.running || state.dispatchRunning ? 'stop_both' : 'start_both',
          })
        }
      />
      <main className={styles.main}>
        <header className={styles.header}>
          <div>
            <h1>{taskPage ? (farming ? '自动收种' : '自动派遣') : '连点器'}</h1>
            <p>
              {taskPage
                ? farming
                  ? '作物成熟后收获，补种高级灵谷并返回原地图'
                  : '小队归来后，重新派遣并返回原地图'
                : '为游戏窗口设置连续点击'}
            </p>
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
          status={
            farming ? state.farm.status : taskPage ? state.dispatchStatus : state.clicker.status
          }
          detail={
            taskPage ? (
              <PollingInterval
                key={page}
                value={farming ? state.farm.interval : state.dispatchInterval}
                disabled={disabled}
                onSave={(interval) => perform({ type: 'configure_polling', farming, interval })}
              />
            ) : (
              '仅在鼠标指向游戏时点击'
            )
          }
          disabled={
            disabled || (farming ? state.farm.stopping : taskPage && state.dispatchStopping)
          }
          onToggle={() =>
            void perform(
              taskPage
                ? {
                    type: farming
                      ? on
                        ? 'stop_farm'
                        : 'start_farm'
                      : on
                        ? 'stop_dispatch'
                        : 'start_dispatch',
                  }
                : { type: 'set_clicker', enabled: !on },
            )
          }
        />
        {taskPage ? (
          <DispatchPage state={farming ? state.farm : state} />
        ) : (
          <ClickerPage state={state} disabled={disabled} perform={perform} />
        )}
      </main>
    </div>
  );
}
