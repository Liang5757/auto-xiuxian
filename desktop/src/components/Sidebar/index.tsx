import logo from '../../../assets/app.png';
import type { Page } from '../../types';
import styles from './index.module.scss';

interface Props {
  page: Page;
  connected: boolean;
  onNavigate(page: Page): void;
}
export default function Sidebar({ page, connected, onNavigate }: Props) {
  return (
    <aside className={styles.sidebar}>
      <div className={styles.brand}>
        <img src={logo} alt="" />
        <strong>修仙助手</strong>
      </div>
      <nav aria-label="功能">
        <button
          className={page === 'dispatch' ? styles.active : ''}
          onClick={() => onNavigate('dispatch')}
          aria-current={page === 'dispatch' ? 'page' : undefined}
        >
          <span aria-hidden="true" className={styles.plane}>
            ➤
          </span>
          自动派遣
        </button>
        <button
          className={page === 'clicker' ? styles.active : ''}
          onClick={() => onNavigate('clicker')}
          aria-current={page === 'clicker' ? 'page' : undefined}
        >
          <span aria-hidden="true" className={styles.cursor}>
            ↖
          </span>
          连点器
        </button>
      </nav>
      <div className={styles.connection}>
        <p>
          <i data-connected={connected} />
          {connected ? '游戏已连接' : '等待游戏启动'}
        </p>
        <span>懒人修仙传</span>
      </div>
    </aside>
  );
}
