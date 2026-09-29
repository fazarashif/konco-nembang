/** Follow the Windows light/dark setting with the design system's window theme tokens. */
export function followSystemTheme(): void {
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const apply = () => {
    document.documentElement.dataset.knTheme = media.matches ? "dark" : "light";
  };
  apply();
  media.addEventListener("change", apply);
}
