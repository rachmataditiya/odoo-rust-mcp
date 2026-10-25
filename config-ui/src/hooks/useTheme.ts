import { useState, useEffect } from 'react';

export type Theme = 'auto' | 'light' | 'dark';

const KEY = 'odoo-mcp.theme';

function getInitialTheme(): Theme {
  try {
    const saved = localStorage.getItem(KEY);
    return saved === 'light' || saved === 'dark' ? saved : 'auto';
  } catch {
    return 'auto';
  }
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  const isDark =
    theme === 'dark' ||
    (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);

  root.classList.toggle('light', !isDark);
  root.classList.toggle('dark', isDark);
  root.style.colorScheme = isDark ? 'dark' : 'light';
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(getInitialTheme);

  useEffect(() => {
    applyTheme(theme);

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = () => {
      if (theme === 'auto') {
        applyTheme('auto');
      }
    };

    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, [theme]);

  const setTheme = (newTheme: Theme) => {
    setThemeState(newTheme);
    try {
      if (newTheme === 'auto') {
        localStorage.removeItem(KEY);
      } else {
        localStorage.setItem(KEY, newTheme);
      }
    } catch {
      // Ignore storage errors
    }
    applyTheme(newTheme);
  };

  return { theme, setTheme };
}
