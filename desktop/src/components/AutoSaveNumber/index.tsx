import { useEffect, useRef, useState } from 'react';
interface Props {
  value: number;
  min: number;
  max: number;
  label: string;
  disabled: boolean;
  onSave(value: number): Promise<boolean>;
}
export default function AutoSaveNumber({ value, min, max, label, disabled, onSave }: Props) {
  const [draft, setDraft] = useState(String(value));
  const latest = useRef({
    draft: String(value),
    value,
    disabled,
    onSave,
    attempted: '',
    timer: undefined as ReturnType<typeof setTimeout> | undefined,
  });
  latest.current.disabled = disabled;
  latest.current.onSave = onSave;
  function save() {
    const current = latest.current;
    clearTimeout(current.timer);
    const number = Number(current.draft);
    if (
      current.disabled ||
      !current.draft ||
      !Number.isInteger(number) ||
      number < min ||
      number > max ||
      number === current.value ||
      current.attempted === current.draft
    )
      return;
    current.attempted = current.draft;
    void current.onSave(number);
  }
  useEffect(() => {
    const current = latest.current;
    if (current.draft === String(current.value) || Number(current.draft) === value) {
      current.draft = String(value);
      setDraft(String(value));
    }
    current.value = value;
  }, [value]);
  useEffect(
    () => () => {
      clearTimeout(latest.current.timer);
      save();
    },
    [],
  );
  const valid =
    draft !== '' && Number.isInteger(Number(draft)) && Number(draft) >= min && Number(draft) <= max;
  return (
    <input
      type="number"
      min={min}
      max={max}
      step={1}
      required
      value={draft}
      disabled={disabled}
      aria-label={label}
      aria-invalid={!valid}
      title={`请输入 ${min}～${max} 的整数`}
      onChange={(event) => {
        const current = latest.current;
        current.draft = event.target.value;
        current.attempted = '';
        setDraft(current.draft);
        clearTimeout(current.timer);
        current.timer = setTimeout(save, 400);
      }}
      onBlur={save}
      onKeyDown={(event) => {
        if (event.key === 'Enter') save();
      }}
    />
  );
}
