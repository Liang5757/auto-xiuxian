import AutoSaveNumber from '../AutoSaveNumber';
import styles from './index.module.scss';
interface Props {
  value: number;
  disabled: boolean;
  onSave(interval: number): Promise<boolean>;
}
export default function PollingInterval({ value, disabled, onSave }: Props) {
  return (
    <label className={styles.field}>
      检测间隔{' '}
      <AutoSaveNumber
        value={value}
        min={1}
        max={86400}
        disabled={disabled}
        label="检测间隔"
        onSave={onSave}
      />{' '}
      s
    </label>
  );
}
