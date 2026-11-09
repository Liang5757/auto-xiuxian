import AutoSaveNumber from '../AutoSaveNumber';
import type { Command, DesktopState } from '../../types';
import styles from './index.module.scss';

interface Props {
  state: DesktopState;
  disabled: boolean;
  perform(command: Command): Promise<boolean>;
}
export default function ClickerPage({ state, disabled, perform }: Props) {
  const config = state.clicker;
  async function configure(update: Partial<Pick<typeof config, 'interval' | 'right' | 'key'>>) {
    const next = { interval: config.interval, right: config.right, key: config.key, ...update };
    return perform({ type: 'configure_clicker', ...next });
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
            <AutoSaveNumber
              value={config.interval}
              min={30}
              max={60000}
              disabled={disabled || config.enabled}
              label="点击间隔"
              onSave={(interval) => configure({ interval })}
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
      </fieldset>
      <p className={styles.note}>自动派遣操作期间，连点会暂时暂停。</p>
    </section>
  );
}
