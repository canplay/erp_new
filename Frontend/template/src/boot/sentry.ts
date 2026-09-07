import { defineBoot } from '#q-app';
import * as Sentry from '@sentry/vue';

export default defineBoot(({ app, router }) => {
  Sentry.init({
    app,
    dsn: 'https://***@example.com/1',
    environment: 'production',
    release: 'qdsh@2025.1.21',
    integrations: [Sentry.browserTracingIntegration({ router }), Sentry.replayIntegration()],
    tracesSampleRate: 1.0,
    tracePropagationTargets: ['localhost', /^https:\/\/example.com/],
    replaysSessionSampleRate: 0.1,
    replaysOnErrorSampleRate: 1.0,
  });

  app.config.errorHandler = (error, vm, info) => {
    Sentry.captureException(error, {
      extra: { component: vm?.$options.name, lifecycleHook: info },
    });
  };

  window.addEventListener('unhandledrejection', (event) => {
    Sentry.captureException(event.reason);
  });
});
