import type { DesktopState } from '../../types';
import styles from './index.module.scss';

export default function DispatchPage({
  state,
}: {
  state: Pick<DesktopState, 'logs' | 'map' | 'success' | 'failure'>;
}) {
  return (
    <>
      <div className={styles.metrics}>
        <section className={styles.metric}>
          <span>任务成功</span>
          <i className={styles.success} aria-hidden="true">
            ✓
          </i>
          <strong>{state.success}</strong>
        </section>
        <section className={styles.metric}>
          <span>任务失败</span>
          <i className={styles.failure} aria-hidden="true">
            !
          </i>
          <strong>{state.failure}</strong>
        </section>
        <section className={styles.metric}>
          <span>当前地图</span>
          <i className={styles.map} aria-hidden="true">
            ⌖
          </i>
          <strong className={styles.mapValue}>{state.map}</strong>
        </section>
      </div>
      <section className={styles.activity} aria-label="运行记录">
        <header>
          <h2>运行记录</h2>
          <span>
            <kbd>Esc</kbd> 停止派遣和收种
          </span>
        </header>
        {state.logs.length === 0 ? (
          <div className={styles.empty}>
            <div className={styles.clock} aria-hidden="true">
              ◷
            </div>
            <p>暂无运行记录</p>
          </div>
        ) : (
          <ol className={styles.logs}>
            {state.logs.map((log) => (
              <li key={log.id}>
                <time>{log.time}</time>
                <span>{log.message}</span>
              </li>
            ))}
          </ol>
        )}
      </section>
    </>
  );
}
