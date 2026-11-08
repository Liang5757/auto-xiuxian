import styles from './index.module.scss';

interface Props {
  checked: boolean;
  disabled?: boolean;
  label: string;
  onChange(): void;
}
export default function Toggle({ checked, disabled, label, onChange }: Props) {
  return (
    <button
      className={styles.toggle}
      type="button"
      role="switch"
      aria-label={label}
      aria-checked={checked}
      disabled={disabled}
      onClick={onChange}
      data-on={checked}
    >
      <span />
    </button>
  );
}
