import { defineBoot } from '#q-app';
import { setupSentry } from '@erp-new-frontend-monorepo/boot/sentry';

export default defineBoot(async ({ app, router }) => {
  await setupSentry(app, router);
});
