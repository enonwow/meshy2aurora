export type StudioTheme = "dark" | "light" | "system";

export interface StudioHeaderProps {
  version: string;
  environment: string;
  theme: StudioTheme;
  onHelp?: () => void;
  onThemeMenu?: () => void;
  onSettings?: () => void;
}

function BrandMark() {
  return (
    <svg
      aria-hidden="true"
      className="studio-header__brand-mark"
      viewBox="0 0 32 40"
      fill="none"
    >
      <path d="M16 1 30 9v22l-14 8L2 31V9L16 1Z" stroke="currentColor" />
      <path d="m2 9 14 8 14-8M16 1v38M2 31l14-8 14 8" stroke="currentColor" />
      <path d="m9 13 14 8-14 8V13Zm14 0L9 21l14 8V13Z" stroke="currentColor" />
    </svg>
  );
}

function HelpIcon() {
  return (
    <svg aria-hidden="true" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="9" stroke="currentColor" />
      <path d="M9.8 9a2.3 2.3 0 1 1 3.7 1.8c-.9.7-1.5 1.1-1.5 2.2" stroke="currentColor" strokeLinecap="round" />
      <circle cx="12" cy="16.8" r=".8" fill="currentColor" />
    </svg>
  );
}

function ThemeIcon({ theme }: { theme: StudioTheme }) {
  if (theme === "light") {
    return (
      <svg aria-hidden="true" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="4" stroke="currentColor" />
        <path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M19 5l-1.5 1.5m-11 11L5 19" stroke="currentColor" strokeLinecap="round" />
      </svg>
    );
  }

  if (theme === "system") {
    return (
      <svg aria-hidden="true" viewBox="0 0 24 24" fill="none">
        <rect x="3" y="4" width="18" height="13" rx="2" stroke="currentColor" />
        <path d="M8 21h8m-4-4v4" stroke="currentColor" strokeLinecap="round" />
      </svg>
    );
  }

  return (
    <svg aria-hidden="true" viewBox="0 0 24 24" fill="none">
      <path d="M20 15.2A8.5 8.5 0 0 1 8.8 4 8.5 8.5 0 1 0 20 15.2Z" stroke="currentColor" strokeLinejoin="round" />
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg aria-hidden="true" viewBox="0 0 24 24" fill="none">
      <path d="M9.6 3.4 10.2 2h3.6l.6 1.4 1.6.7 1.4-.6 2.6 2.6-.6 1.4.7 1.6 1.4.6v3.6l-1.4.6-.7 1.6.6 1.4-2.6 2.6-1.4-.6-1.6.7-.6 1.4h-3.6L9.6 20 8 19.3l-1.4.6L4 17.3l.6-1.4-.7-1.6-1.4-.6v-3.6l1.4-.6.7-1.6L4 6.5l2.6-2.6 1.4.6 1.6-.7Z" stroke="currentColor" strokeLinejoin="round" />
      <circle cx="12" cy="12" r="3" stroke="currentColor" />
    </svg>
  );
}

export function StudioHeader({
  version,
  environment,
  theme,
  onHelp,
  onThemeMenu,
  onSettings,
}: StudioHeaderProps) {
  return (
    <header className="studio-header">
      <div className="studio-header__brand" aria-label="meshy2aurora">
        <BrandMark />
        <span className="studio-header__product-name">meshy2aurora</span>
        <span className="studio-header__version">{version}</span>
        <span className="studio-header__environment">{environment}</span>
      </div>

      <div className="studio-header__actions" aria-label="Studio controls">
        <button type="button" className="studio-header__action studio-header__help" disabled={!onHelp} onClick={onHelp}>
          <HelpIcon />
          <span>Help</span>
        </button>
        <button
          type="button"
          className="studio-header__action studio-header__theme"
          aria-label={`Theme: ${theme}`}
          aria-haspopup="menu"
          disabled={!onThemeMenu}
          onClick={onThemeMenu}
        >
          <ThemeIcon theme={theme} />
          <span aria-hidden="true" className="studio-header__chevron">⌄</span>
        </button>
        <button
          type="button"
          className="studio-header__action studio-header__settings"
          aria-label="Settings"
          disabled={!onSettings}
          onClick={onSettings}
        >
          <SettingsIcon />
        </button>
      </div>
    </header>
  );
}
