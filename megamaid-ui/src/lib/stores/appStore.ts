import { writable } from 'svelte/store';

export type NotificationType = 'info' | 'success' | 'warning' | 'error';

export interface Notification {
  id: string;
  type: NotificationType;
  message: string;
  duration?: number; // in milliseconds, 0 = persistent
}

export interface AppState {
  isLoading: boolean;
  currentView: string;
  theme: 'light' | 'dark' | 'auto';
}

// Default state
const defaultState: AppState = {
  isLoading: false,
  currentView: 'home',
  theme: 'auto',
};

// Stores
export const appState = writable<AppState>(defaultState);
export const notifications = writable<Notification[]>([]);

// Actions
export const appActions = {
  setLoading(isLoading: boolean) {
    appState.update((state) => ({ ...state, isLoading }));
  },

  setCurrentView(view: string) {
    appState.update((state) => ({ ...state, currentView: view }));
  },

  setTheme(theme: 'light' | 'dark' | 'auto') {
    appState.update((state) => ({ ...state, theme }));
    // Apply theme to document
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else if (theme === 'light') {
      document.documentElement.classList.remove('dark');
    } else {
      // Auto mode - use system preference
      const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      if (isDark) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    }
  },

  notify(notification: Omit<Notification, 'id'>) {
    const id = `notification-${Date.now()}-${Math.random()}`;
    const newNotification: Notification = {
      id,
      duration: 5000, // default 5 seconds
      ...notification,
    };

    notifications.update((items) => [...items, newNotification]);

    // Auto-dismiss if duration is set
    if (newNotification.duration && newNotification.duration > 0) {
      setTimeout(() => {
        appActions.dismissNotification(id);
      }, newNotification.duration);
    }

    return id;
  },

  dismissNotification(id: string) {
    notifications.update((items) => items.filter((n) => n.id !== id));
  },

  clearNotifications() {
    notifications.set([]);
  },
};
