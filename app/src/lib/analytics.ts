import { init, trackEvent } from "@aptabase/web";

// App key is injected at build time by Vite from the APTABASE_APP_KEY env var.
declare const __APTABASE_APP_KEY__: string;
const APP_KEY = typeof __APTABASE_APP_KEY__ !== "undefined" ? __APTABASE_APP_KEY__ : "";

if (APP_KEY) {
  init(APP_KEY, {
    appVersion: typeof __APP_VERSION__ !== "undefined" ? __APP_VERSION__ : "0.0.0",
    isDebug: import.meta.env.DEV,
  });
}

/**
 * Wrapper around Aptabase tracking. Fire-and-forget -- never throws.
 */
export const analytics = {
  track: (event: string, props?: Record<string, string | number>) => {
    if (!APP_KEY) return;
    try {
      trackEvent(event, props);
    } catch {
      // Analytics unavailable
    }
  },
};
