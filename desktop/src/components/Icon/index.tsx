type IconName = 'dispatch' | 'farm' | 'clicker' | 'speed' | 'power';

export default function Icon({ name }: { name: IconName }) {
  const paths = {
    dispatch: (
      <>
        <path d="m21 3-7 18-4-7-7-4 18-7Z" />
        <path d="m10 14 5-5" />
      </>
    ),
    farm: (
      <>
        <path d="M12 21v-9M12 16C5 16 3 12 3 7c6 0 9 3 9 9ZM12 12c0-6 3-9 9-9 0 6-3 9-9 9Z" />
      </>
    ),
    clicker: (
      <>
        <rect x="6" y="2" width="12" height="20" rx="6" />
        <path d="M6 10h12M12 2v8M9 6h1" />
      </>
    ),
    speed: (
      <>
        <path d="M4.2 18a9 9 0 1 1 15.6 0M12 13l4-5M5 11h1M18 11h1M12 4v1" />
        <circle cx="12" cy="13" r="1" />
      </>
    ),
    power: (
      <>
        <path d="M6.34 6.34a8 8 0 1 0 11.32 0M12 3v9" />
      </>
    ),
  };
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {paths[name]}
    </svg>
  );
}
