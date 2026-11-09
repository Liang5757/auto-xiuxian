import type { ReactNode } from 'react';
import Toggle from '../Toggle';
import styles from './index.module.scss';

interface Props {
  on: boolean;
  status: string;
  detail: ReactNode;
  disabled: boolean;
  onToggle(): void;
}
export default function StatusCard({ on, status, detail, disabled, onToggle }: Props) {
  return (
    <section className={styles.card} aria-label="运行状态">
      <span className={styles.power} data-on={on} aria-hidden="true">
        ⏻
      </span>
      <div className={styles.description}>
        <h2 aria-live="polite">{status}</h2>
        <div className={styles.detail}>{detail}</div>
      </div>
      <span className={styles.label}>{on ? '已开启' : '已关闭'}</span>
      <Toggle checked={on} disabled={disabled} label="启停当前功能" onChange={onToggle} />
    </section>
  );
}
