/** Brand mark (docs/design/design-assets/logo/mark.svg), inline so the bar follows the theme. */
export function LogoMark({ size = 28 }: { size?: number }) {
  return (
    <svg viewBox="0 0 136 156" width={(size * 136) / 156} height={size} aria-hidden className="logo-mark">
      <circle cx="68" cy="61" r="52" fill="none" stroke="#F8C153" strokeWidth="18" />
      <circle cx="68" cy="61" r="21" fill="#F8C153" />
      <rect x="0" y="136" width="136" height="20" rx="10" fill="currentColor" />
    </svg>
  );
}
