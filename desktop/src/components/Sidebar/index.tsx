import Toggle from '../Toggle';
import logo from '../../../assets/app.png';
import type { Page } from '../../types';
import styles from './index.module.scss';

interface Props {
  page: Page;
  connected: boolean;
  bothOn: boolean;
  anyOn: boolean;
  disabled: boolean;
  toggleBoth(): void;
  onNavigate(page: Page): void;
}
export default function Sidebar({
  page,
  connected,
  bothOn,
  anyOn,
  disabled,
  toggleBoth,
  onNavigate,
}: Props) {
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
          className={page === 'farm' ? styles.active : ''}
          onClick={() => onNavigate('farm')}
          aria-current={page === 'farm' ? 'page' : undefined}
        >
          <span aria-hidden="true" className={styles.crop}>
            ✿
          </span>
          自动收种
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
      <div className={styles.master}>
        <div>
          <strong>派遣与收种</strong>
          <span>{bothOn ? '已开启' : anyOn ? '部分开启' : '已关闭'}</span>
        </div>
        <Toggle
          checked={anyOn}
          disabled={disabled}
          label="派遣与收种总开关"
          onChange={toggleBoth}
        />
      </div>
    </aside>
  );
}
