import { useEffect, useState } from 'react';
import type { Command, DesktopState } from '../../types';
import styles from './index.module.scss';

interface Props {
  state: DesktopState;
  disabled: boolean;
  perform(command: Command): Promise<boolean>;
}
export default function ClickerPage({ state, disabled, perform }: Props) {
  const config = state.clicker;
  const [interval, setInterval] = useState(String(config.interval));
  const [error, setError] = useState('');
  useEffect(() => setInterval(String(config.interval)), [config.interval]);
  async function configure(update: Partial<Pick<typeof config, 'interval' | 'right' | 'key'>>) {
    const next = { interval: config.interval, right: config.right, key: config.key, ...update };
    if (!Number.isInteger(next.interval) || next.interval < 30 || next.interval > 60000) {
      setError('点击间隔应为 30～60000 毫秒');
      return;
    }
    setError('');
    await perform({ type: 'configure_clicker', ...next });
  }
  return (
    <section className={styles.settings}>
      <header>
        <h2>点击设置</h2>
        <span>已点击 {config.count} 次</span>
      </header>
      <fieldset disabled={disabled || config.enabled}>
        <label className={styles.row}>
          <span>点击间隔</span>
          <div className={styles.interval}>
            <input
              type="number"
              min="30"
              max="60000"
              step="10"
              value={interval}
              aria-label="点击间隔"
              onChange={(event) => setInterval(event.target.value)}
              onBlur={() => {
                if (Number(interval) !== config.interval)
                  void configure({ interval: Number(interval) });
              }}
              onKeyDown={(event) => {
                if (event.key === 'Enter') event.currentTarget.blur();
              }}
            />
            <span>毫秒</span>
          </div>
        </label>
        <div className={styles.row}>
          <span>鼠标按键</span>
          <div className={styles.choices}>
            {[false, true].map((right) => (
              <button
                key={String(right)}
                aria-pressed={config.right === right}
                onClick={() => void configure({ right })}
              >
                {right ? '右键' : '左键'}
              </button>
            ))}
          </div>
        </div>
        <div className={styles.row}>
          <span>启停热键</span>
          <div className={styles.choices}>
            {[117, 118, 120].map((key) => (
              <button
                key={key}
                aria-pressed={config.key === key}
                onClick={() => void configure({ key })}
              >
                F{key - 111}
              </button>
            ))}
          </div>
        </div>
        {error && (
          <p className={styles.error} role="alert">
            {error}
          </p>
        )}
      </fieldset>
      <p className={styles.note}>自动派遣操作期间，连点会暂时暂停。</p>
    </section>
  );
}
