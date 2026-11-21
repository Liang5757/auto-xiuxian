import { useEffect, useState } from 'react';
import type { SpeedState } from '../../types';
import StatusCard from '../StatusCard';
import styles from './index.module.scss';

export default function SpeedPage({ state }: { state?: SpeedState }) {
  const [draft, setDraft] = useState(String(state?.factor ?? 2));
  const [pending, setPending] = useState(false);
  const [error, setError] = useState('');
  useEffect(() => {
    if (state) setDraft(String(state.factor));
  }, [state?.factor]);
  const factor = Number(draft);
  const valid =
    draft !== '' &&
    Number.isFinite(factor) &&
    factor >= 0.1 &&
    factor <= 10 &&
    Math.abs(factor * 10 - Math.round(factor * 10)) < 1e-8;
  async function perform(action: () => Promise<{ ok: boolean; error?: string }>) {
    setPending(true);
    setError('');
    try {
      const result = await action();
      if (!result.ok) setError(result.error || '变速操作失败');
      return result.ok;
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
      return false;
    } finally {
      setPending(false);
    }
  }
  function apply(enabled: boolean, value = factor) {
    if (!window.assistant) return;
    void perform(() => window.assistant!.speed({ enabled, factor: value }));
  }
  return (
    <>
      <StatusCard
        on={state?.enabled ?? false}
        status={state?.status ?? '连接中'}
        detail="关闭变速后游戏恢复原速"
        disabled={pending || !state || (!state.enabled && !valid)}
        onToggle={() => apply(!state?.enabled, state?.enabled ? state.factor : factor)}
      />
      <section className={styles.settings}>
        <header>
          <h2>变速设置</h2>
          <span>内置变速</span>
        </header>
        <fieldset disabled={pending || !state}>
          <label className={styles.row}>
            <span>游戏倍率</span>
            <input
              aria-label="游戏倍率"
              type="number"
              min="0.1"
              max="10"
              step="0.1"
              value={draft}
              aria-invalid={!valid}
              onChange={(event) => setDraft(event.target.value)}
            />
            <span>倍</span>
            <button disabled={!valid} onClick={() => apply(state?.enabled ?? false)}>
              应用倍率
            </button>
          </label>
          <div className={styles.choices} aria-label="常用倍率">
            {[1, 2, 3, 5, 10].map((value) => (
              <button
                key={value}
                aria-pressed={factor === value}
                onClick={() => setDraft(String(value))}
              >
                {value}×
              </button>
            ))}
            <button
              onClick={() => {
                setDraft('1');
                apply(false, 1);
              }}
            >
              恢复原速
            </button>
          </div>
        </fieldset>
        {error && (
          <p className={styles.error} role="alert">
            {error}
          </p>
        )}
        <p className={styles.note}>
          支持 0.1～10
          倍，最多一位小数。保存倍率后点击开关启用；重启助手或游戏后需手动开启。仅保留一个游戏窗口，避免同时使用其他变速工具。正常退出助手会恢复原速，隐藏到托盘会继续变速。
        </p>
      </section>
    </>
  );
}
